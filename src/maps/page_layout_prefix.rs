//! ArcMap 10.1 PageLayout 对象体的元素循环前缀。
//!
//! PageLayout.dll 10.1.0.3035 的 Load `0x7b6e4e30` 读取无符号版本、
//! 4 字节成员和有符号元素数；版本小于 7 使用普通对象读取，否则使用
//! ExtraSafeReadObject。调用者须先从对象流消费 PageLayout 对象头并保留
//! 跨流引用表。此处不跳过元素体，也不猜测未知成员的业务含义。

use crate::maps::map_object::MAP_CLASS_ID;
use crate::objects::element_body::{
    ElementBody10_1, ParagraphTextElement10_1, read_element_body_of_class_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// PageLayout 类标识（Windows 字节序）。
pub const PAGE_LAYOUT_CLASS_ID: [u8; 16] = [
    0x6e, 0xd7, 0x94, 0xdd, 0x6d, 0x83, 0xd0, 0x11, 0x87, 0xec, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// Layer.dll 10.1 FrameElement（直接委派 BaseFrameElement Load）。
pub const FRAME_ELEMENT_CLASS_ID: [u8; 16] = [
    0xc5, 0xa2, 0x1b, 0xe0, 0xb2, 0x24, 0xd3, 0x11, 0xb8, 0xaa, 0, 0x60, 0x08, 0x02, 0xe6, 0x03,
];

/// MapFrame 类标识；MapElements.dll 10.1.0.3035 的 GetClassID 0x795709f0。
pub const MAP_FRAME_CLASS_ID: [u8; 16] = [
    0x70, 0xd7, 0x94, 0xdd, 0x6d, 0x83, 0xd0, 0x11, 0x87, 0xec, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageLayoutMapFramePrefix10_1 {
    pub frame_key: u32,
    pub version: u16,
    /// 第一处 IMap 槽在 /PageLayout 内的绝对偏移。
    pub map_offset: usize,
    pub map: ObjectEntry10_1,
    /// 读取窗口中 IMap 槽之后的剩余字节数；嵌套地图框的窗口还包含宿主后段。
    pub remaining_in_element: usize,
}

/// 读取有界 MapFrame 元素的版本与 IMap 槽；已有 Map 必须来自共享对象流会话。
/// 成功将游标推进至 IMap 槽之后，失败不提交游标或登记表。
pub fn read_page_layout_map_frame_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<PageLayoutMapFramePrefix10_1> {
    let (mut prefix, mut region) = read_extra_safe_element_region(reader, max_bytes)?;
    let frame = read_map_frame_prefix(&mut region, prefix.position(), table)?;
    prefix.read_bytes(region.position())?;
    *reader = prefix;
    table.register(frame.frame_key);
    Ok(frame)
}

/// 完整读取一个有界 MapFrame。任何内嵌体未闭合，均不提交游标或键表。
pub fn read_page_layout_map_frame_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<super::map_frame::MapFrame10_1> {
    let (prefix, mut region) = read_extra_safe_element_region(reader, max_bytes)?;
    let frame = read_map_frame_prefix(&mut region, prefix.position(), table)?;
    read_bounded_map_frame_tail(reader, prefix, region, max_bytes, table, frame)
}

fn read_bounded_map_frame_tail<'a>(
    reader: &mut Reader<'a>,
    mut prefix: Reader<'a>,
    mut region: Reader<'a>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    frame: PageLayoutMapFramePrefix10_1,
) -> io::Result<super::map_frame::MapFrame10_1> {
    let mut refs = table.clone();
    refs.register(frame.frame_key);
    let body = super::map_frame::read_map_frame_tail(&mut region, max_bytes, &mut refs, frame)
        .map_err(|err| {
            io::Error::new(
                err.kind(),
                format!(
                    "MapFrame /PageLayout:0x{:x}：{err}",
                    prefix.position() + region.position()
                ),
            )
        })?;
    if region.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "MapFrame 未完整消费声明范围",
        ));
    }
    prefix.read_bytes(region.position())?;
    *reader = prefix;
    *table = refs;
    Ok(body)
}

pub(super) fn read_map_frame_prefix(
    region: &mut Reader<'_>,
    region_offset: usize,
    table: &RefTable10_1,
) -> io::Result<PageLayoutMapFramePrefix10_1> {
    let ObjectHeader::Object {
        class_id,
        key: frame_key,
    } = read_header_10_1(region)?
    else {
        return Err(io::Error::new(ErrorKind::InvalidData, "MapFrame 元素为空"));
    };
    if class_id != MAP_FRAME_CLASS_ID || table.contains(frame_key) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "此槽不是新 MapFrame 对象",
        ));
    }
    let version = region.read_u16_le()?;
    if version > 4 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的 MapFrame 版本",
        ));
    }
    let map_offset = region_offset + region.position();
    let map = match read_header_10_1(region)? {
        ObjectHeader::Null => ObjectEntry10_1::Null,
        ObjectHeader::Object { class_id, key }
            if class_id == MAP_CLASS_ID && table.contains(key) =>
        {
            ObjectEntry10_1::BackReference { class_id, key }
        }
        ObjectHeader::Object { key, .. } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("未支持的 MapFrame IMap 槽：/PageLayout:0x{map_offset:x}，键 {key}"),
            ));
        }
    };
    Ok(PageLayoutMapFramePrefix10_1 {
        frame_key,
        version,
        map_offset,
        map,
        remaining_in_element: region.remaining(),
    })
}

fn read_extra_safe_element_region<'a>(
    reader: &Reader<'a>,
    max_bytes: usize,
) -> io::Result<(Reader<'a>, Reader<'a>)> {
    let mut prefix = reader.clone();
    let size = prefix.read_u32_le()? as usize;
    let seek = prefix.read_u32_le()?;
    if seek != 0 || size < 16 || size > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的页面元素范围",
        ));
    }
    let region = prefix.clone().read_region(size)?;
    Ok((prefix, region))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageLayoutPrefix10_1 {
    pub version: u16,
    pub raw_member: u32,
    pub element_count: u32,
    pub extra_safe: bool,
}

/// 读取元素循环之前的固定前缀；错误时不推进游标。
pub fn read_page_layout_prefix_10_1(
    reader: &mut Reader<'_>,
    max_elements: u32,
) -> io::Result<PageLayoutPrefix10_1> {
    let mut trial = reader.clone();
    let version = u16::from_le_bytes(trial.read_bytes(2)?.try_into().unwrap());
    if version >= 16 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "PageLayout 版本不受支持",
        ));
    }
    let raw_member = u32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if count < 0 || count as u32 > max_elements {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "PageLayout 元素数量无效或超过限额",
        ));
    }
    let prefix = PageLayoutPrefix10_1 {
        version,
        raw_member,
        element_count: count as u32,
        extra_safe: version >= 7,
    };
    *reader = trial;
    Ok(prefix)
}

/// 读取一个页面元素的 ExtraSafe 包装和已支持的元素体。
///
/// 仅支持 B 为零且 A 等于对象头加对象体长度的已观察形态；该长度解释
/// 尚属样本推定。遇到不支持的类、内层停止或范围不匹配时整体回滚，绝不
/// 跳过对象体或虚构引用登记。调用者负责按原文档的跨流顺序提供引用表。
pub fn read_page_layout_element_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, ElementBody10_1)> {
    let (mut trial, mut region) = read_extra_safe_element_region(reader, max_bytes)?;
    let mut refs = table.clone();
    let entry = read_object_entry_10_1(&mut region, &mut refs)?;
    let ObjectEntry10_1::NewObject { class_id, .. } = entry else {
        return Err(io::Error::new(ErrorKind::Unsupported, "页面元素并非新对象"));
    };
    let body = read_element_body_of_class_10_1(&class_id, &mut region, max_bytes, &mut refs)?;
    if let Some(stop) = body.stop() {
        return Err(io::Error::new(ErrorKind::Unsupported, stop.describe()));
    }
    if region.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "页面元素未完整消费声明范围",
        ));
    }
    trial.read_bytes(region.position())?;
    *reader = trial;
    *table = refs;
    Ok((entry, body))
}

/// 已顺序读出的布局元素集合及可识别的尾段。
#[derive(Debug, Clone, PartialEq)]
pub struct PageLayoutElements10_1 {
    pub prefix: PageLayoutPrefix10_1,
    pub map_frames: Vec<super::map_frame::MapFrame10_1>,
    pub surround_frames: Vec<super::surround_frame::MapSurroundFrame10_1>,
    /// 不带对象体的元素槽，保存其序号、包装起点和条目。
    pub indirect_elements: Vec<(u32, usize, ObjectEntry10_1)>,
    pub rectangles: Vec<super::rectangle_element::RectangleElement10_1>,
    pub frame_elements: Vec<(u32, super::map_frame::BaseFrameElement10_1)>,
    pub paragraphs: Vec<(u32, ParagraphTextElement10_1)>,
    pub groups: Vec<super::group_element::GroupElement10_1>,
    pub consumed_elements: u32,
    /// 元素列表末端；元素未读完时为首个未解析元素的位置。
    pub end_offset: usize,
    pub stop: Option<String>,
    pub tail: Option<super::page_layout_tail::PageLayoutTail10_1>,
    pub tail_error: Option<String>,
    /// 文档加载器在 PageLayout 对象体之后读取；非零时后接打印纸张及打印机。
    pub print_settings_flag: Option<u16>,
}

impl PageLayoutElements10_1 {
    pub fn consumed_end_offset(&self) -> usize {
        self.tail.as_ref().map_or(self.end_offset, |tail| {
            tail.end_offset + usize::from(self.print_settings_flag.is_some()) * 2
        })
    }
}

/// 用 /Maps 的同一键表解析 /PageLayout 根与 ExtraSafe 元素集合。
/// 按原生顺序遍历；未知类或嵌套对象会停止，避免遗漏其可能登记的键。
pub(super) fn read_page_layout_elements_10_1(
    page: &[u8],
    max_bytes: usize,
    refs: &mut RefTable10_1,
    is_root_map: impl Fn(u32) -> bool,
) -> io::Result<PageLayoutElements10_1> {
    let mut reader = Reader::new(page);
    match read_header_10_1(&mut reader)? {
        ObjectHeader::Object { class_id, key }
            if class_id == PAGE_LAYOUT_CLASS_ID && !refs.contains(key) =>
        {
            let prefix = read_page_layout_prefix_10_1(&mut reader, 65_536)?;
            refs.register(key);
            let mut result = PageLayoutElements10_1 {
                prefix,
                map_frames: Vec::new(),
                surround_frames: Vec::new(),
                indirect_elements: Vec::new(),
                rectangles: Vec::new(),
                frame_elements: Vec::new(),
                paragraphs: Vec::new(),
                groups: Vec::new(),
                consumed_elements: 0,
                end_offset: reader.position(),
                stop: None,
                tail: None,
                tail_error: None,
                print_settings_flag: None,
            };
            if result.prefix.element_count > 0 && !result.prefix.extra_safe {
                result.stop = Some("尚未接入普通对象形式的页面元素循环".into());
                return Ok(result);
            }
            for index in 0..result.prefix.element_count {
                let offset = reader.position();
                match read_element_or_map_frame(&mut reader, max_bytes, refs, &is_root_map) {
                    Ok(element) => {
                        match element {
                            PageFrame::Map(frame) => result.map_frames.push(*frame),
                            PageFrame::Surround(frame) => result.surround_frames.push(*frame),
                            PageFrame::Indirect(entry) => {
                                result.indirect_elements.push((index, offset, entry))
                            }
                            PageFrame::Rectangle(rect) => result.rectangles.push(*rect),
                            PageFrame::Base(key, body) => result.frame_elements.push((key, *body)),
                            PageFrame::Paragraph(key, body) => result.paragraphs.push((key, *body)),
                            PageFrame::Group(body) => result.groups.push(*body),
                            PageFrame::Other => {}
                        }
                        result.consumed_elements += 1;
                        result.end_offset = reader.position();
                    }
                    Err(error) => {
                        result.stop = Some(error.to_string());
                        break;
                    }
                }
            }
            if result.stop.is_none() {
                match super::page_layout_tail::read_page_layout_tail_10_1(
                    &mut reader,
                    refs,
                    result.prefix.version,
                    result.prefix.element_count,
                    max_bytes,
                    &is_root_map,
                ) {
                    Ok(tail) => {
                        result.tail = Some(tail);
                        // 外层文档读侧 0x7b708d00：ObjectStream::LoadObject
                        // 返回后经当前 IStream 再读两字节，而非 PageLayout::Load。
                        match reader.read_u16_le() {
                            Ok(flag) => {
                                result.print_settings_flag = Some(flag);
                                if flag != 0 {
                                    result.tail_error =
                                        Some("非空打印设置（IPaper / IPrinter）尚未支持".into());
                                }
                            }
                            Err(error) => {
                                result.tail_error = Some(format!("打印设置标记截断：{error}"))
                            }
                        }
                    }
                    Err(error) => result.tail_error = Some(error.to_string()),
                }
            }
            Ok(result)
        }
        _ => Err(io::Error::new(
            ErrorKind::Unsupported,
            "未识别的新 PageLayout 根对象",
        )),
    }
}

enum PageFrame {
    Map(Box<super::map_frame::MapFrame10_1>),
    Surround(Box<super::surround_frame::MapSurroundFrame10_1>),
    Indirect(ObjectEntry10_1),
    Rectangle(Box<super::rectangle_element::RectangleElement10_1>),
    Base(u32, Box<super::map_frame::BaseFrameElement10_1>),
    Paragraph(u32, Box<ParagraphTextElement10_1>),
    Group(Box<super::group_element::GroupElement10_1>),
    Other,
}

fn read_element_or_map_frame(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    is_root_map: &impl Fn(u32) -> bool,
) -> io::Result<PageFrame> {
    use super::surround_frame::{MAP_SURROUND_FRAME_CLASS_ID, read_surround_frame_10_1};
    let (mut prefix, mut region) = read_extra_safe_element_region(reader, max_bytes)?;
    let mut probe = region.clone();
    let header = read_header_10_1(&mut probe)?;
    let indirect = match header {
        ObjectHeader::Null => Some(ObjectEntry10_1::Null),
        ObjectHeader::Object { class_id, key } if refs.contains(key) => {
            Some(ObjectEntry10_1::BackReference { class_id, key })
        }
        _ => None,
    };
    if let Some(entry) = indirect {
        if probe.remaining() != 0 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "页面元素空槽或回引之后仍有额外字节",
            ));
        }
        prefix.read_bytes(probe.position())?;
        *reader = prefix;
        return Ok(PageFrame::Indirect(entry));
    }
    match header {
        ObjectHeader::Object {
            class_id: MAP_FRAME_CLASS_ID,
            ..
        } => {
            let frame = read_map_frame_prefix(&mut region, prefix.position(), refs)?;
            check_root_map_link(&frame, is_root_map)?;
            read_bounded_map_frame_tail(reader, prefix, region, max_bytes, refs, frame)
                .map(|frame| PageFrame::Map(Box::new(frame)))
        }
        ObjectHeader::Object {
            class_id: MAP_SURROUND_FRAME_CLASS_ID,
            ..
        } => {
            let start = prefix.position();
            let frame = read_bounded_page_body(
                reader,
                prefix,
                region,
                refs,
                "MapSurroundFrame",
                |r, staged| read_surround_frame_10_1(r, start, max_bytes, staged, is_root_map),
            )?;
            Ok(PageFrame::Surround(Box::new(frame)))
        }
        ObjectHeader::Object {
            class_id: super::rectangle_element::RECTANGLE_ELEMENT_CLASS_ID,
            ..
        } => {
            let rect = read_bounded_page_body(
                reader,
                prefix,
                region,
                refs,
                "RectangleElement",
                |r, staged| {
                    super::rectangle_element::read_rectangle_element_10_1(r, max_bytes, staged)
                },
            )?;
            Ok(PageFrame::Rectangle(Box::new(rect)))
        }
        ObjectHeader::Object {
            class_id: FRAME_ELEMENT_CLASS_ID,
            key,
        } => {
            let base = read_bounded_page_body(
                reader,
                prefix,
                region,
                refs,
                "FrameElement",
                |r, staged| {
                    r.read_bytes(20)?; // 此处已确认为新 FrameElement 的头。
                    staged.register(key);
                    super::map_frame::base_frame(r, max_bytes, staged)
                },
            )?;
            Ok(PageFrame::Base(key, Box::new(base)))
        }
        ObjectHeader::Object {
            class_id: super::group_element::GROUP_ELEMENT_CLASS_ID,
            ..
        } => read_bounded_page_body(reader, prefix, region, refs, "GroupElement", |r, staged| {
            super::group_element::read_group_element_10_1(r, max_bytes, staged)
        })
        .map(|body| PageFrame::Group(Box::new(body))),
        _ => read_page_layout_element_10_1(reader, max_bytes, refs).map(|(entry, body)| {
            match (entry, body) {
                (
                    ObjectEntry10_1::NewObject { key, .. },
                    ElementBody10_1::ParagraphTextElement(body),
                ) => PageFrame::Paragraph(key, body),
                _ => PageFrame::Other,
            }
        }),
    }
}

fn read_bounded_page_body<'a, T>(
    reader: &mut Reader<'a>,
    mut prefix: Reader<'a>,
    mut region: Reader<'a>,
    refs: &mut RefTable10_1,
    name: &str,
    read_body: impl FnOnce(&mut Reader<'a>, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<T> {
    let mut staged = refs.clone();
    let body = read_body(&mut region, &mut staged).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!(
                "{name} /PageLayout:0x{:x}：{err}",
                prefix.position() + region.position()
            ),
        )
    })?;
    if region.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{name} 未完整消费声明范围"),
        ));
    }
    prefix.read_bytes(region.position())?;
    *reader = prefix;
    *refs = staged;
    Ok(body)
}

pub(super) fn check_root_map_link(
    frame: &PageLayoutMapFramePrefix10_1,
    is_root_map: &impl Fn(u32) -> bool,
) -> io::Result<()> {
    if matches!(frame.map, ObjectEntry10_1::BackReference { key, .. } if !is_root_map(key)) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "MapFrame 指向尚未确认的根 Map 键",
        ));
    }
    Ok(())
}
