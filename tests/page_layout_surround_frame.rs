use mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1;
use mxd2qgz::maps::page_layout_prefix::{MAP_FRAME_CLASS_ID, PAGE_LAYOUT_CLASS_ID};
use mxd2qgz::maps::surround_frame::{MAP_SURROUND_FRAME_CLASS_ID, SurroundFrameBounds10_1};
use mxd2qgz::storage::object_refs::ObjectEntry10_1;
use std::io::Read;

fn layout_root(count: u32) -> Vec<u8> {
    let mut bytes = PAGE_LAYOUT_CLASS_ID.to_vec();
    bytes.extend(1_u32.to_le_bytes());
    bytes.extend(9_u16.to_le_bytes());
    bytes.extend([0; 4]);
    bytes.extend(count.to_le_bytes());
    bytes
}

fn push_element(layout: &mut Vec<u8>, element: &[u8]) {
    layout.extend((element.len() as u32).to_le_bytes());
    layout.extend([0; 4]); // ExtraSafe 的常规 B=0 形式。
    layout.extend(element);
}

fn surround_frame(version: i16) -> Vec<u8> {
    let mut body = MAP_SURROUND_FRAME_CLASS_ID.to_vec();
    body.extend(7_u32.to_le_bytes());
    body.extend(version.to_le_bytes());
    body.extend([0; 48]); // IMapSurround、IMapFrame、IGeometry 或 IEnvelope。
    body.extend([0; 2 + 2 + 16]); // BaseFrameElement、RectShapeElement、空几何。
    body.extend(0_u16.to_le_bytes()); // ElementProperties v0。
    for _ in 0..2 {
        body.extend(2_u32.to_le_bytes());
        body.extend(0_u16.to_le_bytes());
    }
    body.extend([0; 4 + 16 + 16 + 16 + 2]); // 遗留 LONG、属性、边框、背景及 DraftMode。
    if version >= 3 {
        body.extend(12.5_f64.to_le_bytes());
        body.extend(15.25_f64.to_le_bytes());
    }
    body
}

#[test]
fn surround_frame_uses_version_gates_and_indirect_entries_occupy_whole_slots() {
    for version in [i16::MIN, -1, 0, 1, 2, 3] {
        let mut page = layout_root(3);
        push_element(&mut page, &surround_frame(version));
        let backref_offset = page.len();
        let mut backref = MAP_SURROUND_FRAME_CLASS_ID.to_vec();
        backref.extend(7_u32.to_le_bytes());
        push_element(&mut page, &backref);
        let null_offset = page.len();
        push_element(&mut page, &[0; 16]);

        let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
        let layout = report.page_layout.unwrap();
        assert!(layout.stop.is_none(), "{:?}", layout.stop);
        assert_eq!(layout.consumed_elements, 3);
        assert_eq!(layout.end_offset, page.len());
        assert_eq!(layout.surround_frames.len(), 1);
        let frame = &layout.surround_frames[0];
        assert_eq!(frame.version, version as u16);
        assert!(matches!(frame.bounds, SurroundFrameBounds10_1::Geometry(_)) == (version < 2));
        assert_eq!(
            frame.version_three_double_bits,
            (version >= 3).then(|| [12.5_f64.to_bits(), 15.25_f64.to_bits()])
        );
        assert_eq!(
            layout.indirect_elements,
            [(
                1,
                backref_offset,
                ObjectEntry10_1::BackReference {
                    class_id: MAP_SURROUND_FRAME_CLASS_ID,
                    key: 7
                }
            ),]
        );
        assert_eq!(
            layout.placeholder_elements,
            [(2, null_offset, page[null_offset..].to_vec())]
        );
    }
}

#[test]
fn layout_preserves_extra_safe_reference_suffix_and_reads_next_element() {
    let mut page = layout_root(3);
    push_element(&mut page, &surround_frame(2));
    let offset = page.len();
    let mut slot = MAP_SURROUND_FRAME_CLASS_ID.to_vec();
    slot.extend(7_u32.to_le_bytes());
    slot.push(0x99);
    push_element(&mut page, &slot);
    let next_offset = page.len();
    push_element(&mut page, &[0; 16]);
    let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
    let layout = report.page_layout.unwrap();
    assert_eq!(layout.consumed_elements, 3);
    assert_eq!(layout.end_offset, page.len());
    assert_eq!(
        layout.element_reference_suffixes,
        [(1, next_offset - 1, vec![0x99])]
    );
    assert_eq!(
        layout.placeholder_elements,
        [(2, next_offset, page[next_offset..].to_vec())]
    );
    let suffix = report
        .map
        .unknown_fields
        .iter()
        .find(|field| field.offset == (next_offset - 1) as u64)
        .unwrap();
    assert_eq!(suffix.bytes, [0x99]);

    let truncated = parse_mxd_streams_10_1(&[0; 8], Some(&page[..next_offset - 1])).unwrap();
    let layout = truncated.page_layout.unwrap();
    assert_eq!(layout.consumed_elements, 1);
    assert_eq!(layout.end_offset, offset);
    assert!(layout.element_reference_suffixes.is_empty());
}

#[test]
#[ignore = "需要本地提取的 ArcMap 10.1 安装介质 Utilities.mxd"]
fn utilities_consumes_all_page_elements_and_links_three_root_maps() {
    let mut document = cfb::open("temp/arcmap101-media-mxd/Utilities.mxd").unwrap();
    let mut read_stream = |name: &str| {
        let mut bytes = Vec::new();
        document
            .open_stream(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    };
    let maps = read_stream("/Maps");
    let page = read_stream("/PageLayout");
    let report = parse_mxd_streams_10_1(&maps, Some(&page)).unwrap();
    let layout = report.page_layout.unwrap();
    assert_eq!(report.map.frames.len(), 3);
    assert_eq!(layout.consumed_elements, 27);
    assert_eq!(layout.end_offset, 0x3669);
    assert_eq!(layout.surround_frames.len(), 5);
    assert!(!layout.rectangles.is_empty());
    assert!(layout.stop.is_none(), "{:?}", layout.stop);
    let nested: Vec<_> = layout
        .surround_frames
        .iter()
        .filter_map(|frame| frame.map_frame_body.as_deref())
        .map(|frame| {
            (
                frame.prefix.frame_key,
                frame.prefix.map_offset,
                frame.prefix.map,
            )
        })
        .collect();
    for (frame_key, offset, map_key) in [(165, 0x66, 29), (175, 0x491, 85), (183, 0x7c8, 124)] {
        assert!(nested.contains(&(
            frame_key,
            offset,
            ObjectEntry10_1::BackReference {
                class_id: mxd2qgz::maps::map_object::MAP_CLASS_ID,
                key: map_key
            }
        )));
    }
    assert!(layout.indirect_elements.iter().any(|(_, _, entry)| *entry
        == ObjectEntry10_1::BackReference {
            class_id: MAP_FRAME_CLASS_ID,
            key: 165
        }));
    let tail = layout.tail.as_ref().unwrap();
    assert_eq!(tail.element_values.len(), 27);
    assert_eq!(layout.print_settings_flag, Some(0));
    assert_eq!(layout.consumed_end_offset(), page.len());
    assert!(
        !report
            .map
            .unknown_fields
            .iter()
            .any(|f| f.stream == "/PageLayout")
    );
}

fn parse_media_document(name: &str) -> mxd2qgz::maps::mxd_maps::MapsParseReport10_1 {
    let mut doc = cfb::open(format!("temp/arcmap101-media-mxd/{name}")).unwrap();
    let mut read = |stream: &str| {
        let mut bytes = Vec::new();
        doc.open_stream(stream)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    };
    parse_mxd_streams_10_1(&read("/Maps"), Some(&read("/PageLayout"))).unwrap()
}

#[test]
#[ignore = "需要本地提取的 ArcMap 10.1 安装介质 MilitaryPortrait.mxd"]
fn military_portrait_reads_graticule_label_border_and_locator_reference() {
    use mxd2qgz::maps::map_grid::{
        DMS_GRID_LABEL_CLASS_ID, GRATICULE_CLASS_ID, SIMPLE_MAP_GRID_BORDER_CLASS_ID,
    };
    let mut doc = cfb::open("temp/arcmap101-media-mxd/MilitaryPortrait.mxd").unwrap();
    let mut read = |name: &str| {
        let mut bytes = Vec::new();
        doc.open_stream(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    };
    let maps = read("/Maps");
    let mut page = read("/PageLayout");
    let report = parse_mxd_streams_10_1(&maps, Some(&page)).unwrap();
    let layout = report.page_layout.unwrap();
    assert_eq!(layout.consumed_elements, 11);
    assert!(layout.stop.is_none(), "{:?}", layout.stop);
    assert_eq!(layout.end_offset, 0x288d);
    let inset = layout
        .surround_frames
        .iter()
        .filter_map(|frame| frame.map_frame_body.as_deref())
        .find(|frame| frame.prefix.frame_key == 158)
        .unwrap();
    let first = &inset.grids[0];
    assert_eq!(
        first.entry,
        ObjectEntry10_1::NewObject {
            class_id: GRATICULE_CLASS_ID,
            key: 159
        }
    );
    let grid = first.graticule.as_ref().unwrap();
    assert_eq!(grid.version, 5);
    assert_eq!(grid.base.name, "Grid");
    assert_eq!(
        grid.base.label,
        ObjectEntry10_1::NewObject {
            class_id: DMS_GRID_LABEL_CLASS_ID,
            key: 162
        }
    );
    assert_eq!(grid.base.dms_label.as_ref().unwrap().version, 4);
    assert!(
        inset
            .grids
            .iter()
            .filter_map(|slot| slot.graticule.as_ref())
            .any(|grid| grid.base.border
                == ObjectEntry10_1::NewObject {
                    class_id: SIMPLE_MAP_GRID_BORDER_CLASS_ID,
                    key: 192
                }
                && grid.base.simple_border.is_some())
    );
    let locator = layout
        .map_frames
        .iter()
        .flat_map(|frame| &frame.locator_rectangles)
        .find_map(|slot| slot.body.as_deref())
        .unwrap();
    assert_eq!(
        locator.map_frame,
        ObjectEntry10_1::BackReference {
            class_id: MAP_FRAME_CLASS_ID,
            key: 158
        }
    );

    let reference = 0x2232;
    assert_eq!(page[reference..reference + 16], MAP_FRAME_CLASS_ID);
    assert_eq!(page[reference + 16..reference + 20], 158_u32.to_le_bytes());
    page[reference..reference + 16].fill(0x42);
    let edited = parse_mxd_streams_10_1(&maps, Some(&page))
        .unwrap()
        .page_layout
        .unwrap();
    assert_eq!(edited.end_offset, layout.end_offset, "{:?}", edited.stop);
    assert_eq!(edited.map_frames, layout.map_frames);
    assert_eq!(edited.surround_frames, layout.surround_frames);

    // 键 159 是前面已经读取的 Graticule。
    page[reference..reference + 16].copy_from_slice(&MAP_FRAME_CLASS_ID);
    page[reference + 16..reference + 20].copy_from_slice(&159_u32.to_le_bytes());
    let edited = parse_mxd_streams_10_1(&maps, Some(&page)).unwrap();
    let stopped = edited.page_layout.unwrap();
    assert!(stopped.stop.is_some());
    assert!(stopped.end_offset < reference);
    assert_eq!(
        edited.map.unknown_fields.last().unwrap().bytes,
        page[stopped.end_offset..]
    );
}

#[test]
#[ignore = "需要本地提取的 ArcMap 10.1 安装介质 ConservationLandscape.mxd"]
fn conservation_landscape_reads_multilayer_line_inside_simple_fill() {
    use mxd2qgz::objects::display_symbol::SymbolSlotFull10_1 as Symbol;

    let report = parse_media_document("ConservationLandscape.mxd");
    let layout = report.page_layout.unwrap();
    assert_eq!(layout.consumed_elements, 9);
    assert!(layout.stop.is_none(), "{:?}", layout.stop);
    assert_eq!(layout.end_offset, 0x18a5);
    let filled = layout.rectangles.iter().any(|rect| {
        let Symbol::MultiLayerFillSymbol(body) = &rect.fill else { return false };
        body.layers.iter().any(|layer| matches!(layer,
            Symbol::SimpleFillSymbol(fill) if matches!(fill.outline, Symbol::MultiLayerLineSymbol(_))
        ))
    });
    assert!(filled);
}

#[test]
#[ignore = "需要本地 10.1 安装介质 CentralAmericaCaribbean.mxd"]
fn central_america_paragraph_reads_text_and_remaining_layout_elements() {
    let report = parse_media_document("CentralAmericaCaribbean.mxd");
    let layout = report.page_layout.unwrap();
    assert!(layout.stop.is_none(), "{:?}", layout.stop);
    assert_eq!(layout.consumed_elements, 7);
    assert_eq!(layout.end_offset, 0x1dc9);
    let (key, paragraph) = &layout.paragraphs[0];
    assert_eq!((*key, paragraph.version), (550, 1));
    let decoration = paragraph.decoration.as_ref().unwrap();
    assert_eq!(decoration.double_1, 5.0);
    assert_eq!(decoration.raw_int32, 1);
    assert_eq!(decoration.double_2, 5.0);
    assert_eq!(
        paragraph.text_element.as_ref().unwrap().text,
        r#"<dyn type="dataFrame" name="Central America & Caribbean" property="name"/>"#
    );
    assert_eq!(
        layout.map_frames[0].prefix.map,
        ObjectEntry10_1::BackReference {
            class_id: mxd2qgz::maps::map_object::MAP_CLASS_ID,
            key: 38
        }
    );
}

#[test]
#[ignore = "需要完整的 10.1 安装介质 66 个 MXD/MXT"]
fn desktop_media_reads_each_layout_object_and_empty_print_settings() {
    let media = std::fs::read_dir("temp/arcmap101-media-mxd").unwrap();
    let mut count = 0;
    for item in media {
        let path = item.unwrap().path();
        if !path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("mxd") || ext.eq_ignore_ascii_case("mxt"))
        {
            continue;
        }
        let filename = path.file_name().unwrap().to_str().unwrap();
        let report = parse_media_document(filename);
        let layout = report
            .page_layout
            .unwrap_or_else(|| panic!("{filename} 未进入布局"));
        assert!(layout.stop.is_none(), "{filename}: {:?}", layout.stop);
        assert_eq!(
            layout.consumed_elements, layout.prefix.element_count,
            "{filename}"
        );
        assert!(layout.tail.is_some(), "{filename}: {:?}", layout.tail_error);
        assert_eq!(layout.print_settings_flag, Some(0), "{filename}");
        assert!(
            layout.tail_error.is_none(),
            "{filename}: {:?}",
            layout.tail_error
        );
        assert!(
            !report
                .map
                .unknown_fields
                .iter()
                .any(|field| field.stream == "/PageLayout"),
            "{filename}"
        );
        count += 1;
    }
    assert_eq!(count, 66);
}

#[test]
fn group_element_reads_nested_surround_frame_and_shares_its_key() {
    use mxd2qgz::maps::group_element::{GROUP_ELEMENT_CLASS_ID, GroupChild10_1};

    let frame = surround_frame(2);
    let mut group = GROUP_ELEMENT_CLASS_ID.to_vec();
    group.extend(5_u32.to_le_bytes());
    group.extend(2_u16.to_le_bytes());
    group.extend(0_u16.to_le_bytes()); // ElementProperties v0。
    group.extend([2, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0]);
    group.extend([0; 4 + 16]);
    group.extend(1_u32.to_le_bytes());
    let frame_at = group.len();
    group.extend(&frame);
    group.extend(&frame[20 + 2 + 48..]); // 组尾的 BaseFrameElement。

    let mut page = layout_root(2);
    push_element(&mut page, &group);
    push_element(&mut page, &frame[..20]);
    let layout = parse_mxd_streams_10_1(&[0; 8], Some(&page))
        .unwrap()
        .page_layout
        .unwrap();
    assert_eq!(layout.consumed_elements, 2);
    assert!(matches!(
        &layout.groups[0].children[..],
        [GroupChild10_1::SurroundFrame(frame)] if frame.key == 7
    ));
    assert!(matches!(
        &layout.indirect_elements[..],
        [(1, _, ObjectEntry10_1::BackReference { key: 7, .. })]
    ));

    let mut bad_page = layout_root(1);
    let group_at = bad_page.len();
    push_element(&mut bad_page, &group[..frame_at + frame.len() - 1]);
    let layout = parse_mxd_streams_10_1(&[0; 8], Some(&bad_page))
        .unwrap()
        .page_layout
        .unwrap();
    assert_eq!(layout.consumed_elements, 0);
    assert_eq!(layout.end_offset, group_at);
    assert!(layout.groups.is_empty());
}

#[test]
fn group_element_reads_nested_map_frame_and_rolls_back_truncation() {
    use mxd2qgz::maps::group_element::{GROUP_ELEMENT_CLASS_ID, GroupChild10_1};

    let surrogate = surround_frame(2);
    let base = &surrogate[20 + 2 + 48..];
    let mut frame = MAP_FRAME_CLASS_ID.to_vec();
    frame.extend(7_u32.to_le_bytes());
    frame.extend(4_u16.to_le_bytes());
    frame.extend([0; 16 + 8]); // IMap 空槽和两个空集合。
    frame.extend(base);

    let mut group = GROUP_ELEMENT_CLASS_ID.to_vec();
    group.extend(5_u32.to_le_bytes());
    group.extend(2_u16.to_le_bytes());
    group.extend(0_u16.to_le_bytes()); // ElementProperties v0。
    group.extend([2, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0]);
    group.extend([0; 4 + 16]);
    group.extend(1_u32.to_le_bytes());
    group.extend(&frame);
    let frame_end = group.len();
    group.extend(base);

    let mut page = layout_root(2);
    push_element(&mut page, &group);
    push_element(&mut page, &frame[..20]);
    let layout = parse_mxd_streams_10_1(&[0; 8], Some(&page))
        .unwrap()
        .page_layout
        .unwrap();
    assert_eq!(layout.consumed_elements, 2);
    assert!(matches!(
        &layout.groups[0].children[..],
        [GroupChild10_1::MapFrame(frame)] if frame.prefix.frame_key == 7
    ));
    assert!(matches!(
        &layout.indirect_elements[..],
        [(1, _, ObjectEntry10_1::BackReference { key: 7, .. })]
    ));

    let mut bad_page = layout_root(1);
    let group_start = bad_page.len();
    push_element(&mut bad_page, &group[..frame_end - 1]);
    let layout = parse_mxd_streams_10_1(&[0; 8], Some(&bad_page))
        .unwrap()
        .page_layout
        .unwrap();
    assert_eq!(layout.end_offset, group_start);
    assert_eq!(layout.consumed_elements, 0);
    assert!(layout.groups.is_empty());
    assert!(layout.stop.is_some());
}

#[test]
fn scene_graphics_consumes_surround_frames_and_references() {
    use mxd2qgz::layers::graphics_3d::{GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1};
    use mxd2qgz::storage::object_refs::RefTable10_1;
    use mxd2qgz::stream::reader::Reader;

    for version in 0..=3 {
        let frame = surround_frame(version);
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend([2, 0, 0, 0, 0, 0]); // 空容器名称。
        bytes.extend([0; 8 + 16 + 16]); // 定宽前缀、关联图层和包络。
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend(&frame);
        bytes.extend(&frame[..20]);
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let layer =
            read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
                .unwrap();
        assert_eq!(
            layer.elements[0].surround_frame.as_ref().unwrap().version,
            version as u16
        );
        assert!(matches!(
            layer.elements[1].element.entry,
            ObjectEntry10_1::BackReference { key: 7, .. }
        ));
        assert_eq!(reader.position(), end);
    }
}

#[test]
fn standalone_layout_reads_new_map_inside_map_frame() {
    use mxd2qgz::maps::map_object::MapSuffixStop;

    let mut doc = cfb::open("samples/测试底图.mxd").unwrap();
    let mut maps = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut maps)
        .unwrap();
    let base = surround_frame(2);
    let base = &base[20 + 2 + 48..];
    let mut frame = MAP_FRAME_CLASS_ID.to_vec();
    frame.extend(0x7fff_u32.to_le_bytes());
    frame.extend(4_u16.to_le_bytes());
    frame.extend(&maps[4..0x25569]); // 参考 Map 的完整对象头和对象体。
    frame.extend([0; 8]);
    frame.extend(base);
    let mut reference = MAP_FRAME_CLASS_ID.to_vec();
    reference.extend(0x8000_u32.to_le_bytes());
    reference.extend(4_u16.to_le_bytes());
    reference.extend(&maps[4..24]); // 同一 Map 的对象头。
    reference.extend([0; 8]);
    reference.extend(base);
    let mut page = layout_root(2);
    push_element(&mut page, &frame);
    push_element(&mut page, &reference);

    let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
    assert!(
        report
            .diagnostics
            .iter()
            .any(|note| note.contains("回引 内嵌 Map 键"))
    );
    let layout = report.page_layout.unwrap();
    assert!(layout.stop.is_none(), "{:?}", layout.stop);
    assert_eq!(layout.end_offset, page.len());
    let parsed_frame = &layout.map_frames[0];
    assert!(matches!(
        parsed_frame.prefix.map,
        ObjectEntry10_1::NewObject { .. }
    ));
    assert_eq!(
        parsed_frame.embedded_map.as_ref().unwrap().1.stop,
        MapSuffixStop::Complete
    );
    assert!(matches!(
        layout.map_frames[1].prefix.map,
        ObjectEntry10_1::BackReference { .. }
    ));
    assert!(layout.map_frames[1].embedded_map.is_none());

    use mxd2qgz::maps::page_layout_prefix::read_page_layout_map_frame_10_1;
    use mxd2qgz::storage::object_refs::RefTable10_1;
    use mxd2qgz::stream::reader::Reader;

    let mut truncated = Vec::new();
    push_element(&mut truncated, &frame[..frame.len() - 1]);
    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&truncated);
    assert!(read_page_layout_map_frame_10_1(&mut reader, 4_000_000, &mut refs).is_err());
    assert!(refs.is_empty());
    assert_eq!(reader.position(), 0);
}

#[test]
fn surround_frame_rejects_map_frame_reference_to_another_class() {
    let mut page = layout_root(2);
    push_element(&mut page, &surround_frame(2));
    let boundary = page.len();

    let mut second = surround_frame(2);
    second[16..20].copy_from_slice(&8_u32.to_le_bytes());
    let mut frame_reference = MAP_FRAME_CLASS_ID.to_vec();
    frame_reference.extend(7_u32.to_le_bytes());
    second.splice(38..54, frame_reference);
    push_element(&mut page, &second);

    let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
    let layout = report.page_layout.unwrap();
    assert_eq!(layout.consumed_elements, 1);
    assert_eq!(layout.end_offset, boundary);
    assert_eq!(
        report.map.unknown_fields.last().unwrap().bytes,
        page[boundary..]
    );
}

#[test]
fn surround_frame_rejects_surround_reference_to_another_class() {
    use mxd2qgz::maps::map_surround_family::MARKER_NORTH_ARROW_CLASS_ID;

    let mut page = layout_root(2);
    push_element(&mut page, &surround_frame(2));
    let boundary = page.len();

    let mut second = surround_frame(2);
    second[16..20].copy_from_slice(&8_u32.to_le_bytes());
    let mut reference = MARKER_NORTH_ARROW_CLASS_ID.to_vec();
    reference.extend(7_u32.to_le_bytes());
    second.splice(22..38, reference);
    push_element(&mut page, &second);

    let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
    let layout = report.page_layout.unwrap();
    assert_eq!(layout.consumed_elements, 1);
    assert_eq!(layout.end_offset, boundary);
    assert_eq!(
        report.map.unknown_fields.last().unwrap().bytes,
        page[boundary..]
    );
}
