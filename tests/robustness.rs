//! 跨读取器的异常输入健壮性测试（通过 `mxd2qgz::` 公共 API 访问）。
//!
//! 这些测试证明的是各严格读取器自身的健壮性共性：
//! - 对每个读取器的合法输入，任意截断前缀（含空输入）都返回明确错误且不 panic；
//! - 失败后游标不推进，且同一失败输入在同一 Reader 上重读得到相同错误类别
//!   （clone-trial 事务性的可观察行为）；
//! - `u32::MAX`/`usize::MAX` 级别的声明长度、码元数或限额参数不引发算术溢出、
//!   panic 或巨大分配，而是在分配前被明确拒绝；
//! - 未知类标识与键按 `object_stream` 契约原样保留；VariantStream 的整个
//!   `u16` VARTYPE 空间都有显式分支；
//! - 当前读取器不递归、不自我放大，顺序读取在输入耗尽时有界完成。
//!
//! 这些测试不证明 MXD 格式规则、字段归属、版本语义、对象边界或引用关系
//! 已经验证；它们只刻画合成输入上的实现性质，格式结论仍以原生研究记录
//! 为准。底层 `reader::Reader` 原语的同等性质已由 `tests/reader.rs` 与
//! `src/reader.rs` 单元测试覆盖，此处仅在超大长度一节触及公共原语，不重复。
//! 循环引用依赖尚未实现的对象体与引用表编码，待对象引用编码确定后覆盖
//! （TODO「自动验证」节）；当前仅在注释中说明并在顺序读取测试中证明
//! 现有读取器结构上不递归。

use mxd2qgz::map_object::read_map_prefix_10_1;
use mxd2qgz::maps_stream::{read_root_prefix_10_1, read_root_prefix_10_8};
use mxd2qgz::native_string::{read_string_10_1, read_string_10_8};
use mxd2qgz::object_stream::{ObjectHeader, read_header_10_1, read_header_10_8};
use mxd2qgz::reader::Reader;
use mxd2qgz::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
use mxd2qgz::variant_stream::{VariantValue10_8, read_value_10_8};
use mxd2qgz::version_stream::{read_arcmap_header_10_1, read_layout_10_1, read_layout_10_8};
use std::io::{self, ErrorKind};

/// 构造 AfCore String::Write 布局的合法输入：u32 字节长度前缀 + 含终止零的 UTF-16LE。
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

/// 构造 VariantStream VT_BSTR 型载荷：u32 码元数 + UTF-16LE + 两个终止零字节。
fn bstr_payload_bytes(value: &str) -> Vec<u8> {
    let units: Vec<u16> = value.encode_utf16().collect();
    let mut bytes = Vec::with_capacity(4 + units.len() * 2 + 2);
    bytes.extend_from_slice(&(units.len() as u32).to_le_bytes());
    for unit in units {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes.extend_from_slice(&[0, 0]);
    bytes
}

/// 构造 VariantStream 值：u16 VARTYPE + 载荷。
fn value_bytes(value_type: u16, payload: &[u8]) -> Vec<u8> {
    value_type
        .to_le_bytes()
        .into_iter()
        .chain(payload.iter().copied())
        .collect()
}

/// 参与统一截断扫描的读取器；每个变体对应一个公共读取函数入口。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReaderKind {
    NativeString10_1,
    NativeString10_8,
    Layout10_1,
    ArcMapHeader10_1,
    Layout10_8,
    MapsRoot10_8,
    MapsRoot10_1,
    MapPrefix10_1,
    ObjectHeader10_1,
    ObjectHeader10_8,
    VariantString10_8,
    VariantFixed10_8,
}

fn all_kinds() -> [ReaderKind; 12] {
    [
        ReaderKind::NativeString10_1,
        ReaderKind::NativeString10_8,
        ReaderKind::Layout10_1,
        ReaderKind::ArcMapHeader10_1,
        ReaderKind::Layout10_8,
        ReaderKind::MapsRoot10_8,
        ReaderKind::MapsRoot10_1,
        ReaderKind::MapPrefix10_1,
        ReaderKind::ObjectHeader10_1,
        ReaderKind::ObjectHeader10_8,
        ReaderKind::VariantString10_8,
        ReaderKind::VariantFixed10_8,
    ]
}

/// 每个读取器一个确定合法的完整输入；完整输入必须被接受并完整消费。
fn sample(kind: ReaderKind) -> Vec<u8> {
    match kind {
        ReaderKind::NativeString10_1 | ReaderKind::NativeString10_8 => encoded("地图😀"),
        ReaderKind::Layout10_1 | ReaderKind::Layout10_8 => {
            let mut bytes = encoded("10.1");
            bytes.extend(0x13_i16.to_le_bytes());
            bytes
        }
        ReaderKind::ArcMapHeader10_1 => {
            let mut bytes = encoded("10.1");
            bytes.extend(0x13_i16.to_le_bytes());
            bytes.extend(7_i32.to_le_bytes());
            bytes
        }
        ReaderKind::MapsRoot10_8 => {
            let mut bytes = 1_i32.to_le_bytes().to_vec();
            bytes.extend([0xa5_u8; 16]);
            bytes.extend(7_u32.to_le_bytes());
            bytes
        }
        ReaderKind::MapsRoot10_1 => {
            // 10.1 读侧在对象循环结束后还须读取 4 字节尾部值，合法样本必须
            // 包含它才能被完整消费。
            let mut bytes = 1_i32.to_le_bytes().to_vec();
            bytes.extend([0xa5_u8; 16]);
            bytes.extend(7_u32.to_le_bytes());
            bytes.extend(1_u32.to_le_bytes());
            bytes
        }
        ReaderKind::MapPrefix10_1 => {
            // 版本 51 + 两个名称 + 嵌套 PCS 头 + 保存版本 0 的最小 PCS 体
            // （仅版本 + WKT 分支），全部被读取器完整消费。
            let mut bytes = 51_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("新建数据框"));
            bytes.extend(encoded(""));
            bytes.extend(PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
            bytes.extend(0x1e_u32.to_le_bytes());
            let wkt = b"PROJCS[\"x\"]";
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend((wkt.len() as u32).to_le_bytes());
            bytes.extend(wkt);
            bytes
        }
        ReaderKind::ObjectHeader10_1 | ReaderKind::ObjectHeader10_8 => {
            let mut bytes = vec![0xa5_u8; 16];
            bytes.extend(7_u32.to_le_bytes());
            bytes
        }
        ReaderKind::VariantString10_8 => value_bytes(8, &bstr_payload_bytes("地图")),
        ReaderKind::VariantFixed10_8 => value_bytes(5, &[0; 8]),
    }
}

/// 以统一限额调用目标读取器，只保留成败与游标行为，供跨读取器断言复用。
fn try_read(kind: ReaderKind, reader: &mut Reader<'_>) -> io::Result<()> {
    match kind {
        ReaderKind::NativeString10_1 => read_string_10_1(reader, 1024).map(|_| ()),
        ReaderKind::NativeString10_8 => read_string_10_8(reader, 1024).map(|_| ()),
        ReaderKind::Layout10_1 => read_layout_10_1(reader, 1024).map(|_| ()),
        ReaderKind::ArcMapHeader10_1 => read_arcmap_header_10_1(reader, 1024).map(|_| ()),
        ReaderKind::Layout10_8 => read_layout_10_8(reader, 1024).map(|_| ()),
        ReaderKind::MapsRoot10_8 => read_root_prefix_10_8(reader, 16).map(|_| ()),
        ReaderKind::MapsRoot10_1 => read_root_prefix_10_1(reader, 16).map(|_| ()),
        ReaderKind::MapPrefix10_1 => read_map_prefix_10_1(reader, 1024).map(|_| ()),
        ReaderKind::ObjectHeader10_1 => read_header_10_1(reader).map(|_| ()),
        ReaderKind::ObjectHeader10_8 => read_header_10_8(reader).map(|_| ()),
        ReaderKind::VariantString10_8 | ReaderKind::VariantFixed10_8 => {
            read_value_10_8(reader, u32::MAX).map(|_| ())
        }
    }
}

#[test]
fn every_reader_rejects_every_truncated_prefix_without_panicking_or_advancing() {
    for kind in all_kinds() {
        let input = sample(kind);
        // 完整输入必须成功且被完整消费，证明该样本对目标读取器确实合法。
        let mut reader = Reader::new(&input);
        try_read(kind, &mut reader)
            .unwrap_or_else(|error| panic!("{kind:?}: 合法样本被拒绝: {error}"));
        assert_eq!(reader.position(), input.len(), "{kind:?}");
        // 0..n 的每个截断前缀：必须失败、不 panic、游标不推进，且重读结果一致。
        for end in 0..input.len() {
            let mut reader = Reader::new(&input[..end]);
            let first = try_read(kind, &mut reader)
                .err()
                .unwrap_or_else(|| panic!("{kind:?}: 截断前缀 end={end} 未返回错误"));
            assert_eq!(reader.position(), 0, "{kind:?}: end={end}");
            let second = try_read(kind, &mut reader)
                .err()
                .unwrap_or_else(|| panic!("{kind:?}: 重读 end={end} 未返回错误"));
            assert_eq!(first.kind(), second.kind(), "{kind:?}: end={end}");
            assert_eq!(reader.position(), 0, "{kind:?}: end={end}");
        }
    }
}

#[test]
fn non_eof_failures_leave_the_cursor_local_and_repeatable() {
    // 对象头读取没有非 EOF 失败类别，其事务性已由截断扫描覆盖。
    let surrogate = {
        // 声明 4 字节，内容是孤立高代理项。
        let mut bytes = 4_u32.to_le_bytes().to_vec();
        bytes.extend([0x00, 0xd8, 0x00, 0x00]);
        bytes
    };
    let short_out_of_range = {
        let mut bytes = encoded("x");
        bytes.extend(20_i16.to_le_bytes());
        bytes
    };
    let cases: Vec<(ReaderKind, Vec<u8>, ErrorKind, &str)> = vec![
        (
            ReaderKind::NativeString10_8,
            3_u32.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "奇数字节长度",
        ),
        (
            ReaderKind::NativeString10_1,
            u32::MAX.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "非法声明长度",
        ),
        (
            ReaderKind::Layout10_1,
            surrogate.clone(),
            ErrorKind::InvalidData,
            "孤立高代理项",
        ),
        (
            ReaderKind::Layout10_8,
            surrogate,
            ErrorKind::InvalidData,
            "孤立高代理项",
        ),
        (
            ReaderKind::ArcMapHeader10_1,
            short_out_of_range,
            ErrorKind::InvalidData,
            "short 超出头部支持范围",
        ),
        (
            ReaderKind::MapsRoot10_8,
            (-1_i32).to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "负数量",
        ),
        (
            ReaderKind::MapsRoot10_8,
            17_i32.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "数量超过限额",
        ),
        (
            ReaderKind::MapsRoot10_1,
            (-1_i32).to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "10.1 负数量",
        ),
        (
            ReaderKind::MapsRoot10_1,
            17_i32.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "10.1 数量超过限额",
        ),
        (
            ReaderKind::VariantString10_8,
            value_bytes(0xffff, &[1, 2, 3, 4]),
            ErrorKind::InvalidData,
            "未知 VARTYPE",
        ),
        (
            ReaderKind::VariantString10_8,
            value_bytes(9, &[1, 2, 3, 4]),
            ErrorKind::Unsupported,
            "Variant 对象分支",
        ),
        (
            ReaderKind::VariantFixed10_8,
            value_bytes(13, &[1, 2, 3, 4]),
            ErrorKind::Unsupported,
            "Variant 对象分支",
        ),
    ];
    for (kind, bytes, expected, label) in cases {
        let mut reader = Reader::new(&bytes);
        let first = try_read(kind, &mut reader)
            .err()
            .unwrap_or_else(|| panic!("{label}: 该输入应当失败"));
        assert_eq!(first.kind(), expected, "{label}");
        assert_eq!(reader.position(), 0, "{label}: 失败后游标必须回到入口");
        // 失败可重复：同一 Reader 再次读取得到相同错误类别，游标仍在入口。
        let second = try_read(kind, &mut reader)
            .err()
            .unwrap_or_else(|| panic!("{label}: 重读应当失败"));
        assert_eq!(second.kind(), expected, "{label}: 重读结果不一致");
        assert_eq!(reader.position(), 0, "{label}: 重读后游标仍在入口");
    }
}

#[test]
fn huge_declared_lengths_and_limits_fail_explicitly_without_overflow() {
    // AfCore 字符串与 Version 布局：u32::MAX-1 是最大偶数声明长度，在
    // usize::MAX 限额下于读取阶段明确短读；u32::MAX 是奇数，在校验阶段被拒。
    let reads: [&dyn Fn(&mut Reader<'_>, usize) -> io::Result<()>; 5] = [
        &|r: &mut Reader<'_>, m: usize| read_string_10_1(r, m).map(|_| ()),
        &|r: &mut Reader<'_>, m: usize| read_string_10_8(r, m).map(|_| ()),
        &|r: &mut Reader<'_>, m: usize| read_layout_10_1(r, m).map(|_| ()),
        &|r: &mut Reader<'_>, m: usize| read_arcmap_header_10_1(r, m).map(|_| ()),
        &|r: &mut Reader<'_>, m: usize| read_layout_10_8(r, m).map(|_| ()),
    ];
    for read in reads {
        for (declared, expected) in [
            (u32::MAX - 1, ErrorKind::UnexpectedEof),
            (u32::MAX, ErrorKind::InvalidData),
        ] {
            let bytes = declared.to_le_bytes();
            let mut reader = Reader::new(&bytes);
            let error = read(&mut reader, usize::MAX)
                .err()
                .unwrap_or_else(|| panic!("declared={declared:#x}: 应当失败"));
            assert_eq!(error.kind(), expected, "declared={declared:#x}");
            assert_eq!(reader.position(), 0, "declared={declared:#x}");
        }
    }

    // VariantStream：n*2+2 在 u32 上的溢出边界。0x7fff_fffe 仍可表达并在
    // 读取阶段短读；更大的码元数在任何分配前被拒。限额取 u32::MAX 以隔离限额检查。
    for (units, expected) in [
        (0x7fff_fffe_u32, ErrorKind::UnexpectedEof),
        (0x7fff_ffff, ErrorKind::InvalidData),
        (0x8000_0000, ErrorKind::InvalidData),
        (u32::MAX, ErrorKind::InvalidData),
    ] {
        let bytes = value_bytes(8, &units.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let error = read_value_10_8(&mut reader, u32::MAX)
            .err()
            .unwrap_or_else(|| panic!("units={units:#x}: 应当失败"));
        assert_eq!(error.kind(), expected, "units={units:#x}");
        assert_eq!(reader.position(), 0, "units={units:#x}");
    }

    // /Maps：i32::MAX 数量在限额之内也不做任何分配，进入对象循环时立即短读。
    // 10.1 与 10.8 数量校验边界一致；10.1 在对象循环之后还须读取 4 字节尾部值。
    let count_only = i32::MAX.to_le_bytes();
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_8(&mut reader, i32::MAX as u32).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
    // 超限额数量被拒；限额参数 u32::MAX 无法由 i32 表示，在读取任何对象前被拒。
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_8(&mut reader, 16).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_8(&mut reader, u32::MAX).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_1(&mut reader, i32::MAX as u32).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_1(&mut reader, 16).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_1(&mut reader, u32::MAX).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert_eq!(reader.position(), 0);

    // 底层 UTF-16 读取：usize::MAX（奇数）与 usize::MAX-1（偶数）分别被
    // 奇偶校验和边界检查拒绝，游标不变。
    let mut reader = Reader::new(&[0, 0]);
    for (length, expected) in [
        (usize::MAX, ErrorKind::InvalidData),
        (usize::MAX - 1, ErrorKind::UnexpectedEof),
    ] {
        let error = reader.read_utf16_le(length).unwrap_err();
        assert_eq!(error.kind(), expected, "length=0x{length:x}");
        assert_eq!(reader.position(), 0, "length=0x{length:x}");
    }
}

#[test]
fn unknown_class_ids_and_keys_are_preserved_verbatim_without_interpretation() {
    // 合成的未知 GUID；读取器契约是不校验注册，原样保留磁盘字节顺序与键的
    // 全部位模式。全零 16 字节才是空头；其他任何内容都按对象头处理。
    let class_id = [
        0xd8, 0x1c, 0x6c, 0x2e, 0x31, 0xae, 0x44, 0xd8, 0x8e, 0x7a, 0x11, 0x0b, 0x2f, 0x93, 0x7d,
        0xe4,
    ];
    for read_header in [read_header_10_1, read_header_10_8] {
        for key in [0_u32, 1, 0x7fff_ffff, 0x8000_0000, 0xdead_beef, u32::MAX] {
            let mut bytes = class_id.to_vec();
            bytes.extend(key.to_le_bytes());
            bytes.extend(b"opaque body");
            let mut reader = Reader::new(&bytes);
            let header = read_header(&mut reader)
                .unwrap_or_else(|error| panic!("key={key:#x}: 未知类标识不应被拒绝: {error}"));
            assert_eq!(header, ObjectHeader::Object { class_id, key });
            assert_eq!(reader.position(), 20);
            assert_eq!(reader.read_bytes(11).unwrap(), b"opaque body");
        }
    }
    // Null 判定只取决于 16 字节是否全零：首字节或末字节非零都按对象处理。
    for index in [0usize, 15] {
        let mut id = [0_u8; 16];
        id[index] = 1;
        let mut bytes = id.to_vec();
        bytes.extend(0_u32.to_le_bytes());
        for read_header in [read_header_10_1, read_header_10_8] {
            let mut reader = Reader::new(&bytes);
            assert_eq!(
                read_header(&mut reader).unwrap(),
                ObjectHeader::Object {
                    class_id: id,
                    key: 0
                }
            );
        }
    }
    // 键被截断时失败且游标不推进（与截断扫描结论一致）。
    let mut reader = Reader::new(&class_id);
    let error = read_header_10_1(&mut reader).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
}

#[test]
fn every_vartype_value_has_an_explicit_branch_without_panicking() {
    // 已知分支之外的整个 u16 空间都必须显式拒绝；9/13 是显式不支持的对象分支。
    // 载荷取 8 个非零字节，使固定宽度分支完整、字符串分支因声明码元数
    // 0xa5a5a5a5 超过限额而被拒；字符串分支的完整行为由模块测试覆盖。
    let payload = [0xa5_u8; 8];
    for value_type in 0..=u16::MAX {
        let bytes = value_bytes(value_type, &payload);
        let mut reader = Reader::new(&bytes);
        let result = read_value_10_8(&mut reader, 4);
        match value_type {
            0 => {
                assert_eq!(result.unwrap(), VariantValue10_8::Empty, "VT={value_type}");
                assert_eq!(reader.position(), 2);
            }
            1 => {
                assert_eq!(result.unwrap(), VariantValue10_8::Null, "VT={value_type}");
                assert_eq!(reader.position(), 2);
            }
            2 | 11 => {
                assert_eq!(
                    result.unwrap(),
                    VariantValue10_8::Raw2([0xa5, 0xa5]),
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 4);
            }
            3 | 4 => {
                assert_eq!(
                    result.unwrap(),
                    VariantValue10_8::Raw4([0xa5; 4]),
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 6);
            }
            5 | 7 => {
                assert_eq!(
                    result.unwrap(),
                    VariantValue10_8::Raw8(payload),
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 10);
            }
            17 => {
                assert_eq!(
                    result.unwrap(),
                    VariantValue10_8::Raw1([0xa5]),
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 3);
            }
            8 => {
                assert_eq!(
                    result.unwrap_err().kind(),
                    ErrorKind::InvalidData,
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 0);
            }
            9 | 13 => {
                assert_eq!(
                    result.unwrap_err().kind(),
                    ErrorKind::Unsupported,
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 0);
            }
            _ => {
                assert_eq!(
                    result.unwrap_err().kind(),
                    ErrorKind::InvalidData,
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 0);
            }
        }
    }
}

#[test]
fn current_readers_complete_boundedly_without_recursive_self_application() {
    // 循环引用（对象经引用表互相引用）需要对象体与引用表编码才能表达；
    // 当前读取器不解析对象体，也不维护对象表，自引用结构在其输入布局中
    // 不可表达。待对象引用编码确定后补充缺失引用、重复标识及循环引用的
    // 覆盖（TODO「自动验证」节）。此处断言现有读取器不递归、不自我放大：
    // 大量连续对象在输入耗尽时以有界步骤完成。
    let mut header = vec![0xa5_u8; 16];
    header.extend(7_u32.to_le_bytes());
    let bytes = header.repeat(512);
    let mut reader = Reader::new(&bytes);
    for index in 0..512 {
        let read_header = if index % 2 == 0 {
            read_header_10_8
        } else {
            read_header_10_1
        };
        assert_eq!(
            read_header(&mut reader).unwrap(),
            ObjectHeader::Object {
                class_id: [0xa5; 16],
                key: 7
            }
        );
    }
    assert_eq!(reader.remaining(), 0);

    // Variant 固定宽度值顺序读取同样有界。
    let value = value_bytes(4, &[1, 2, 3, 4]);
    let bytes = value.repeat(1024);
    let mut reader = Reader::new(&bytes);
    for _ in 0..1024 {
        assert_eq!(
            read_value_10_8(&mut reader, 4).unwrap(),
            VariantValue10_8::Raw4([1, 2, 3, 4])
        );
    }
    assert_eq!(reader.remaining(), 0);

    // 长字符串一次性读取，不因长度递归。
    let text = "地图".repeat(500);
    let bytes = encoded(&text);
    let mut reader = Reader::new(&bytes);
    assert_eq!(read_string_10_8(&mut reader, bytes.len()).unwrap(), text);
    assert_eq!(reader.remaining(), 0);
}
