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

fn frame_body(version: i16) -> Vec<u8> {
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
fn parses_the_signed_map_frame_version_gates() {
    for version in [i16::MIN, -1, 0, 1, 2, 3, 4] {
        let bytes = wrap(&frame_body(version));
        let mut r = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        refs.register(42);
        let frame = read_page_layout_map_frame_10_1(&mut r, 4096, &mut refs).unwrap();
        assert_eq!(r.remaining(), 0);
        assert_eq!(frame.prefix.version, version as u16);
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

    let bytes = wrap(&5_u16.to_le_bytes());
    let mut r = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let err = read_page_layout_map_frame_10_1(&mut r, 4096, &mut refs).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(r.position(), 0);
    assert!(refs.is_empty());
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

    use mxd2qgz::objects::number_format::LATLON_FORMAT_CLASS_ID;
    use mxd2qgz::storage::object_refs::read_object_entry_10_1;

    let offset = 2 + 20 + 4 + 8 + 16;
    for version in [i16::MIN, -1, 0, 1, 2, 3, 4, 5] {
        let mut body = frame_body(2);
        body[offset..offset + 4].copy_from_slice(&1_u32.to_le_bytes());
        let mut grid = GRATICULE_CLASS_ID.to_vec();
        grid.extend(44_u32.to_le_bytes());
        grid.extend(version.to_le_bytes());
        grid.extend(i16::MIN.to_le_bytes());
        let name: Vec<_> = "格网\0".encode_utf16().flat_map(u16::to_le_bytes).collect();
        grid.extend((name.len() as u32).to_le_bytes());
        grid.extend(name);
        grid.extend([0; 32]);
        grid.extend(DMS_GRID_LABEL_CLASS_ID);
        grid.extend(45_u32.to_le_bytes());
        let label_version = if version < 0 { version } else { 4 };
        grid.extend(label_version.to_le_bytes());
        grid.extend([0; 4 + 3 * 16 + 4]);
        let format_offset = offset + 4 + grid.len();
        grid.extend([0x42; 16]);
        grid.extend(47_u32.to_le_bytes());
        grid.extend([0; 8]);
        if label_version == 4 {
            grid.extend([0; 10 + 2 + 2]);
        }
        grid.extend([0; 16 + 8 + 2 + 16 + 8 + 8]);
        grid.extend(SIMPLE_MAP_GRID_BORDER_CLASS_ID);
        grid.extend(46_u32.to_le_bytes());
        grid.extend((-1_i16).to_le_bytes());
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
        let mut header = LATLON_FORMAT_CLASS_ID.to_vec();
        header.extend(47_u32.to_le_bytes());
        read_object_entry_10_1(&mut Reader::new(&header), &mut refs).unwrap();
        let original = refs.clone();
        let mut wrong = body.clone();
        wrong[format_offset + 16..format_offset + 20].copy_from_slice(&43_u32.to_le_bytes());
        let wrong = wrap(&wrong);
        let mut reader = Reader::new(&wrong);
        let mut staged = original.clone();
        assert_eq!(
            read_page_layout_map_frame_10_1(&mut reader, 4096, &mut staged)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::Unsupported
        );
        assert_eq!(reader.position(), 0);
        assert_eq!(staged, original);
        let frame =
            read_page_layout_map_frame_10_1(&mut Reader::new(&bytes), 4096, &mut refs).unwrap();
        let grid = frame.grids[0].graticule.as_ref().unwrap();
        assert_eq!(grid.base.name, "格网");
        assert_eq!(grid.base.version, 0x8000);
        assert_eq!(grid.base.simple_border.as_ref().unwrap().version, 0xffff);
        assert_eq!(grid.version, version as u16);
        let label = grid.base.dms_label.as_ref().unwrap();
        assert_eq!(label.version, label_version as u16);
        assert_eq!(
            label.format,
            ObjectEntry10_1::BackReference {
                class_id: LATLON_FORMAT_CLASS_ID,
                key: 47,
            }
        );
        assert_eq!(label.shorts_2c.is_some(), label_version >= 2);
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

#[test]
fn map_frame_reads_measured_and_index_grids_with_a_shared_calibrated_border() {
    use mxd2qgz::maps::map_grid::{
        CALIBRATED_MAP_GRID_BORDER_CLASS_ID, INDEX_GRID_CLASS_ID, MEASURED_GRID_CLASS_ID,
    };
    use mxd2qgz::objects::display_symbol::{SIMPLE_FILL_SYMBOL_CLASS_ID, SymbolSlotFull10_1};
    use mxd2qgz::objects::envelope::SpatialReferenceEntry10_1;
    use mxd2qgz::objects::spatial_reference_slots::UNKNOWN_COORDINATE_SYSTEM_CLASS_ID;

    fn string(bytes: &mut Vec<u8>, value: &str) {
        let chars: Vec<_> = value.encode_utf16().chain([0]).collect();
        bytes.extend((chars.len() as u32 * 2).to_le_bytes());
        bytes.extend(chars.into_iter().flat_map(u16::to_le_bytes));
    }

    fn grid_base(bytes: &mut Vec<u8>) {
        bytes.extend(1_u16.to_le_bytes());
        string(bytes, "网格");
        bytes.extend([0; 4 * 16 + 8 + 2 + 16 + 8 + 8]);
    }

    let mut grids = Vec::new();
    let mut border_version_offset = 0;
    for version in [1_u16, 2] {
        grids.extend(MEASURED_GRID_CLASS_ID);
        grids.extend((43 + u32::from(version)).to_le_bytes());
        grids.extend(version.to_le_bytes());
        grid_base(&mut grids);
        if version == 1 {
            grids.extend([0; 16]);
        } else {
            grids.extend(CALIBRATED_MAP_GRID_BORDER_CLASS_ID);
            grids.extend(51_u32.to_le_bytes());
            border_version_offset = grids.len();
            grids.extend(1_u16.to_le_bytes());
            grids.extend(SIMPLE_FILL_SYMBOL_CLASS_ID);
            grids.extend(52_u32.to_le_bytes());
            grids.extend(1_u16.to_le_bytes());
            grids.extend([0; 16 + 16 + 12]);
            grids.extend(SIMPLE_FILL_SYMBOL_CLASS_ID);
            grids.extend(52_u32.to_le_bytes());
            grids.extend([2_f64, 3_f64].into_iter().flat_map(f64::to_le_bytes));
            grids.extend(7_u16.to_le_bytes());
        }
        grids.extend(
            [10_f64, 20_f64, 30_f64, 40_f64]
                .into_iter()
                .flat_map(f64::to_le_bytes),
        );
        grids.extend((-3_i32).to_le_bytes());
        if version == 1 {
            grids.extend([0; 16]);
        } else {
            grids.extend(UNKNOWN_COORDINATE_SYSTEM_CLASS_ID);
            grids.extend(77_u32.to_le_bytes());
            grids.extend(5_u16.to_le_bytes());
        }
    }

    grids.extend(INDEX_GRID_CLASS_ID);
    grids.extend(46_u32.to_le_bytes());
    grids.extend(1_u16.to_le_bytes());
    grid_base(&mut grids);
    let border_ref_offset = grids.len();
    grids.extend([0x42; 16]);
    grids.extend(51_u32.to_le_bytes());
    grids.extend(4_i32.to_le_bytes());
    grids.extend(6_i32.to_le_bytes());
    let index_count_offset = grids.len();
    grids.extend(2_i32.to_le_bytes());
    string(&mut grids, "甲");
    string(&mut grids, "");
    grids.extend(1_i32.to_le_bytes());
    string(&mut grids, "七");

    let mut body = frame_body(2);
    let grid_count_offset = 2 + 20 + 4 + 8 + 16;
    let grids_offset = grid_count_offset + 4;
    body[grid_count_offset..grids_offset].copy_from_slice(&3_u32.to_le_bytes());
    body.splice(grids_offset..grids_offset, grids);

    let bytes = wrap(&body);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    refs.register(42);
    refs.register(77);
    let original = refs.clone();
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert!(refs.contains(52));
    assert_eq!(reader.remaining(), 0);
    let old = frame.grids[0].measured_grid.as_ref().unwrap();
    assert_eq!(old.spatial_reference, SpatialReferenceEntry10_1::Null);
    assert_eq!(old.short_30, None);
    let measured = frame.grids[1].measured_grid.as_ref().unwrap();
    assert_eq!(measured.short_30, Some(5));
    assert_eq!(
        measured.doubles.map(f64::from_bits),
        [10.0, 20.0, 30.0, 40.0]
    );
    assert_eq!(measured.long_34, -3);
    assert_eq!(
        measured.spatial_reference,
        SpatialReferenceEntry10_1::BackReference {
            class_id: UNKNOWN_COORDINATE_SYSTEM_CLASS_ID,
            key: 77,
        }
    );
    let border = measured.base.calibrated_border.as_ref().unwrap();
    assert!(matches!(
        border.fills[0],
        SymbolSlotFull10_1::SimpleFillSymbol(_)
    ));
    assert!(matches!(
        border.fills[1],
        SymbolSlotFull10_1::BackReference { key: 52, .. }
    ));
    assert_eq!(border.doubles.map(f64::from_bits), [2.0, 3.0]);
    assert_eq!(border.short_1c, 7);
    let index = frame.grids[2].index_grid.as_ref().unwrap();
    assert_eq!(
        index.base.border,
        ObjectEntry10_1::BackReference {
            class_id: CALIBRATED_MAP_GRID_BORDER_CLASS_ID,
            key: 51,
        }
    );
    assert_eq!(index.longs, [4, 6]);
    assert_eq!(index.label_counts, [2, 1]);
    assert_eq!(index.labels[0], ["甲", ""]);
    assert_eq!(index.labels[1], ["七"]);

    let mut invalid_border = body.clone();
    let border_version = grids_offset + border_version_offset;
    invalid_border[border_version..border_version + 2].copy_from_slice(&2_u16.to_le_bytes());
    let mut wrong_border_ref = body.clone();
    let key = grids_offset + border_ref_offset + 16;
    wrong_border_ref[key..key + 4].copy_from_slice(&52_u32.to_le_bytes());
    let mut invalid_index = body.clone();
    let index_count = grids_offset + index_count_offset;
    invalid_index[index_count..index_count + 4].copy_from_slice(&i32::MAX.to_le_bytes());
    for invalid in [
        &invalid_border,
        &wrong_border_ref,
        &invalid_index,
        &body[..body.len() - 1],
    ] {
        let bytes = wrap(invalid);
        let mut reader = Reader::new(&bytes);
        let mut refs = original.clone();
        assert!(read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, original);
    }
}

#[test]
fn map_frame_reads_numeric_grid_label_versions_and_rolls_back_truncation() {
    use mxd2qgz::layers::feature_layer_tail::NumberFormatSlot10_1;
    use mxd2qgz::maps::map_grid::{
        CORNER_GRID_LABEL_CLASS_ID, FORMATTED_GRID_LABEL_CLASS_ID, GRATICULE_CLASS_ID,
        MIXED_FONT_GRID_LABEL_CLASS_ID, NumericGridLabelTail10_1,
    };
    use mxd2qgz::objects::number_format::CURRENCY_FORMAT_CLASS_ID;

    for (class_id, max_version) in [
        (FORMATTED_GRID_LABEL_CLASS_ID, 2_i16),
        (MIXED_FONT_GRID_LABEL_CLASS_ID, 3),
        (CORNER_GRID_LABEL_CLASS_ID, 2),
    ] {
        for version in 1..=max_version {
            let mut grid = GRATICULE_CLASS_ID.to_vec();
            grid.extend(44_u32.to_le_bytes());
            grid.extend(1_u16.to_le_bytes());
            grid.extend(1_u16.to_le_bytes()); // MapGrid 基体版本。
            grid.extend([2, 0, 0, 0, 0, 0]);
            grid.extend([0; 32]);
            grid.extend(class_id);
            grid.extend(45_u32.to_le_bytes());
            grid.extend(version.to_le_bytes());
            grid.extend([0; 16]);
            grid.extend(CURRENCY_FORMAT_CLASS_ID);
            grid.extend(46_u32.to_le_bytes());
            grid.extend(1_u16.to_le_bytes());
            grid.extend(0.25_f64.to_le_bytes());
            match class_id {
                FORMATTED_GRID_LABEL_CLASS_ID if version >= 2 => grid.extend(17_u16.to_le_bytes()),
                MIXED_FONT_GRID_LABEL_CLASS_ID => {
                    grid.extend([0; 16]);
                    if version >= 2 {
                        grid.extend([0; 10]);
                    }
                    if version >= 3 {
                        grid.extend(19_u16.to_le_bytes());
                    }
                }
                CORNER_GRID_LABEL_CLASS_ID => {
                    grid.extend([0; 16 + 18]);
                    grid.extend((-2_i32).to_le_bytes());
                    grid.extend(3_i32.to_le_bytes());
                    if version >= 2 {
                        for letter in [b'A', b'B', b'C'] {
                            grid.extend([4, 0, 0, 0, letter, 0, 0, 0]);
                        }
                    }
                }
                _ => {}
            }
            let label_end = grid.len();
            grid.extend([0; 16 + 8 + 2 + 16 + 8 + 8 + 16]);
            grid.extend([0; 2 + 4 * 8 + 4]); // Graticule v1 后继段。

            let count = 2 + 20 + 4 + 8 + 16;
            let mut body = frame_body(2);
            body[count..count + 4].copy_from_slice(&1_u32.to_le_bytes());
            body.splice(count + 4..count + 4, grid);
            let label_start = count + 4 + 20 + 2 + 2 + 6 + 32;
            let mut referenced = body.clone();
            referenced.drain(label_start + 20..count + 4 + label_end);
            referenced[label_start..label_start + 16].fill(0x42);
            let referenced = wrap(&referenced);
            let mut refs = RefTable10_1::new();
            refs.register(42);
            let mut header = class_id.to_vec();
            header.extend(45_u32.to_le_bytes());
            mxd2qgz::storage::object_refs::read_object_entry_10_1(
                &mut Reader::new(&header),
                &mut refs,
            )
            .unwrap();
            let mut reader = Reader::new(&referenced);
            let result = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
            let base = &result.grids[0].graticule.as_ref().unwrap().base;
            assert_eq!(
                base.label,
                ObjectEntry10_1::BackReference { class_id, key: 45 }
            );
            assert!(base.numeric_label.is_none());
            assert_eq!(reader.remaining(), 0);

            let format_start = label_start + 20 + 2 + 16;
            let mut format_ref = body.clone();
            format_ref.drain(format_start + 20..format_start + 22);
            format_ref[format_start..format_start + 16].fill(0x42);

            let mut header = CURRENCY_FORMAT_CLASS_ID.to_vec();
            header.extend(46_u32.to_le_bytes());
            let mut refs = RefTable10_1::new();
            refs.register(42);
            mxd2qgz::storage::object_refs::read_object_entry_10_1(
                &mut Reader::new(&header),
                &mut refs,
            )
            .unwrap();
            let serialized = wrap(&format_ref);
            let mut reader = Reader::new(&serialized);
            let result = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
            let label = result.grids[0]
                .graticule
                .as_ref()
                .unwrap()
                .base
                .numeric_label
                .as_ref()
                .unwrap();
            assert!(matches!(
                label.format,
                NumberFormatSlot10_1::BackReference {
                    class_id: CURRENCY_FORMAT_CLASS_ID,
                    key: 46,
                }
            ));
            assert_eq!(reader.remaining(), 0);

            format_ref[format_start + 16..format_start + 20].copy_from_slice(&45_u32.to_le_bytes());
            let serialized = wrap(&format_ref);
            let mut reader = Reader::new(&serialized);
            let mut refs = RefTable10_1::new();
            refs.register(42);
            let saved = refs.clone();
            assert_eq!(
                read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs)
                    .unwrap_err()
                    .kind(),
                std::io::ErrorKind::Unsupported
            );
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, saved);

            let bytes = wrap(&body);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            refs.register(42);
            let original = refs.clone();
            let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
            let graticule = frame.grids[0].graticule.as_ref().unwrap();
            let label = graticule.base.numeric_label.as_ref().unwrap();
            assert_eq!(label.version, version);
            assert_eq!(f64::from_bits(label.double_bits), 0.25);
            assert!(
                matches!(&label.format, NumberFormatSlot10_1::CurrencyFormat(format) if format.key == 46)
            );
            match &label.tail {
                NumericGridLabelTail10_1::Formatted { short_18 } => {
                    assert_eq!(*short_18, (version >= 2).then_some(17))
                }
                NumericGridLabelTail10_1::MixedFont {
                    shorts_20,
                    short_10,
                    ..
                } => {
                    assert_eq!(*shorts_20, (version >= 2).then_some([0; 5]));
                    assert_eq!(*short_10, (version >= 3).then_some(19));
                }
                NumericGridLabelTail10_1::Corner { longs, strings, .. } => {
                    assert_eq!(*longs, [-2, 3]);
                    assert_eq!(
                        strings
                            .as_ref()
                            .map(|strings| strings.each_ref().map(String::as_str)),
                        (version >= 2).then_some(["A", "B", "C"])
                    );
                }
            }
            assert!(refs.contains(46));
            assert_eq!(reader.remaining(), 0);

            let bytes = wrap(&body[..count + 4 + label_end - 1]);
            let mut reader = Reader::new(&bytes);
            let mut refs = original.clone();
            assert!(read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, original);
        }
    }
}

#[test]
fn map_frame_reads_mgrs_and_custom_overlay_grid_bodies() {
    use mxd2qgz::layers::feature_layer::FEATURE_CLASS_NAME_CLASS_ID;
    use mxd2qgz::layers::raster_dataset_name::{
        RASTER_CATALOG_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1,
    };
    use mxd2qgz::maps::map_grid::{CUSTOM_OVERLAY_GRID_CLASS_ID, MGRS_GRID_CLASS_ID};
    use mxd2qgz::maps::standalone_table::TABLE_NAME_CLASS_ID;
    use mxd2qgz::objects::dataset_name_more::{
        DatasetNameMoreBody10_1, DatasetNameMoreSlot10_1, REL_QUERY_TABLE_NAME_CLASS_ID,
        RELATIONSHIP_CLASS_NAME_CLASS_ID, XY_EVENT_2_FIELDS_PROPERTIES_CLASS_ID,
        XY_EVENT_SOURCE_NAME_CLASS_ID,
    };
    use mxd2qgz::objects::display_symbol::{SIMPLE_LINE_SYMBOL_CLASS_ID, SymbolSlotFull10_1};

    fn string(bytes: &mut Vec<u8>, value: &str) {
        let chars: Vec<_> = value.encode_utf16().chain([0]).collect();
        bytes.extend((chars.len() as u32 * 2).to_le_bytes());
        bytes.extend(chars.into_iter().flat_map(u16::to_le_bytes));
    }

    fn base(bytes: &mut Vec<u8>) {
        bytes.extend(1_u16.to_le_bytes());
        string(bytes, "网格");
        bytes.extend([0; 4 * 16 + 8 + 2 + 16 + 8 + 8 + 16]);
    }

    let mut grids = Vec::new();
    let mut final_line_offset = 0;
    for version in 1_u16..=3 {
        grids.extend(MGRS_GRID_CLASS_ID);
        grids.extend((50 + u32::from(version)).to_le_bytes());
        grids.extend(version.to_le_bytes());
        base(&mut grids);
        grids.extend((-11_i32).to_le_bytes());
        grids.extend([0; 16]);
        grids.extend(22_u16.to_le_bytes());
        grids.extend([0; 32]);
        grids.extend(3.5_f64.to_le_bytes());
        grids.extend(44_u16.to_le_bytes());
        grids.extend([0; 16]);
        grids.extend((-55_i32).to_le_bytes());
        grids.extend(66_i32.to_le_bytes());
        if version >= 2 {
            grids.extend(77_u16.to_le_bytes());
        }
        if version == 3 {
            grids.extend((-88_i32).to_le_bytes());
            final_line_offset = grids.len();
            grids.extend(SIMPLE_LINE_SYMBOL_CLASS_ID);
            grids.extend(71_u32.to_le_bytes());
        }
    }
    grids.extend(CUSTOM_OVERLAY_GRID_CLASS_ID);
    grids.extend(54_u32.to_le_bytes());
    grids.extend(1_u16.to_le_bytes());
    base(&mut grids);
    let source_offset = grids.len();
    grids.extend(FEATURE_CLASS_NAME_CLASS_ID);
    grids.extend(72_u32.to_le_bytes());
    let source_body_offset = grids.len();
    grids.extend(2_u16.to_le_bytes());
    for value in ["格网要素", "", "Shapefile 要素类", "Shape"] {
        string(&mut grids, value);
    }
    grids.extend(1_i32.to_le_bytes());
    grids.extend(1_i32.to_le_bytes());
    grids.extend([0; 2 + 16 + 2]);
    let source_end = grids.len();
    string(&mut grids, "格网编号");

    let mut body = frame_body(2);
    let count = 2 + 20 + 4 + 8 + 16;
    let start = count + 4;
    body[count..start].copy_from_slice(&4_u32.to_le_bytes());
    body.splice(start..start, grids);

    let mut refs = RefTable10_1::new();
    refs.register(42);
    refs.register(71);
    let original = refs.clone();
    let bytes = wrap(&body);
    let mut reader = Reader::new(&bytes);
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert_eq!(reader.remaining(), 0);
    for version in 1_i16..=3 {
        let grid = frame.grids[(version - 1) as usize]
            .mgrs_grid
            .as_ref()
            .unwrap();
        assert_eq!(grid.version, version);
        assert_eq!(grid.base.name, "网格");
        assert_eq!(grid.long_ac, -11);
        assert_eq!(grid.short_44, 22);
        assert_eq!(f64::from_bits(grid.double_34), 3.5);
        assert_eq!(grid.short_2c, 44);
        assert_eq!(grid.longs, [-55, 66]);
        assert_eq!(grid.short_1c, (version >= 2).then_some(77));
        assert_eq!(grid.v3.is_some(), version == 3);
        if let Some(v3) = &grid.v3 {
            assert_eq!(v3.long_18, -88);
            assert!(matches!(
                v3.line_14,
                SymbolSlotFull10_1::BackReference { key: 71, .. }
            ));
        }
    }
    let overlay = frame.grids[3].custom_overlay_grid.as_ref().unwrap();
    assert_eq!(overlay.string_44, "格网编号");
    assert!(overlay.feature_class_name.is_some());
    assert!(refs.contains(72));

    let mut catalog = body.clone();
    catalog[start + source_offset..start + source_offset + 16]
        .copy_from_slice(&RASTER_CATALOG_NAME_CLASS_ID);
    let bytes = wrap(&catalog);
    let mut reader = Reader::new(&bytes);
    let mut refs = original.clone();
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    let overlay = frame.grids[3].custom_overlay_grid.as_ref().unwrap();
    assert!(matches!(
        overlay.other_source_name.as_deref(),
        Some(DatasetNameMoreBody10_1::RasterTable(name))
            if matches!(name.as_ref(), RasterDatasetNameEntryBody10_1::Catalog(body) if body.key == 72)
    ));
    assert_eq!(overlay.string_44, "格网编号");
    assert_eq!(reader.remaining(), 0);
    assert!(refs.contains(72));

    let mut event_source = XY_EVENT_SOURCE_NAME_CLASS_ID.to_vec();
    event_source.extend(72_u32.to_le_bytes());
    event_source.extend(1_u16.to_le_bytes());
    string(&mut event_source, "格网事件");
    string(&mut event_source, "");
    let event_table_offset = event_source.len();
    event_source.extend(TABLE_NAME_CLASS_ID);
    event_source.extend(73_u32.to_le_bytes());
    event_source.extend(1_u16.to_le_bytes());
    for value in ["坐标表", "", ""] {
        string(&mut event_source, value);
    }
    event_source.extend([0; 16]);
    event_source.extend(XY_EVENT_2_FIELDS_PROPERTIES_CLASS_ID);
    event_source.extend(74_u32.to_le_bytes());
    event_source.extend(1_u16.to_le_bytes());
    for value in ["x", "y", "z"] {
        string(&mut event_source, value);
    }
    event_source.extend([0; 16]);
    let mut event_body = body.clone();
    event_body.splice(start + source_offset..start + source_end, event_source);
    let bytes = wrap(&event_body);
    let mut reader = Reader::new(&bytes);
    let mut refs = original.clone();
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    let overlay = frame.grids[3].custom_overlay_grid.as_ref().unwrap();
    assert!(matches!(
        overlay.other_source_name.as_deref(),
        Some(DatasetNameMoreBody10_1::XyEventSource(_))
    ));
    assert_eq!(overlay.string_44, "格网编号");
    assert_eq!(reader.remaining(), 0);
    assert!([72, 73, 74].into_iter().all(|key| refs.contains(key)));

    let mut table_source = TABLE_NAME_CLASS_ID.to_vec();
    table_source.extend(72_u32.to_le_bytes());
    table_source.extend(1_u16.to_le_bytes());
    for value in ["格网数据表", "", ""] {
        string(&mut table_source, value);
    }
    table_source.extend([0; 16]);
    let mut table_body = body.clone();
    table_body.splice(start + source_offset..start + source_end, table_source);
    let bytes = wrap(&table_body);
    let mut reader = Reader::new(&bytes);
    let mut refs = original.clone();
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    let overlay = frame.grids[3].custom_overlay_grid.as_ref().unwrap();
    assert!(matches!(
        overlay.other_source_name.as_deref(),
        Some(DatasetNameMoreBody10_1::Table(name)) if name.strings[0] == "格网数据表"
    ));
    assert!(refs.contains(72));
    assert_eq!(overlay.string_44, "格网编号");
    assert_eq!(reader.remaining(), 0);

    let mut cad_source = mxd2qgz::objects::dataset_name_more::CAD_DRAWING_NAME_CLASS_ID.to_vec();
    cad_source.extend(72_u32.to_le_bytes());
    cad_source.extend(1_u16.to_le_bytes());
    for value in ["格网图纸", "", ""] {
        string(&mut cad_source, value);
    }
    cad_source.extend([0; 16]);
    let mut cad_body = body.clone();
    cad_body.splice(start + source_offset..start + source_end, cad_source);
    let bytes = wrap(&cad_body);
    let mut reader = Reader::new(&bytes);
    let mut refs = original.clone();
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    let overlay = frame.grids[3].custom_overlay_grid.as_ref().unwrap();
    assert!(matches!(
        overlay.other_source_name.as_deref(),
        Some(DatasetNameMoreBody10_1::CadDrawing(name)) if name.key == 72
    ));
    assert!(refs.contains(72));
    assert_eq!(overlay.string_44, "格网编号");
    assert_eq!(reader.remaining(), 0);

    let mut joined_source = REL_QUERY_TABLE_NAME_CLASS_ID.to_vec();
    joined_source.extend(72_u32.to_le_bytes());
    joined_source.extend(3_u16.to_le_bytes());
    string(&mut joined_source, "连接结果");
    string(&mut joined_source, "");
    joined_source.extend(RELATIONSHIP_CLASS_NAME_CLASS_ID);
    joined_source.extend(73_u32.to_le_bytes());
    joined_source.extend(1_u16.to_le_bytes());
    for value in ["关系类", "", ""] {
        string(&mut joined_source, value);
    }
    joined_source.extend([0; 12 + 2 + 16]); // 标量、零选择子和空工作空间。
    joined_source.extend(1_u16.to_le_bytes());
    joined_source.extend([0; 32]); // 空查询过滤器和空选择集。
    string(&mut joined_source, "格网字段");
    joined_source.extend([0; 6]);

    let mut joined_body = body.clone();
    joined_body.splice(start + source_offset..start + source_end, joined_source);
    let bytes = wrap(&joined_body);
    let mut reader = Reader::new(&bytes);
    let mut refs = original.clone();
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    let overlay = frame.grids[3].custom_overlay_grid.as_ref().unwrap();
    let Some(DatasetNameMoreBody10_1::RelQueryTable(name)) = overlay.other_source_name.as_deref()
    else {
        panic!("预期关系查询名称");
    };
    assert!(matches!(
        &name.source_slot,
        DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::RelationshipClass(body)) if body.key == 73
    ));
    assert_eq!(name.string_c, "格网字段");
    assert_eq!(overlay.string_44, "格网编号");
    assert!(refs.contains(72) && refs.contains(73));
    assert_eq!(reader.remaining(), 0);

    use mxd2qgz::objects::dataset_name::TOPOLOGY_NAME_CLASS_ID;
    let mut topology = TOPOLOGY_NAME_CLASS_ID.to_vec();
    topology.extend(72_u32.to_le_bytes());
    topology.extend(1_u16.to_le_bytes());
    string(&mut topology, "拓扑名称");
    string(&mut topology, "");
    topology.extend([0; 16]); // 空 IDatasetName。
    let mut topology_body = body.clone();
    topology_body.splice(start + source_offset..start + source_end, topology);
    let bytes = wrap(&topology_body);
    let mut reader = Reader::new(&bytes);
    let mut refs = original.clone();
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    let overlay = frame.grids[3].custom_overlay_grid.as_ref().unwrap();
    assert!(matches!(
        overlay.other_source_name.as_deref(),
        Some(DatasetNameMoreBody10_1::Topology(name)) if name.string_a == "拓扑名称"
    ));
    assert_eq!(overlay.string_44, "格网编号");
    assert!(refs.contains(72));
    assert_eq!(reader.remaining(), 0);

    let mut reader = Reader::new(&bytes[..bytes.len() - 1]);
    let mut refs = original.clone();
    assert!(read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(!refs.contains(72));

    let mut unknown_event_table = event_body;
    unknown_event_table[start + source_offset + event_table_offset] ^= 0xff;
    let mut unknown_source = body.clone();
    unknown_source[start + source_offset] ^= 0xff;
    let mut unknown_version = body.clone();
    unknown_version[start + 20..start + 22].copy_from_slice(&4_u16.to_le_bytes());
    for invalid in [
        unknown_source.as_slice(),
        unknown_event_table.as_slice(),
        unknown_version.as_slice(),
        &body[..start + final_line_offset + 17],
        &body[..start + source_body_offset + 1],
    ] {
        let bytes = wrap(invalid);
        let mut reader = Reader::new(&bytes);
        let mut refs = original.clone();
        assert!(read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, original);
    }
}

#[test]
fn locator_rectangle_reads_a_new_map_frame_with_shared_keys() {
    use mxd2qgz::maps::map_frame::LOCATOR_RECTANGLE_CLASS_ID;

    fn body_with_locator(key: u32, count: u32) -> Vec<u8> {
        let mut body = frame_body(4);
        if count == 0 {
            return body;
        }
        let mut locator = LOCATOR_RECTANGLE_CLASS_ID.to_vec();
        locator.extend(key.to_le_bytes());
        locator.extend(1_u16.to_le_bytes());
        locator.extend(MAP_FRAME_CLASS_ID);
        locator.extend((key + 1).to_le_bytes());
        locator.extend(body_with_locator(key + 2, count - 1));
        locator.extend([0; 16 * 3]);
        locator.extend(17_u16.to_le_bytes());
        body[26..30].copy_from_slice(&1_u32.to_le_bytes());
        body.splice(30..30, locator);
        body
    }

    let bytes = wrap(&body_with_locator(50, 1));
    let mut refs = RefTable10_1::new();
    refs.register(42);
    let mut reader = Reader::new(&bytes);
    let outer = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    let locator = outer.locator_rectangles[0].body.as_ref().unwrap();
    assert_eq!(locator.short_1c, 17);
    let inner = locator.map_frame_body.as_ref().unwrap();
    assert_eq!(inner.prefix.frame_key, 51);
    assert_eq!(inner.prefix.map_offset, 8 + 20 + 30 + 20 + 2 + 20 + 2);
    assert_eq!(
        inner.prefix.map,
        ObjectEntry10_1::BackReference {
            class_id: MAP_CLASS_ID,
            key: 42
        }
    );
    assert!(refs.contains(50) && refs.contains(51));
    assert_eq!(reader.remaining(), 0);

    let version_offset = 8 + 20 + 30 + 20;
    for version in [i16::MIN, -1, 0] {
        let mut old = bytes.clone();
        old[version_offset..version_offset + 2].copy_from_slice(&version.to_le_bytes());
        let mut reader = Reader::new(&old);
        let mut refs = RefTable10_1::new();
        refs.register(42);
        let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
        let locator = frame.locator_rectangles[0].body.as_ref().unwrap();
        assert_eq!(locator.version, version as u16);
        assert_eq!(locator.short_1c, 17);
        assert!(locator.shadow.is_none());
        assert_eq!(locator.short_24, None);
        assert!(locator.map_frame_body.is_some());
        assert_eq!(reader.remaining(), 0);
    }

    let mut unsupported = bytes.clone();
    unsupported[version_offset..version_offset + 2].copy_from_slice(&4_u16.to_le_bytes());
    let mut truncated = bytes.clone();
    truncated.pop();
    let size = (truncated.len() as u32 - 8).to_le_bytes();
    truncated[..4].copy_from_slice(&size);
    for invalid in [unsupported, truncated, wrap(&body_with_locator(50, 9))] {
        let mut refs = RefTable10_1::new();
        refs.register(42);
        let saved = refs.clone();
        let mut reader = Reader::new(&invalid);
        assert!(read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, saved);
    }
}

#[test]
fn old_map_frame_tail_reads_a_new_map_frame_before_its_scalars() {
    let mut outer = frame_body(3);
    let slot = outer.len() - (16 + 4 + 8 + 4 + 16);
    let mut inner = MAP_FRAME_CLASS_ID.to_vec();
    inner.extend(51_u32.to_le_bytes());
    inner.extend(frame_body(4));
    outer.splice(slot..slot + 16, inner);
    let bytes = wrap(&outer);
    let mut refs = RefTable10_1::new();
    refs.register(42);
    let mut reader = Reader::new(&bytes);
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    let tail = frame.version_three_tail.unwrap();
    assert_eq!(tail.map_frame_body.unwrap().prefix.frame_key, 51);
    assert_eq!(tail.mode_bits, 0xffff_0001);
    assert_eq!(tail.scale_bits, 1000_f64.to_bits());
    assert!(refs.contains(51));
    assert_eq!(reader.remaining(), 0);

    outer.splice(slot..slot + 20 + frame_body(4).len(), {
        let mut wrong = MAP_FRAME_CLASS_ID.to_vec();
        wrong.extend(42_u32.to_le_bytes());
        wrong
    });
    let mut refs = RefTable10_1::new();
    let mut map = MAP_CLASS_ID.to_vec();
    map.extend(42_u32.to_le_bytes());
    mxd2qgz::storage::object_refs::read_object_entry_10_1(&mut Reader::new(&map), &mut refs)
        .unwrap();
    let saved = refs.clone();
    let bytes = wrap(&outer);
    let mut reader = Reader::new(&bytes);
    assert!(read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert_eq!(refs, saved);
}

#[test]
fn old_map_frame_reads_a_complete_inline_feature_layer() {
    use mxd2qgz::layers::feature_layer::FEATURE_LAYER_CLASS_ID;

    let mut body = frame_body(3);
    let slot = body.len() - 16;
    body.truncate(slot);
    body.extend(FEATURE_LAYER_CLASS_ID);
    body.extend(51_u32.to_le_bytes());
    body.extend(1_u16.to_le_bytes());
    for _ in 0..2 {
        body.extend([2, 0, 0, 0, 0, 0]);
    }
    body.extend([0; 6 + 16 * 3]); // 三个 short、名称、渲染器及 GUID。
    body.extend([2, 0, 0, 0, 0, 0]);
    body.extend([0; 16 + 2 + 4 + 16]); // 比例值、short、空超链接循环和颜色。
    for _ in 0..2 {
        body.extend([2, 0, 0, 0, 0, 0]);
    }
    body.extend([0; 4 + 4 + 16 + 2 + 16 + 4 + 2]);
    let bytes = wrap(&body);
    let mut refs = RefTable10_1::new();
    refs.register(42);
    let mut reader = Reader::new(&bytes);
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    let layer = frame.version_three_tail.unwrap();
    assert_eq!(layer.layer_body.unwrap().version, 1);
    assert!(refs.contains(51));
    assert_eq!(reader.remaining(), 0);

    body.pop();
    let truncated = wrap(&body);
    let mut refs = RefTable10_1::new();
    refs.register(42);
    let saved = refs.clone();
    let mut reader = Reader::new(&truncated);
    assert!(read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert_eq!(refs, saved);
}

#[test]
fn map_frame_resolves_root_map_by_registered_class() {
    use mxd2qgz::storage::object_refs::read_object_entry_10_1;

    let mut body = frame_body(4);
    for (registered, header) in [(MAP_CLASS_ID, [0x42; 16]), ([0x42; 16], MAP_CLASS_ID)] {
        body[2..18].copy_from_slice(&header);
        let bytes = wrap(&body);
        let mut definition = registered.to_vec();
        definition.extend(42_u32.to_le_bytes());
        let mut refs = RefTable10_1::new();
        read_object_entry_10_1(&mut Reader::new(&definition), &mut refs).unwrap();
        let saved = refs.clone();
        let mut reader = Reader::new(&bytes);
        let result = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs);
        if registered == MAP_CLASS_ID {
            assert_eq!(
                result.unwrap().prefix.map,
                ObjectEntry10_1::BackReference {
                    class_id: MAP_CLASS_ID,
                    key: 42,
                }
            );
            assert_eq!(reader.remaining(), 0);
        } else {
            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::Unsupported);
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, saved);
        }
    }
}

#[test]
fn base_frame_and_rect_shape_use_signed_version_gates() {
    let mut body = frame_body(4);
    let base = 30;
    body[base..base + 2].copy_from_slice(&i16::MIN.to_le_bytes());
    body[base + 2..base + 4].copy_from_slice(&(-1_i16).to_le_bytes());
    body.truncate(body.len() - 16); // 旧 BaseFrameElement 无 IShadow。
    body.drain(base + 20..base + 26); // 旧 RectShapeElement 无 LONG 和 short。
    let bytes = wrap(&body);
    let mut refs = RefTable10_1::new();
    refs.register(42);
    let mut reader = Reader::new(&bytes);
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert_eq!(frame.base.version, 0x8000);
    assert_eq!(frame.base.geometry_version, 0xffff);
    assert_eq!(frame.base.geometry_long, None);
    assert_eq!(frame.base.geometry_short, None);
    assert_eq!(frame.base.draft_mode_bits, 0xffff);
    assert!(frame.base.shadow.is_none());
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn map_frame_extent_uses_registered_envelope_class() {
    use mxd2qgz::objects::envelope::ENVELOPE_CLASS_ID;
    use mxd2qgz::storage::object_refs::read_object_entry_10_1;

    for registered in [ENVELOPE_CLASS_ID, MAP_CLASS_ID] {
        let mut refs = RefTable10_1::new();
        refs.register(42);
        let mut definition = registered.to_vec();
        definition.extend(51_u32.to_le_bytes());
        read_object_entry_10_1(&mut Reader::new(&definition), &mut refs).unwrap();
        let saved = refs.clone();

        let mut body = frame_body(2);
        let mut reference = MAP_CLASS_ID.to_vec();
        reference.extend(51_u32.to_le_bytes());
        body.splice(34..50, reference);
        let bytes = wrap(&body);
        let mut reader = Reader::new(&bytes);
        let result = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs);
        if registered == ENVELOPE_CLASS_ID {
            let extent = result.unwrap().extent_settings.unwrap();
            assert_eq!(
                extent.envelope,
                ObjectEntry10_1::BackReference {
                    class_id: ENVELOPE_CLASS_ID,
                    key: 51,
                }
            );
            assert_eq!(reader.remaining(), 0);
        } else {
            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::Unsupported);
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, saved);
        }
    }
}

#[test]
fn rect_shape_geometry_reference_retains_registered_class() {
    use mxd2qgz::objects::envelope::ENVELOPE_CLASS_ID;
    use mxd2qgz::storage::object_refs::read_object_entry_10_1;

    let mut refs = RefTable10_1::new();
    refs.register(42);
    let mut definition = ENVELOPE_CLASS_ID.to_vec();
    definition.extend(51_u32.to_le_bytes());
    read_object_entry_10_1(&mut Reader::new(&definition), &mut refs).unwrap();

    let mut body = frame_body(4);
    let mut reference = [0x42; 16].to_vec();
    reference.extend(51_u32.to_le_bytes());
    body.splice(34..50, reference);
    let bytes = wrap(&body);
    let mut reader = Reader::new(&bytes);
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert_eq!(
        frame.base.geometry.entry,
        ObjectEntry10_1::BackReference {
            class_id: ENVELOPE_CLASS_ID,
            key: 51,
        }
    );
    assert!(frame.base.geometry.body.is_none());
    assert_eq!(frame.base.geometry_long, Some(0x1020_3040));
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn map_frame_nonpositive_collection_counts_preserve_the_base_frame() {
    let mut body = frame_body(4);
    body[22..26].copy_from_slice(&i32::MIN.to_le_bytes());
    body[26..30].copy_from_slice(&(-1_i32).to_le_bytes());
    let bytes = wrap(&body);
    let mut refs = RefTable10_1::new();
    refs.register(42);
    let mut reader = Reader::new(&bytes);
    let frame = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert_eq!(frame.grid_count_raw, i32::MIN);
    assert_eq!(frame.locator_count_raw, -1);
    assert!(frame.grids.is_empty());
    assert!(frame.locator_rectangles.is_empty());
    assert_eq!(frame.base.draft_mode_bits, 0xffff);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn map_frame_collections_use_registered_grid_and_locator_classes() {
    use mxd2qgz::maps::map_frame::LOCATOR_RECTANGLE_CLASS_ID;
    use mxd2qgz::maps::map_grid::GRATICULE_CLASS_ID;
    use mxd2qgz::storage::object_refs::read_object_entry_10_1;

    let mut body = frame_body(4);
    let mut collections = Vec::new();
    for key in [50_u32, 51] {
        collections.extend(1_i32.to_le_bytes());
        collections.extend([0x42; 16]);
        collections.extend(key.to_le_bytes());
    }
    body.splice(22..30, collections);
    let bytes = wrap(&body);
    for (grid, locator) in [
        (GRATICULE_CLASS_ID, LOCATOR_RECTANGLE_CLASS_ID),
        (MAP_CLASS_ID, LOCATOR_RECTANGLE_CLASS_ID),
        (GRATICULE_CLASS_ID, MAP_CLASS_ID),
    ] {
        let mut refs = RefTable10_1::new();
        refs.register(42);
        for (class_id, key) in [(grid, 50_u32), (locator, 51)] {
            let mut header = class_id.to_vec();
            header.extend(key.to_le_bytes());
            read_object_entry_10_1(&mut Reader::new(&header), &mut refs).unwrap();
        }
        let saved = refs.clone();
        let mut reader = Reader::new(&bytes);
        let result = read_page_layout_map_frame_10_1(&mut reader, 4096, &mut refs);
        if grid == MAP_CLASS_ID || locator == MAP_CLASS_ID {
            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::Unsupported);
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, saved);
        } else {
            let frame = result.unwrap();
            assert_eq!(
                frame.grids[0].entry,
                ObjectEntry10_1::BackReference {
                    class_id: grid,
                    key: 50
                }
            );
            assert_eq!(
                frame.locator_rectangles[0].entry,
                ObjectEntry10_1::BackReference {
                    class_id: locator,
                    key: 51
                }
            );
            assert_eq!(reader.remaining(), 0);
        }
    }
}
