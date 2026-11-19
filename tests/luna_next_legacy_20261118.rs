//! 本轮旧档目标的独立宿主边界核验。
//!
//! 合成的 Hyperlink v0、GroupLayer v10 和 Envelope v0 字段边界分别由
//! `hyperlink.rs`、`terra_round9_old.rs` 与 `envelope_v0_round2.rs` 覆盖；
//! 这里额外从参考 MXD 的 `/Maps` 根顺序检查已到达的 FeatureLayer 槽，避免
//! 把 CLSID 搜索结果当作 Hyperlink 实例。该样本没有 Hyperlink 实例，不能
//! 代替旧档 v0 样本。

use mxd2qgz::{
    layers::{
        feature_layer::FEATURE_LAYER_CLASS_ID,
        group_layer::GROUP_LAYER_CLASS_ID,
        layer_list::{LayerBodyPrefix10_1, LayerEntryObject},
    },
    maps::map_object::{MAP_CLASS_ID, read_map_suffix_10_1},
    storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1},
    stream::reader::Reader,
};
use std::io::Read;

#[test]
fn reference_map_root_reaches_featurelayer_hyperlink_counters() {
    let mut doc = cfb::open("samples/测试底图.mxd").unwrap();
    let mut maps = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut maps)
        .unwrap();

    let mut reader = Reader::new(&maps);
    let mut refs = RefTable10_1::new();
    assert_eq!(reader.read_u32_le().unwrap(), 1, "根 Map 计数");
    assert!(matches!(
        read_object_entry_10_1(&mut reader, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
    let map = read_map_suffix_10_1(&mut reader, 16_000_000, &mut refs).unwrap();

    let mut reached = Vec::new();
    for entry in &map.layer_list.entries {
        let Some(LayerBodyPrefix10_1::FeatureLayer(layer)) = &entry.parsed else {
            continue;
        };
        let Some(successor) = &layer.successor else {
            continue;
        };
        if let mxd2qgz::layers::layer_list::LayerEntryObject::Entry(ObjectEntry10_1::NewObject {
            class_id: FEATURE_LAYER_CLASS_ID,
            key,
        }) = &entry.object
        {
            reached.push((entry.offset, *key, successor.hyperlink_count));
        }
    }

    assert!(
        !reached.is_empty(),
        "Map 根顺序应到达至少一个 FeatureLayer 后继段"
    );
    assert!(
        reached.iter().all(|(_, _, count)| *count == 0),
        "参考样本已到达的 Hyperlink 宿主计数：{reached:?}"
    );
    eprintln!("参考样本中按 /Maps 根顺序到达的 Hyperlink 宿主：{reached:?}");
}

#[test]
#[ignore = "需要 10.1 SDK Java 与 SDK .NET 提取文件"]
fn sdk_group_layers_are_reached_from_map_roots_and_are_v11_or_v12() {
    fn rooted_groups(path: &str) -> Vec<(usize, u32, u16, u32, usize)> {
        let mut doc = cfb::open(path).unwrap();
        let mut maps = Vec::new();
        doc.open_stream("/Maps")
            .unwrap()
            .read_to_end(&mut maps)
            .unwrap();
        let mut reader = Reader::new(&maps);
        let mut refs = RefTable10_1::new();
        assert!(reader.read_u32_le().unwrap() > 0);
        assert!(matches!(
            read_object_entry_10_1(&mut reader, &mut refs).unwrap(),
            ObjectEntry10_1::NewObject {
                class_id: MAP_CLASS_ID,
                ..
            }
        ));
        let map = read_map_suffix_10_1(&mut reader, 16_000_000, &mut refs).unwrap();

        map.layer_list
            .entries
            .iter()
            .filter_map(|entry| {
                let Some(LayerBodyPrefix10_1::GroupLayer(group)) = &entry.parsed else {
                    return None;
                };
                let LayerEntryObject::Entry(ObjectEntry10_1::NewObject {
                    class_id: GROUP_LAYER_CLASS_ID,
                    key,
                }) = &entry.object
                else {
                    return None;
                };
                if entry.offset == 0x15a {
                    assert_eq!(
                        group.stop,
                        mxd2qgz::layers::group_layer::GroupLayerStop10_1::Complete
                    );
                    assert_eq!(entry.body.as_ref().unwrap().len, 0);
                }
                Some((
                    entry.offset,
                    *key,
                    group.version,
                    group.child_count,
                    group.children.len(),
                ))
            })
            .collect()
    }

    let brazil = rooted_groups(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/temp/terra-round8-old/sdk-java/brazil.mxd.4896861F_D89C_46BA_ABEC_17591162B2B1",
    ));
    assert_eq!(
        brazil
            .iter()
            .map(|g| (g.0, g.2, g.3, g.4))
            .collect::<Vec<_>>(),
        [(0x15a, 11, 0, 0), (0x1f6, 11, 7, 7), (0x8d70, 11, 13, 13)]
    );

    let schematics = rooted_groups(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/temp/terra-round8-old/sdk-dotnet/SchematicsForServerSample.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2",
    ));
    assert_eq!(
        schematics
            .iter()
            .map(|g| (g.0, g.2, g.3, g.4))
            .collect::<Vec<_>>(),
        [(0x162, 12, 7, 7), (0xb234, 12, 1, 1)]
    );
}
