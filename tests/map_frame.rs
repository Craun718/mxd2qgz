use mxd2qgz::maps::map_object::MAP_CLASS_ID;
use mxd2qgz::maps::page_layout_prefix::{MAP_FRAME_CLASS_ID, read_page_layout_map_frame_10_1};
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use mxd2qgz::stream::reader::Reader;

fn wrap(body: &[u8]) -> Vec<u8> {
    let mut bytes = ((20 + body.len()) as u32).to_le_bytes().to_vec();
    bytes.extend(0_u32.to_le_bytes());
    bytes.extend(MAP_FRAME_CLASS_ID);
    bytes.extend(43_u32.to_le_bytes());
    bytes.extend(body);
    bytes
}

fn frame_body(version: u16) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(MAP_CLASS_ID);
    bytes.extend(42_u32.to_le_bytes());
    if version < 4 {
        bytes.extend(3_u32.to_le_bytes());
        bytes.extend(500_f64.to_le_bytes());
        bytes.extend([0; 16]);
    }
    if version < 2 {
        bytes.extend([0; 16]);
        bytes.extend(9_u16.to_le_bytes());
        bytes.extend(10_u16.to_le_bytes());
        bytes.extend([0; 16]);
    }
    bytes.extend([0; 8]); // 两个空集合
    bytes.extend(2_u16.to_le_bytes()); // BaseFrameElement
    bytes.extend(3_u16.to_le_bytes()); // 几何基类
    bytes.extend([0; 16]);
    bytes.extend(0x1020_3040_u32.to_le_bytes());
    bytes.extend(0x5060_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes()); // ElementProperties
    for _ in 0..2 {
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend(0_u16.to_le_bytes());
    }
    bytes.extend([0; 4 + 16]); // 旧式 CustomProperty
    bytes.extend([0; 32]); // IBorder + IBackground
    bytes.extend(0xffff_u16.to_le_bytes());
    bytes.extend([0; 16]); // IShadow
    if version == 3 {
        bytes.extend([0; 16]); // IMapFrame
        bytes.extend(0xffff_0001_u32.to_le_bytes());
        bytes.extend(1000_f64.to_le_bytes());
        bytes.extend(7_u32.to_le_bytes());
        bytes.extend([0; 16]); // IFeatureLayer
    }
    bytes
}

#[test]
fn parses_the_v0_to_v4_map_frame_gates() {
    for version in 0..=4 {
        let bytes = wrap(&frame_body(version));
        let mut r = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        refs.register(42);
        let frame = read_page_layout_map_frame_10_1(&mut r, 4096, &mut refs).unwrap();
        assert_eq!(r.remaining(), 0);
        assert_eq!(
            frame.prefix.map,
            ObjectEntry10_1::BackReference {
                class_id: MAP_CLASS_ID,
                key: 42
            }
        );
        assert!(refs.contains(43));
        assert_eq!(frame.extent_settings.is_some(), version < 4);
        assert_eq!(frame.legacy_frame.is_some(), version < 2);
        assert_eq!(frame.version_three_tail.is_some(), version == 3);
        assert_eq!(frame.base.geometry_long, Some(0x1020_3040));
        assert_eq!(frame.base.geometry_short, Some(0x5060));
        assert_eq!(frame.base.draft_mode_bits, 0xffff);
    }
}

#[test]
fn nested_truncation_and_unknown_grid_leave_the_outer_cursor_and_map_keys_intact() {
    let body = frame_body(2);
    let mut refs = RefTable10_1::new();
    refs.register(42);
    let original = refs.clone();
    // 对每个截断都重写外层长度，迫使内层读取器检查各自边界。
    for length in 0..body.len() {
        let bytes = wrap(&body[..length]);
        let mut r = Reader::new(&bytes);
        assert!(
            read_page_layout_map_frame_10_1(&mut r, 4096, &mut refs).is_err(),
            "{length}"
        );
        assert_eq!(r.position(), 0);
        assert_eq!(refs, original);
    }
    let grids_offset = 2 + 20 + 4 + 8 + 16;
    let mut unknown_grid = body;
    unknown_grid[grids_offset..grids_offset + 4].copy_from_slice(&1_u32.to_le_bytes());
    unknown_grid.splice(
        grids_offset + 4..grids_offset + 4,
        [0x99; 16].into_iter().chain(44_u32.to_le_bytes()),
    );
    let bytes = wrap(&unknown_grid);
    let mut r = Reader::new(&bytes);
    let error = read_page_layout_map_frame_10_1(&mut r, 4096, &mut refs).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    assert!(error.to_string().contains("IMapGrid"));
    assert_eq!(r.position(), 0);
    assert_eq!(refs, original);
}

#[test]
fn layout_stops_at_the_first_unknown_element_and_preserves_the_tail() {
    use mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1;
    use mxd2qgz::maps::page_layout_prefix::PAGE_LAYOUT_CLASS_ID;

    let mut page = PAGE_LAYOUT_CLASS_ID.to_vec();
    page.extend(41_u32.to_le_bytes());
    page.extend(9_u16.to_le_bytes());
    page.extend(0_u32.to_le_bytes());
    page.extend(1_u32.to_le_bytes());
    let element_offset = page.len();
    page.extend(20_u32.to_le_bytes());
    page.extend(0_u32.to_le_bytes());
    page.extend([0x99; 16]);
    page.extend(42_u32.to_le_bytes());
    page.extend([0x41, 0x42]); // 元素集合之后另有未知布局字段

    let maps = [0; 8]; // 0 个根 Map，后接 4 字节根尾
    let report = parse_mxd_streams_10_1(&maps, Some(&page)).unwrap();
    assert_eq!(report.confirmed_tail, Some((4, 0)));
    let layout = report.page_layout.unwrap();
    assert_eq!(layout.prefix.element_count, 1);
    assert_eq!(layout.consumed_elements, 0);
    assert!(layout.stop.is_some());
    assert_eq!(layout.end_offset, element_offset);
    let unknown = report.map.unknown_fields.last().unwrap();
    assert_eq!(unknown.stream, "/PageLayout");
    assert_eq!(unknown.offset, element_offset as u64);
    assert_eq!(unknown.bytes, page[element_offset..]);
}

#[test]
fn graticule_version_gates_and_truncation_preserve_map_frame_transaction() {
    use mxd2qgz::maps::map_grid::{
        DMS_GRID_LABEL_CLASS_ID, GRATICULE_CLASS_ID, SIMPLE_MAP_GRID_BORDER_CLASS_ID,
    };

    let offset = 2 + 20 + 4 + 8 + 16;
    for version in 0_u16..=5 {
        let mut body = frame_body(2);
        body[offset..offset + 4].copy_from_slice(&1_u32.to_le_bytes());
        let mut grid = GRATICULE_CLASS_ID.to_vec();
        grid.extend(44_u32.to_le_bytes());
        grid.extend(version.to_le_bytes());
        grid.extend(1_u16.to_le_bytes());
        let name: Vec<_> = "格网\0".encode_utf16().flat_map(u16::to_le_bytes).collect();
        grid.extend((name.len() as u32).to_le_bytes());
        grid.extend(name);
        grid.extend([0; 32]);
        grid.extend(DMS_GRID_LABEL_CLASS_ID);
        grid.extend(45_u32.to_le_bytes());
        grid.extend(4_u16.to_le_bytes());
        grid.extend([0; 4 + 3 * 16 + 4 + 16 + 8 + 10 + 2 + 2]);
        grid.extend([0; 16 + 8 + 2 + 16 + 8 + 8]);
        grid.extend(SIMPLE_MAP_GRID_BORDER_CLASS_ID);
        grid.extend(46_u32.to_le_bytes());
        grid.extend(1_u16.to_le_bytes());
        grid.extend([0; 16]);
        grid.extend([0; 2 + 4 * 8 + 4]);
        if version >= 2 {
            grid.extend([0; 2]);
        }
        if version >= 3 {
            grid.extend([0; 2 * 8 + 2 * 16 + 8 + 2]);
        }
        if version >= 4 {
            grid.extend([0; 8]);
        }
        if version >= 5 {
            grid.extend([0; 2 + 2 * 8]);
        }

        body.splice(offset + 4..offset + 4, grid);
        let bytes = wrap(&body);
        let mut refs = RefTable10_1::new();
        refs.register(42);
        let original = refs.clone();
        let frame =
            read_page_layout_map_frame_10_1(&mut Reader::new(&bytes), 4096, &mut refs).unwrap();
        let grid = frame.grids[0].graticule.as_ref().unwrap();
        assert_eq!(grid.base.name, "格网");
        assert!(grid.base.simple_border.is_some());
        assert!(grid.base.dms_label.is_some());
        assert_eq!(grid.v3.is_some(), version >= 3);
        assert_eq!(grid.double_24.is_some(), version >= 4);
        assert_eq!(grid.v5.is_some(), version >= 5);
        assert!(refs.contains(46));

        for length in offset + 4..body.len() {
            let bytes = wrap(&body[..length]);
            let mut reader = Reader::new(&bytes);
            let mut refs = original.clone();
            assert!(
                read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).is_err(),
                "v{version}，长度 {length}"
            );
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, original);
        }
    }
}
