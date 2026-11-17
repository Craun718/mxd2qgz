use mxd2qgz::objects::spatial_reference::read_projected_coordinate_system_10_1;
use mxd2qgz::objects::spatial_reference_slots::read_geographic_coordinate_system_10_1;
use mxd2qgz::stream::reader::Reader;

fn body(version: u16, vertical: bool) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(1_u32.to_le_bytes());
    bytes.push(b'X');
    for value in [&[0x2d, 0x4e][..], &[0xff][..], &[][..], &[0, 0][..]] {
        bytes.extend((value.len() as u32).to_le_bytes());
        bytes.extend(value);
    }
    bytes.push(u8::from(vertical));
    if vertical {
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(3_u32.to_le_bytes());
        bytes.extend(b"met");
    }
    bytes
}

#[test]
fn version_two_reads_domains_without_gcs_extent() {
    let bytes = body(2, false);
    let mut pcs_reader = Reader::new(&bytes);
    let pcs = read_projected_coordinate_system_10_1(&mut pcs_reader, 32).unwrap();
    assert_eq!(pcs_reader.position(), bytes.len());
    assert_eq!(pcs.domains.unwrap().strings[1], [0xff]);
    let mut gcs_reader = Reader::new(&bytes);
    let gcs = read_geographic_coordinate_system_10_1(&mut gcs_reader, 32).unwrap();
    assert_eq!(gcs_reader.position(), bytes.len());
    assert_eq!(gcs.domains.unwrap().strings[0], [0x2d, 0x4e]);
    assert!(gcs.extent.is_none());
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        assert!(read_projected_coordinate_system_10_1(&mut reader, 32).is_err());
        assert_eq!(reader.position(), 0);
        assert!(read_geographic_coordinate_system_10_1(&mut reader, 32).is_err());
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn pcs_vertical_segment_precedes_precision_and_is_transactional() {
    let mut bytes = body(6, true);
    bytes.extend(0x1234_u16.to_le_bytes());
    for n in 0..10 {
        bytes.extend((n as f64).to_le_bytes());
    }
    let mut reader = Reader::new(&bytes);
    let pcs = read_projected_coordinate_system_10_1(&mut reader, 32).unwrap();
    assert_eq!(reader.position(), bytes.len());
    assert_eq!(pcs.domains.unwrap().vertical_unit.unwrap().unit_wkt, b"met");
    assert_eq!(pcs.precision.unwrap().flags, 0x1234);
    assert_eq!(pcs.trailing_doubles.unwrap(), [7.0, 8.0, 9.0]);
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        assert!(read_projected_coordinate_system_10_1(&mut reader, 32).is_err());
        assert_eq!(reader.position(), 0);
    }
    let mut reader = Reader::new(&bytes);
    assert!(read_projected_coordinate_system_10_1(&mut reader, 2).is_err());
    assert_eq!(reader.position(), 0);
}
