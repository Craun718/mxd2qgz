use mxd2qgz::maps::map_object::MAP_CLASS_ID;
use mxd2qgz::maps::map_object::{MapSuffixStop, read_map_suffix_10_1};
use mxd2qgz::maps::page_layout_prefix::{
    PAGE_LAYOUT_CLASS_ID, read_page_layout_element_10_1, read_page_layout_prefix_10_1,
};
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use mxd2qgz::stream::reader::Reader;
use std::collections::HashSet;
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
    let map_keys: HashSet<u32> = (0..10_000)
        .filter(|key| session_refs.contains(*key))
        .collect();
    assert!(map_keys.len() > 100);

    let mut page_reader = Reader::new(&page);
    assert!(matches!(
        read_object_entry_10_1(&mut page_reader, &mut session_refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: PAGE_LAYOUT_CLASS_ID,
            ..
        }
    ));
    let prefix = read_page_layout_prefix_10_1(&mut page_reader, 100_000).unwrap();
    let mut found = Vec::new();

    for index in 0..prefix.element_count {
        let offset = page_reader.position();
        let mut probe = page_reader.clone();
        let size = probe.read_u32_le().unwrap() as usize;
        let seek = probe.read_u32_le().unwrap();
        assert_eq!(seek, 0, "element {index} at {offset:#x}");
        let mut region = probe.read_region(size).unwrap();
        let mut probe_refs = session_refs.clone();
        if let ObjectEntry10_1::BackReference { class_id, key } =
            read_object_entry_10_1(&mut region, &mut probe_refs).unwrap()
        {
            if map_keys.contains(&key) {
                found.push((index, offset, class_id, key, "element-slot"));
                break;
            }
            panic!("PageLayout 根元素回引键 {key} 不属于已消费的 Maps 键集");
        }

        let (entry, body) =
            match read_page_layout_element_10_1(&mut page_reader, 16_000_000, &mut session_refs) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!(
                        "LinearRef PageLayout stops at element {index}, offset {offset:#x}: {error}"
                    );
                    break;
                }
            };
        let ObjectEntry10_1::NewObject { class_id, key } = entry else {
            unreachable!("page element reader only returns new objects")
        };

        // Debug 输出来自已解析的强类型槽；只认明确的 BackReference 变体，
        // 并将其键与 Maps 完整读取后冻结的键集求交。
        let debug = format!("{body:?}");
        let mut rest = debug.as_str();
        while let Some(start) = rest.find("BackReference {") {
            rest = &rest[start + "BackReference {".len()..];
            let Some(key_start) = rest.find("key: ") else {
                break;
            };
            let value = &rest[key_start + 5..];
            let digits: String = value.chars().take_while(char::is_ascii_digit).collect();
            let Ok(reference_key) = digits.parse::<u32>() else {
                continue;
            };
            if map_keys.contains(&reference_key) {
                found.push((index, offset, class_id, reference_key, "nested-slot"));
            }
            rest = value;
        }
        if !found.is_empty() {
            eprintln!(
                "found LinearRef Maps-to-PageLayout back-reference(s): {found:?}; element key={key}"
            );
            break;
        }
    }

    assert!(
        !found.is_empty(),
        "no Maps-key back-reference was reached before the first unsupported PageLayout body"
    );
}
