use mxd2qgz::objects::geodata_xform::{
    COMPOSITE_XFORM_CLASS_ID, GeodataXformBody10_1, IDENTITY_XFORM_CLASS_ID,
    POLYNOMIAL_XFORM_CLASS_ID, read_geodata_xform_slot_10_1,
};
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use mxd2qgz::stream::reader::Reader;

fn header(bytes: &mut Vec<u8>, class: [u8; 16], key: u32) {
    bytes.extend(class);
    bytes.extend(key.to_le_bytes());
}

#[test]
fn composite_uses_one_table_for_nested_bodies_and_references() {
    let mut bytes = Vec::new();
    header(&mut bytes, COMPOSITE_XFORM_CLASS_ID, 51);
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    header(&mut bytes, IDENTITY_XFORM_CLASS_ID, 52);
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend([0; 16]); // Identity 的空空间参考。
    header(&mut bytes, IDENTITY_XFORM_CLASS_ID, 52);
    bytes.extend(31_u16.to_le_bytes());
    for cut in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
    bytes.push(0x95);
    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
    let GeodataXformBody10_1::Composite {
        children, short_34, ..
    } = *slot.body.unwrap()
    else {
        panic!("应读为复合变换");
    };
    assert!(matches!(
        children[0].body.as_deref(),
        Some(GeodataXformBody10_1::Identity { .. })
    ));
    assert!(matches!(
        children[1].entry,
        ObjectEntry10_1::BackReference { key: 52, .. }
    ));
    assert_eq!(short_34, Some(31));
    assert_eq!(reader.read_bytes(1).unwrap(), [0x95]);
    assert_eq!(refs.len(), 2);
}

#[test]
fn polynomial_reads_point_and_coefficient_arrays_in_native_order() {
    let mut bytes = Vec::new();
    header(&mut bytes, POLYNOMIAL_XFORM_CLASS_ID, 1);
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]);
    for flag in [1_i32, -1, 1] {
        bytes.extend(flag.to_le_bytes());
    }
    for value in 1..=18 {
        bytes.extend((value as f64).to_le_bytes());
    }
    bytes.extend(4_u32.to_le_bytes());
    bytes.extend([b'P', 0, 0, 0]);
    bytes.push(0x87);
    let mut reader = Reader::new(&bytes);
    let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut RefTable10_1::new()).unwrap();
    let GeodataXformBody10_1::Polynomial(body) = *slot.body.unwrap() else {
        panic!("应读为多项式变换");
    };
    assert_eq!(
        body.point_arrays.unwrap(),
        [vec![[1.0, 2.0]], vec![[3.0, 4.0]]]
    );
    assert_eq!(
        body.coefficients_e4.unwrap(),
        [vec![5.0, 6.0], vec![7.0, 8.0]]
    );
    assert_eq!(
        body.coefficients_e8.unwrap(),
        [vec![9.0, 10.0], vec![11.0, 12.0]]
    );
    assert_eq!(
        body.arrays_c8_cc.unwrap(),
        [vec![13.0, 14.0, 15.0], vec![16.0, 17.0, 18.0]]
    );
    assert_eq!(body.name.as_deref(), Some("P"));
    assert_eq!(reader.read_bytes(1).unwrap(), [0x87]);

    // 同一位置的有符号计数若异常，不能借用后续字节或保留对象键。
    for n in [-1, i32::MAX] {
        let mut bad = bytes.clone();
        bad[44..48].copy_from_slice(&n.to_le_bytes());
        let mut reader = Reader::new(&bad);
        let mut refs = RefTable10_1::new();
        assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn composite_rejects_excessive_nesting_before_committing_keys() {
    let mut bytes = Vec::new();
    for key in 1..=25 {
        header(&mut bytes, COMPOSITE_XFORM_CLASS_ID, key);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
    }
    bytes.extend([0; 16]);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let err = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap_err();
    assert!(err.to_string().contains("嵌套深度"));
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}
