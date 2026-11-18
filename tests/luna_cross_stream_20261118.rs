use mxd2qgz::maps::map_object::MAP_CLASS_ID;
use mxd2qgz::maps::map_object::{MapSuffixStop, read_map_suffix_10_1};
use mxd2qgz::maps::page_layout_prefix::{
    PAGE_LAYOUT_CLASS_ID, read_page_layout_map_frame_10_1, read_page_layout_map_frame_prefix_10_1,
    read_page_layout_prefix_10_1,
};
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use mxd2qgz::stream::reader::Reader;
use std::io::Read;

const LINEARREF_PATH: &str =
    "temp/terra-round8-old/sdk-java/LinearRef.mxd.4896861F_D89C_46BA_ABEC_17591162B2B1";

fn read_storage_stream(document_path: &str, stream_name: &str) -> Vec<u8> {
    let mut document = cfb::open(document_path).unwrap();
    let mut bytes = Vec::new();
    document
        .open_stream(&format!("/{stream_name}"))
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    bytes
}

#[test]
#[ignore = "需要本地提取的 ArcMap 10.1 安装介质 CentralUSA.mxd"]
fn central_usa_maps_then_page_layout_preserves_one_reference_table() {
    let maps = read_storage_stream("temp/arcmap101-media-mxd/CentralUSA.mxd", "Maps");
    let page = read_storage_stream("temp/arcmap101-media-mxd/CentralUSA.mxd", "PageLayout");

    // /Maps 根先有 1 个 Map；这里只消费其对象头，不把未知 Map 体伪装成已解析。
    let mut shared = RefTable10_1::new();
    let mut maps_reader = Reader::new(&maps);
    assert_eq!(maps_reader.read_u32_le().unwrap(), 1);
    assert_eq!(
        read_object_entry_10_1(&mut maps_reader, &mut shared).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            key: 38,
        }
    );
    assert!(shared.contains(38));

    // 对应原生 SetStream：换 Reader/底层流，但沿用同一会话登记表。
    let mut page_reader = Reader::new(&page);
    assert_eq!(
        read_object_entry_10_1(&mut page_reader, &mut shared).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: PAGE_LAYOUT_CLASS_ID,
            key: 736,
        }
    );
    assert!(shared.contains(38));
    assert!(shared.contains(736));

    // 独立解析 PageLayout 从空表开始：其根可分类，但前一流的键 38 不存在。
    let mut isolated = RefTable10_1::new();
    let mut isolated_page_reader = Reader::new(&page);
    assert_eq!(
        read_object_entry_10_1(&mut isolated_page_reader, &mut isolated).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: PAGE_LAYOUT_CLASS_ID,
            key: 736,
        }
    );
    assert!(isolated.contains(736));
    assert!(!isolated.contains(38));
}

#[test]
#[ignore = "需要本地提取的 10.1 SDK Java LinearRef.mxd"]
fn linearref_page_layout_checks_references_against_completed_maps_session() {
    let maps = read_storage_stream(LINEARREF_PATH, "Maps");
    let page = read_storage_stream(LINEARREF_PATH, "PageLayout");

    let mut maps_reader = Reader::new(&maps);
    let map_count = maps_reader.read_u32_le().unwrap();
    assert!(map_count > 0);
    let mut session_refs = RefTable10_1::new();
    assert!(matches!(
        read_object_entry_10_1(&mut maps_reader, &mut session_refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
    let map = read_map_suffix_10_1(&mut maps_reader, 16_000_000, &mut session_refs).unwrap();
    assert_eq!(map.stop, MapSuffixStop::Complete);
    assert_eq!(maps_reader.position(), 0x8409);
    assert_eq!(maps_reader.read_u32_le().unwrap(), 1);
    assert_eq!(maps_reader.remaining(), 0);
    assert!(session_refs.contains(31));

    let mut page_reader = Reader::new(&page);
    assert_eq!(
        read_object_entry_10_1(&mut page_reader, &mut session_refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: PAGE_LAYOUT_CLASS_ID,
            key: 416
        }
    );
    let prefix = read_page_layout_prefix_10_1(&mut page_reader, 100_000).unwrap();
    assert!(prefix.extra_safe);
    assert_eq!(prefix.element_count, 1);

    // 仅解析 /PageLayout 时，原样的 IMap 头无法识别为回引。
    let mut isolated_reader = page_reader.clone();
    let mut isolated_refs = RefTable10_1::new();
    isolated_refs.register(416);
    assert!(
        read_page_layout_map_frame_prefix_10_1(&mut isolated_reader, 4096, &mut isolated_refs)
            .is_err()
    );
    assert_eq!(isolated_reader.position(), 0x1e);
    assert!(!isolated_refs.contains(417));

    let body = read_page_layout_map_frame_10_1(&mut page_reader, 4096, &mut session_refs).unwrap();
    let frame = body.prefix;
    assert_eq!(frame.version, 2);
    assert_eq!(frame.frame_key, 417);
    assert_eq!(frame.map_offset, 0x3c);
    assert_eq!(
        frame.map,
        ObjectEntry10_1::BackReference {
            class_id: MAP_CLASS_ID,
            key: 31
        }
    );
    assert!(session_refs.contains(417));
    assert_eq!(page_reader.position(), 0x27f);
    assert_eq!(body.base.properties.type_description, "Data Frame");
    assert!(session_refs.contains(422));
    assert_eq!(frame.remaining_in_element, 559);

    let report = mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1(&maps, Some(&page)).unwrap();
    let layout = report.page_layout.unwrap();
    assert_eq!(layout.consumed_elements, 1);
    assert_eq!(layout.map_frames.len(), 1);
    assert_eq!(layout.map_frames[0].prefix.map, frame.map);
    assert_eq!(layout.end_offset, 0x27f);
    assert_eq!(layout.tail.as_ref().unwrap().end_offset, 0x48c);
    assert_eq!(layout.print_settings_flag, Some(0));
    assert_eq!(layout.consumed_end_offset(), page.len());
    assert!(
        !report
            .map
            .unknown_fields
            .iter()
            .any(|field| field.stream == "/PageLayout")
    );
}

#[test]
fn page_layout_map_frame_does_not_borrow_bytes_from_next_element() {
    use mxd2qgz::maps::page_layout_prefix::MAP_FRAME_CLASS_ID;

    let mut bytes = Vec::new();
    // IMap 回引需要 20 字节，声明范围只给了 19 字节。
    bytes.extend(41_u32.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    bytes.extend(MAP_FRAME_CLASS_ID);
    bytes.extend(8_u32.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(MAP_CLASS_ID);
    bytes.extend(3_u32.to_le_bytes());
    bytes.push(0x71);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    refs.register(3);
    let expected_refs = refs.clone();
    assert!(read_page_layout_map_frame_prefix_10_1(&mut reader, 1024, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert_eq!(refs, expected_refs);

    bytes[..4].copy_from_slice(&43_u32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let frame = read_page_layout_map_frame_prefix_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert_eq!(frame.map_offset, 30);
    assert_eq!(frame.remaining_in_element, 1);
    assert_eq!(reader.position(), 50);
    assert_eq!(reader.read_bytes(1).unwrap(), &[0x71]);
}
