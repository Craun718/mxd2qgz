//! 从本地介质的完整 /Maps 读取图例，默认不要求安装介质。
use mxd2qgz::{
    layers::legend_item::LegendItemClassFormatSlot10_1,
    maps::{
        map_object::{MAP_CLASS_ID, MapSurroundElement10_1, read_map_suffix_10_1},
        map_surround::{LegendItemSlot10_1, LegendStop10_1},
        map_surround_family::SlotEntry10_1,
    },
    storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1},
    stream::reader::Reader,
};
use std::io::Read;

#[test]
#[ignore = "需要本地安装介质中的地图模板"]
fn media_templates_read_surrounds_and_legend_items_from_map_root() {
    for (path, map_version, legend_offset, extent_offset) in [
        (
            "temp/arcmap101-media-mxd/LandscapeClassic.mxd",
            45,
            0xa78,
            0xef4,
        ),
        (
            "temp/arcmap108-media-mxd/africa.mxt.2319C344_B117_4BE0_8B98_93D6E4E94502",
            31,
            0x8b28,
            0xa892,
        ),
    ] {
        let mut doc = cfb::open(path).unwrap();
        let mut maps = Vec::new();
        doc.open_stream("/Maps")
            .unwrap()
            .read_to_end(&mut maps)
            .unwrap();
        let mut reader = Reader::new(&maps);
        reader.read_bytes(4).unwrap();
        let mut refs = RefTable10_1::new();
        let entry = read_object_entry_10_1(&mut reader, &mut refs).unwrap();
        assert!(matches!(
            entry,
            ObjectEntry10_1::NewObject {
                class_id: MAP_CLASS_ID,
                ..
            }
        ));
        let body = reader.position();
        assert_eq!(
            u16::from_le_bytes(maps[body..body + 2].try_into().unwrap()),
            map_version
        );
        let suffix = read_map_suffix_10_1(&mut reader, 1_000_000, &mut refs).unwrap();
        assert_eq!(suffix.map_surround_count, Some(3));
        assert_eq!(suffix.map_surrounds.len(), 3);
        assert_eq!(suffix.extent_offset, Some(extent_offset));
        assert!(suffix.extent.is_some());
        if map_version == 45 {
            assert_eq!(suffix.map_surrounds[1].offset, 0x885);
            let Some(MapSurroundElement10_1::MarkerNorthArrow(arrow)) =
                &suffix.map_surround_parsed[1]
            else {
                panic!("第二个装饰物不是 MarkerNorthArrow");
            };
            assert_eq!(arrow.version, 2);
            assert_eq!(arrow.legacy_marker_long, None);
            let mut page = Vec::new();
            doc.open_stream("/PageLayout")
                .unwrap()
                .read_to_end(&mut page)
                .unwrap();
            let parsed =
                mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1(&maps, Some(&page)).unwrap();
            let layout = parsed.page_layout.unwrap();
            assert_eq!(layout.consumed_end_offset(), page.len());
            assert!(layout.surround_frames.iter().any(|frame| {
                frame.surround
                    == ObjectEntry10_1::BackReference {
                        class_id: mxd2qgz::maps::map_surround_family::MARKER_NORTH_ARROW_CLASS_ID,
                        key: arrow.key,
                    }
                    && frame.surround_body.is_none()
            }));
            assert!(arrow.stop.is_none());
            let SlotEntry10_1::Body(marker) = &arrow.marker_element else {
                panic!("没有内联 MarkerElement");
            };
            assert!(marker.stop.is_none());
        }
        assert_eq!(suffix.map_surrounds[2].offset, legend_offset);
        let Some(MapSurroundElement10_1::Legend(legend)) = &suffix.map_surround_parsed[2] else {
            panic!("第三个装饰物不是 Legend");
        };
        assert_eq!(legend.stop, LegendStop10_1::Completed);
        if map_version == 31 {
            assert_eq!(legend.items.len(), 3);
            for (item, offset) in legend.items.iter().zip([0x8ede, 0x956e, 0x9e76]) {
                assert_eq!(item.offset, offset);
                let LegendItemSlot10_1::HorizontalLegendItem(horizontal) = &item.entry else {
                    panic!("图例项不是 HorizontalLegendItem");
                };
                assert!(horizontal.stop.is_none());
                assert_eq!((horizontal.version, horizontal.base_version), (1, 1));
                let LegendItemClassFormatSlot10_1::ClassFormat(format) = &horizontal.class_format
                else {
                    panic!("没有新 LegendClassFormat 对象");
                };
                assert!(format.stop.is_none());
            }
        }
    }
}
