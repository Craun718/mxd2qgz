use mxd2qgz::objects::envelope::SpatialReferenceEntry10_1;
use mxd2qgz::objects::geometry::{
    GeometryBody10_1, MULTIPATCH_CLASS_ID, MultiPatchBody10_1, read_geometry_body_10_1,
};
use mxd2qgz::objects::multi_patch::MultiPatchRecord10_1;
use mxd2qgz::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;
use std::io::{self, Write};

fn mesh_core(kind: u32) -> Vec<u8> {
    let mut bytes = kind.to_le_bytes().to_vec();
    for value in [0.0_f64, 0.0, 4.0, 5.0] {
        bytes.extend(value.to_le_bytes());
    }
    for value in [1_u32, 3, 0, 0x100] {
        bytes.extend(value.to_le_bytes()); // 面片数、点数、起点和保留标志位的类型字。
    }
    for value in [0.0_f64, 0.0, 4.0, 0.0, 0.0, 5.0] {
        bytes.extend(value.to_le_bytes());
    }
    for value in [2.0_f64, 4.0, 2.0, 3.0, 4.0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

fn with_measures(record: &mut Vec<u8>) {
    for value in [8.0_f64, 10.0, 8.0, 9.0, 10.0] {
        record.extend(value.to_le_bytes());
    }
}

fn general_record() -> Vec<u8> {
    let mut record = mesh_core(0xdd00_0036);
    record.extend(3_u32.to_le_bytes());
    with_measures(&mut record);
    for value in [2_u32, 10, 11, 1] {
        record.extend(value.to_le_bytes()); // 点 ID 计数、ID、法线计数。
    }
    for value in [0.0_f32, 0.0, 1.0] {
        record.extend(value.to_le_bytes());
    }
    for value in [2_u32, 2, 0] {
        record.extend(value.to_le_bytes()); // 纹理点数、维数、面片纹理起点。
    }
    for value in [0.0_f32, 0.0, 0.5, 1.0] {
        record.extend(value.to_le_bytes());
    }
    for value in [2_u32, 3, 0, 2, 3] {
        record.extend(value.to_le_bytes()); // 材质数、版本、两个区间外加最终偏移。
    }
    record.extend([0xaa, 0xbb, 0xcc]);
    record
}

fn object_body(version: u16, record: &[u8]) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend((record.len() as u32).to_le_bytes());
    bytes.extend(record);
    bytes.extend(PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
    bytes.extend(71_u32.to_le_bytes());
    bytes
}

fn read_body(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Box<MultiPatchBody10_1>> {
    let GeometryBody10_1::MultiPatch(body) =
        read_geometry_body_10_1(&MULTIPATCH_CLASS_ID, reader, max, refs)?
    else {
        panic!("几何类应为 MultiPatch");
    };
    Ok(body)
}

#[test]
fn multi_patch_decodes_classic_and_general_records_without_consuming_following_field() {
    let classic = mesh_core(0x1f);
    let mut classic_with_m = mesh_core(0x20);
    with_measures(&mut classic_with_m);
    let mut general_v1 = general_record();
    let revision_at = general_v1.len() - 3 - 4 * 4;
    general_v1.drain(revision_at..revision_at + 4);
    for (version, record, measures, general) in [
        (1, classic, false, false),
        (2, classic_with_m, true, false),
        (1, general_v1, true, true),
        (2, general_record(), true, true),
    ] {
        let mut bytes = object_body(version, &record);
        bytes.extend(0x1234_u16.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        refs.register(71);
        let body = read_body(&mut reader, 1024, &mut refs).unwrap();
        assert_eq!(body.record_bytes, record);
        assert!(matches!(
            body.spatial_reference,
            SpatialReferenceEntry10_1::BackReference { key: 71, .. }
        ));
        let Some(MultiPatchRecord10_1::Mesh(mesh)) = body.record else {
            panic!("缺少已解码的面片");
        };
        assert_eq!(mesh.part_starts, [0]);
        assert_eq!(mesh.part_types, [0x100]);
        assert_eq!(mesh.envelope, [0.0, 0.0, 4.0, 5.0]);
        assert_eq!(mesh.xy, [[0.0, 0.0], [4.0, 0.0], [0.0, 5.0]]);
        assert_eq!(mesh.z.values, [2.0, 3.0, 4.0]);
        assert_eq!(
            mesh.m.as_ref().map(|m| m.values.as_slice()),
            measures.then_some([8.0, 9.0, 10.0].as_slice())
        );
        if general {
            assert_eq!(mesh.point_ids, [10, 11]);
            assert_eq!(mesh.normals, [[0.0, 0.0, 1.0]]);
            let texture = mesh.texture_coordinates.unwrap();
            assert_eq!((texture.dimension, texture.vertex_count), (2, 2));
            assert_eq!(texture.part_starts, [0]);
            assert_eq!(texture.values, [0.0, 0.0, 0.5, 1.0]);
            let materials = mesh.materials.unwrap();
            assert_eq!(materials.revision, (version == 2).then_some(3));
            assert_eq!(materials.offsets, [0, 2, 3]);
            assert_eq!(materials.payload, [0xaa, 0xbb, 0xcc]);
        }
        assert_eq!(reader.read_u16_le().unwrap(), 0x1234);
    }
}

fn packed_record(tag: u32, record: &[u8]) -> Vec<u8> {
    use flate2::{Compression, write::ZlibEncoder};
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(record).unwrap();
    let packed = encoder.finish().unwrap();
    let mut result = tag.to_le_bytes().to_vec();
    result.extend((record.len() as u32).to_le_bytes());
    result.extend((packed.len() as u32).to_le_bytes());
    result.extend(packed);
    result
}

#[test]
fn multi_patch_decodes_legacy_and_flagged_zlib_wrappers() {
    let record = mesh_core(0x1f);
    for (version, tag) in [(1, 0x38), (1, 0x1f), (1, 0), (2, 0x38), (2, 0x8080_0036)] {
        let packed = packed_record(tag, &record);
        let bytes = object_body(version, &packed);
        let mut refs = RefTable10_1::new();
        refs.register(71);
        let mut reader = Reader::new(&bytes);
        let body = read_body(&mut reader, 1024, &mut refs).unwrap();
        assert_eq!(body.record_bytes, packed);
        assert!(
            matches!(body.record, Some(MultiPatchRecord10_1::Mesh(mesh)) if mesh.xy.len() == 3)
        );
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn multi_patch_invalid_records_preserve_reader_and_reference_table() {
    let record = general_record();
    let mut count = record.clone();
    count[36..40].copy_from_slice(&u32::MAX.to_le_bytes());
    let mut part_start = record.clone();
    part_start[44..48].copy_from_slice(&9_u32.to_le_bytes());
    let mut packed = packed_record(0x8080_0036, &record);
    let mut wrong_length = packed.clone();
    wrong_length[4..8].copy_from_slice(&((record.len() + 1) as u32).to_le_bytes());
    let mut limit = packed.clone();
    limit[4..8].copy_from_slice(&2048_u32.to_le_bytes());
    let last_byte = packed.last_mut().unwrap();
    *last_byte ^= 0xff;

    for invalid in [
        count,
        part_start,
        packed,
        wrong_length,
        limit,
        record[..record.len() - 1].to_vec(),
    ] {
        let bytes = object_body(2, &invalid);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        refs.register(71);
        assert!(read_body(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs.len(), 1);
    }
}

#[test]
fn legacy_measure_flags_decode_normals_and_uv_while_preserving_raw_values() {
    let normal = 0x0010_0000_u32 | (255 << 1) | (255 << 10) | 1;
    let texture = 0x0020_0000_u32;
    let uv = (42_000 << 16) | 21_000_u32;
    let both = (u64::from(normal | texture) << 32) | u64::from(uv);
    let texture_only = (u64::from(texture) << 32) | u64::from(uv);
    let mut record = mesh_core(0x1f);
    for bits in [both, both, both, texture_only, 0] {
        record.extend(bits.to_le_bytes());
    }
    let bytes = object_body(2, &record);
    let mut refs = RefTable10_1::new();
    refs.register(71);
    let body = read_body(&mut Reader::new(&bytes), 1024, &mut refs).unwrap();
    let Some(MultiPatchRecord10_1::Mesh(mesh)) = body.record else {
        panic!("缺少面片");
    };
    assert_eq!(mesh.m.unwrap().values[0].to_bits(), both);
    assert_eq!(mesh.normals, [[0.0, 0.0, -1.0], [0.0; 3], [0.0; 3]]);
    let texture = mesh.texture_coordinates.unwrap();
    assert_eq!(texture.part_starts, [0]);
    assert_eq!(texture.values, [0.0, 1.0, 0.0, 1.0, 0.0, 0.0]);
}

#[test]
fn old_general_shape_tags_keep_the_versioned_material_header() {
    for version in [1, 2] {
        for tag in [0x29_u32, 0x2a] {
            let mut record = general_record();
            let raw_kind = 0xdd00_0000 | tag;
            record[..4].copy_from_slice(&raw_kind.to_le_bytes());
            let body_bytes = object_body(version, &record);
            let mut refs = RefTable10_1::new();
            refs.register(71);
            let body = read_body(&mut Reader::new(&body_bytes), 4096, &mut refs).unwrap();
            let Some(MultiPatchRecord10_1::Mesh(mesh)) = body.record else {
                panic!("缺少旧式面片");
            };
            assert_eq!(mesh.shape_type, raw_kind);
            assert_eq!(mesh.part_starts, [0]);
            assert_eq!(mesh.z.values, [2.0, 3.0, 4.0]);
            assert_eq!(mesh.materials.unwrap().revision, Some(3));
        }
    }
}

fn with_materials(version: u16, materials: &[&[u8]]) -> Vec<u8> {
    let mut record = general_record();
    record.truncate(record.len() - (5 * 4 + 3)); // 替换材质计数、表头和负载。
    record.extend((materials.len() as u32).to_le_bytes());
    if version == 2 {
        record.extend(3_u32.to_le_bytes());
    }
    record.extend(0_u32.to_le_bytes());
    let mut offset = 0;
    for material in materials {
        offset += material.len() as u32;
        record.extend(offset.to_le_bytes());
    }
    for material in materials {
        record.extend(*material);
    }
    object_body(version, &record)
}

#[test]
fn reads_material_commands_from_normal_geometry_dispatch() {
    use mxd2qgz::objects::multi_patch_material::MultiPatchMaterialCommand10_1::{
        Color, CullBackface, EdgeColor, EdgeWidth, Texture, TextureReference, TransparencyPercent,
    };

    for version in [1, 2] {
        let reference_opcode = if version == 1 { 8 } else { 5 };
        let mut commands = vec![1, 12, 34, 56, 3, 40, 6, reference_opcode];
        commands.extend(1_i32.to_le_bytes());
        let expected_encoding = (version == 2).then_some(3);
        if version == 1 {
            commands.extend([2, 3, 1, 0, 1, 0, 17, 18, 19]);
            commands.extend([7, 3, 2, 0, 3, 0, 1, 0, 0, 0, 21]);
        } else {
            commands.extend([2, 3, 1, 0, 1, 0]);
            commands.extend(3_u32.to_le_bytes());
            commands.extend(3_u32.to_le_bytes());
            commands.extend([17, 18, 19]);
            commands.extend([9, 11, 12, 13, 10, 7]);
        }
        let body_bytes = with_materials(version, &[&commands, &[6]]);
        let mut refs = RefTable10_1::new();
        refs.register(71);
        let body = read_body(&mut Reader::new(&body_bytes), 4096, &mut refs).unwrap();
        let Some(MultiPatchRecord10_1::Mesh(mesh)) = body.record else {
            panic!("缺少通用面片");
        };
        let materials = mesh.materials.unwrap();
        let decoded = materials
            .commands(0)
            .unwrap()
            .collect::<io::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(
            decoded[..5],
            [
                Color([12, 34, 56]),
                TransparencyPercent(40),
                CullBackface,
                TextureReference {
                    opcode: reference_opcode,
                    material_index: 1,
                },
                Texture {
                    opcode: 2,
                    bytes_per_pixel: 3,
                    width: 1,
                    height: 1,
                    encoding: expected_encoding,
                    bytes: &[17, 18, 19]
                },
            ]
        );
        if version == 1 {
            assert_eq!(
                decoded[5..],
                [Texture {
                    opcode: 7,
                    bytes_per_pixel: 3,
                    width: 2,
                    height: 3,
                    encoding: None,
                    bytes: &[21]
                },]
            );
        } else {
            assert_eq!(decoded[5..], [EdgeColor([11, 12, 13]), EdgeWidth(7)]);
        }
        assert_eq!(
            materials.commands(1).unwrap().next().unwrap().unwrap(),
            CullBackface
        );
    }
}

#[test]
fn unknown_or_short_command_does_not_consume_following_material() {
    use mxd2qgz::objects::multi_patch_material::MultiPatchMaterialCommand10_1::CullBackface;

    let bytes = with_materials(2, &[&[6, 88, 73], &[1, 4], &[6]]);
    let mut refs = RefTable10_1::new();
    refs.register(71);
    let body = read_body(&mut Reader::new(&bytes), 4096, &mut refs).unwrap();
    let Some(MultiPatchRecord10_1::Mesh(mesh)) = body.record else {
        panic!("缺少通用面片");
    };
    let materials = mesh.materials.unwrap();
    let mut first = materials.commands(0).unwrap();
    assert_eq!(first.next().unwrap().unwrap(), CullBackface);
    assert_eq!(
        first.next().unwrap().unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert!(first.next().is_none());
    let mut second = materials.commands(1).unwrap();
    assert_eq!(
        second.next().unwrap().unwrap_err().kind(),
        io::ErrorKind::UnexpectedEof
    );
    assert!(second.next().is_none());
    assert_eq!(
        materials.commands(2).unwrap().next().unwrap().unwrap(),
        CullBackface
    );
    assert_eq!(materials.payload, [6, 88, 73, 1, 4, 6]);
}

#[test]
fn legacy_material_attributes_leave_the_next_opcode_and_material_intact() {
    use mxd2qgz::objects::multi_patch_material::MultiPatchMaterialCommand10_1::{
        Color, CullBackface, Shininess, TransparencyPercent,
    };

    let bytes = with_materials(1, &[&[3, 40, 1, 11, 22, 33], &[4, 9], &[6]]);
    let mut refs = RefTable10_1::new();
    refs.register(71);
    let mut reader = Reader::new(&bytes);
    let body = read_body(&mut reader, 4096, &mut refs).unwrap();
    let Some(MultiPatchRecord10_1::Mesh(mesh)) = body.record else {
        panic!("缺少旧式面片");
    };
    let materials = mesh.materials.unwrap();
    assert_eq!(materials.offsets, [0, 6, 8, 9]);
    for (index, expected) in [
        vec![TransparencyPercent(40), Color([11, 22, 33])],
        vec![Shininess(9)],
        vec![CullBackface],
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            materials
                .commands(index)
                .unwrap()
                .collect::<io::Result<Vec<_>>>()
                .unwrap(),
            expected
        );
    }
    assert_eq!(reader.remaining(), 0);
}
