use mxd2qgz::layers::layer_list::LayerBodyPrefix10_1;
use mxd2qgz::layers::raster_dataset_name::{
    RasterDatasetNameEntryBody10_1, RasterDatasetWorkspaceSlot10_1,
};
use mxd2qgz::maps::map_object::read_map_suffix_10_1;
use mxd2qgz::storage::object_refs::{RefTable10_1, read_object_entry_10_1};
use mxd2qgz::stream::reader::Reader;
use std::io::Read;

#[test]
fn filegdb_raster_name_reaches_end_record_in_reference_map() {
    let mut doc = cfb::open("samples/测试底图.mxd").unwrap();
    let mut maps = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut maps)
        .unwrap();
    let mut reader = Reader::new(&maps);
    let mut refs = RefTable10_1::new();
    assert_eq!(reader.read_u32_le().unwrap(), 1);
    read_object_entry_10_1(&mut reader, &mut refs).unwrap();
    let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
    let entry = &map.layer_list.entries[4];
    let Some(LayerBodyPrefix10_1::RasterLayer(raster)) = &entry.parsed else {
        panic!("第五条应为 FileGDB 栅格");
    };
    let Some(RasterDatasetNameEntryBody10_1::FileGeodatabase { name, records }) =
        &raster.names[0].body
    else {
        panic!("应读完 FileGDB 栅格名称");
    };
    assert_eq!(name.delegated_name.key, 260);
    let Some(RasterDatasetWorkspaceSlot10_1::WorkspaceName(workspace)) =
        &name.delegated_name.workspace
    else {
        panic!("委派名称体应包含实际 WorkspaceName 新对象");
    };
    assert_eq!(workspace.key, 261);
    assert!(workspace.stop.is_none());
    assert!(refs.contains(260));
    assert!(refs.contains(261));
    assert!(name.delegated_name.stop.is_none());
    assert_eq!(
        (records.tag1_value, records.tag4_value, records.tag7_value),
        (1, 1, Some(1))
    );
    assert!(matches!(
        records.tag6_object,
        mxd2qgz::layers::fgdb_raster_dataset_name::FgdbRasterIoObject10_1::Null
    ));
    assert!(records.stop.is_none());
    // end_offset 相对 RasterLayer 体；对象体前是 flag、ExtraSafe 前缀和对象头。
    assert_eq!(entry.offset + 30 + records.end_offset.unwrap(), 0x51d9);
    assert_eq!(reader.position(), 0x25569);
}
