//! `object_refs` 对象条目分类的合成与仓库参考样本回归测试。

use mxd2qgz::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use mxd2qgz::reader::Reader;
use std::io::Read;

fn entry_bytes(class_id: [u8; 16], key: u32) -> Vec<u8> {
    let mut bytes = class_id.to_vec();
    bytes.extend(key.to_le_bytes());
    bytes
}

/// 仓库参考样本 samples/测试底图.mxd /Maps 流内偏移的辅助观察回归对照：
/// 首对象 @0x04 键 29（Map）、回引用条目 @0x2d2c（FeatureLayer 键 32，
/// 仅 20 字节头无体）。
/// 偏移与键来自原生研究记录（docs/arcmap-10.1-object-body.md 第五轮、
/// docs/arcmap-10.1-map-object.md 第四轮），样本观察不是目标软件执行验证，
/// 不能据此声称格式规则已验证。
#[test]
fn sample_basemap_object_entries_match_previous_auxiliary_observation() {
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut maps = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut maps)
        .unwrap();

    // 0x00..0x04 是 4 字节数量；0x04 起是首对象条目。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&maps[0x04..]);
    let entry = read_object_entry_10_1(&mut reader, &mut table).unwrap();
    let ObjectEntry10_1::NewObject { class_id, key } = entry else {
        panic!("首对象条目应为新对象，实际 {entry:?}");
    };
    assert_eq!(key, 29, "样本首对象键序起点");
    assert_ne!(class_id, [0; 16], "首对象类标识非全零");
    assert_eq!(reader.position(), 20);

    // 图层列表条目 2 @0x2d2c：FeatureLayer CLSID + 键 32，A=20=仅头无体的
    // 回引用（命中键 32 既有对象；docs/arcmap-10.1-map-object.md 第四轮 0x2d22 表）。
    // 用键 32 预置登记表模拟「此前已在会话中见过」；同一字节在空表下会
    // 被分类为新对象——流内无法局部区分回引与新对象，这正是原生语义。
    let back_ref_bytes: Vec<u8> = maps[0x2d2c..0x2d2c + 20].to_vec();
    let class_id: [u8; 16] = back_ref_bytes[..16].try_into().unwrap();
    let mut seeded = RefTable10_1::new();
    assert!(!seeded.contains(32));
    seeded.register(32);
    let mut reader = Reader::new(&back_ref_bytes);
    let entry = read_object_entry_10_1(&mut reader, &mut seeded).unwrap();
    assert_eq!(entry, ObjectEntry10_1::BackReference { class_id, key: 32 });
    assert_eq!(reader.position(), 20);
    // 空表下同一回引字节按新对象分类：登记状态决定判定，流内无专用标记。
    let mut reader = Reader::new(&back_ref_bytes);
    assert!(matches!(
        read_object_entry_10_1(&mut reader, &mut RefTable10_1::new()).unwrap(),
        ObjectEntry10_1::NewObject { key: 32, .. }
    ));
}

/// 合成循环引用：条目分类状态机对 A→B→A 循环给出回引判定（同模块单元
/// 测试已覆盖；此处从公共 API 再走一遍与截断健壮性互补的路径）。
#[test]
fn cyclic_synthetic_entries_classify_by_seen_key_state() {
    let a = entry_bytes([0x11; 16], 1);
    let b = entry_bytes([0x22; 16], 2);
    let mut stream = a.clone();
    stream.extend_from_slice(&b);
    stream.extend_from_slice(&a); // B 的体内对 A 的回引

    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&stream);
    assert!(matches!(
        read_object_entry_10_1(&mut reader, &mut table).unwrap(),
        ObjectEntry10_1::NewObject { key: 1, .. }
    ));
    assert!(matches!(
        read_object_entry_10_1(&mut reader, &mut table).unwrap(),
        ObjectEntry10_1::NewObject { key: 2, .. }
    ));
    assert_eq!(
        read_object_entry_10_1(&mut reader, &mut table).unwrap(),
        ObjectEntry10_1::BackReference {
            class_id: [0x11; 16],
            key: 1
        }
    );
    assert!(reader.position() == 60 && table.len() == 2);
}
