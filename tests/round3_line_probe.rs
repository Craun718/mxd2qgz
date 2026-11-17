use mxd2qgz::objects::element_body::{read_line_element_body_10_1, read_text_element_body_10_1};
use mxd2qgz::storage::object_refs::{RefTable10_1, read_object_entry_10_1};
use mxd2qgz::stream::reader::Reader;

#[test]
#[ignore = "需要本地 CentralUSA 介质流"]
fn central_usa_first_two_elements_without_assumed_references() {
    let data = std::fs::read("temp/arcmap101-media-streams/CentralUSA.mxd.PageLayout").unwrap();
    let mut table = RefTable10_1::new();
    let mut root = Reader::new(&data);
    println!("root: {:?}", read_object_entry_10_1(&mut root, &mut table));
    for (start, end) in [(0x26, 0xa04), (0xa0c, 0xb29)] {
        let mut reader = Reader::new(&data[start..end]);
        let entry = read_object_entry_10_1(&mut reader, &mut table);
        println!("entry {start:x}: {entry:?}; keys {}", table.len());
        if start == 0x26 {
            let result = read_text_element_body_10_1(&mut reader, 1_000_000, &mut table);
            println!(
                "text: {result:#?}; offset {:x}; keys {}",
                start + reader.position(),
                table.len()
            );
        } else {
            let result = read_line_element_body_10_1(&mut reader, 1_000_000, &mut table);
            println!(
                "line: {result:#?}; offset {:x}; keys {}",
                start + reader.position(),
                table.len()
            );
        }
    }
}

#[test]
#[ignore = "需要本地 CentralUSA 介质流"]
fn central_usa_line_alone_has_no_assumed_references() {
    let data = std::fs::read("temp/arcmap101-media-streams/CentralUSA.mxd.PageLayout").unwrap();
    let start = 0xa0c;
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&data[start..0xb29]);
    println!(
        "entry: {:?}",
        read_object_entry_10_1(&mut reader, &mut table)
    );
    let result = read_line_element_body_10_1(&mut reader, 1_000_000, &mut table);
    println!(
        "line alone: {result:#?}; cursor {:x}; keys {}",
        start + reader.position(),
        table.len()
    );
}
