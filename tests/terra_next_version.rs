use mxd2qgz::stream::reader::Reader;
use mxd2qgz::version::version_stream::read_arcmap_header_10_8;

#[test]
fn count_candidate_preserves_all_bits_and_original_bytes() {
    let mut data = vec![4, 0, 0, 0, b'x', 0, 0, 0, 25, 0];
    data.extend_from_slice(&[0x78, 0x56, 0x34, 0xf2, 0xaa]);
    let mut reader = Reader::new(&data);
    let header = read_arcmap_header_10_8(&mut reader, 64).unwrap();
    assert_eq!(header.conditional_bytes, Some([0x78, 0x56, 0x34, 0xf2]));
    assert_eq!(
        header.write_table_entry_count_candidate(),
        Some(0xf234_5678)
    );
    assert_eq!(reader.remaining(), 1);
}

#[test]
fn count_candidate_is_absent_when_no_conditional_field_exists() {
    let data = [4, 0, 0, 0, b'x', 0, 0, 0, 10, 0];
    let mut reader = Reader::new(&data);
    let header = read_arcmap_header_10_8(&mut reader, 64).unwrap();
    assert_eq!(header.write_table_entry_count_candidate(), None);
}
