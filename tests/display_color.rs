//! `display_color`（DisplayCore.dll 颜色族）严格读取器的合成输入测试。
//!
//! 原生证据见 `docs/arcmap-10.1-display-tails.md`：HsvColor/GrayColor/
//! HlsColor/RgbColor 共享 Color 核心与 CmykColor 版本分支的字段序来自
//! 10.1 原生 DLL 反编译；仓库参考样本无这四类实例（无字节佐证）。本文件
//! 全部使用合成输入，仅验证读取器自身契约（正常/异常/截断/事务性），不
//! 代表 MXD 格式已验证，未在目标软件中核对。

use mxd2qgz::objects::display_color::{
    CMYK_COLOR_CLASS_ID, ColorBody10_1, ColorSlot10_1, GRAY_COLOR_CLASS_ID, HLS_COLOR_CLASS_ID,
    HSV_COLOR_CLASS_ID, RGB_COLOR_CLASS_ID, read_cmyk_color_body_10_1, read_color_slot_10_1,
    read_core_color_body_10_1, read_rgb_color_body_10_1,
};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

/// 追加对象条目头（16 字节类标识 + 4 字节键）。
fn push_entry(bytes: &mut Vec<u8>, class_id: &[u8; 16], key: u32) {
    bytes.extend_from_slice(class_id);
    bytes.extend(key.to_le_bytes());
}

/// 共享 Color 核心体（版本 1/1）：flag==0 走 3 个 double 分支，flag!=0 走
/// 4 字节分支；尾随两字节取非常量值验证位模式保留。
fn core_color_body(flag: u8) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.push(flag);
    if flag == 0 {
        for value in [1.5_f64, -2.5, 100.0001] {
            bytes.extend(value.to_le_bytes());
        }
    } else {
        bytes.extend(0xA1B2_C3D4_u32.to_le_bytes());
    }
    bytes.push(0x11);
    bytes.push(0xEE);
    bytes
}

/// CmykColor 体：按版本分支组装（版本 ≥2 的 byte/标志/串段与版本 ≥3/4 的
/// CMYK/尾段）。`string` 为 None 时标志字节写 0。
fn cmyk_body(version: u16, with_string: bool, with_core: bool) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    if version >= 2 {
        bytes.push(0x5A); // byte → this-0x10
        if with_string {
            bytes.push(1); // 标志非零：后随 String + short
            let text: Vec<u8> = "x"
                .encode_utf16()
                .chain([0])
                .flat_map(u16::to_le_bytes)
                .collect();
            bytes.extend((text.len() as u32).to_le_bytes());
            bytes.extend(text);
            bytes.extend(7_i16.to_le_bytes());
        } else {
            bytes.push(0);
        }
    }
    if version >= 3 {
        bytes.extend(0x0302_0100_u32.to_le_bytes()); // CMYK 位模式
        if version >= 4 {
            bytes.push(0x21); // byte → this-0x24
            bytes.push(0x43); // 补码字节原样位模式
        }
    } else if with_core {
        bytes.extend(&core_color_body(0)[2..]);
    }
    bytes
}

#[test]
fn color_transparency_obeys_the_signed_version_layout() {
    for (version, expected) in [
        (0xffff_u16, Some(0x11)),
        (2, Some(0x11)),
        (3, None),
        (4, Some(0xbc)),
    ] {
        let mut bytes = cmyk_body(if version == 0xffff { 1 } else { version }, false, true);
        bytes[..2].copy_from_slice(&version.to_le_bytes());
        if version == 0xffff {
            bytes[2..4].copy_from_slice(&0x8000u16.to_le_bytes());
        }
        bytes.push(0xad);
        let mut reader = Reader::new(&bytes);
        let color =
            read_cmyk_color_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 2).unwrap();
        assert_eq!(color.version, version);
        assert_eq!(
            ColorBody10_1::Cmyk(Box::new(color)).transparency(),
            expected
        );
        assert_eq!(reader.read_bytes(1).unwrap(), &[0xad]);
    }
}

#[test]
fn core_color_body_parses_both_branches_and_keeps_bit_patterns() {
    for _class_id in [HSV_COLOR_CLASS_ID, GRAY_COLOR_CLASS_ID, HLS_COLOR_CLASS_ID] {
        for flag in [0_u8, 1] {
            let body = core_color_body(flag);
            let mut reader = Reader::new(&body);
            let result = read_core_color_body_10_1(&mut reader, &mut RefTable10_1::new(), 91, "X")
                .unwrap_or_else(|error| panic!("flag={flag}: 应当成功: {error}"));
            assert_eq!(result.key, 91);
            assert_eq!(result.version_outer, 1);
            assert_eq!(result.version_inner, 1);
            assert_eq!(result.branch_flag, flag);
            assert_eq!(reader.position(), body.len());
            if flag == 0 {
                assert_eq!(result.doubles, Some([1.5, -2.5, 100.0001]));
                assert_eq!(result.packed_rgb_raw, None);
            } else {
                assert_eq!(result.doubles, None);
                assert_eq!(result.packed_rgb_raw, Some(0xA1B2_C3D4));
            }
            assert_eq!(result.trailing_byte, 0x11);
            // 补码字节保留流内原样位模式（装载时的 0xff-x 取补不写入读取器）。
            assert_eq!(result.complement_byte_raw, 0xEE);
        }
    }
}

#[test]
fn core_color_body_rejects_versions_and_truncation_without_advancing() {
    // 外层或内层版本超限时拒绝，游标不动。
    for bad in [[2_u8, 0, 1, 1, 0].as_slice(), &[1, 0, 2, 1, 0][..]] {
        let mut bytes = bad.to_vec();
        bytes.extend(&core_color_body(0)[4..]);
        let mut reader = Reader::new(&bytes);
        let error = read_core_color_body_10_1(&mut reader, &mut RefTable10_1::new(), 1, "X")
            .err()
            .unwrap_or_else(|| panic!("bad={bad:?}: 应当失败"));
        assert_eq!(error.kind(), ErrorKind::InvalidData, "bad={bad:?}");
        assert_eq!(reader.position(), 0, "bad={bad:?}");
    }
    // 逐字节截断：每个前缀都失败、游标不动、重读一致。
    let body = core_color_body(0);
    for end in 0..body.len() {
        let mut reader = Reader::new(&body[..end]);
        let first = read_core_color_body_10_1(&mut reader, &mut RefTable10_1::new(), 1, "X")
            .err()
            .unwrap_or_else(|| panic!("end={end}: 应当失败"));
        assert_eq!(reader.position(), 0, "end={end}");
        let second = read_core_color_body_10_1(&mut reader, &mut RefTable10_1::new(), 1, "X")
            .err()
            .unwrap_or_else(|| panic!("end={end}: 重读应当失败"));
        assert_eq!(first.kind(), second.kind(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
    }
}

#[test]
fn cmyk_body_covers_all_version_branches() {
    // 版本 ≤1：仅共享核心段；版本 ==2：byte/标志段 + 核心段；版本 ==3：无
    // 尾段；版本 ==4：补齐两字节。全部完整消费。
    for version in [0_u16, 1, 2, 3, 4] {
        let with_string = version >= 2;
        let with_core = version <= 2;
        let body = cmyk_body(version, with_string, with_core);
        let mut reader = Reader::new(&body);
        let result = read_cmyk_color_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 77)
            .unwrap_or_else(|error| panic!("version={version}: 应当成功: {error}"));
        assert_eq!(result.key, 77);
        assert_eq!(result.version, version);
        assert_eq!(reader.position(), body.len(), "version={version}");
        if version >= 2 {
            assert_eq!(result.byte_10, Some(0x5A));
            assert_eq!(result.string_flag, Some(1));
            assert_eq!(result.string.as_deref(), Some("x"));
            assert_eq!(result.string_short, Some(7));
        } else {
            assert_eq!(result.byte_10, None);
            assert_eq!(result.string_flag, None);
        }
        if version >= 3 {
            assert_eq!(result.cmyk_raw, Some(0x0302_0100));
        } else {
            assert_eq!(result.cmyk_raw, None);
            assert!(result.core.is_some(), "version={version}: 核心段应存在");
        }
        if version >= 4 {
            assert_eq!(result.byte_24, Some(0x21));
            assert_eq!(result.complement_byte_raw, Some(0x43));
        } else {
            assert_eq!(result.byte_24, None);
            assert_eq!(result.complement_byte_raw, None);
        }
        if with_core {
            let core = result.core.unwrap();
            assert_eq!(core.version_outer, version);
            assert_eq!(core.version_inner, 1);
            assert_eq!(core.branch_flag, 0);
        } else {
            assert_eq!(result.core, None);
        }
    }
    // 标志为零时 String/short 不读取。
    let mut bytes = 4_u16.to_le_bytes().to_vec();
    bytes.extend(&cmyk_body(4, false, false)[2..]);
    let mut reader = Reader::new(&bytes);
    let result = read_cmyk_color_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1).unwrap();
    assert_eq!(result.string_flag, Some(0));
    assert_eq!(result.string, None);
    assert_eq!(result.string_short, None);
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn cmyk_v2_reads_packed_core_then_next_object() {
    let mut bytes = Vec::new();
    push_entry(&mut bytes, &CMYK_COLOR_CLASS_ID, 99);
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend([1, 0]); // 无字符串。
    bytes.extend(&core_color_body(1)[2..]); // 仅写核心自己的版本。
    let end = bytes.len();
    bytes.extend([0; 16]); // 下一个 IColor 槽为空。
    let mut refs = RefTable10_1::new();
    for cut in 20..end {
        let mut reader = Reader::new(&bytes[..cut]);
        assert!(read_color_slot_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
    let mut reader = Reader::new(&bytes);
    let ColorSlot10_1::Color {
        body: ColorBody10_1::Cmyk(color),
        ..
    } = read_color_slot_10_1(&mut reader, 1024, &mut refs).unwrap()
    else {
        panic!("应读取 CmykColor");
    };
    assert_eq!(color.core.unwrap().packed_rgb_raw, Some(0xa1b2_c3d4));
    assert_eq!(reader.position(), end);
    assert!(refs.contains(99));
    assert_eq!(
        read_color_slot_10_1(&mut reader, 1024, &mut refs).unwrap(),
        ColorSlot10_1::Null
    );
}

#[test]
fn cmyk_body_rejects_versions_limits_and_truncation() {
    // 版本 5 超过上限，失败时游标不动。
    let mut bytes = 5_u16.to_le_bytes().to_vec();
    bytes.extend(&cmyk_body(4, false, false)[2..]);
    let mut reader = Reader::new(&bytes);
    let error =
        read_cmyk_color_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    // 字符串声明长度超过限额被拒（含合成边界 u32::MAX）。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.push(0);
    bytes.push(1);
    bytes.extend(u32::MAX.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let error =
        read_cmyk_color_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    // 逐字节截断扫描：版本 4 含串分支的全输入。
    let body = cmyk_body(4, true, false);
    for end in 0..body.len() {
        let mut reader = Reader::new(&body[..end]);
        let first = read_cmyk_color_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1);
        assert!(first.is_err(), "end={end}: 应当失败");
        assert_eq!(reader.position(), 0, "end={end}");
        let second = read_cmyk_color_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1);
        assert_eq!(
            first.unwrap_err().kind(),
            second.unwrap_err().kind(),
            "end={end}"
        );
        assert_eq!(reader.position(), 0, "end={end}");
    }
}

#[test]
fn color_slot_dispatches_all_five_closed_classes() {
    // 新对象 + 各类体：全部解析为 Color 变体，键登记，游标完整消费。
    let cases: [(&[u8; 16], u8); 5] = [
        (&RGB_COLOR_CLASS_ID, 0),
        (&HSV_COLOR_CLASS_ID, 1),
        (&GRAY_COLOR_CLASS_ID, 0),
        (&HLS_COLOR_CLASS_ID, 1),
        (&CMYK_COLOR_CLASS_ID, 0),
    ];
    for (class_id, flag) in cases {
        let mut bytes = Vec::new();
        push_entry(&mut bytes, class_id, 300);
        if *class_id == CMYK_COLOR_CLASS_ID {
            bytes.extend(&cmyk_body(4, false, false));
        } else {
            bytes.extend(&core_color_body(flag));
        }
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let slot = read_color_slot_10_1(&mut reader, 1024, &mut table).unwrap();
        match slot {
            ColorSlot10_1::Color { class_id: id, body } => {
                assert_eq!(id, *class_id);
                match body {
                    ColorBody10_1::Rgb(body) => assert_eq!(body.key, 300),
                    ColorBody10_1::Core { class_id: id, body } => {
                        assert_eq!(id, *class_id);
                        assert_eq!(body.key, 300);
                    }
                    ColorBody10_1::Cmyk(body) => assert_eq!(body.key, 300),
                }
            }
            other => panic!("期望 Color 变体，得到 {other:?}"),
        }
        assert_eq!(reader.position(), bytes.len());
        assert!(table.contains(300));
    }
    // 空标记与回引原样返回。
    let bytes = [0_u8; 16].to_vec();
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_color_slot_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap(),
        ColorSlot10_1::Null
    );
    assert_eq!(reader.position(), 16);
    let mut bytes = Vec::new();
    push_entry(&mut bytes, &RGB_COLOR_CLASS_ID, 400);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    table.register(400);
    match read_color_slot_10_1(&mut reader, 1024, &mut table).unwrap() {
        ColorSlot10_1::BackReference { class_id, key } => {
            assert_eq!(class_id, RGB_COLOR_CLASS_ID);
            assert_eq!(key, 400);
        }
        other => panic!("期望回引，得到 {other:?}"),
    }
    assert_eq!(reader.position(), 20);
}

#[test]
fn unknown_color_class_rolls_back_cursor_and_key() {
    // 未闭合颜色类（合成 GUID）：读取器回退到槽位起点，键不保留，登记表
    // 预存键不受影响。
    let unknown = [0xab_u8; 16];
    let mut bytes = Vec::new();
    push_entry(&mut bytes, &unknown, 501);
    bytes.extend([0x77; 8]); // 未知体的假想内容，必须保持未消费
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    table.register(7);
    match read_color_slot_10_1(&mut reader, 1024, &mut table).unwrap() {
        ColorSlot10_1::Unsupported { class_id, key } => {
            assert_eq!(class_id, unknown);
            assert_eq!(key, 501);
        }
        other => panic!("期望 Unsupported，得到 {other:?}"),
    }
    assert_eq!(reader.position(), 0, "游标回退到槽位起点");
    assert_eq!(table.len(), 1, "未知键不保留");
    assert!(table.contains(7));
    // 回退后槽位字节可重读：同一输入换 Null 头验证对位不受影响。
    let bytes = [0_u8; 16].to_vec();
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_color_slot_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap(),
        ColorSlot10_1::Null
    );
}

#[test]
fn rgb_slot_matches_direct_rgb_reader() {
    // 槽位分派与 RgbColor 直读的结果一致（同构佐证）。
    let mut bytes = Vec::new();
    push_entry(&mut bytes, &RGB_COLOR_CLASS_ID, 37);
    bytes.extend(&core_color_body(0));
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let slot = read_color_slot_10_1(&mut reader, 1024, &mut table).unwrap();
    let mut direct_reader = Reader::new(&bytes[20..]);
    let direct =
        read_rgb_color_body_10_1(&mut direct_reader, &mut RefTable10_1::new(), 37).unwrap();
    match slot {
        ColorSlot10_1::Color {
            class_id,
            body: ColorBody10_1::Rgb(body),
        } => {
            assert_eq!(class_id, RGB_COLOR_CLASS_ID);
            assert_eq!(*body, direct);
        }
        other => panic!("期望 Rgb 体，得到 {other:?}"),
    }
}

#[test]
fn color_slot_failures_are_transactional() {
    // 槽内体被截断：错误、游标回入口、登记表无新键。
    let mut bytes = Vec::new();
    push_entry(&mut bytes, &HSV_COLOR_CLASS_ID, 600);
    let body = core_color_body(0);
    bytes.extend(&body[..body.len() - 3]);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let error = read_color_slot_10_1(&mut reader, 1024, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty(), "截断失败不登记键");
    // 重读一致。
    let error = read_color_slot_10_1(&mut reader, 1024, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());
}
