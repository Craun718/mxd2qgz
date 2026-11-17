use mxd2qgz::objects::envelope::read_envelope_10_1;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

#[test]
fn version_zero_reads_flags_and_optional_pairs_without_spatial_reference() {
    for flags in [0_u32, 0x40, 0x80, 0xc0] {
        let mut bytes = 0_u16.to_le_bytes().to_vec();
        for value in [1.0_f64, 2.0, 3.0, 4.0] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(flags.to_le_bytes());
        if flags & 0x40 != 0 {
            for value in [5.0_f64, 6.0] {
                bytes.extend(value.to_le_bytes());
            }
        }
        if flags & 0x80 != 0 {
            for value in [7.0_f64, 8.0] {
                bytes.extend(value.to_le_bytes());
            }
        }
        let body_len = bytes.len();
        bytes.extend(b"tail");
        let mut reader = Reader::new(&bytes);
        let parsed = read_envelope_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
        assert_eq!(parsed.coords, [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(parsed.attribute_flags, Some(flags));
        assert_eq!(parsed.z_domain, (flags & 0x40 != 0).then_some([5.0, 6.0]));
        assert_eq!(parsed.m_domain, (flags & 0x80 != 0).then_some([7.0, 8.0]));
        assert_eq!(parsed.spatial_reference, None);
        assert_eq!(reader.position(), body_len);
        assert_eq!(reader.read_bytes(4).unwrap(), b"tail");
        for end in 0..body_len {
            let mut reader = Reader::new(&bytes[..end]);
            let mut table = RefTable10_1::new();
            table.register(42);
            assert!(read_envelope_10_1(&mut reader, 1024, &mut table).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(table.len(), 1);
            assert!(table.contains(42));
        }
    }
}
