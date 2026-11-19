//! Hatch.dll 10.1 的图层刻度扩展、刻度类、模板和定义。

use super::image_server_rules::{RuleSlot10_1, slot, unsupported_class};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::objects::query_filter::{
    QueryFilterBody10_1, QueryFilterObject10_1, read_query_filter_slot_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const HATCH_LAYER_EXTENSION_CLASS_ID: [u8; 16] = [
    0xdc, 0x48, 0xad, 0x37, 0xbe, 0x12, 0xa0, 0x45, 0xa4, 0x06, 0x26, 0xc4, 0x82, 0x7e, 0x4a, 0xf5,
];
const HATCH_CLASS_ID: [u8; 16] = [
    0x32, 0x7c, 0xe2, 0xfb, 0xe5, 0x80, 0xef, 0x49, 0xb2, 0xa4, 0xb7, 0xf9, 0x56, 0x99, 0x2c, 0x7c,
];
const HATCH_TEMPLATE_CLASS_ID: [u8; 16] = [
    0xbc, 0x75, 0x0a, 0x44, 0xc7, 0xa9, 0x50, 0x4c, 0xa4, 0xa7, 0x0f, 0xab, 0xb0, 0xe4, 0xc5, 0x21,
];
const HATCH_LINE_DEFINITION_CLASS_ID: [u8; 16] = [
    0xf5, 0x49, 0x1f, 0x1e, 0x67, 0x10, 0x73, 0x40, 0xad, 0x2e, 0x12, 0xfa, 0x4a, 0x19, 0x50, 0xf8,
];
const HATCH_MARKER_DEFINITION_CLASS_ID: [u8; 16] = [
    0xe9, 0x85, 0x07, 0xce, 0x51, 0xca, 0x83, 0x4b, 0xba, 0x73, 0x3f, 0xbf, 0xda, 0x99, 0xf3, 0xe2,
];

#[derive(Debug, Clone, PartialEq)]
pub struct HatchLayerExtension10_1 {
    pub version: i16,
    pub flag: u8,
    pub count: i32,
    pub classes: Vec<(String, RuleSlot10_1<HatchClass10_1>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HatchClass10_1 {
    pub version: i16,
    pub flag: u8,
    pub template: RuleSlot10_1<HatchTemplate10_1>,
    pub filter: QueryFilterObject10_1<QueryFilterBody10_1>,
    pub doubles: [f64; 2],
    pub definitions: [RuleSlot10_1<HatchDefinition10_1>; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct HatchTemplate10_1 {
    pub version: i16,
    pub name: String,
    pub double_before_definition: f64,
    pub definition: RuleSlot10_1<HatchDefinition10_1>,
    pub double_after_definition: f64,
    pub long_f8: i32,
    pub flags_f4_f3: [u8; 2],
    pub long_f0: i32,
    pub flag_ec: u8,
    pub count: i32,
    pub definitions: Vec<(i32, RuleSlot10_1<HatchDefinition10_1>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HatchDefinition10_1 {
    pub version: i16,
    pub symbol: SymbolSlotFull10_1,
    pub text_symbol: SymbolSlotFull10_1,
    pub double: f64,
    pub long_40: i32,
    pub flags: [u8; 3],
    pub strings: [String; 3],
    pub longs: [i32; 3],
    /// 线定义在共享字段后追加两个 double，标记定义读完共享字段即结束。
    pub line_doubles: Option<[f64; 2]>,
}

fn read_version(r: &mut Reader<'_>, label: &str) -> io::Result<i16> {
    let version = r.read_u16_le()? as i16;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("{label} 未支持版本 {version}"),
        ));
    }
    Ok(version)
}

fn bounded_count(r: &Reader<'_>, max: usize, raw: i32, item_min: usize) -> io::Result<usize> {
    let count = raw.max(0) as usize;
    if count > r.remaining().min(max) / item_min {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "刻度条目数量超出可读范围",
        ));
    }
    Ok(count)
}

fn read_hatch_slot<'a, T>(
    r: &mut Reader<'a>,
    refs: &mut RefTable10_1,
    label: &str,
    accepts: impl Fn([u8; 16]) -> bool,
    read: impl FnOnce(&mut Reader<'a>, &mut RefTable10_1, [u8; 16]) -> io::Result<T>,
) -> io::Result<RuleSlot10_1<T>> {
    let parsed = slot(r, refs, |r, refs, class, _| {
        if !accepts(class) {
            return Err(unsupported_class(label, &class));
        }
        read(r, refs, class)
    })?;
    if let ObjectEntry10_1::BackReference { class_id, .. } = parsed.entry
        && !accepts(class_id)
    {
        return Err(unsupported_class(label, &class_id));
    }
    Ok(parsed)
}

fn read_symbol(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<SymbolSlotFull10_1> {
    let parsed = read_symbol_slot_full_10_1(r, max, refs, MAX_SYMBOL_NESTING_DEPTH)?;
    if let Some(reason) = slot_full_stop_10_1(&parsed, DisplaySlotKind10_1::Symbol) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("刻度符号未完整读取：{reason:?}"),
        ));
    }
    Ok(parsed)
}

fn read_definition(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<HatchDefinition10_1>> {
    // 线和标记定义共用 0x758239a0，仅线定义在后面追加标量。
    read_hatch_slot(
        r,
        refs,
        "IHatchDefinition",
        |class| {
            matches!(
                class,
                HATCH_LINE_DEFINITION_CLASS_ID | HATCH_MARKER_DEFINITION_CLASS_ID
            )
        },
        |r, refs, class| {
            Ok(HatchDefinition10_1 {
                version: read_version(r, "HatchDefinition")?,
                symbol: read_symbol(r, max, refs)?,
                text_symbol: read_symbol(r, max, refs)?,
                double: r.read_f64_le()?,
                long_40: r.read_u32_le()? as i32,
                flags: r.read_bytes(3)?.try_into().unwrap(),
                strings: [
                    read_string_10_1(r, max)?,
                    read_string_10_1(r, max)?,
                    read_string_10_1(r, max)?,
                ],
                longs: [
                    r.read_u32_le()? as i32,
                    r.read_u32_le()? as i32,
                    r.read_u32_le()? as i32,
                ],
                line_doubles: (class == HATCH_LINE_DEFINITION_CLASS_ID)
                    .then(|| Ok::<_, io::Error>([r.read_f64_le()?, r.read_f64_le()?]))
                    .transpose()?,
            })
        },
    )
}

fn read_template(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<HatchTemplate10_1> {
    let version = read_version(r, "HatchTemplate")?;
    let name = read_string_10_1(r, max)?;
    let double_before_definition = r.read_f64_le()?;
    let definition = read_definition(r, max, refs)?;
    let double_after_definition = r.read_f64_le()?;
    let long_f8 = r.read_u32_le()? as i32;
    let flags_f4_f3 = r.read_bytes(2)?.try_into().unwrap();
    let long_f0 = r.read_u32_le()? as i32;
    let flag_ec = r.read_bytes(1)?[0];
    let count = r.read_u32_le()? as i32;
    let mut definitions = Vec::new();
    for _ in 0..bounded_count(r, max, count, 4 + 16)? {
        definitions.push((r.read_u32_le()? as i32, read_definition(r, max, refs)?));
    }
    Ok(HatchTemplate10_1 {
        version,
        name,
        double_before_definition,
        definition,
        double_after_definition,
        long_f8,
        flags_f4_f3,
        long_f0,
        flag_ec,
        count,
        definitions,
    })
}

fn read_class(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<HatchClass10_1> {
    let version = read_version(r, "HatchClass")?;
    let flag = r.read_bytes(1)?[0];
    let template = read_hatch_slot(
        r,
        refs,
        "IHatchTemplate",
        |class| class == HATCH_TEMPLATE_CLASS_ID,
        |r, refs, _| read_template(r, max, refs),
    )?;
    let mut stop = None;
    let filter = read_query_filter_slot_10_1(r, max, refs, &mut stop)?;
    if let Some(reason) = stop {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("HatchClass 查询过滤器未完整读取：{reason:?}"),
        ));
    }
    let doubles = [r.read_f64_le()?, r.read_f64_le()?];
    let definitions = [
        read_definition(r, max, refs)?,
        read_definition(r, max, refs)?,
        read_definition(r, max, refs)?,
        read_definition(r, max, refs)?,
    ];
    Ok(HatchClass10_1 {
        version,
        flag,
        template,
        filter,
        doubles,
        definitions,
    })
}

pub(super) fn read_bounded_hatch_layer_extension(
    bytes: &[u8],
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Box<HatchLayerExtension10_1>> {
    let mut r = Reader::new(bytes);
    let mut staged = refs.clone();
    let version = read_version(&mut r, "HatchLayerExtension")?;
    let flag = r.read_bytes(1)?[0];
    let count = r.read_u32_le()? as i32;
    let mut classes = Vec::new();
    for _ in 0..bounded_count(&r, max, count, 4 + 16)? {
        let name = read_string_10_1(&mut r, max)?;
        let hatch = read_hatch_slot(
            &mut r,
            &mut staged,
            "IHatchClass",
            |class| class == HATCH_CLASS_ID,
            |r, refs, _| read_class(r, max, refs),
        )?;
        classes.push((name, hatch));
    }
    if r.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "HatchLayerExtension 对象体消费与声明长度不一致",
        ));
    }
    *refs = staged;
    Ok(Box::new(HatchLayerExtension10_1 {
        version,
        flag,
        count,
        classes,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::raster_layer_tail::read_raster_layer_tail_10_1;

    #[test]
    fn hatch_extensions_preserve_nested_definition_boundaries_and_keys() {
        let empty_string = [2, 0, 0, 0, 0, 0];
        let append_header = |bytes: &mut Vec<u8>, class: [u8; 16], key: u32| {
            bytes.extend(class);
            bytes.extend(key.to_le_bytes());
        };
        let append_definition = |bytes: &mut Vec<u8>, line: Option<[f64; 2]>| {
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend([0; 32]); // 空符号和空文字符号。
            bytes.extend(0.75_f64.to_le_bytes());
            bytes.extend(5_i32.to_le_bytes());
            bytes.extend([1, 0, 1]);
            bytes.extend(empty_string.repeat(3));
            bytes.extend([0; 12]);
            if let Some(values) = line {
                for value in values {
                    bytes.extend(value.to_le_bytes());
                }
            }
        };

        let mut body = 1_i16.to_le_bytes().to_vec();
        body.push(1);
        body.extend(2_i32.to_le_bytes());
        body.extend(empty_string);
        append_header(&mut body, HATCH_CLASS_ID, 72);
        body.extend(1_i16.to_le_bytes());
        body.push(1);
        append_header(&mut body, HATCH_TEMPLATE_CLASS_ID, 73);
        body.extend(1_i16.to_le_bytes());
        body.extend(empty_string);
        body.extend(1.5_f64.to_le_bytes());
        let line_at = body.len();
        append_header(&mut body, HATCH_LINE_DEFINITION_CLASS_ID, 74);
        append_definition(&mut body, Some([22.5, -3.25]));
        body.extend(3_f64.to_le_bytes());
        body.extend(5_i32.to_le_bytes());
        body.extend([0, 1]);
        body.extend(11_i32.to_le_bytes());
        body.push(0);
        body.extend(2_i32.to_le_bytes());
        body.extend(25_i32.to_le_bytes());
        append_header(&mut body, HATCH_MARKER_DEFINITION_CLASS_ID, 75);
        append_definition(&mut body, None);
        body.extend(30_i32.to_le_bytes());
        append_header(&mut body, HATCH_LINE_DEFINITION_CLASS_ID, 74);
        body.extend([0; 16]); // 空查询过滤器。
        body.extend(500_f64.to_le_bytes());
        body.extend(2000_f64.to_le_bytes());
        append_header(&mut body, HATCH_MARKER_DEFINITION_CLASS_ID, 75);
        body.extend([0; 3 * 16]);
        body.extend(empty_string);
        append_header(&mut body, HATCH_CLASS_ID, 72);

        for version in [3, 8] {
            let build = |body: &[u8]| {
                let mut bytes = vec![0; 26];
                bytes.extend(1_i32.to_le_bytes());
                if version == 3 {
                    bytes.extend((body.len() as u64).to_le_bytes());
                    bytes.extend(HATCH_LAYER_EXTENSION_CLASS_ID);
                    bytes.extend(body);
                } else {
                    bytes.extend((20 + body.len() as u32).to_le_bytes());
                    bytes.extend(0_u32.to_le_bytes());
                    append_header(&mut bytes, HATCH_LAYER_EXTENSION_CLASS_ID, 76);
                    bytes.extend(body);
                    bytes.extend([0; 8 + 32 + 8 + 16]);
                }
                bytes
            };

            let bytes = build(&body);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let tail = read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).unwrap();
            let hatch = if version == 3 {
                let mut legacy = Reader::new(&bytes[26..]);
                let mut legacy_refs = RefTable10_1::new();
                let shared = crate::layers::layer_extensions::read_layer_extensions_10_1(
                    &mut legacy,
                    4096,
                    &mut legacy_refs,
                )
                .unwrap();
                assert_eq!(legacy.remaining(), 0);
                assert_eq!(legacy_refs.len(), 4);
                let entry = &tail.legacy_extensions[0].hatch_layer_extension;
                assert_eq!(entry, &shared[0].hatch_layer_extension);
                entry
            } else {
                assert!(refs.contains(76));
                &tail.extra_safe_extensions[0].hatch_layer_extension
            }
            .as_ref()
            .unwrap();

            assert_eq!(reader.remaining(), 0);
            assert!((72..=75).all(|key| refs.contains(key)));
            assert_eq!(hatch.flag, 1);
            let class = hatch.classes[0].1.body.as_ref().unwrap();
            assert_eq!(class.doubles, [500.0, 2000.0]);
            let template = class.template.body.as_ref().unwrap();
            assert_eq!(
                template.definition.body.as_ref().unwrap().line_doubles,
                Some([22.5, -3.25])
            );
            assert_eq!(template.double_after_definition, 3.0);
            assert_eq!(template.definitions[0].0, 25);
            assert!(
                template.definitions[0]
                    .1
                    .body
                    .as_ref()
                    .unwrap()
                    .line_doubles
                    .is_none()
            );
            assert!(matches!(
                template.definitions[1].1.entry,
                ObjectEntry10_1::BackReference { key: 74, .. }
            ));
            assert!(matches!(
                class.definitions[0].entry,
                ObjectEntry10_1::BackReference { key: 75, .. }
            ));
            assert!(matches!(
                hatch.classes[1].1.entry,
                ObjectEntry10_1::BackReference { key: 72, .. }
            ));

            let mut unknown_definition = body.clone();
            unknown_definition[line_at..line_at + 16].fill(0xa9);
            let mut oversized_count = body.clone();
            oversized_count[3..7].copy_from_slice(&i32::MAX.to_le_bytes());
            for broken in [
                &body[..body.len() - 1],
                &unknown_definition,
                &oversized_count,
            ] {
                let bytes = build(broken);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                assert!(
                    read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }
}
