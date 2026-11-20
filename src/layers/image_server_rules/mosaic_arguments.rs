//! MosaicDB.dll 10.1.0.3035 的栅格函数参数持久化布局。

use super::class_ids::class_id;
use super::{
    RasterFunctionArgumentsBody10_1, RuleSlot10_1, read_arguments_properties, read_basic_arguments,
    signed_version, slot, unsupported, unsupported_class, version,
};
use crate::layers::raster_dataset_name::{
    FUNCTION_RASTER_DATASET_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1, RasterNameSlot10_1,
    read_name_body, read_name_slot,
};
use crate::objects::dataset_name_more::{DatasetNameMoreSlot10_1, read_tabular_name_slot_10_1};
use crate::objects::property_set::PropertySetObjectStream10_1;
use crate::objects::string_array::{StringArraySlot10_1, read_string_array_slot_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io;

#[derive(Clone, Copy)]
pub(super) enum Layout {
    Base,
    Prefix,
    Suffix(u16, u16),
    RasterDatasetCrawler,
    TableCrawler,
    InlineBuilder(i16),
    AddRasters,
    UriProductNameFilter,
    RasterType,
    OverviewItemUri,
}

pub(super) fn layout(id: [u8; 16]) -> Option<Layout> {
    CLASSES
        .iter()
        .find_map(|(class, layout)| (*class == id).then_some(*layout))
}

const ADD_RASTERS: [u8; 16] = class_id(0x50088aff, 0x736a, 0x4448, 0xb00a18eb0c4971ea);
const SYNCHRONIZE: [u8; 16] = class_id(0x737c8139, 0x853b, 0x4c31, 0xa58aa1c4d7624e34);

#[derive(Debug, Clone, PartialEq)]
pub struct ItemUri10_1 {
    pub version: u16,
    pub strings: [String; 5],
    pub value: f64,
    pub strings_array: Option<StringArraySlot10_1>,
    pub properties: Option<RuleSlot10_1<PropertySetObjectStream10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MosaicArgumentsPayload10_1 {
    RasterDatasetCrawler {
        long: i32,
        name: RasterNameSlot10_1,
    },
    TableCrawler {
        name: DatasetNameMoreSlot10_1,
        long: Option<i32>,
    },
    InlineRasterDatasetBuilder {
        /// 内联构建器的基类版本及属性集记录在外层参数体。
        version: u16,
    },
    AddRasters {
        /// 内联 SynchronizeParameters 的基类版本及属性集。
        version: u16,
        properties: RuleSlot10_1<PropertySetObjectStream10_1>,
    },
    RasterType {
        name: RasterNameSlot10_1,
        add_rasters: Option<RuleSlot10_1<RasterFunctionArgumentsBody10_1>>,
        synchronize: Option<RuleSlot10_1<RasterFunctionArgumentsBody10_1>>,
    },
    OverviewItemUri {
        item_uri: ItemUri10_1,
        cell_size_version: u16,
        cell_size_properties: RuleSlot10_1<PropertySetObjectStream10_1>,
        function_name: RasterDatasetNameEntryBody10_1,
    },
    UriProductNames {
        text: String,
        short: u16,
        arrays: [StringArraySlot10_1; 2],
    },
}

pub(super) fn read(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    layout: Layout,
    depth: usize,
) -> io::Result<RasterFunctionArgumentsBody10_1> {
    use MosaicArgumentsPayload10_1 as Payload;

    let prefix = match layout {
        Layout::Prefix => Some(version(r, 1, "URIPropertiesFilter")?),
        Layout::UriProductNameFilter => Some(signed_version(r, 940, "URIProductNameFilter")?),
        _ => None,
    };
    if matches!(layout, Layout::UriProductNameFilter) && prefix.is_some_and(|n| (n as i16) <= 1) {
        let payload = Payload::UriProductNames {
            text: read_string_10_1(r, max)?,
            short: r.read_u16_le()?,
            arrays: [
                read_string_array_slot_10_1(r, max, refs)?,
                read_string_array_slot_10_1(r, max, refs)?,
            ],
        };
        return Ok(RasterFunctionArgumentsBody10_1 {
            has_basic_arguments: false,
            version: 0,
            properties: RuleSlot10_1 {
                entry: ObjectEntry10_1::Null,
                body: None,
            },
            merge: None,
            local_arguments_version: None,
            mosaic_version: prefix,
            mosaic_payload: Some(Box::new(payload)),
        });
    }

    let (base_version, properties) = read_basic_arguments(r, max, refs)?;
    let (mosaic_version, payload) = match layout {
        Layout::Suffix(min, max) => (Some(suffix(r, min, max)?), None),
        Layout::RasterDatasetCrawler => {
            let version = suffix(r, 1, 3)?;
            let payload = (version == 3)
                .then(|| {
                    Ok::<_, io::Error>(Payload::RasterDatasetCrawler {
                        long: r.read_u32_le()? as i32,
                        name: read_name_slot(r, max, refs, depth)?,
                    })
                })
                .transpose()?;
            (Some(version), payload)
        }
        Layout::TableCrawler => {
            let version = suffix(r, 1, 3)?;
            let payload = (version < 3)
                .then(|| {
                    let mut stop = None;
                    let name = read_tabular_name_slot_10_1(r, max, refs, &mut stop)?;
                    if let Some(reason) = stop {
                        return Err(unsupported(format!(
                            "TableCrawler 的 IName 未读完：{reason:?}"
                        )));
                    }
                    let long = (version == 2)
                        .then(|| r.read_u32_le().map(|n| n as i32))
                        .transpose()?;
                    Ok(Payload::TableCrawler { name, long })
                })
                .transpose()?;
            (Some(version), payload)
        }
        Layout::InlineBuilder(max_version) => {
            let version = suffix(r, 3, 3)?;
            let outer_version = signed_version(r, max_version, "MosaicDB 内联构建器")?;
            (
                Some(outer_version),
                Some(Payload::InlineRasterDatasetBuilder { version }),
            )
        }
        Layout::AddRasters => {
            let (version, properties) = read_basic_arguments(r, max, refs)?;
            (
                None,
                Some(Payload::AddRasters {
                    version,
                    properties,
                }),
            )
        }
        Layout::RasterType => {
            let name = read_name_slot(r, max, refs, depth)?;
            let version = suffix(r, 2, 943)?;
            let (add_rasters, synchronize) = if version == 943 {
                (
                    Some(read_parameter_slot(r, max, refs, &[ADD_RASTERS], depth)?),
                    Some(read_parameter_slot(
                        r,
                        max,
                        refs,
                        &[SYNCHRONIZE, ADD_RASTERS],
                        depth,
                    )?),
                )
            } else {
                (None, None)
            };
            (
                Some(version),
                Some(Payload::RasterType {
                    name,
                    add_rasters,
                    synchronize,
                }),
            )
        }
        Layout::OverviewItemUri => {
            let item_uri = read_item_uri(r, max, refs)?;
            let (cell_size_version, cell_size_properties) = read_basic_arguments(r, max, refs)?;
            let function_name = read_name_body(
                r,
                max,
                refs,
                FUNCTION_RASTER_DATASET_NAME_CLASS_ID,
                0,
                depth,
            )?;
            (
                None,
                Some(Payload::OverviewItemUri {
                    item_uri,
                    cell_size_version,
                    cell_size_properties,
                    function_name,
                }),
            )
        }
        Layout::Base | Layout::Prefix | Layout::UriProductNameFilter => (prefix, None),
    };
    Ok(RasterFunctionArgumentsBody10_1 {
        has_basic_arguments: true,
        version: base_version,
        properties,
        merge: None,
        local_arguments_version: None,
        mosaic_version,
        mosaic_payload: payload.map(Box::new),
    })
}

fn read_parameter_slot(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    allowed: &[[u8; 16]],
    depth: usize,
) -> io::Result<RuleSlot10_1<RasterFunctionArgumentsBody10_1>> {
    let parameter = slot(r, refs, |r, refs, id, _| {
        if !allowed.contains(&id) {
            return Err(unsupported_class("RasterType 参数", &id));
        }
        let layout = if id == ADD_RASTERS {
            Layout::AddRasters
        } else {
            Layout::Base
        };
        read(r, max, refs, layout, depth)
    })?;
    if let ObjectEntry10_1::BackReference { class_id, .. } = parameter.entry
        && !allowed.contains(&class_id)
    {
        return Err(unsupported_class("RasterType 参数回引", &class_id));
    }
    Ok(parameter)
}

fn read_item_uri(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<ItemUri10_1> {
    let version = suffix(r, 1, 941)?;
    let strings = [
        read_string_10_1(r, max)?,
        read_string_10_1(r, max)?,
        read_string_10_1(r, max)?,
        read_string_10_1(r, max)?,
        read_string_10_1(r, max)?,
    ];
    let value = r.read_f64_le()?;
    let strings_array = (version > 1)
        .then(|| read_string_array_slot_10_1(r, max, refs))
        .transpose()?;
    let properties = (version == 941)
        .then(|| read_arguments_properties(r, max, refs))
        .transpose()?;
    Ok(ItemUri10_1 {
        version,
        strings,
        value,
        strings_array,
        properties,
    })
}

fn suffix(r: &mut Reader<'_>, min: u16, max: u16) -> io::Result<u16> {
    let value = version(r, max, "MosaicDB 参数版本尾字段")?;
    if value < min {
        return Err(unsupported(format!(
            "MosaicDB 参数版本尾字段不支持 {value}"
        )));
    }
    Ok(value)
}

#[rustfmt::skip]
const CLASSES: [([u8; 16], Layout); 65] = [
    (class_id(0x7fde6784, 0x3e1f, 0x416f, 0xb80167750e5baa1d), Layout::Base), // CellSizeLevel
    (class_id(0x3c8017ce, 0x5091, 0x4b09, 0xba2e62f15454589a), Layout::Base), // DefineOverviewsParameters
    (class_id(0xa411f679, 0x9803, 0x479d, 0x885a2318a4b373e9), Layout::Base), // AnalyzeParameters
    (class_id(0xd54b5641, 0x8abd, 0x4f43, 0xacf45278afb1d79d), Layout::Base), // BuildBoundaryParameters
    (class_id(0xeeb74063, 0x1aad, 0x4578, 0x8131ede565724f61), Layout::Base), // BuildFootprintsParameters
    (class_id(0xfdf16309, 0xe5e2, 0x421b, 0x9ace0d2360b13e96), Layout::Base), // BuildSeamlinesParameters
    (class_id(0x0b50d766, 0x210d, 0x4526, 0x8919c324ba171507), Layout::Base), // CalculateCellSizeRangesParameters
    (class_id(0x56965c22, 0x3ade, 0x4a4a, 0xb35a523c21438921), Layout::Base), // CompositeURIFilter
    (class_id(0xe9de847a, 0x2888, 0x4ff1, 0xb384be60ffdae6f6), Layout::Base), // ComputeDirtyAreaParameters
    (class_id(0x9031f735, 0x38d2, 0x400b, 0xa6138bc0b511eed6), Layout::Base), // CreateMosaicDatasetParameters
    (class_id(0xb32bf91b, 0x51ef, 0x43dc, 0xb74ef686c30f23a1), Layout::Base), // DefineNoDataParameters
    (class_id(0x6651dd88, 0x26f4, 0x4816, 0xa43187f65f01901c), Layout::Base), // DeleteMosaicDatasetParameters
    (class_id(0x5c8a7664, 0x37e1, 0x424c, 0xbc9877a7430c3159), Layout::Base), // EnumMosaicDatasetJobs
    (class_id(0x34a5dcbe, 0xb169, 0x4a81, 0x884ddb98ea237f82), Layout::Base), // GenerateOverviewsParameters
    (class_id(0xa60416bb, 0x87a4, 0x483f, 0x9fd6239159c32a62), Layout::Base), // ImportGeometryParameters
    (class_id(0x61e87bba, 0xaa07, 0x44ad, 0x8f1c296c31d6ce9d), Layout::Base), // MergeItemsParameters
    (class_id(0xbe17e20a, 0xc88f, 0x4871, 0x891ed15561818267), Layout::Base), // MosaicDatasetSelectionParameters
    (class_id(0x950d3cd2, 0x730b, 0x46f9, 0x90a181a73ff3441f), Layout::Base), // QueryPathsParameters
    (class_id(0x98ab60aa, 0x951a, 0x4835, 0x94ca2c45561b4c57), Layout::Base), // RasterTypeDataSourceHandlerParameters
    (class_id(0xce6af0dc, 0x6ebd, 0x4655, 0xb287c851d1c52ab6), Layout::Base), // RemoveItemsParameters
    (class_id(0x9edf6058, 0x1368, 0x4a4b, 0x9fa6e437e24fc8ca), Layout::Base), // ReplacePathsParameters
    (class_id(0xa1cd9dbc, 0x65e7, 0x42e6, 0x8412eb9a1b2de168), Layout::Base), // SplitMergedItemsParameters
    (SYNCHRONIZE, Layout::Base), // SynchronizeParameters
    (class_id(0xbc454e95, 0x8235, 0x445d, 0x8ca1b298a80a8263), Layout::Base), // RemoveSlivers
    (class_id(0xde221c73, 0x5c88, 0x4a0d, 0xb78e13e3520e2061), Layout::Base), // SeamlineTaskInfo
    (class_id(0x4a5ff13e, 0x95ae, 0x411b, 0xa8ae3dfdcbbfbdec), Layout::Suffix(1, 1)), // ArrayCrawler
    (class_id(0x1b13f960, 0x8914, 0x4440, 0xa2d508b1b16c916d), Layout::Suffix(1, 1)), // FileCrawler
    (class_id(0x939d5a4f, 0x0ab2, 0x4aab, 0x96a6c66f481c118b), Layout::Suffix(1, 1)), // FolderCrawler
    (class_id(0xd796c3ce, 0x7d3a, 0x4bbd, 0x8d4374212b14e2b5), Layout::Suffix(1, 1)), // IkonosFileCrawler
    (class_id(0xcd87c31a, 0xcf00, 0x49af, 0x9dcf561cf08e2001), Layout::Suffix(1, 1)), // PixFileCrawler
    (class_id(0x6d09023d, 0x535f, 0x48fb, 0xbd5fbc500c451fea), Layout::Suffix(1, 1)), // QuickBirdFileCrawler
    (class_id(0x233d70dd, 0xb202, 0x4b84, 0x8f0e60f590bba259), Layout::Suffix(1, 1)), // CsvCrawler
    (class_id(0xffa21592, 0x9238, 0x4402, 0x98f60253312512dd), Layout::Suffix(1, 1)), // GenericObjectCrawler
    (class_id(0x5ebaeaa3, 0xb866, 0x4422, 0xb22eec86162a1444), Layout::Suffix(1, 1)), // OverviewCrawler
    (class_id(0xe51f2125, 0xd25d, 0x4d2e, 0x97f19753a5a92562), Layout::Suffix(1, 1)), // MosaicDatasetJob
    (class_id(0xaa3e2f92, 0xb23b, 0x427b, 0x8e2a68c0eeca4dfa), Layout::Prefix), // URIPropertiesFilter
    (class_id(0x3f2555fa, 0x4674, 0x4685, 0xb77cbe3d3ed75e5e), Layout::Suffix(2, 2)), // ISDefBuilder
    (class_id(0xbc7c6260, 0xc8fb, 0x40ac, 0x98458067f1185f92), Layout::Suffix(2, 2)), // RPDefBuilder
    (class_id(0xda19b110, 0x389b, 0x4300, 0x9c8a49687e41e4bd), Layout::Suffix(0, 2)), // LasBuilder
    (class_id(0x1e7b6221, 0x2ac3, 0x43d3, 0xa5fb0a986d25c99a), Layout::Suffix(0, 2)), // LasDatasetBuilder
    (class_id(0xea7603f9, 0xa5b5, 0x40c4, 0xa96a2044965279d8), Layout::Suffix(0, 2)), // TerrainBuilder
    (class_id(0x0b5fc20f, 0x59a7, 0x46dc, 0x8f75c58ca4c8c9e1), Layout::Suffix(0, 1)), // ApplanixBuilder
    (class_id(0xbadffa86, 0x5c20, 0x44b1, 0x8a660abc611b8aae), Layout::Suffix(0, 1)), // IkonosBuilder
    (class_id(0x32012433, 0x8ddf, 0x4aca, 0x968631def28d0e28), Layout::Suffix(0, 1)), // ISATBuilder
    (class_id(0x6e34d5d3, 0x4ae1, 0x43bc, 0xafb30c12022184d6), Layout::Suffix(0, 1)), // MatchATBuilder
    (class_id(0x24939700, 0xba0b, 0x439d, 0x954e09d6f140cb71), Layout::Suffix(0, 1)), // OverviewBuilder
    (class_id(0x18453678, 0xa175, 0x4ef1, 0x90fab793b1f33e6f), Layout::Suffix(0, 1)), // SpotBuilder
    (class_id(0xca963330, 0x4125, 0x4d21, 0xb05ddfb766c79243), Layout::Suffix(0, 1)), // KompsatBuilder
    (class_id(0x661cf6c7, 0xcd79, 0x4e72, 0xa0237df0a413041f), Layout::Suffix(0, 1)), // LandsatBuilder
    (class_id(0x8e0a8dd6, 0xdf21, 0x4e83, 0x859f64e53841981c), Layout::Suffix(0, 1)), // LandsatMSSBuilder
    (class_id(0xbf0d25e4, 0xb429, 0x417a, 0x9ea12abed24d106d), Layout::Suffix(0, 1)), // QuickBirdBuilder
    (class_id(0xa3c4e211, 0xe019, 0x45b9, 0x9709e7c609bf82d8), Layout::Suffix(0, 1)), // RadarBuilder
    (class_id(0xdf477dc1, 0x4ad8, 0x4e27, 0x9ea12b971536ac36), Layout::Suffix(0, 1)), // RapidEyeBuilder
    (class_id(0x62f7d26f, 0x0053, 0x4514, 0xb9f5e58a1b0aab84), Layout::Suffix(3, 3)), // RasterDatasetBuilder
    (class_id(0xea44519b, 0x5080, 0x4001, 0x8054cd5a93b761cb), Layout::Suffix(3, 3)), // TableBuilder
    (class_id(0x6a5bb65a, 0x9cf2, 0x4aa4, 0x9eb1d7e8de9a6dd6), Layout::RasterDatasetCrawler),
    (class_id(0x8b77c68d, 0x71ba, 0x4707, 0xaa032fb6e4a2c52b), Layout::TableCrawler),
    (class_id(0x1e076602, 0xa69e, 0x4ff6, 0x9c2ce23f1ff2cd80), Layout::InlineBuilder(2)), // HREBuilder
    (class_id(0x2c2351f9, 0x9b7c, 0x4208, 0x8d1734d7496007ee), Layout::InlineBuilder(2)), // NITFBuilder
    (class_id(0xc055b1ff, 0xf26c, 0x488c, 0xb8111f871b00f4a6), Layout::InlineBuilder(1)), // PixFileBuilder
    (class_id(0xa9965bfe, 0x7862, 0x4077, 0xa76d83c3a943d258), Layout::InlineBuilder(940)), // RPFBuilder
    (ADD_RASTERS, Layout::AddRasters), // AddRastersParameters
    (class_id(0x227397d1, 0xc518, 0x4edb, 0xaddfa07160aad029), Layout::UriProductNameFilter),
    (class_id(0xc7ee2957, 0xe230, 0x4be0, 0xae55a459b9647e21), Layout::RasterType),
    (class_id(0x550753de, 0x479a, 0x46ba, 0xa1c76eb86070d89c), Layout::OverviewItemUri),
];
