//! OLE StdFont / StdPicture 持久化体（`src/objects/ole_font.rs`）与
//! TextSymbol IFontDisp 槽类分派的单元测试。
//!
//! 字段序证据见 `docs/arcmap-10.1-ole-stdfont.md`：StdFont 形态由 10.1 介质
//! 分发的 oleaut32 2.40.4275 Load/Save 反编译给出，并与仓库参考样本 /Maps 流
//! 37 处 StdFont 新对象逐字节互证（本文件含样本首例 `0xd6c` 键 68 的体字节
//! 原样用例）。StdPicture 样本 0 实例，全部为合成输入。仅验证读取器自身契
//! 约，不代表 MXD 格式规则已验证，未经目标软件人工检查。

use mxd2qgz::objects::element_body::{
    ElementBodyStop10_1, FontSlot10_1, read_text_symbol_body_10_1,
};
use mxd2qgz::objects::ole_font::{
    STDFONT_CLASS_ID, STDPICTURE_CLASS_ID, read_std_font_body_10_1, read_std_picture_body_10_1,
};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

/// AfCore String::Write 布局：u32 字节长度前缀（含终止零）+ UTF-16LE。
fn encoded(text: &str) -> Vec<u8> {
    let payload: Vec<_> = text
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect();
    let mut bytes = (payload.len() as u32).to_le_bytes().to_vec();
    bytes.extend(payload);
    bytes
}

/// 16 字节对象头：类标识 + 键。
fn entry(class_id: &[u8; 16], key: u32) -> Vec<u8> {
    let mut bytes = class_id.to_vec();
    bytes.extend(key.to_le_bytes());
    bytes
}

/// 16 字节空标记。
fn null_entry() -> Vec<u8> {
    [0_u8; 16].to_vec()
}

/// 按字段序构造 StdFont 体。
fn std_font_body(
    version: u8,
    charset: Option<u16>,
    flags: u8,
    weight: u16,
    size_cy_lo: i32,
    name: &[u8],
) -> Vec<u8> {
    let mut b = vec![version];
    if let Some(charset) = charset {
        b.extend(charset.to_le_bytes());
    }
    b.push(flags);
    b.extend(weight.to_le_bytes());
    b.extend(size_cy_lo.to_le_bytes());
    b.push(name.len() as u8);
    b.extend_from_slice(name);
    b
}

/// StdPicture 体（Save 头形态）：魔数 + 第二字 + 数据长 + 数据。
fn std_picture_body(word2: u16, data: &[u8]) -> Vec<u8> {
    let mut b = 0x746c_u16.to_le_bytes().to_vec();
    b.extend(word2.to_le_bytes());
    b.extend((data.len() as u32).to_le_bytes());
    b.extend_from_slice(data);
    b
}

/// 样本首例 StdFont 体字节原样（/Maps 流 `0xd6c` 键 68 的对象头之后）：
/// 版本 1、charset 134（GB2312）、标志 0、weight 400、CY 120000（12.0pt）、
/// 名字 4 字节 GBK「宋体」。
const SAMPLE_FONT_BODY: &[u8] = &[
    0x01, 0x86, 0x00, 0x00, 0x90, 0x01, 0xc0, 0xd4, 0x01, 0x00, 0x04, 0xcb, 0xce, 0xcc, 0xe5,
];

#[test]
fn reads_font_saved_by_native_oleaut32_and_wine() {
    // 两种实现均写出该 ASCII 字体体，三个布尔属性全部开启。
    let bytes = [
        0x01, 0x86, 0x00, 0x0e, 0x8a, 0x02, 0xcc, 0x55, 0x01, 0x00, 0x06, b'T', b'a', b'h', b'o',
        b'm', b'a', 0x53,
    ];
    let mut reader = Reader::new(&bytes);
    let font = read_std_font_body_10_1(&mut reader, 32).unwrap();
    assert_eq!(font.version, 1);
    assert_eq!(font.charset, Some(134));
    assert_eq!(font.flags, 0x0e);
    assert_eq!(font.weight, 650);
    assert_eq!(font.size_cy_lo, 87500);
    assert_eq!(font.name_bytes, b"Tahoma");
    assert_eq!(reader.read_bytes(1).unwrap(), [0x53]);
}

/// TextSymbol 体（版本 4）合成前缀：字段 1-14 全为空标记/零，字段序按
/// element_body 模块文档。返回（字节，字体槽起点偏移）——字体槽条目、
/// StdFont/StdPicture 体与版本门尾段由各测试自行拼接。
fn text_symbol_prefix_v4() -> (Vec<u8>, usize) {
    let mut b = Vec::new();
    b.extend(4_u16.to_le_bytes()); // 版本 4
    b.extend(null_entry()); // 颜色槽 #1
    b.extend([0_u8; 12]); // 字段 3 int32 ×3
    b.extend([0_u8; 2]); // 字段 4 byte ×2
    b.extend([0_u8; 24]); // 字段 5 double ×3
    b.extend([0_u8; 8]); // 字段 6 int32 ×2
    b.extend(null_entry()); // 颜色槽 #2
    b.extend([0_u8; 16]); // 字段 8 double ×2
    b.extend([0_u8; 8]); // 字段 8 int32 ×2
    b.extend([0_u8; 24]); // 字段 8 double ×3
    b.push(0); // 字段 8 byte
    b.extend(0_f64.to_le_bytes()); // 字段 8 double
    b.extend(0_i32.to_le_bytes()); // 字段 8 int32
    b.extend(0_f64.to_le_bytes()); // 字段 8 double
    b.push(0); // 字段 8 byte
    b.extend([0x11_u8; 16]); // 字段 9 原样 GUID
    b.extend(null_entry()); // 字段 10 ITextPath 槽
    b.extend(null_entry()); // 字段 11 ISymbol 槽 #1
    b.extend(encoded("")); // 字段 12 String
    b.extend(0_f64.to_le_bytes()); // 字段 13 double
    b.extend(0_i32.to_le_bytes()); // 字段 13 int32
    b.extend(0_f64.to_le_bytes()); // 字段 13 double
    b.extend(null_entry()); // 字段 14 ISymbol 槽 #2（空标记可消费）
    let font_slot_offset = b.len();
    (b, font_slot_offset)
}

/// 版本门尾段（版本 4 全读）：short + GUID{D7099B91-…} + short（样本观测
/// 位模式）。
fn tail_segment_v4() -> Vec<u8> {
    let mut b = 0x0000_u16.to_le_bytes().to_vec();
    b.extend([
        0x91, 0x9b, 0x09, 0xd7, 0x98, 0xe2, 0x9e, 0x49, 0x97, 0x57, 0x72, 0xbb, 0x39, 0xe5, 0x5c,
        0xf3,
    ]);
    b.extend(0x0000_u16.to_le_bytes());
    b
}

#[test]
fn std_font_parses_sample_first_instance_body() {
    // 样本首例（键 68 @0xd6c）体字节原样：全部字段命中样本观测值。
    let mut reader = Reader::new(SAMPLE_FONT_BODY);
    let body = read_std_font_body_10_1(&mut reader, 1024).unwrap();
    assert_eq!(body.version, 1);
    assert_eq!(body.charset, Some(134));
    assert_eq!(body.flags, 0);
    assert_eq!(body.weight, 400);
    assert_eq!(body.size_cy_lo, 120_000);
    assert_eq!(body.name_bytes, [0xcb, 0xce, 0xcc, 0xe5]); // GBK「宋体」
    assert_eq!(reader.position(), SAMPLE_FONT_BODY.len());
}

#[test]
fn std_font_parses_marker_font_shape_and_bold_variant() {
    // 样本 CharacterMarker 语境形态（键 1567 @0x173db）：charset 0（ANSI）、
    // 名字 19 字节 ASCII「ESRI Default Marker」、weight 400、CY 120000。
    let name = b"ESRI Default Marker";
    let bytes = std_font_body(1, Some(0), 0, 400, 120_000, name);
    assert_eq!(bytes.len(), 11 + name.len());
    let mut reader = Reader::new(&bytes);
    let body = read_std_font_body_10_1(&mut reader, 1024).unwrap();
    assert_eq!(body.charset, Some(0));
    assert_eq!(body.weight, 400);
    assert_eq!(body.name_bytes, name.as_slice());
    assert_eq!(reader.position(), bytes.len());
    // bold 形态（weight 700、CY 97500 = 9.75pt）照常解析。
    let bold = std_font_body(1, Some(134), 0, 700, 97_500, &[0x61]);
    let mut reader = Reader::new(&bold);
    let body = read_std_font_body_10_1(&mut reader, 1024).unwrap();
    assert_eq!(body.weight, 700);
    assert_eq!(body.size_cy_lo, 97_500);
}

#[test]
fn std_font_version_zero_skips_charset() {
    // 版本 0：原生 TEST AL,AL 条件读——charset 字段不存在。
    let bytes = std_font_body(0, None, 0, 400, 80_000, &[0x41]);
    let mut reader = Reader::new(&bytes);
    let body = read_std_font_body_10_1(&mut reader, 1024).unwrap();
    assert_eq!(body.version, 0);
    assert_eq!(body.charset, None);
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn std_font_rejects_version_and_name_len_like_native() {
    // 版本 >1：原生无符号比较拒绝（E_FAIL）；名字节数 >0x20：同拒绝。
    let v2 = std_font_body(2, Some(0), 0, 400, 0, &[]);
    let mut reader = Reader::new(&v2);
    let error = read_std_font_body_10_1(&mut reader, 1024).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0, "失败不推进游标");
    let long_name = std_font_body(1, Some(0), 0, 400, 0, &[0x41; 0x21]);
    let mut reader = Reader::new(&long_name);
    let error = read_std_font_body_10_1(&mut reader, 1024).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    // 调用者限额同样约束名字长度（≤0x20 但超限额）。
    let limited = std_font_body(1, Some(0), 0, 400, 0, &[0x41; 4]);
    let mut reader = Reader::new(&limited);
    let error = read_std_font_body_10_1(&mut reader, 2).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
}

#[test]
fn std_font_truncation_leaves_cursor_unchanged() {
    // 逐前缀截断：全部报错且游标不动（体无嵌套对象条目，登记表不参与）。
    for end in 0..SAMPLE_FONT_BODY.len() {
        let mut reader = Reader::new(&SAMPLE_FONT_BODY[..end]);
        assert!(read_std_font_body_10_1(&mut reader, 1024).is_err());
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn std_picture_parses_header_and_empty_picture() {
    // 头形态：魔数 0x746c + 第二字 0 + 数据长 0 = 空图。
    let bytes = std_picture_body(0, &[]);
    let mut reader = Reader::new(&bytes);
    let body = read_std_picture_body_10_1(&mut reader, 1024).unwrap();
    assert_eq!(body.word2, 0);
    assert_eq!(body.data_size, 0);
    assert_eq!(body.image_data, Vec::<u8>::new());
    assert_eq!(reader.position(), bytes.len());
    // 非空数据（BITMAP 魔数 'BM' 开头的合成负载）原样保留。
    let data = [0x42, 0x4d, 1, 2, 3, 4];
    let bytes = std_picture_body(0, &data);
    let mut reader = Reader::new(&bytes);
    let body = read_std_picture_body_10_1(&mut reader, 1024).unwrap();
    assert_eq!(body.data_size, 6);
    assert_eq!(body.image_data, data);
    assert_eq!(reader.position(), bytes.len());
    // 第二字非 0 位模式原样保留（Load 不校验）。
    let bytes = std_picture_body(0xa5a5, &[]);
    let mut reader = Reader::new(&bytes);
    let body = read_std_picture_body_10_1(&mut reader, 1024).unwrap();
    assert_eq!(body.word2, 0xa5a5);
}

#[test]
fn std_picture_rejects_non_header_magic_and_oversized_declared_data() {
    // 非 0x746c 魔数（旧格式无头形态）：明确拒绝，游标不动。
    let mut bytes = 0x1234_u16.to_le_bytes().to_vec();
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let error = read_std_picture_body_10_1(&mut reader, 1024).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    // 声明数据长超限额：拒绝（含溢出安全的 u32 声明值）。
    let mut bytes = 0x746c_u16.to_le_bytes().to_vec();
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(u32::MAX.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let error = read_std_picture_body_10_1(&mut reader, 1024).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    // 声明长未超限额但流被截断：UnexpectedEof，游标不动。
    let bytes = std_picture_body(0, &[1, 2, 3, 4]);
    let mut reader = Reader::new(&bytes[..bytes.len() - 2]);
    let error = read_std_picture_body_10_1(&mut reader, 1024).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
}

#[test]
fn text_symbol_font_slot_parses_stdfont_and_tail_segment() {
    // 样本推进形态：IFontDisp 槽 StdFont 新对象（键 68）整体解析，版本门
    // 尾段可达并按位模式保留；键 68 按原生「登记先于持久化」语义登记。
    let (mut body, _) = text_symbol_prefix_v4();
    body.extend(entry(&STDFONT_CLASS_ID, 68));
    body.extend(std_font_body(
        1,
        Some(134),
        0,
        400,
        120_000,
        &[0xcb, 0xce, 0xcc, 0xe5],
    ));
    body.extend(tail_segment_v4());
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&body);
    let parsed = read_text_symbol_body_10_1(&mut reader, 1024, &mut table).unwrap();
    assert!(parsed.stop.is_none(), "TextSymbol 体应完整消费");
    assert_eq!(reader.position(), body.len());
    let FontSlot10_1::StdFont(font_body) = &parsed.font else {
        panic!("字体槽应为 StdFont 体：{:?}", parsed.font);
    };
    assert_eq!(font_body.charset, Some(134));
    assert_eq!(font_body.weight, 400);
    assert_eq!(font_body.size_cy_lo, 120_000);
    assert_eq!(font_body.name_bytes, [0xcb, 0xce, 0xcc, 0xe5]);
    // 版本门尾段（StdFont 体闭合后样本可达）。
    assert_eq!(parsed.tail_short_1, Some(0x0000));
    assert_eq!(
        parsed.tail_path_clsid,
        Some([
            0x91, 0x9b, 0x09, 0xd7, 0x98, 0xe2, 0x9e, 0x49, 0x97, 0x57, 0x72, 0xbb, 0x39, 0xe5,
            0x5c, 0xf3,
        ])
    );
    assert_eq!(parsed.tail_short_2, Some(0x0000));
    assert!(table.contains(68), "StdFont 键应登记");
}

#[test]
fn text_symbol_font_slot_unknown_class_stops_at_slot_start() {
    // 其余字体实现类（含 StdPicture 类标识）：维持停止回退——游标停在该槽
    // 对象头起点、键不登记、字段保持 Null、尾段不可达。
    for class_id in [[0x77_u8; 16], STDPICTURE_CLASS_ID] {
        let (mut body, slot_offset) = text_symbol_prefix_v4();
        body.extend(entry(&class_id, 62));
        body.extend(tail_segment_v4()); // 停止后不可达
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&body);
        let parsed = read_text_symbol_body_10_1(&mut reader, 1024, &mut table).unwrap();
        match &parsed.stop {
            Some(ElementBodyStop10_1::NestedObjectNotClosed {
                slot,
                offset,
                class_id: stopped,
                key,
            }) => {
                assert_eq!(*slot, "IFontDisp 字体槽");
                assert_eq!(*offset, slot_offset);
                assert_eq!(*stopped, class_id);
                assert_eq!(*key, 62);
            }
            other => panic!("应停止于字体槽：{other:?}"),
        }
        assert_eq!(parsed.font, FontSlot10_1::Null);
        assert_eq!(reader.position(), slot_offset, "游标停在该槽对象头起点");
        assert!(!table.contains(62), "未知字体类键不登记");
        assert_eq!(parsed.tail_short_1, None, "尾段不可达");
        assert_eq!(parsed.tail_path_clsid, None, "尾段不可达");
        assert_eq!(parsed.tail_short_2, None, "尾段不可达");
    }
}

#[test]
fn text_symbol_font_slot_back_reference_and_null() {
    // 回引条目（键已登记）与空标记：无体、无停止。
    let (mut body, _) = text_symbol_prefix_v4();
    body.extend(entry(&STDFONT_CLASS_ID, 68));
    body.extend(tail_segment_v4());
    let mut table = RefTable10_1::new();
    table.register(68);
    let mut reader = Reader::new(&body);
    let parsed = read_text_symbol_body_10_1(&mut reader, 1024, &mut table).unwrap();
    assert!(parsed.stop.is_none());
    assert_eq!(reader.position(), body.len());
    assert_eq!(
        parsed.font,
        FontSlot10_1::BackReference {
            class_id: STDFONT_CLASS_ID,
            key: 68,
        }
    );
    // 空标记形态。
    let (mut body, _) = text_symbol_prefix_v4();
    body.extend(null_entry());
    body.extend(tail_segment_v4());
    let mut reader = Reader::new(&body);
    let parsed = read_text_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert!(parsed.stop.is_none());
    assert_eq!(parsed.font, FontSlot10_1::Null);
}

#[test]
fn text_symbol_stdfont_body_error_fails_without_committing() {
    // StdFont 体截断：TextSymbol 整体失败（体自身错误上抛语义），游标与
    // 登记表不提交。
    let (mut body, _) = text_symbol_prefix_v4();
    body.extend(entry(&STDFONT_CLASS_ID, 68));
    // 体缺名字的最后 1 字节（4 字节名只放 3 字节，共 14 字节）。
    body.extend(&std_font_body(1, Some(134), 0, 400, 120_000, &[0xcb, 0xce, 0xcc])[..14]);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&body);
    assert!(read_text_symbol_body_10_1(&mut reader, 1024, &mut table).is_err());
    assert_eq!(reader.position(), 0, "整体失败不推进游标");
    assert!(table.is_empty(), "登记表不提交");
}
