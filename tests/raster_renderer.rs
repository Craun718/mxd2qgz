//! RasterRGBRenderer 旧版与全色锐化记录的合成布局，依据 10.1 Load 指令。

use mxd2qgz::layers::raster_renderer::{
    RASTER_RGB_RENDERER_CLASS_ID, RasterRendererPrefix10_1, RasterRendererSharedLayout10_1,
    read_raster_renderer_slot_prefix_10_1,
};
use mxd2qgz::objects::display_color::{ColorSlot10_1, RGB_COLOR_CLASS_ID};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

fn rgb(version: i32) -> (Vec<u8>, usize, usize) {
    let mut bytes = RASTER_RGB_RENDERER_CLASS_ID.to_vec();
    bytes.extend(71_u32.to_le_bytes());
    bytes.extend(version.to_le_bytes());
    for band in [2_i32, 1, 0] {
        bytes.extend(band.to_le_bytes());
    }
    bytes.extend([0x73; 0x930]);
    bytes.push(0xd1);
    bytes.extend(8_i32.to_le_bytes());
    bytes.extend(1.5_f64.to_le_bytes());
    bytes.extend(123_i32.to_le_bytes());
    bytes.extend((-42_i32).to_le_bytes());
    for i in 1..=8 {
        bytes.extend(f64::from(i).to_le_bytes());
    }
    let color_offset = bytes.len();
    bytes.extend(RGB_COLOR_CLASS_ID);
    bytes.extend(72_u32.to_le_bytes());
    bytes.extend([1, 0, 1, 0, 1, 0x01, 0x02, 0x03, 0x04, 0x4a, 0x4b]);
    bytes.extend(RGB_COLOR_CLASS_ID);
    bytes.extend(72_u32.to_le_bytes());
    let shared_offset = bytes.len();
    bytes.extend(1_i32.to_le_bytes()); // 旧共用体不含 RasterIO。
    for value in [15_i32, 5, 17, 18] {
        bytes.extend(value.to_le_bytes());
    }
    if version >= 2 {
        bytes.extend((-1_i32).to_le_bytes());
    }
    if version >= 3 {
        bytes.extend(90_i32.to_le_bytes());
    }
    (bytes, color_offset, shared_offset)
}

#[test]
fn rgb_versions_one_to_three_consume_their_own_tail_and_keep_color_keys() {
    for version in 1..=3 {
        let (mut bytes, _, _) = rgb(version);
        bytes.push(0x4f);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let slot = read_raster_renderer_slot_prefix_10_1(&mut reader, 4096, &mut table).unwrap();
        assert!(slot.body.as_ref().unwrap().is_complete());
        let Some(RasterRendererPrefix10_1::LegacyRgb(body)) = slot.body else {
            panic!("应读到旧版 RGB");
        };
        assert_eq!(body.version, version);
        assert_eq!(body.bands, [2, 1, 0]);
        assert_eq!(body.lookup_table, [0x73; 0x930]);
        assert_eq!(body.byte_value, 0xd1);
        assert_eq!(body.double_value, 1.5);
        assert_eq!(body.longs, [8, 123, -42]);
        assert_eq!(
            body.double_groups,
            [[1.0, 2.0, 3.0, 4.0], [5.0, 6.0, 7.0, 8.0]]
        );
        assert!(matches!(body.colors[0], ColorSlot10_1::Color { .. }));
        assert_eq!(
            body.colors[1],
            ColorSlot10_1::BackReference {
                key: 72,
                class_id: RGB_COLOR_CLASS_ID
            },
        );
        assert_eq!(body.shared.version, 1);
        assert_eq!(
            body.shared.layout,
            RasterRendererSharedLayout10_1::Legacy([15, 5, 17, 18])
        );
        assert_eq!(body.trailing_flag, (version >= 2).then_some(-1));
        assert_eq!(body.trailing_long, (version >= 3).then_some(90));
        assert!(table.contains(71) && table.contains(72));
        assert_eq!(reader.read_bytes(1).unwrap(), [0x4f]);
    }
}

#[test]
fn malformed_legacy_rgb_keeps_outer_and_color_keys_uncommitted() {
    fn assert_transactional_failure(bytes: &[u8]) {
        let mut reader = Reader::new(bytes);
        let mut table = RefTable10_1::new();
        assert!(read_raster_renderer_slot_prefix_10_1(&mut reader, 4096, &mut table).is_err());
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }

    let (bytes, color, shared) = rgb(3);
    for cut in 0..bytes.len() {
        assert_transactional_failure(&bytes[..cut]);
    }
    let mut unknown = bytes.clone();
    unknown[color] ^= 0x80;
    assert_transactional_failure(&unknown);
    let mut invalid_version = bytes.clone();
    invalid_version[shared..shared + 4].copy_from_slice(&0_i32.to_le_bytes());
    assert_transactional_failure(&invalid_version);
    invalid_version = bytes;
    invalid_version[20..24].copy_from_slice(&0_i32.to_le_bytes());
    assert_transactional_failure(&invalid_version);
}

fn record(bytes: &mut Vec<u8>, tag: i32, value: &[u8]) {
    bytes.extend(tag.to_le_bytes());
    bytes.extend((value.len() as i32).to_le_bytes());
    bytes.extend(value);
}

fn tagged_rgb_filter_prefix() -> Vec<u8> {
    use mxd2qgz::layers::raster_renderer::PANSHARPENING_FILTER_CLASS_ID;

    let mut bytes = RASTER_RGB_RENDERER_CLASS_ID.to_vec();
    bytes.extend(71_u32.to_le_bytes());
    bytes.extend(4_i32.to_le_bytes());
    record(&mut bytes, 0x00ff_00ff, &[]);
    for (tag, width) in (1..=11).zip([4, 4, 4, 0x930, 1, 4, 8, 4, 4, 32, 2]) {
        record(&mut bytes, tag, &vec![0; width]);
    }
    for (tag, class_id) in [(12_i32, [0; 16]), (18, PANSHARPENING_FILTER_CLASS_ID)] {
        bytes.extend(tag.to_le_bytes());
        bytes.extend((-1_i32).to_le_bytes());
        bytes.extend(class_id);
    }
    bytes.extend(1_i32.to_le_bytes());
    bytes
}

fn finish_tagged_rgb_filter(bytes: &mut Vec<u8>) {
    bytes.extend(3_i32.to_le_bytes());
    for weight in [0.125_f64, 0.25, 0.5, 0.75] {
        bytes.extend(weight.to_le_bytes());
    }
    record(bytes, 19, &63_i32.to_le_bytes());
    bytes.extend([0xff; 8]);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]); // 共用体 v1。
}

#[test]
fn tagged_rgb_filter_consumes_empty_rasters_and_preserves_next_record() {
    use mxd2qgz::layers::raster_renderer::{PansharpeningFilter10_1, PansharpeningSlot10_1};

    let mut bytes = tagged_rgb_filter_prefix();
    let raster_a = bytes.len();
    bytes.extend([0; 32]);
    finish_tagged_rgb_filter(&mut bytes);
    let body_end = bytes.len();
    bytes.push(0x65);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let slot = read_raster_renderer_slot_prefix_10_1(&mut reader, 4096, &mut table).unwrap();
    let Some(RasterRendererPrefix10_1::Rgb {
        tail: Some(tail), ..
    }) = slot.body
    else {
        panic!("应读完含过滤器的 RGB 对象");
    };
    assert_eq!(
        tail.pansharpening,
        Some(PansharpeningSlot10_1::Filter(Box::new(
            PansharpeningFilter10_1 {
                version: 1,
                rasters: [None, None],
                method: 3,
                weights: [0.125, 0.25, 0.5, 0.75],
            }
        ))),
    );
    assert_eq!(tail.tag19, Some(63));
    assert_eq!(reader.position(), body_end);
    assert_eq!(reader.read_bytes(1).unwrap(), [0x65]);
    assert_eq!(table.len(), 1); // 过滤器在 RasterIO 内，不带共享键。

    let mut truncated = Reader::new(&bytes[..raster_a + 31]);
    let mut truncated_table = RefTable10_1::new();
    let prefix = read_raster_renderer_slot_prefix_10_1(&mut truncated, 4096, &mut truncated_table)
        .unwrap()
        .body
        .unwrap();
    assert!(!prefix.is_complete());
    let mut nonempty = bytes.clone();
    nonempty[raster_a + 16] = 0x7e;
    let mut unsupported = Reader::new(&nonempty);
    let mut unsupported_table = RefTable10_1::new();
    let partial =
        read_raster_renderer_slot_prefix_10_1(&mut unsupported, 4096, &mut unsupported_table)
            .unwrap()
            .body
            .unwrap();
    assert!(!partial.is_complete());
    assert_eq!(unsupported.position(), truncated.position());
    assert_eq!(unsupported_table, truncated_table);
}

#[test]
fn tagged_rgb_filter_reads_raster_names_and_references_with_the_original_stream() {
    use mxd2qgz::layers::fgdb_raster_dataset_name::FGDB_RASTER_DATASET_NAME_CLASS_ID;
    use mxd2qgz::layers::raster_dataset_name::{
        ACCESS_RASTER_DATASET_NAME_CLASS_ID, RASTER_DATASET_NAME_CLASS_ID,
        RASTER_DATASET_NAME83_CLASS_ID, SDE_RASTER_DATASET_NAME_CLASS_ID,
    };
    use mxd2qgz::layers::raster_renderer::{LoadedRasterName10_1, PansharpeningSlot10_1};
    use mxd2qgz::objects::envelope::ENVELOPE_CLASS_ID;
    use mxd2qgz::objects::geodata_xform::IDENTITY_XFORM_CLASS_ID;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    fn entry(bytes: &mut Vec<u8>, class_id: [u8; 16], key: u32) {
        bytes.extend(class_id);
        bytes.extend(key.to_le_bytes());
    }
    fn string(bytes: &mut Vec<u8>, text: &str) {
        let units: Vec<u16> = text.encode_utf16().chain([0]).collect();
        bytes.extend(((units.len() * 2) as u32).to_le_bytes());
        bytes.extend(units.into_iter().flat_map(u16::to_le_bytes));
    }

    for class_id in [
        RASTER_DATASET_NAME_CLASS_ID,
        RASTER_DATASET_NAME83_CLASS_ID,
        FGDB_RASTER_DATASET_NAME_CLASS_ID,
        ACCESS_RASTER_DATASET_NAME_CLASS_ID,
        SDE_RASTER_DATASET_NAME_CLASS_ID,
    ] {
        let mut bytes = tagged_rgb_filter_prefix();
        let first_raster = bytes.len();
        entry(&mut bytes, class_id, 72);
        bytes.extend(4_i32.to_le_bytes());
        for text in ["raster", "", "", ""] {
            string(&mut bytes, text);
        }
        bytes.extend([0; 16]); // 空工作空间。
        bytes.extend([0; 8]); // 名称版本 4 的两个 int32。
        if matches!(
            class_id,
            FGDB_RASTER_DATASET_NAME_CLASS_ID
                | ACCESS_RASTER_DATASET_NAME_CLASS_ID
                | SDE_RASTER_DATASET_NAME_CLASS_ID
        ) {
            record(&mut bytes, 0xff00fa, &[]);
            record(&mut bytes, 1, &29_i32.to_le_bytes());
            record(&mut bytes, 2, &[b'a', 0, 0, 0]);
            record(&mut bytes, 3, &[b'b', 0, 0, 0]);
            record(&mut bytes, 4, &7_i32.to_le_bytes());
            record(&mut bytes, 5, &[0; 2]);
            bytes.extend(6_i32.to_le_bytes());
            bytes.extend([0xff; 4]);
            bytes.extend([0; 16]);
            if class_id == ACCESS_RASTER_DATASET_NAME_CLASS_ID {
                record(&mut bytes, 7, &1_i32.to_le_bytes());
            }
            bytes.extend([0xff; 8]); // 数据库名称自身的 RasterIO 结束对。
        } else if class_id == RASTER_DATASET_NAME83_CLASS_ID {
            record(&mut bytes, 0xff00fa, &[]);
            record(&mut bytes, 1, &31_i32.to_le_bytes());
            record(&mut bytes, 2, &[b'c', 0, 0, 0]);
            bytes.extend([0xff; 8]);
        }

        let bands = bytes.len();
        bytes.extend(2_i32.to_le_bytes());
        string(&mut bytes, "红");
        string(&mut bytes, "绿");
        entry(&mut bytes, IDENTITY_XFORM_CLASS_ID, 73);
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend([0; 16]); // IdentityXform 的空坐标参考。
        entry(&mut bytes, ENVELOPE_CLASS_ID, 74);
        bytes.extend(1_u16.to_le_bytes());
        for ordinate in [1.0_f64, 2.0, 3.0, 4.0] {
            bytes.extend(ordinate.to_le_bytes());
        }
        let reference = bytes.len();
        entry(&mut bytes, [0x7d; 16], 72); // 以登记对象的类为准。
        let second_band_count = if class_id == RASTER_DATASET_NAME83_CLASS_ID {
            -1_i32
        } else {
            1
        };
        bytes.extend(second_band_count.to_le_bytes());
        if second_band_count > 0 {
            string(&mut bytes, "全色");
        }
        entry(&mut bytes, IDENTITY_XFORM_CLASS_ID, 73);
        entry(&mut bytes, ENVELOPE_CLASS_ID, 74);
        finish_tagged_rgb_filter(&mut bytes);
        let end = bytes.len();
        bytes.push(0x6f);

        let mut r = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let body = read_raster_renderer_slot_prefix_10_1(&mut r, 4096, &mut refs)
            .unwrap()
            .body
            .unwrap();
        let RasterRendererPrefix10_1::Rgb {
            tail: Some(tail), ..
        } = body
        else {
            panic!("应完整消费包含两个非空栅格槽的 RGB：{body:?}");
        };
        let Some(PansharpeningSlot10_1::Filter(filter)) = tail.pansharpening else {
            panic!("应读到全色锐化体");
        };
        let first = filter.rasters[0].as_ref().unwrap();
        match first.name_body.as_ref().unwrap() {
            LoadedRasterName10_1::Dataset(name) => assert_eq!(name.key, 72),
            LoadedRasterName10_1::Dataset83 { name, records } => {
                assert_eq!(name.key, 72);
                let records = records.as_ref().unwrap();
                assert_eq!((records.first_long, records.text.as_str()), (31, "c"));
            }
            LoadedRasterName10_1::FileGeodatabase { name, records } => {
                assert_eq!(name.delegated_name.key, 72);
                assert_eq!(records.tag1_value, 29);
            }
            LoadedRasterName10_1::Database(name) => {
                assert_eq!(name.delegated_name.key, 72);
                let records = name.records.as_ref().unwrap();
                assert_eq!(records.longs_1_4, [29, 7]);
                assert_eq!(records.strings_2_3_5, ["a", "b", ""]);
                assert!(records.tag6_name.is_none());
                assert_eq!(
                    records.tag7_long,
                    (class_id == ACCESS_RASTER_DATASET_NAME_CLASS_ID).then_some(1)
                );
            }
            other => panic!("预期测试构造的栅格数据集名称，读取为 {other:?}"),
        }
        assert_eq!(first.declared_band_count, 2);
        assert_eq!(first.band_names, ["红", "绿"]);
        assert!(first.transform.body.is_some());
        assert_eq!(
            first.extent.body.as_ref().unwrap().coords,
            [1.0, 2.0, 3.0, 4.0]
        );
        let second = filter.rasters[1].as_ref().unwrap();
        assert_eq!(
            second.name_entry,
            ObjectEntry10_1::BackReference { class_id, key: 72 }
        );
        assert!(second.name_body.is_none());
        assert_eq!(second.declared_band_count, second_band_count);
        if second_band_count > 0 {
            assert_eq!(second.band_names, ["全色"]);
        } else {
            assert!(second.band_names.is_empty());
        }
        assert!(matches!(
            second.transform.entry,
            ObjectEntry10_1::BackReference { key: 73, .. }
        ));
        assert!(matches!(
            second.extent.entry,
            ObjectEntry10_1::BackReference { key: 74, .. }
        ));
        assert_eq!(tail.tag19, Some(63));
        assert_eq!(refs.len(), 4);
        assert_eq!(r.position(), end);

        let assert_uncommitted = |input: &[u8]| {
            let mut r = Reader::new(input);
            let mut refs = RefTable10_1::new();
            let body = read_raster_renderer_slot_prefix_10_1(&mut r, 4096, &mut refs)
                .unwrap()
                .body
                .unwrap();
            assert!(!body.is_complete());
            assert!(r.position() < first_raster);
            assert!(refs.contains(71) && refs.len() == 1);
        };
        let mut wrong_class = bytes.clone();
        wrong_class[reference..reference + 16].copy_from_slice(&class_id);
        wrong_class[reference + 16..reference + 20].copy_from_slice(&73_u32.to_le_bytes());
        assert_uncommitted(&wrong_class);
        for cut in first_raster..end {
            assert_uncommitted(&bytes[..cut]);
        }
        bytes[bands..bands + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        assert_uncommitted(&bytes);
    }
}

#[test]
fn tagged_rgb_filter_reuses_cache_and_netcdf_name_children_across_slots() {
    use mxd2qgz::layers::raster_dataset_name::{
        CACHE_RASTER_DATASET_NAME_CLASS_ID, NETCDF_RASTER_DATASET_NAME_CLASS_ID,
        RasterDatasetWorkspaceSlot10_1,
    };
    use mxd2qgz::layers::raster_renderer::{LoadedRasterName10_1, PansharpeningSlot10_1};
    use mxd2qgz::objects::envelope::ENVELOPE_CLASS_ID;
    use mxd2qgz::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let empty_string = [2, 0, 0, 0, 0, 0];
    let mut bytes = tagged_rgb_filter_prefix();
    let first_name_offset = bytes.len();
    bytes.extend(CACHE_RASTER_DATASET_NAME_CLASS_ID);
    bytes.extend(72_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(empty_string.repeat(2));
    bytes.extend(WORKSPACE_NAME_CLASS_ID);
    bytes.extend(73_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(empty_string.repeat(3));
    bytes.extend([0; 21]);
    bytes.extend([0; 16]); // 缓存名称的空瓦片读取器。
    bytes.extend(ENVELOPE_CLASS_ID);
    bytes.extend(74_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    for ordinate in [1.0_f64, 2.0, 3.0, 4.0] {
        bytes.extend(ordinate.to_le_bytes());
    }
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend([0; 16]); // 空栅格变换。
    bytes.extend(ENVELOPE_CLASS_ID);
    bytes.extend(74_u32.to_le_bytes());

    let second_name_offset = bytes.len();
    bytes.extend(NETCDF_RASTER_DATASET_NAME_CLASS_ID);
    bytes.extend(75_u32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(empty_string.repeat(8));
    bytes.extend([0; 32]); // 空名称和值数组。
    bytes.extend(WORKSPACE_NAME_CLASS_ID);
    bytes.extend(73_u32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(ENVELOPE_CLASS_ID);
    bytes.extend(74_u32.to_le_bytes());
    finish_tagged_rgb_filter(&mut bytes);
    let end = bytes.len();
    bytes.push(0x6f);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let renderer = read_raster_renderer_slot_prefix_10_1(&mut reader, 4096, &mut refs)
        .unwrap()
        .body
        .unwrap();
    let RasterRendererPrefix10_1::Rgb {
        tail: Some(tail), ..
    } = renderer
    else {
        panic!("应完整读取 RGB 尾段");
    };
    let Some(PansharpeningSlot10_1::Filter(filter)) = tail.pansharpening else {
        panic!("应读取全色锐化过滤器");
    };
    let cache = filter.rasters[0].as_ref().unwrap();
    let netcdf = filter.rasters[1].as_ref().unwrap();
    assert!(matches!(
        cache.name_body.as_ref(),
        Some(LoadedRasterName10_1::Cache(name)) if name.envelope.body.is_some()
    ));
    assert!(matches!(
        netcdf.name_body.as_ref(),
        Some(LoadedRasterName10_1::NetCdf(name))
            if matches!(name.workspace, RasterDatasetWorkspaceSlot10_1::BackReference { key: 73, .. })
    ));
    for raster in [cache, netcdf] {
        assert_eq!(
            raster.extent.entry,
            ObjectEntry10_1::BackReference {
                class_id: ENVELOPE_CLASS_ID,
                key: 74,
            }
        );
    }
    assert_eq!(tail.tag19, Some(63));
    assert_eq!(reader.position(), end);
    assert_eq!(refs.len(), 5);

    let mut unknown = bytes.clone();
    unknown[second_name_offset] ^= 0xff;
    for invalid in [&bytes[..second_name_offset + 27], &unknown[..]] {
        let mut reader = Reader::new(invalid);
        let mut refs = RefTable10_1::new();
        let renderer = read_raster_renderer_slot_prefix_10_1(&mut reader, 4096, &mut refs)
            .unwrap()
            .body
            .unwrap();
        assert!(!renderer.is_complete());
        assert!(reader.position() < first_name_offset);
        assert!(refs.contains(71) && refs.len() == 1);
    }
}

#[test]
fn simple_raster_renderer_v1_and_v2_stop_after_their_shared_body() {
    use mxd2qgz::layers::raster_renderer::SIMPLE_RASTER_RENDERER_CLASS_ID;

    for version in [1_i32, 2] {
        let mut bytes = SIMPLE_RASTER_RENDERER_CLASS_ID.to_vec();
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(version.to_le_bytes());
        if version == 2 {
            bytes.extend(0x8123_u16.to_le_bytes());
        }
        bytes.extend(1_i32.to_le_bytes()); // 共用体版本。
        for value in [2_i32, 3, 4, 5] {
            bytes.extend(value.to_le_bytes());
        }
        let mut reader = Reader::new(&bytes[..bytes.len() - 1]);
        let mut table = RefTable10_1::new();
        assert!(read_raster_renderer_slot_prefix_10_1(&mut reader, 1024, &mut table).is_err());
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());

        bytes.push(0x7e);
        let mut reader = Reader::new(&bytes);
        let slot = read_raster_renderer_slot_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
        assert!(slot.body.as_ref().unwrap().is_complete());
        let Some(RasterRendererPrefix10_1::Simple {
            version: actual,
            short,
            shared,
        }) = slot.body
        else {
            panic!("应读到 SimpleRasterRenderer");
        };
        assert_eq!(actual, version);
        assert_eq!(short, (version == 2).then_some(0x8123));
        assert_eq!(
            shared.layout,
            RasterRendererSharedLayout10_1::Legacy([2, 3, 4, 5])
        );
        assert_eq!(reader.read_bytes(1).unwrap(), [0x7e]);
        assert!(table.contains(76));
    }
}

#[test]
fn discrete_renderer_preserves_unsigned_colors_and_the_ramp_key() {
    use mxd2qgz::layers::raster_renderer::DISCRETE_COLOR_RENDERER_CLASS_ID;
    use mxd2qgz::objects::color_ramp::{ColorRampSlot10_1, RANDOM_COLOR_RAMP_CLASS_ID};

    let mut bytes = DISCRETE_COLOR_RENDERER_CLASS_ID.to_vec();
    bytes.extend(71_u32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    let count_offset = bytes.len();
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(0x80a1_23b4_u32.to_le_bytes());
    bytes.extend(0xffff_ffff_u32.to_le_bytes());
    bytes.extend(4_u32.to_le_bytes()); // AfCore String：4 字节含终止零。
    bytes.extend(('色' as u16).to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    let ramp_offset = bytes.len();
    bytes.extend(RANDOM_COLOR_RAMP_CLASS_ID);
    bytes.extend(72_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes()); // 自由函数 BSTR 布局中的空名称。
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(18_i32.to_le_bytes());
    bytes.extend([0; 12]);
    bytes.extend(1_i32.to_le_bytes()); // 共用体版本。
    bytes.extend([0; 16]);
    let end = bytes.len();
    bytes.push(0x76);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let slot = read_raster_renderer_slot_prefix_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(slot.body.as_ref().unwrap().is_complete());
    let Some(RasterRendererPrefix10_1::Discrete(body)) = slot.body else {
        panic!("应读到离散色渲染器");
    };
    assert_eq!(body.colors, [0x80a1_23b4, 0xffff_ffff]);
    assert_eq!(body.text, "色");
    assert!(
        matches!(body.ramp, ColorRampSlot10_1::Random(ramp) if ramp.key == 72 && ramp.seed_int32 == 18)
    );
    assert_eq!(body.shared.version, 1);
    assert_eq!(reader.position(), end);
    assert_eq!(reader.read_bytes(1).unwrap(), [0x76]);
    assert!(refs.contains(71) && refs.contains(72));

    let assert_failure = |input: &[u8], budget| {
        let mut reader = Reader::new(input);
        let mut refs = RefTable10_1::new();
        assert!(read_raster_renderer_slot_prefix_10_1(&mut reader, budget, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    };
    for cut in 0..end {
        assert_failure(&bytes[..cut], 1024);
    }
    assert_failure(&bytes, 7);
    let mut unknown = bytes.clone();
    unknown[ramp_offset] ^= 1;
    assert_failure(&unknown, 1024);
    for count in [-1_i32, i32::MAX] {
        bytes[count_offset..count_offset + 4].copy_from_slice(&count.to_le_bytes());
        assert_failure(&bytes, 1024);
    }
}

#[test]
fn raster_unique_value_reads_both_layouts_and_nested_values() {
    use mxd2qgz::layers::raster_unique_value::{
        RASTER_UNIQUE_VALUE_COLORS_CLASS_ID, RASTER_UNIQUE_VALUE_RENDERER_CLASS_ID,
        RasterUniqueValueColorKey10_1, RasterUniqueValueLayout10_1, UNIQUE_VALUES_CLASS_ID,
    };
    use mxd2qgz::objects::color_ramp::ColorRampSlot10_1;
    use mxd2qgz::objects::geometry::{GEOMETRY_BAG_CLASS_ID, GeometryBody10_1, POINT_CLASS_ID};
    use mxd2qgz::objects::property_set::{
        PROPERTY_SET_CLASS_ID, PropertyValue10_1,
        PropertyValueObjectStream10_1::{self, Inline},
        VariantArrayObjectValues10_1,
    };
    use mxd2qgz::objects::string_array::STR_ARRAY_CLASS_ID;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;
    use mxd2qgz::stream::variant_array::ClassVariantArray10_1;

    fn string(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(value.to_le_bytes());
        bytes.extend(0_u16.to_le_bytes());
    }

    for (version, legacy_colors) in [
        (1_i32, None),
        (1, Some(0_i32)),
        (2, Some(1)),
        (3, None),
        (4, None),
    ] {
        let legacy = version < 3;
        let mut bytes = RASTER_UNIQUE_VALUE_RENDERER_CLASS_ID.to_vec();
        bytes.extend(81_u32.to_le_bytes());
        bytes.extend(version.to_le_bytes());
        for c in if legacy { &b"ab"[..] } else { &b"abc"[..] } {
            string(&mut bytes, u16::from(*c));
        }
        bytes.extend(7_u16.to_le_bytes());
        bytes.extend([0; 16]); // ISymbol。
        string(&mut bytes, b'x'.into());
        if legacy {
            bytes.extend(17_i32.to_le_bytes());
        }
        bytes.extend(1_i32.to_le_bytes()); // 一个空 ILegendGroup 槽。
        bytes.extend([0; 16]);
        if legacy {
            bytes.extend([0; 16]); // ILegendItem。
        }
        bytes.extend(1_i32.to_le_bytes()); // 类别数。
        bytes.extend(0xfe76_1234_u32.to_le_bytes());
        let value_count = bytes.len();
        bytes.extend((if version == 4 { 6_i32 } else { 2 }).to_le_bytes());
        bytes.extend(3_u16.to_le_bytes());
        bytes.extend([0; 14]);
        bytes.extend(19_i32.to_le_bytes());
        bytes.extend(0x2011_u16.to_le_bytes()); // 字节数组。
        bytes.extend([0; 14]);
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend([0x80, 0xff]);
        if version == 4 {
            bytes.extend(9_u16.to_le_bytes());
            bytes.extend([0; 30]); // 保留区及空 GUID，无键。
            bytes.extend(13_u16.to_le_bytes());
            bytes.extend([0; 14]);
            bytes.extend(PROPERTY_SET_CLASS_ID);
            bytes.extend(83_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(13_u16.to_le_bytes());
            bytes.extend([0; 14]);
            bytes.extend(STR_ARRAY_CLASS_ID);
            bytes.extend(84_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            string(&mut bytes, '乙' as u16);
            bytes.extend(13_u16.to_le_bytes());
            bytes.extend([0; 14]);
            bytes.extend(GEOMETRY_BAG_CLASS_ID);
            bytes.extend(85_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend([0; 16]); // 缓存几何。
            bytes.extend(POINT_CLASS_ID);
            bytes.extend(86_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(20_i32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(14.5_f64.to_le_bytes());
            bytes.extend(27.5_f64.to_le_bytes());
            bytes.extend([0; 17]); // 空空间参考和标志字节。
        }
        bytes.extend(9_u16.to_le_bytes());
        let tail_object = bytes.len();
        if legacy {
            if let Some(string_flag) = legacy_colors {
                bytes.extend(RASTER_UNIQUE_VALUE_COLORS_CLASS_ID);
                bytes.extend(82_u32.to_le_bytes());
                bytes.extend(1_i32.to_le_bytes());
                bytes.extend(1_i32.to_le_bytes());
                bytes.extend(string_flag.to_le_bytes());
                if string_flag == 0 {
                    bytes.extend(9.25_f64.to_le_bytes());
                } else {
                    string(&mut bytes, '甲' as u16);
                }
                bytes.extend(2_i32.to_le_bytes());
                bytes.extend(0xaa00_ff11_u32.to_le_bytes());
                bytes.extend(0x0000_1234_u32.to_le_bytes());
            } else {
                bytes.extend([0; 16]);
            }
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(13.25_f64.to_le_bytes());
            bytes.extend(23_i32.to_le_bytes());
        } else {
            bytes.extend(UNIQUE_VALUES_CLASS_ID);
            bytes.extend(82_u32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend((if version == 4 { 4_i32 } else { 1 }).to_le_bytes());
            bytes.extend(8_u16.to_le_bytes()); // Variant 的标量 BSTR。
            bytes.extend([0; 14]);
            string(&mut bytes, '项' as u16);
            bytes.extend(33_i32.to_le_bytes());
            if version == 4 {
                bytes.extend(9_u16.to_le_bytes());
                bytes.extend([0; 14]);
                bytes.extend(PROPERTY_SET_CLASS_ID);
                bytes.extend(83_u32.to_le_bytes()); // 回引类别值里的对象。
                bytes.extend(34_i32.to_le_bytes());
                bytes.extend(13_u16.to_le_bytes());
                bytes.extend([0; 14]);
                bytes.extend(STR_ARRAY_CLASS_ID);
                bytes.extend(84_u32.to_le_bytes());
                bytes.extend(35_i32.to_le_bytes());
                bytes.extend(13_u16.to_le_bytes());
                bytes.extend([0; 14]);
                bytes.extend(POINT_CLASS_ID);
                bytes.extend(86_u32.to_le_bytes()); // 回引几何集合内部登记的点。
                bytes.extend(36_i32.to_le_bytes());
                bytes.extend([0; 16]); // v4 自身 IColorRamp。
            }
        }
        bytes.extend(1_i32.to_le_bytes()); // 共用体。
        bytes.extend([0; 16]);
        let end = bytes.len();
        bytes.push(0x7e);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_raster_renderer_slot_prefix_10_1(&mut reader, 4096, &mut refs)
            .unwrap()
            .body
            .unwrap();
        assert!(parsed.is_complete());
        let RasterRendererPrefix10_1::UniqueValue(body) = parsed else {
            panic!("应读到栅格唯一值渲染器");
        };
        assert_eq!(body.strings.len(), if legacy { 2 } else { 3 });
        assert_eq!(body.label, "x");
        assert_eq!(body.legend_groups.len(), 1);
        assert_eq!(body.classes[0].raw_key, 0xfe76_1234);
        assert_eq!(
            body.classes[0].values[..2],
            [
                Inline(PropertyValue10_1::Raw4(19_i32.to_le_bytes())),
                Inline(PropertyValue10_1::Array(ClassVariantArray10_1::Fixed {
                    vartype: 17,
                    element_bytes: 1,
                    raw: vec![0x80, 0xff],
                })),
            ]
        );
        if version == 4 {
            assert_eq!(
                body.classes[0].values[2],
                Inline(PropertyValue10_1::NullObject)
            );
            assert!(matches!(
                &body.classes[0].values[3],
                PropertyValueObjectStream10_1::Object { class_id: PROPERTY_SET_CLASS_ID, key: 83, body }
                    if body.version == 1 && body.entries.is_empty()
            ));
            assert_eq!(
                body.classes[0].values[4],
                PropertyValueObjectStream10_1::ArrayObject {
                    class_id: STR_ARRAY_CLASS_ID,
                    key: 84,
                    version: 1,
                    values: VariantArrayObjectValues10_1::Strings(vec!["乙".into()]),
                }
            );
            let PropertyValueObjectStream10_1::Geometry(geometry) = &body.classes[0].values[5]
            else {
                panic!("应读到 Variant 几何对象");
            };
            assert_eq!(
                (geometry.class_id, geometry.key),
                (GEOMETRY_BAG_CLASS_ID, 85)
            );
            let GeometryBody10_1::GeometryBag(bag) = geometry.body() else {
                panic!("应读到几何集合");
            };
            assert!(matches!(
                bag.members[0].body.as_deref(),
                Some(GeometryBody10_1::Point(point)) if point.record.x == Some(14.5)
                   && point.record.y == Some(27.5)
            ));
        } else {
            assert_eq!(body.classes[0].values.len(), 2);
        }
        assert_eq!((body.first_short, body.second_short), (7, 9));
        match body.layout {
            RasterUniqueValueLayout10_1::Legacy {
                before_groups,
                colors,
                samples,
                ..
            } => {
                assert!(legacy);
                assert_eq!(before_groups, 17);
                assert_eq!(samples, [(13.25, 23)]);
                if let Some(string_flag) = legacy_colors {
                    assert_eq!(
                        colors.entry,
                        ObjectEntry10_1::NewObject {
                            class_id: RASTER_UNIQUE_VALUE_COLORS_CLASS_ID,
                            key: 82,
                        }
                    );
                    let colors = colors.body.unwrap();
                    assert_eq!((colors.version, colors.string_flag), (1, string_flag));
                    assert_eq!(colors.entries.len(), 1);
                    assert_eq!(
                        colors.entries[0].value,
                        if string_flag == 0 {
                            RasterUniqueValueColorKey10_1::Number(9.25)
                        } else {
                            RasterUniqueValueColorKey10_1::Text("甲".into())
                        }
                    );
                    assert_eq!(colors.entries[0].colors, [0xaa00_ff11, 0x1234]);
                } else {
                    assert_eq!(colors.entry, ObjectEntry10_1::Null);
                    assert!(colors.body.is_none());
                }
            }
            RasterUniqueValueLayout10_1::Modern {
                unique_values,
                ramp,
            } => {
                assert!(!legacy);
                assert_eq!(
                    unique_values.entry,
                    ObjectEntry10_1::NewObject {
                        class_id: UNIQUE_VALUES_CLASS_ID,
                        key: 82,
                    }
                );
                let values = unique_values.body.unwrap().values;
                assert_eq!(
                    values[0],
                    (Inline(PropertyValue10_1::String("项".into())), 33)
                );
                if version == 4 {
                    assert_eq!(
                        values[1],
                        (
                            PropertyValueObjectStream10_1::BackReference {
                                class_id: PROPERTY_SET_CLASS_ID,
                                key: 83
                            },
                            34,
                        )
                    );
                    assert_eq!(
                        values[2],
                        (
                            PropertyValueObjectStream10_1::BackReference {
                                class_id: STR_ARRAY_CLASS_ID,
                                key: 84,
                            },
                            35,
                        )
                    );
                    assert_eq!(
                        values[3],
                        (
                            PropertyValueObjectStream10_1::BackReference {
                                class_id: POINT_CLASS_ID,
                                key: 86,
                            },
                            36,
                        )
                    );
                }
                assert_eq!(values.len(), if version == 4 { 4 } else { 1 });
                assert_eq!(ramp, (version == 4).then_some(ColorRampSlot10_1::Null));
            }
        }
        assert_eq!(reader.read_bytes(1).unwrap(), [0x7e]);
        assert_eq!(reader.position(), end + 1);
        assert_eq!(
            refs.len(),
            if version == 4 {
                6
            } else if legacy && legacy_colors.is_none() {
                1
            } else {
                2
            }
        );

        let assert_rollback = |input: &[u8]| {
            let mut reader = Reader::new(input);
            let mut refs = RefTable10_1::new();
            assert!(read_raster_renderer_slot_prefix_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        };
        for cut in 0..end {
            assert_rollback(&bytes[..cut]);
        }
        let mut broken = bytes.clone();
        broken[tail_object] ^= 1;
        assert_rollback(&broken);
        if legacy_colors.is_some() {
            let count = tail_object + 24;
            let mut broken = bytes.clone();
            broken[count..count + 4].copy_from_slice(&i32::MAX.to_le_bytes());
            assert_rollback(&broken);
        }
        bytes[value_count..value_count + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        assert_rollback(&bytes);
    }
}

#[test]
fn raster_classify_reads_legacy_and_current_versions_with_optional_raster_io() {
    use mxd2qgz::layers::feature_layer_tail::{LEGEND_CLASS_CLASS_ID, NumberFormatSlot10_1};
    use mxd2qgz::layers::raster_classify_renderer::{
        ClassificationIo10_1, ClassificationMethod10_1, RASTER_CLASSIFY_RENDERER_CLASS_ID,
    };
    use mxd2qgz::layers::raster_unique_value::UNIQUE_VALUES_CLASS_ID;
    use mxd2qgz::objects::number_format::CURRENCY_FORMAT_CLASS_ID;
    use mxd2qgz::objects::property_set::{
        PropertyValue10_1, PropertyValueObjectStream10_1::Inline,
    };

    fn text(bytes: &mut Vec<u8>, c: u16) {
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(c.to_le_bytes());
        bytes.extend(0_u16.to_le_bytes());
    }

    for version in 1_i32..=5 {
        let mut bytes = RASTER_CLASSIFY_RENDERER_CLASS_ID.to_vec();
        bytes.extend(91_u32.to_le_bytes());
        bytes.extend(version.to_le_bytes());
        if version == 1 {
            bytes.extend((-7_i32).to_le_bytes());
        }
        text(&mut bytes, b'a'.into());
        text(&mut bytes, b'b'.into());
        if version == 1 {
            text(&mut bytes, b'x'.into());
        } else if version == 2 {
            bytes.push(0); // 无分类方法 GUID。
        } else {
            bytes.push(1);
            bytes.extend([0x4e; 16]); // 内联 GUID 无 ObjectStream 键。
        }
        text(&mut bytes, b'c'.into());
        bytes.extend(1_i32.to_le_bytes()); // 一个空 ILegendGroup。
        bytes.extend([0; 16]);
        bytes.extend([0; 16]); // ILegendItem。
        if version == 1 {
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend((-4_i32).to_le_bytes());
            bytes.extend(7_i32.to_le_bytes());
        }
        let break_count = bytes.len();
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(10.0_f64.to_le_bytes());
        bytes.extend(20.5_f64.to_le_bytes());
        if version >= 2 {
            bytes.extend(4_u16.to_le_bytes());
            bytes.extend(CURRENCY_FORMAT_CLASS_ID);
            bytes.extend(92_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend(0.125_f64.to_le_bytes());
        }
        if version >= 3 {
            for value in [2_i32, 15] {
                bytes.extend(3_u16.to_le_bytes());
                bytes.extend([0; 14]);
                bytes.extend(value.to_le_bytes());
            }
            if version == 3 {
                bytes.extend((-1_i16).to_le_bytes());
            } else {
                bytes.extend(0_u16.to_le_bytes());
                bytes.extend(LEGEND_CLASS_CLASS_ID);
                bytes.extend(93_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend([0; 16]); // LegendClass 符号。
                text(&mut bytes, b'z'.into());
                text(&mut bytes, b'w'.into());
                bytes.extend([0; 16]); // LegendClass 格式。
            }
        }
        if version >= 4 {
            bytes.extend(UNIQUE_VALUES_CLASS_ID);
            bytes.extend(94_u32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            record(&mut bytes, 0xff00ff, &[]);
            if version == 5 {
                record(&mut bytes, 1, &47_i32.to_le_bytes());
            }
            record(&mut bytes, 2, &3.5_f64.to_le_bytes());
            bytes.extend([0xff; 8]);
        }
        if version == 5 {
            bytes.extend(17_u16.to_le_bytes());
        }
        bytes.extend(1_i32.to_le_bytes()); // 共用体。
        bytes.extend([0; 16]);
        let end = bytes.len();
        bytes.push(0xa7);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let slot = read_raster_renderer_slot_prefix_10_1(&mut reader, 1024, &mut refs).unwrap();
        assert!(slot.body.as_ref().unwrap().is_complete());
        let Some(RasterRendererPrefix10_1::Classify(body)) = slot.body else {
            panic!("应读到栅格分类渲染器");
        };
        assert_eq!(body.strings, ["a", "b", "c"]);
        assert_eq!(body.breaks, [10.0, 20.5]);
        assert_eq!(body.legend_groups.len(), 1);
        assert_eq!(body.legacy_long, (version == 1).then_some(-7));
        assert_eq!(body.legacy_values, (version == 1).then_some(vec![-4, 7]));
        assert_eq!(
            body.classification_method,
            match version {
                1 => ClassificationMethod10_1::LegacyString("x".into()),
                2 => ClassificationMethod10_1::Guid {
                    flag: 0,
                    value: None
                },
                _ => ClassificationMethod10_1::Guid {
                    flag: 1,
                    value: Some([0x4e; 16])
                },
            }
        );
        if version >= 2 {
            let numeric = body.numeric_fields.as_ref().unwrap();
            assert!(
                matches!(&numeric.format, NumberFormatSlot10_1::CurrencyFormat(f) if f.key == 92)
            );
            assert_eq!(
                (numeric.short_a, numeric.short_b, numeric.double),
                (4, 3, 0.125)
            );
        } else {
            assert!(body.numeric_fields.is_none());
        }
        if version >= 3 {
            let bounds = body.value_bounds.as_ref().unwrap();
            assert_eq!(
                bounds.values,
                [
                    Inline(PropertyValue10_1::Raw4(2_i32.to_le_bytes())),
                    Inline(PropertyValue10_1::Raw4(15_i32.to_le_bytes()))
                ]
            );
            if version == 3 {
                assert_eq!(bounds.legend_selector, -1);
                assert!(bounds.legend_class.is_none());
            } else {
                assert_eq!(
                    bounds
                        .legend_class
                        .as_ref()
                        .unwrap()
                        .body
                        .as_ref()
                        .unwrap()
                        .label,
                    "z"
                );
            }
        }
        assert_eq!(
            body.records,
            (version >= 4).then_some(ClassificationIo10_1 {
                tag1: (version == 5).then_some(47),
                tag2: Some(3.5),
            })
        );
        assert_eq!(body.trailing_short, (version == 5).then_some(17));
        assert_eq!(body.shared.version, 1);
        assert_eq!(reader.read_bytes(1).unwrap(), [0xa7]);
        assert_eq!(reader.position(), end + 1);
        assert_eq!(
            refs.len(),
            match version {
                1 => 1,
                2 | 3 => 2,
                _ => 4,
            }
        );

        let reject = |input: &[u8]| {
            let mut reader = Reader::new(input);
            let mut refs = RefTable10_1::new();
            assert!(read_raster_renderer_slot_prefix_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        };
        for cut in 0..end {
            reject(&bytes[..cut]);
        }
        bytes[break_count..break_count + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        reject(&bytes);
    }
}

#[test]
fn raster_stretch_reads_versioned_records_and_raw_tail_before_end() {
    use mxd2qgz::layers::feature_layer_tail::{LEGEND_CLASS_CLASS_ID, LEGEND_GROUP_CLASS_ID};
    use mxd2qgz::layers::raster_renderer::RasterHistogramSlot10_1;
    use mxd2qgz::layers::raster_stretch_renderer::{
        RASTER_STRETCH_RENDERER_CLASS_ID, RasterStretchLayout10_1,
    };
    use mxd2qgz::objects::color_ramp::RANDOM_COLOR_RAMP_CLASS_ID;

    fn text(bytes: &mut Vec<u8>, c: u16) {
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(c.to_le_bytes());
        bytes.extend(0_u16.to_le_bytes());
    }

    fn obj(bytes: &mut Vec<u8>, tag: i32, class_id: [u8; 16]) {
        bytes.extend(tag.to_le_bytes());
        bytes.extend((-1_i32).to_le_bytes());
        bytes.extend(class_id);
    }

    for version in 1_i32..=9 {
        let mut bytes = RASTER_STRETCH_RENDERER_CLASS_ID.to_vec();
        bytes.extend(91_u32.to_le_bytes());
        bytes.extend(version.to_le_bytes());
        let mut sample_count = None;
        if version < 3 {
            bytes.extend(2_i32.to_le_bytes());
            text(&mut bytes, b's'.into());
            bytes.extend([0x31; 0x930]);
            bytes.extend(8_i32.to_le_bytes());
            bytes.extend(1.5_f64.to_le_bytes());
            bytes.extend(5_i32.to_le_bytes());
            bytes.extend(17_i32.to_le_bytes());
            bytes.extend(2.5_f64.to_le_bytes());
            bytes.extend([0; 5 * 16]);
            if version == 2 {
                bytes.extend(19_i32.to_le_bytes());
            }
        } else {
            text(&mut bytes, b's'.into());
            record(&mut bytes, 0xff00ff, &[]);
            record(&mut bytes, 1, &2_i32.to_le_bytes());
            record(&mut bytes, 2, &[0x31; 0x930]);
            record(&mut bytes, 3, &8_i32.to_le_bytes());
            record(&mut bytes, 4, &1.5_f64.to_le_bytes());
            record(&mut bytes, 5, &5_i32.to_le_bytes());
            record(&mut bytes, 6, &2.5_f64.to_le_bytes());
            if version == 9 {
                obj(&mut bytes, 7, RANDOM_COLOR_RAMP_CLASS_ID);
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(0_u32.to_le_bytes()); // BSTR 码元数。
                bytes.extend([0; 22]); // 随机梯度的三个标量及六个 short。
                obj(&mut bytes, 8, [0; 16]);
            }
            obj(&mut bytes, 9, LEGEND_GROUP_CLASS_ID);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(2_u16.to_le_bytes());
            text(&mut bytes, b'g'.into());
            bytes.extend(i32::from(version == 9).to_le_bytes());
            if version == 9 {
                bytes.extend(LEGEND_CLASS_CLASS_ID);
                bytes.extend(92_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend([0; 16]); // ISymbol。
                text(&mut bytes, b'c'.into());
                text(&mut bytes, b'd'.into());
                bytes.extend([0; 16]); // ILegendClassFormat。
            }
            record(&mut bytes, 10, &53_i32.to_le_bytes());
            record(&mut bytes, 11, &71_i32.to_le_bytes());
            record(&mut bytes, 12, &72_u16.to_le_bytes());
            let pair: Vec<_> = [7.5_f64, 8.5]
                .into_iter()
                .flat_map(f64::to_le_bytes)
                .collect();
            record(&mut bytes, 13, &pair);
            if version == 9 {
                obj(
                    &mut bytes,
                    17,
                    [
                        0xc1, 0x27, 0x30, 0xba, 0xca, 0x49, 0x88, 0x47, 0x8b, 0x5a, 0x3a, 0x6b,
                        0x38, 0x7d, 0xe7, 0x8c,
                    ],
                );
                bytes.extend(3_i32.to_le_bytes());
                bytes.extend([0; 6 * 8]);
                bytes.extend(1_i32.to_le_bytes());
                bytes.extend(10.0_f64.to_le_bytes());
                bytes.extend(11.0_f64.to_le_bytes());
            }
            if version >= 5 {
                record(&mut bytes, 18, &81_i32.to_le_bytes());
                record(&mut bytes, 19, &3.5_f64.to_le_bytes());
                record(&mut bytes, 20, &83_i32.to_le_bytes());
                sample_count = Some(bytes.len() + 8);
                record(&mut bytes, 21, &1_i32.to_le_bytes());
                record(&mut bytes, 22, &i32::from(version == 9).to_le_bytes());
                bytes.extend(9.5_f64.to_le_bytes());
                if version == 9 {
                    text(&mut bytes, '色' as u16);
                }
            }
            if version >= 6 {
                bytes.extend(&pair);
            }
            if version >= 7 {
                bytes.extend((-1_i32).to_le_bytes());
            }
            if version == 9 {
                bytes.extend(&pair);
            }
            bytes.extend([0xff; 8]);
        }
        bytes.extend(1_i32.to_le_bytes()); // 共用体。
        bytes.extend([0; 16]);
        let end = bytes.len();
        bytes.push(0x52);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let slot = read_raster_renderer_slot_prefix_10_1(&mut reader, 4096, &mut refs)
            .unwrap_or_else(|error| panic!("v{version}: {error}"));
        assert!(slot.body.as_ref().unwrap().is_complete());
        let Some(RasterRendererPrefix10_1::Stretch(body)) = slot.body else {
            panic!("应读到栅格拉伸渲染器");
        };
        assert_eq!(
            (body.band, body.label.as_str(), body.stretch_type),
            (2, "s", 8)
        );
        assert_eq!(body.lookup, [0x31; 0x930]);
        assert_eq!((body.double_a, body.long_a, body.double_b), (1.5, 5, 2.5));
        match body.layout {
            RasterStretchLayout10_1::Legacy {
                extra_long,
                trailing_long,
                ..
            } => {
                assert!(version < 3);
                assert_eq!(extra_long, 17);
                assert_eq!(trailing_long, (version == 2).then_some(19));
            }
            RasterStretchLayout10_1::RasterIo(io) => {
                assert!(version >= 3);
                assert_eq!(io.tag11, Some(71));
                assert_eq!(io.tag12, Some(72));
                assert_eq!(io.tag13, Some([7.5, 8.5]));
                assert_eq!(
                    io.extra.as_ref().map(|extra| &extra.values[..]),
                    (version >= 5)
                        .then(|| if version == 9 {
                            [(9.5, Some("色".into()))]
                        } else {
                            [(9.5, None)]
                        })
                        .as_ref()
                        .map(|array| &array[..])
                );
                assert_eq!(io.pair_v6, (version >= 6).then_some([7.5, 8.5]));
                assert_eq!(io.long_v7, (version >= 7).then_some(-1));
                assert_eq!(io.pair_v9, (version == 9).then_some([7.5, 8.5]));
                if version == 9 {
                    assert_eq!(io.ramp.as_ref().unwrap()[..16], RANDOM_COLOR_RAMP_CLASS_ID);
                    assert_eq!(io.color.as_ref().unwrap(), &[0; 16]);
                    assert_eq!(io.legend_group[..16], LEGEND_GROUP_CLASS_ID);
                    assert_eq!(io.tag10, Some(53));
                    assert!(
                        matches!(io.histogram, Some(RasterHistogramSlot10_1::Histogram(h)) if h.bins == [10.0])
                    );
                }
            }
        }
        assert_eq!(reader.read_bytes(1).unwrap(), [0x52]);
        assert_eq!(refs.len(), if version == 9 { 2 } else { 1 });
        assert_eq!(refs.contains(92), version == 9);

        let reject = |input: &[u8]| {
            let mut reader = Reader::new(input);
            let mut refs = RefTable10_1::new();
            assert!(read_raster_renderer_slot_prefix_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        };
        for cut in 0..end {
            reject(&bytes[..cut]);
        }
        if let Some(count) = sample_count {
            bytes[count..count + 4].copy_from_slice(&i32::MAX.to_le_bytes());
            reject(&bytes);
        }
    }
}

#[test]
#[ignore = "需要本地 10.1 SDK 安装介质提取工程"]
fn sdk_raster_renderers_consume_color_symbols_and_finish_their_bodies() {
    use mxd2qgz::layers::feature_layer_tail::{
        LegendClassEntry10_1, LegendGroupSlot10_1, read_legend_group_body_10_1,
    };
    use mxd2qgz::layers::raster_stretch_renderer::RasterStretchLayout10_1;
    use mxd2qgz::objects::display_symbol::SymbolSlotFull10_1;
    use std::io::Read;

    for (path, start, end, keys, stretch) in [
        (
            "temp/terra-round8-old/sdk-dotnet/BloomfieldTownship.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2",
            0x4b73,
            0x6d0f,
            &[218, 233, 248][..],
            true,
        ),
        (
            "temp/terra-round8-old/sdk-java/portland.mxd.4896861F_D89C_46BA_ABEC_17591162B2B1",
            0x73b30,
            0x75da0,
            &[953, 956, 959, 962, 965][..],
            false,
        ),
    ] {
        let mut doc = cfb::open(path).unwrap();
        let mut bytes = Vec::new();
        doc.open_stream("/Maps")
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        let mut reader = Reader::new(&bytes);
        reader.read_bytes(start).unwrap();
        let mut refs = RefTable10_1::new();
        let slot =
            read_raster_renderer_slot_prefix_10_1(&mut reader, 1_000_000, &mut refs).unwrap();
        let body = slot.body.unwrap();
        assert!(body.is_complete());
        assert_eq!(reader.position(), end);
        let group = match body {
            RasterRendererPrefix10_1::Stretch(body) if stretch => {
                let RasterStretchLayout10_1::RasterIo(body) = body.layout else {
                    panic!("预期 RasterIO 拉伸渲染器");
                };
                let mut raw = Reader::new(&body.legend_group);
                raw.read_bytes(16).unwrap(); // 标签 9 的图例组有 CLSID，无键。
                let group =
                    read_legend_group_body_10_1(&mut raw, 1_000_000, &mut RefTable10_1::new(), 0)
                        .unwrap();
                assert_eq!(raw.remaining(), 0);
                group
            }
            RasterRendererPrefix10_1::Classify(body) if !stretch => {
                let LegendGroupSlot10_1::LegendGroup(group) =
                    body.legend_groups.into_iter().next().unwrap()
                else {
                    panic!("预期分类渲染器图例组");
                };
                *group
            }
            other => panic!("渲染器类型不符: {other:?}"),
        };
        assert!(group.stop.is_none());
        assert_eq!(group.classes.len(), keys.len());
        for (index, (entry, key)) in group.classes.into_iter().zip(keys).enumerate() {
            let LegendClassEntry10_1::NewObject(class) = entry else {
                panic!("预期新建图例类");
            };
            assert!(class.stop.is_none());
            match class.symbol {
                SymbolSlotFull10_1::ColorRampSymbol(body) if stretch => {
                    assert_eq!((body.key, body.version), (*key, 5));
                    assert_eq!(body.tail.unwrap().integer, index as i32);
                }
                SymbolSlotFull10_1::ColorSymbol(body) if !stretch => {
                    assert_eq!(
                        (body.key, body.version, body.integers),
                        (*key, 1, Some([13, 0, 0]))
                    );
                }
                slot => panic!("图例符号类不符: {slot:?}"),
            }
            assert!(refs.contains(*key));
        }
    }
}
