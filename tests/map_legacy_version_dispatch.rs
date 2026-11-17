use mxd2qgz::maps::map_object::MAP_CLASS_ID;
use mxd2qgz::maps::mxd_maps::parse_maps_10_1;

fn map_with_version(version: u16, suffix: &[u8]) -> (Vec<u8>, usize) {
    let mut bytes = 1_i32.to_le_bytes().to_vec();
    bytes.extend(MAP_CLASS_ID);
    bytes.extend(7_u32.to_le_bytes());
    bytes.extend(version.to_le_bytes());
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend([0_u8; 16]);
    let offset = bytes.len();
    bytes.extend(suffix);
    (bytes, offset)
}

#[test]
fn legacy_versions_stop_at_prefix_and_preserve_all_remaining_bytes() {
    for version in [0, 19, 22] {
        let suffix = [0xff, 0xff, 0xff, 0xff, 1, 2, 3];
        let (bytes, offset) = map_with_version(version, &suffix);
        let report = parse_maps_10_1(&bytes).unwrap();
        assert!(report.layer_list.is_none());
        let frame = &report.map.frames[0];
        assert!(frame.children.is_empty());
        assert!(frame.extent.is_none());
        assert_eq!(frame.unknown_fields.len(), 1);
        assert_eq!(frame.unknown_fields[0].offset, offset as u64);
        assert_eq!(frame.unknown_fields[0].bytes, suffix);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|s| s.contains(&format!("Map 架构版本 {version} 小于 23")))
        );
    }
}

#[test]
fn version_23_enters_new_layout_and_rejects_invalid_count() {
    let (bytes, _) = map_with_version(23, &(-1_i32).to_le_bytes());
    let error = parse_maps_10_1(&bytes).unwrap_err();
    assert!(error.to_string().contains("计数"));
}

#[test]
fn legacy_zero_bytes_are_preserved_as_unknown() {
    // 根读取器还要窥读 4 字节，提供后缀后再检查准确边界。
    let (bytes, offset) = map_with_version(19, &[0; 4]);
    let report = parse_maps_10_1(&bytes).unwrap();
    assert_eq!(report.map.frames[0].unknown_fields[0].offset, offset as u64);
    assert_eq!(report.map.frames[0].unknown_fields[0].bytes, [0; 4]);
}
