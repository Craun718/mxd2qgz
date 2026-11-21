//! Map.dll 10.1.0.3035 Load 0x79400ab0..0x794015af 的持久化后段。
//! 按读侧条件门处理；值的业务语义未确认时，仅保留原始位与偏移。

use crate::layers::dep_tail::{
    TIME_EXTENT_CLASS_ID, TIME_REFERENCE_CLASS_ID, TimeExtentBody10_1, TimeReferenceBody10_1,
    read_time_extent_body_10_1, read_time_reference_body_10_1,
};
use crate::layers::image_server_rules::{
    MosaicTimeValue10_1, TIME_INSTANT_CLASS_ID, read_time_value,
};
use crate::objects::element_body::{
    PARAGRAPH_TEXT_ELEMENT_CLASS_ID, ParagraphTextElement10_1, read_paragraph_text_element_10_1,
};
use crate::objects::envelope::{ENVELOPE_CLASS_ID, Envelope10_1, read_envelope_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub enum LateMapValue10_1 {
    Short(u16),
    Long(u32),
    DoubleBits(u64),
    DoubleListBits(Vec<u64>),
    String(String),
    Object {
        entry: ObjectEntry10_1,
        envelope: Option<Box<Envelope10_1>>,
    },
    ParagraphTextElement {
        entry: ObjectEntry10_1,
        body: Box<ParagraphTextElement10_1>,
    },
    TimeExtent(Box<TimeExtentBody10_1>),
    TimeInstant {
        key: u32,
        body: Box<MosaicTimeValue10_1>,
    },
    TimeReference(Box<TimeReferenceBody10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LateMapField10_1 {
    pub offset: usize,
    /// 已确认的 Load 成员偏移或局部字段标识。
    pub field: &'static str,
    pub value: LateMapValue10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LateMapTail10_1 {
    pub fields: Vec<LateMapField10_1>,
    /// 未读字段及出错原因；此时游标停在该字段起点。
    pub stop: Option<String>,
}

struct TailReader<'a, 'b> {
    stream: &'a mut Reader<'b>,
    refs: &'a mut RefTable10_1,
    max_bytes: usize,
    fields: Vec<LateMapField10_1>,
}

impl TailReader<'_, '_> {
    fn field(
        &mut self,
        name: &'static str,
        read: impl FnOnce(&mut Reader<'_>, &mut RefTable10_1) -> io::Result<LateMapValue10_1>,
    ) -> io::Result<()> {
        let offset = self.stream.position();
        let mut trial = self.stream.clone();
        let value = read(&mut trial, self.refs)
            .map_err(|err| io::Error::new(err.kind(), format!("{name} @ 0x{offset:x}：{err}")))?;
        self.fields.push(LateMapField10_1 {
            offset,
            field: name,
            value,
        });
        *self.stream = trial;
        Ok(())
    }

    fn short(&mut self, name: &'static str) -> io::Result<()> {
        self.field(name, |r, _| Ok(LateMapValue10_1::Short(r.read_u16_le()?)))
    }

    fn long(&mut self, name: &'static str) -> io::Result<()> {
        self.field(name, |r, _| Ok(LateMapValue10_1::Long(r.read_u32_le()?)))
    }

    fn double(&mut self, name: &'static str) -> io::Result<()> {
        self.field(name, |r, _| {
            Ok(LateMapValue10_1::DoubleBits(r.read_f64_le()?.to_bits()))
        })
    }

    fn string(&mut self, name: &'static str) -> io::Result<()> {
        let limit = self.max_bytes;
        self.field(name, |r, _| {
            Ok(LateMapValue10_1::String(read_string_10_1(r, limit)?))
        })
    }

    fn doubles(&mut self, name: &'static str) -> io::Result<()> {
        let limit = self.max_bytes;
        self.field(name, |r, _| {
            let count = bounded_count(r, limit, 8)?;
            let values = (0..count)
                .map(|_| r.read_f64_le().map(f64::to_bits))
                .collect::<io::Result<Vec<_>>>()?;
            Ok(LateMapValue10_1::DoubleListBits(values))
        })
    }

    // TIME_INSTANT_CLASS_ID 表示 ITimeValue 槽，兼容 TimeExtent 实现。
    fn object(&mut self, name: &'static str, new_class: Option<[u8; 16]>) -> io::Result<()> {
        let limit = self.max_bytes;
        self.field(name, |r, refs| {
            let (entry, envelope) = match read_header_10_1(r)? {
                ObjectHeader::Null => (ObjectEntry10_1::Null, None),
                ObjectHeader::Object { class_id, key } if refs.contains(key) => {
                    (ObjectEntry10_1::BackReference { class_id, key }, None)
                }
                ObjectHeader::Object { class_id, key }
                    if class_id == TIME_EXTENT_CLASS_ID
                        && matches!(
                            new_class,
                            Some(TIME_EXTENT_CLASS_ID | TIME_INSTANT_CLASS_ID)
                        ) =>
                {
                    let mut refs_trial = refs.clone();
                    refs_trial.register(key);
                    let body = read_time_extent_body_10_1(r, limit, &mut refs_trial, key)?;
                    if let Some(stop) = &body.stop {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            format!("TimeExtent 对象体未完整读取：{stop:?}"),
                        ));
                    }
                    *refs = refs_trial;
                    return Ok(LateMapValue10_1::TimeExtent(Box::new(body)));
                }
                ObjectHeader::Object {
                    class_id: TIME_INSTANT_CLASS_ID,
                    key,
                } if new_class == Some(TIME_INSTANT_CLASS_ID) => {
                    let mut refs_trial = refs.clone();
                    refs_trial.register(key);
                    let body =
                        read_time_value(r, &mut refs_trial, limit, TIME_INSTANT_CLASS_ID, key)?;
                    *refs = refs_trial;
                    return Ok(LateMapValue10_1::TimeInstant {
                        key,
                        body: Box::new(body),
                    });
                }
                ObjectHeader::Object { class_id, key }
                    if Some(class_id) == new_class && class_id == TIME_REFERENCE_CLASS_ID =>
                {
                    let mut refs_trial = refs.clone();
                    refs_trial.register(key);
                    let body = read_time_reference_body_10_1(r, limit, &mut refs_trial, key)?;
                    if body.short_b.is_none() {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            "TimeReference 的时区槽尚未完整读取",
                        ));
                    }
                    *refs = refs_trial;
                    return Ok(LateMapValue10_1::TimeReference(Box::new(body)));
                }
                ObjectHeader::Object { class_id, key }
                    if Some(class_id) == new_class && class_id == ENVELOPE_CLASS_ID =>
                {
                    let mut refs_trial = refs.clone();
                    refs_trial.register(key);
                    let envelope = read_envelope_10_1(r, limit, &mut refs_trial)?;
                    *refs = refs_trial;
                    (
                        ObjectEntry10_1::NewObject { class_id, key },
                        Some(Box::new(envelope)),
                    )
                }
                ObjectHeader::Object { class_id, key }
                    if Some(class_id) == new_class
                        && class_id == PARAGRAPH_TEXT_ELEMENT_CLASS_ID =>
                {
                    let mut refs_trial = refs.clone();
                    refs_trial.register(key);
                    let body = read_paragraph_text_element_10_1(r, limit, &mut refs_trial)?;
                    if let Some(stop) = &body.stop {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            format!("段落文本元素体未完整读取：{stop:?}"),
                        ));
                    }
                    *refs = refs_trial;
                    return Ok(LateMapValue10_1::ParagraphTextElement {
                        entry: ObjectEntry10_1::NewObject { class_id, key },
                        body: Box::new(body),
                    });
                }
                ObjectHeader::Object { class_id, key } => {
                    return Err(io::Error::new(
                        ErrorKind::Unsupported,
                        format!("新对象类 {class_id:02x?} 键 {key} 的体尚未支持"),
                    ));
                }
            };
            Ok(LateMapValue10_1::Object { entry, envelope })
        })
    }

    fn read(&mut self, version: u16) -> io::Result<()> {
        // 0x79400ab0：v34 起的属性，包括三个对象槽。
        self.long("+0x1d0")?;
        self.double("+0x1d8")?;
        self.object("+0x1e0", Some(ENVELOPE_CLASS_ID))?;
        self.object("+0x1e4", None)?;
        self.double("+0x1e8")?;
        self.long("+0x1f0")?;
        self.object("+0x1f4", None)?;
        self.short("+0x3ea")?;

        if version >= 36 {
            // 0x79400c76
            self.doubles("用户比例尺集合")?;
        }
        if version >= 37 {
            // 0x79400cf3
            self.long("ScaleSnapping")?;
        }
        if version >= 38 {
            // 0x79400d44
            self.double("+0x408")?;
            self.long("+0x410")?;
            self.double("+0x418")?;
            self.object("+0x420", Some(TIME_EXTENT_CLASS_ID))?;
            self.object("v38 ITimeValue", Some(TIME_INSTANT_CLASS_ID))?;
            self.object("v38 ITimeReference", Some(TIME_REFERENCE_CLASS_ID))?;
        }
        if version >= 39 {
            // 0x79400ed3
            self.object("+0x424", Some(TIME_EXTENT_CLASS_ID))?;
            self.long("v39 LONG")?;
            self.string("+0x42c")?;
            self.short("+0x430")?;
            self.long("+0x434")?;
            self.long("+0x438")?;
            if version < 45 {
                // 0x7940104c
                self.short("v39..44 short")?;
            }
        }
        if version >= 41 {
            // 0x794010be
            self.long("+0x43c")?;
            self.object("+0x440", None)?;
        }
        if version == 42 {
            // 0x7940113b
            self.short("v42 short")?;
            self.short("+0x44a")?;
        }
        if version >= 43 {
            // 0x794011a7
            self.short("+0x44a")?;
            self.string("+0x428")?;
        }
        if version >= 44 {
            // 0x794011e4
            self.object("+0x44c", Some(TIME_INSTANT_CLASS_ID))?;
        }
        if version >= 45 {
            // 0x79401217
            self.short("+0x450")?;
        }
        if version >= 47 {
            // 0x794012c3
            self.short("+0x454")?;
        }
        if version >= 48 {
            // 0x794012ee
            self.long("v48 LONG")?;
        }
        if version == 49 {
            // 0x7940134e
            let offset = self.stream.position();
            let mut trial = self.stream.clone();
            let count = bounded_count(&mut trial, self.max_bytes, 18).map_err(|err| {
                io::Error::new(err.kind(), format!("v49 对象计数 @ 0x{offset:x}：{err}"))
            })?;
            self.long("v49 对象计数")?;
            for _ in 0..count {
                self.object("v49 集合对象", None)?;
                self.short("v49 集合 short")?;
            }
        }
        if version >= 51 {
            // 0x794013f2
            self.short("+0x452")?;
            self.object("v51 对象", Some(PARAGRAPH_TEXT_ELEMENT_CLASS_ID))?;
        }
        Ok(())
    }
}

fn bounded_count(reader: &mut Reader<'_>, max_bytes: usize, min_item: usize) -> io::Result<usize> {
    let count = reader.read_u32_le()? as i32;
    let count = usize::try_from(count)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Map 后段集合计数为负"))?;
    let size = count
        .checked_mul(min_item)
        .filter(|&size| size <= max_bytes)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "Map 后段集合超过限额"))?;
    if size > reader.remaining() {
        return Err(io::Error::new(ErrorKind::UnexpectedEof, "Map 后段集合截断"));
    }
    Ok(count)
}

/// 从已完整读取的版本 33 集合之后读取，直到对象终点或首个未支持的字段。
/// 未知新对象没有通用体长，保留其头和后续字节，且不登记该键。
pub fn read_map_late_tail_10_1(
    reader: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    max_bytes: usize,
    version: u16,
) -> io::Result<LateMapTail10_1> {
    if !(34..=51).contains(&version) {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "Map 后段版本须为 34..=51",
        ));
    }
    let mut parser = TailReader {
        stream: reader,
        refs,
        max_bytes,
        fields: Vec::new(),
    };
    let stop = parser.read(version).err().map(|err| err.to_string());
    Ok(LateMapTail10_1 {
        fields: parser.fields,
        stop,
    })
}
