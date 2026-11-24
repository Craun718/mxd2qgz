//! GdbCore.dll 10.1 的 QueryFilter 与 SpatialFilter 持久化体。

use crate::layers::feature_id_set::{
    FID_SET_CLASS_ID, FeatureIdSetBody10_1, read_fid_set_body_10_1,
};
use crate::objects::dataset_name_more::DatasetNameMoreStop10_1;
use crate::objects::envelope::{
    ENVELOPE_CLASS_ID, Envelope10_1, SpatialReferenceEntry10_1, read_envelope_10_1,
    read_spatial_reference_body_10_1,
};
use crate::objects::geometry::{
    GEOMETRY_BAG_CLASS_ID, GeometryBody10_1, MULTIPATCH_CLASS_ID, MULTIPOINT_CLASS_ID,
    POINT_CLASS_ID, POLYGON_CLASS_ID, POLYLINE_CLASS_ID, read_keyed_geometry_body_10_1,
};
use crate::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
use crate::objects::spatial_reference_slots::{
    GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID, UNKNOWN_COORDINATE_SYSTEM_CLASS_ID,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::{read_bstr_string_10_1, read_string_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const QUERY_FILTER_CLASS_ID: [u8; 16] = [
    0x95, 0xbd, 0xfe, 0xfd, 0x75, 0xed, 0xd0, 0x11, 0x9a, 0x95, 0x08, 0x00, 0x09, 0xec, 0x73, 0x4b,
];
pub const SPATIAL_FILTER_CLASS_ID: [u8; 16] = [
    0x96, 0xbd, 0xfe, 0xfd, 0x75, 0xed, 0xd0, 0x11, 0x9a, 0x95, 0x08, 0x00, 0x09, 0xec, 0x73, 0x4b,
];
pub(crate) const FILTER_DEFS_CLASS_ID: [u8; 16] = [
    0x1a, 0x15, 0x31, 0x31, 0x5c, 0x4b, 0x26, 0x45, 0x98, 0xbe, 0x71, 0x17, 0x46, 0xb0, 0x3d, 0xf8,
];
pub(crate) const XML_FILTER_DEF_CLASS_ID: [u8; 16] = [
    0xdb, 0x75, 0xf9, 0x47, 0xc0, 0x77, 0x8c, 0x4a, 0xb8, 0xa0, 0x8e, 0x9f, 0xe7, 0x7a, 0x97, 0x95,
];

#[derive(Debug, Clone, PartialEq)]
pub enum QueryFilterObject10_1<T> {
    Null,
    BackReference { class_id: [u8; 16], key: u32 },
    Body { key: u32, body: Box<T> },
    Unsupported { class_id: [u8; 16], key: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryFilterBody10_1 {
    Query(Box<QueryFilterCore10_1>),
    Spatial {
        version: u16,
        core: Box<QueryFilterCore10_1>,
        spatial: Option<Box<SpatialFilterFields10_1>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryFilterCore10_1 {
    pub version: u16,
    pub sub_fields: String,
    pub where_clause: String,
    /// 持久化接口 +0x18 的第三个 String。
    pub string_c: String,
    pub output_reference: QueryFilterObject10_1<SpatialReferenceEntry10_1>,
    pub spatial_resolution: Option<f64>,
    pub fid_set: Option<QueryFilterObject10_1<FeatureIdSetBody10_1>>,
    pub postfix_clause: Option<String>,
    pub definitions: Option<FilterDefinitions10_1>,
    pub prefix_clause: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterDefinitions10_1 {
    Inline(Vec<QueryFilterObject10_1<XmlFilterDef10_1>>),
    Collection(QueryFilterObject10_1<FilterDefinitionsBody10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterDefinitionsBody10_1 {
    pub version: u16,
    pub entries: Vec<QueryFilterObject10_1<XmlFilterDef10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct XmlFilterDef10_1 {
    pub version: u16,
    pub strings: [String; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpatialFilterGeometry10_1 {
    Geometry(GeometryBody10_1),
    Envelope(Envelope10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpatialFilterFields10_1 {
    pub search_order: i32,
    pub spatial_rel: i32,
    pub spatial_rel_description: String,
    pub geometry: QueryFilterObject10_1<SpatialFilterGeometry10_1>,
    pub geometry_field: Option<String>,
    /// FilterOwnsGeometry，保留磁盘 short 原值。
    pub filter_owns_geometry: Option<u16>,
}

type Stop = Option<DatasetNameMoreStop10_1>;

fn read_version(reader: &mut Reader<'_>, max: i16, class: &str) -> io::Result<u16> {
    let version = reader.read_u16_le()?;
    if (version as i16) > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{class} 不支持版本 {version}"),
        ));
    }
    Ok(version)
}

fn read_slot<T>(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    stop: &mut Stop,
    accepts_reference: impl FnOnce([u8; 16]) -> bool,
    read: impl FnOnce(
        &mut Reader<'_>,
        &mut RefTable10_1,
        [u8; 16],
        u32,
        &mut Stop,
    ) -> io::Result<Option<T>>,
) -> io::Result<QueryFilterObject10_1<T>> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let slot = match read_object_entry_10_1(&mut trial, &mut staged)? {
        ObjectEntry10_1::Null => QueryFilterObject10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            let registered = staged.registered_class(key);
            let class_id = registered.unwrap_or(class_id);
            if registered.is_some() && !accepts_reference(class_id) {
                *stop = Some(DatasetNameMoreStop10_1::SlotUnsupported { class_id, key });
                return Ok(QueryFilterObject10_1::Unsupported { class_id, key });
            }
            QueryFilterObject10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            let Some(body) = read(&mut trial, &mut staged, class_id, key, stop)? else {
                *stop = Some(DatasetNameMoreStop10_1::SlotUnsupported { class_id, key });
                return Ok(QueryFilterObject10_1::Unsupported { class_id, key });
            };
            if let Some(inner) = stop.take() {
                *stop = Some(DatasetNameMoreStop10_1::Nested {
                    class_id,
                    key,
                    inner: Box::new(inner),
                });
            }
            QueryFilterObject10_1::Body {
                key,
                body: Box::new(body),
            }
        }
    };
    *reader = trial;
    *table = staged;
    Ok(slot)
}

pub(crate) fn read_xml_filter_def_body_10_1(
    reader: &mut Reader<'_>,
    max: usize,
) -> io::Result<XmlFilterDef10_1> {
    let version = reader.read_u16_le()?;
    if version != 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("XMLFilterDef 不支持版本 {version}"),
        ));
    }
    Ok(XmlFilterDef10_1 {
        version,
        strings: [
            read_bstr_string_10_1(reader, max)?,
            read_bstr_string_10_1(reader, max)?,
        ],
    })
}

pub(crate) fn read_filter_defs_body_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    stop: &mut Stop,
) -> io::Result<FilterDefinitionsBody10_1> {
    Ok(FilterDefinitionsBody10_1 {
        version: read_version(reader, 1, "FilterDefs")?,
        entries: read_definitions(reader, max, table, stop)?,
    })
}

fn read_definitions(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    stop: &mut Stop,
) -> io::Result<Vec<QueryFilterObject10_1<XmlFilterDef10_1>>> {
    // 两个原生宿主均仅在有符号计数为正时进入循环。
    let count = (reader.read_u32_le()? as i32).max(0) as usize;
    if count > reader.remaining() / 16 {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "FilterDefs 条目截断",
        ));
    }
    let mut entries = Vec::new();
    for _ in 0..count {
        entries.push(read_slot(
            reader,
            table,
            stop,
            |class| class == XML_FILTER_DEF_CLASS_ID,
            |reader, _, class, _, _| {
                if class != XML_FILTER_DEF_CLASS_ID {
                    return Ok(None);
                }
                read_xml_filter_def_body_10_1(reader, max).map(Some)
            },
        )?);
        if stop.is_some() {
            break;
        }
    }
    Ok(entries)
}

fn read_core(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    stop: &mut Stop,
) -> io::Result<QueryFilterCore10_1> {
    let version = read_version(reader, 4, "QueryFilter")?;
    let sub_fields = read_string_10_1(reader, max)?;
    let where_clause = read_string_10_1(reader, max)?;
    let string_c = read_string_10_1(reader, max)?;
    let output_reference = read_slot(
        reader,
        table,
        stop,
        |class| {
            matches!(
                class,
                PROJECTED_COORDINATE_SYSTEM_CLASS_ID
                    | GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID
                    | UNKNOWN_COORDINATE_SYSTEM_CLASS_ID
            )
        },
        |reader, _, class_id, key, _| {
            read_spatial_reference_body_10_1(
                reader,
                max,
                &ObjectEntry10_1::NewObject { class_id, key },
            )
        },
    )?;
    let mut core = QueryFilterCore10_1 {
        version,
        sub_fields,
        where_clause,
        string_c,
        output_reference,
        spatial_resolution: None,
        fid_set: None,
        postfix_clause: None,
        definitions: None,
        prefix_clause: None,
    };
    if stop.is_some() {
        return Ok(core);
    }
    core.spatial_resolution = Some(reader.read_f64_le()?);
    core.fid_set = Some(read_slot(
        reader,
        table,
        stop,
        |class| class == FID_SET_CLASS_ID,
        |reader, table, class, key, _| {
            if class == FID_SET_CLASS_ID {
                read_fid_set_body_10_1(reader, table, key).map(Some)
            } else {
                Ok(None)
            }
        },
    )?);
    if stop.is_some() || version == 1 {
        return Ok(core);
    }
    core.postfix_clause = Some(read_bstr_string_10_1(reader, max)?);
    core.definitions = Some(if version == 2 {
        FilterDefinitions10_1::Inline(read_definitions(reader, max, table, stop)?)
    } else {
        FilterDefinitions10_1::Collection(read_slot(
            reader,
            table,
            stop,
            |class| class == FILTER_DEFS_CLASS_ID,
            |reader, table, class, _, stop| {
                if class != FILTER_DEFS_CLASS_ID {
                    return Ok(None);
                }
                read_filter_defs_body_10_1(reader, max, table, stop).map(Some)
            },
        )?)
    });
    if stop.is_none() && version == 4 {
        core.prefix_clause = Some(read_bstr_string_10_1(reader, max)?);
    }
    Ok(core)
}

fn read_spatial_fields(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    stop: &mut Stop,
) -> io::Result<SpatialFilterFields10_1> {
    let search_order = reader.read_u32_le()? as i32;
    let spatial_rel = reader.read_u32_le()? as i32;
    let spatial_rel_description = read_string_10_1(reader, max)?;
    let geometry = read_slot(
        reader,
        table,
        stop,
        |class| {
            matches!(
                class,
                ENVELOPE_CLASS_ID
                    | POINT_CLASS_ID
                    | MULTIPOINT_CLASS_ID
                    | POLYLINE_CLASS_ID
                    | POLYGON_CLASS_ID
                    | GEOMETRY_BAG_CLASS_ID
                    | MULTIPATCH_CLASS_ID
            )
        },
        |reader, table, class, key, _| {
            Ok(Some(match class {
                ENVELOPE_CLASS_ID => {
                    SpatialFilterGeometry10_1::Envelope(read_envelope_10_1(reader, max, table)?)
                }
                POINT_CLASS_ID
                | MULTIPOINT_CLASS_ID
                | POLYLINE_CLASS_ID
                | POLYGON_CLASS_ID
                | GEOMETRY_BAG_CLASS_ID
                | MULTIPATCH_CLASS_ID => SpatialFilterGeometry10_1::Geometry(
                    read_keyed_geometry_body_10_1(&class, key, reader, max, table)?,
                ),
                _ => return Ok(None),
            }))
        },
    )?;
    let (geometry_field, filter_owns_geometry) = if stop.is_none() {
        (
            Some(read_string_10_1(reader, max)?),
            Some(reader.read_u16_le()?),
        )
    } else {
        (None, None)
    };
    Ok(SpatialFilterFields10_1 {
        search_order,
        spatial_rel,
        geometry_field,
        geometry,
        spatial_rel_description,
        filter_owns_geometry,
    })
}

pub(crate) fn read_query_filter_slot_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    stop: &mut Stop,
) -> io::Result<QueryFilterObject10_1<QueryFilterBody10_1>> {
    read_slot(
        reader,
        table,
        stop,
        |class| matches!(class, QUERY_FILTER_CLASS_ID | SPATIAL_FILTER_CLASS_ID),
        |reader, table, class, _, stop| {
            read_query_filter_body_10_1(reader, max, table, class, stop)
        },
    )
}

pub(crate) fn read_query_filter_body_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    class: [u8; 16],
    stop: &mut Stop,
) -> io::Result<Option<QueryFilterBody10_1>> {
    Ok(Some(match class {
        QUERY_FILTER_CLASS_ID => {
            QueryFilterBody10_1::Query(Box::new(read_core(reader, max, table, stop)?))
        }
        SPATIAL_FILTER_CLASS_ID => {
            let version = read_version(reader, 1, "SpatialFilter")?;
            let core = Box::new(read_core(reader, max, table, stop)?);
            let spatial = if stop.is_none() {
                Some(Box::new(read_spatial_fields(reader, max, table, stop)?))
            } else {
                None
            };
            QueryFilterBody10_1::Spatial {
                version,
                core,
                spatial,
            }
        }
        _ => return Ok(None),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn append_header(bytes: &mut Vec<u8>, class: [u8; 16], key: u32) {
        bytes.extend(class);
        bytes.extend(key.to_le_bytes());
    }

    #[test]
    fn spatial_filter_variant_reads_multipatch_and_rolls_back_on_short_geometry() {
        use crate::objects::property_set::{
            PropertyValueObjectStream10_1, read_padded_variant_object_stream_10_1,
        };

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        append_header(&mut bytes, SPATIAL_FILTER_CLASS_ID, 16);
        bytes.extend([1, 0, 1, 0]); // 空间过滤器与基础过滤器版本。
        bytes.extend([2, 0, 0, 0, 0, 0].repeat(3));
        bytes.extend([0; 16 + 8 + 16]); // 空输出坐标系、分辨率和空 FIDSet。
        bytes.extend([0; 8]);
        bytes.extend([2, 0, 0, 0, 0, 0]);
        append_header(&mut bytes, MULTIPATCH_CLASS_ID, 17);
        let mut record = 0x1f_u32.to_le_bytes().to_vec();
        record.extend([0; 32]); // XY 范围。
        for value in [1_u32, 3, 0, 0x100] {
            record.extend(value.to_le_bytes());
        }
        record.extend([0; 6 * 8 + 5 * 8]); // 三点的 XY、Z 范围与 Z 值。
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend((record.len() as u32).to_le_bytes());
        bytes.extend(record);
        bytes.extend([0; 16]);
        let end_geometry = bytes.len();
        bytes.extend([4, 0, 0, 0, b'g', 0, 0, 0]);
        bytes.extend(1_u16.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xaa);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_padded_variant_object_stream_10_1(&mut reader, 512, &mut refs).unwrap();
        let PropertyValueObjectStream10_1::QueryFilter(filter) = result else {
            panic!("缺少空间过滤器");
        };
        let QueryFilterBody10_1::Spatial {
            spatial: Some(spatial),
            ..
        } = filter.body()
        else {
            panic!("缺少空间字段");
        };
        assert!(
            matches!(&spatial.geometry, QueryFilterObject10_1::Body { key: 17, body }
            if matches!(body.as_ref(), SpatialFilterGeometry10_1::Geometry(GeometryBody10_1::MultiPatch(_))))
        );
        assert_eq!(spatial.geometry_field.as_deref(), Some("g"));
        assert_eq!(spatial.filter_owns_geometry, Some(1));
        assert!(refs.contains(16) && refs.contains(17));
        assert_eq!(reader.position(), end);

        let mut reader = Reader::new(&bytes[..end_geometry - 1]);
        let mut refs = RefTable10_1::new();
        assert!(read_padded_variant_object_stream_10_1(&mut reader, 512, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn spatial_filter_reads_geometry_bag_and_its_following_fields() {
        let mut bytes = vec![0; 8];
        bytes.extend([2, 0, 0, 0, 0, 0]);
        append_header(&mut bytes, GEOMETRY_BAG_CLASS_ID, 17);
        bytes.extend([0; 2 + 4 + 16 + 16 + 1]);
        bytes.extend([4, 0, 0, 0, b'g', 0, 0, 0]);
        bytes.extend(1_u16.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xaa);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let mut stop = None;
        let body = read_spatial_fields(&mut reader, 128, &mut refs, &mut stop).unwrap();
        assert!(
            matches!(body.geometry, QueryFilterObject10_1::Body { key: 17, body }
            if matches!(*body, SpatialFilterGeometry10_1::Geometry(GeometryBody10_1::GeometryBag(_))))
        );
        assert_eq!(body.geometry_field.as_deref(), Some("g"));
        assert_eq!(body.filter_owns_geometry, Some(1));
        assert!(refs.contains(17) && stop.is_none());
        assert_eq!(reader.position(), end);
    }

    #[test]
    fn output_reference_checks_registered_spatial_reference_class() {
        for registered in [PROJECTED_COORDINATE_SYSTEM_CLASS_ID, QUERY_FILTER_CLASS_ID] {
            let mut header = Vec::new();
            append_header(&mut header, registered, 7);
            let mut refs = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&header), &mut refs).unwrap();
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([2, 0, 0, 0, 0, 0].repeat(3));
            let offset = bytes.len();
            append_header(&mut bytes, PROJECTED_COORDINATE_SYSTEM_CLASS_ID, 7);
            bytes.extend([0; 8 + 16]);
            let mut reader = Reader::new(&bytes);
            let mut stop = None;
            let body = read_core(&mut reader, 128, &mut refs, &mut stop).unwrap();
            if registered == PROJECTED_COORDINATE_SYSTEM_CLASS_ID {
                assert!(matches!(
                    body.output_reference,
                    QueryFilterObject10_1::BackReference { key: 7, .. }
                ));
                assert_eq!(reader.position(), bytes.len());
            } else {
                assert!(matches!(
                    body.output_reference,
                    QueryFilterObject10_1::Unsupported { key: 7, .. }
                ));
                assert_eq!(reader.position(), offset);
                assert!(stop.is_some());
            }
            assert!(refs.contains(7));
        }
    }

    #[test]
    fn xml_filter_member_reference_checks_the_registered_class() {
        for registered in [XML_FILTER_DEF_CLASS_ID, FILTER_DEFS_CLASS_ID] {
            let mut header = Vec::new();
            append_header(&mut header, registered, 11);
            let mut refs = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&header), &mut refs).unwrap();
            let before = refs.clone();
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(1_u32.to_le_bytes());
            let claimed = if registered == XML_FILTER_DEF_CLASS_ID {
                FILTER_DEFS_CLASS_ID
            } else {
                XML_FILTER_DEF_CLASS_ID
            };
            append_header(&mut bytes, claimed, 11);
            let mut reader = Reader::new(&bytes);
            let mut stop = None;
            let body = read_filter_defs_body_10_1(&mut reader, 128, &mut refs, &mut stop).unwrap();
            assert_eq!(refs, before);
            if registered == XML_FILTER_DEF_CLASS_ID {
                assert!(stop.is_none());
                assert_eq!(
                    body.entries,
                    [QueryFilterObject10_1::BackReference {
                        class_id: registered,
                        key: 11,
                    }]
                );
                assert_eq!(reader.position(), bytes.len());
            } else {
                assert!(stop.is_some());
                assert_eq!(
                    body.entries,
                    [QueryFilterObject10_1::Unsupported {
                        class_id: registered,
                        key: 11,
                    }]
                );
                assert_eq!(reader.position(), 6);
            }
        }
    }

    #[test]
    fn filter_versions_follow_native_comparisons() {
        for (version, supported) in [(u16::MAX, true), (0, true), (1, true), (2, false)] {
            let mut bytes = version.to_le_bytes().to_vec();
            bytes.extend([0; 8]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let defs = read_filter_defs_body_10_1(&mut reader, 128, &mut refs, &mut None);
            assert_eq!(defs.is_ok(), supported);
            if let Ok(defs) = defs {
                assert_eq!(defs.version, version);
                assert_eq!(reader.position(), 6);
            }
            let mut reader = Reader::new(&bytes);
            let xml = read_xml_filter_def_body_10_1(&mut reader, 128);
            assert_eq!(xml.is_ok(), version == 1);
            if xml.is_ok() {
                assert_eq!(reader.position(), 10);
            }
        }
    }

    #[test]
    fn filter_definition_lists_skip_nonpositive_counts() {
        for count in [i32::MIN, -1, 0] {
            let mut defs = 1_u16.to_le_bytes().to_vec();
            defs.extend(count.to_le_bytes());
            defs.push(0xaa);
            let mut reader = Reader::new(&defs);
            let mut refs = RefTable10_1::new();
            let body = read_filter_defs_body_10_1(&mut reader, 128, &mut refs, &mut None).unwrap();
            assert!(body.entries.is_empty());
            assert_eq!(reader.read_bytes(1).unwrap(), &[0xaa]);

            let mut query = Vec::new();
            append_header(&mut query, QUERY_FILTER_CLASS_ID, 31);
            query.extend(2_u16.to_le_bytes());
            query.extend([2, 0, 0, 0, 0, 0].repeat(3));
            query.extend([0; 16 + 8 + 16 + 4]); // 空空间参考、分辨率、空 FIDSet 和空后缀。
            query.extend(count.to_le_bytes());
            query.push(0xaa);
            let mut reader = Reader::new(&query);
            let mut stop = None;
            let result =
                read_query_filter_slot_10_1(&mut reader, 128, &mut refs, &mut stop).unwrap();
            let QueryFilterObject10_1::Body { body, .. } = result else {
                panic!("缺少过滤器对象体");
            };
            let QueryFilterBody10_1::Query(body) = *body else {
                panic!("缺少 QueryFilter");
            };
            assert!(
                matches!(body.definitions, Some(FilterDefinitions10_1::Inline(entries))
                if entries.is_empty())
            );
            assert!(stop.is_none() && refs.contains(31));
            assert_eq!(reader.read_bytes(1).unwrap(), &[0xaa]);
        }
    }

    #[test]
    fn filter_definition_variants_preserve_nested_keys_and_boundaries() {
        use crate::objects::property_set::{
            PropertyValueObjectStream10_1, read_padded_variant_object_stream_10_1,
        };

        for collection in [false, true] {
            let mut bytes = 13_u16.to_le_bytes().to_vec();
            bytes.extend([0; 14]);
            if collection {
                append_header(&mut bytes, FILTER_DEFS_CLASS_ID, 4);
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(1_u32.to_le_bytes());
            }
            append_header(&mut bytes, XML_FILTER_DEF_CLASS_ID, 5);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 8]); // 两个空 BSTR。
            let end = bytes.len();
            bytes.push(0xaa);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let value =
                read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
            match value {
                PropertyValueObjectStream10_1::FilterDefinitions(value) => {
                    assert!(collection);
                    assert_eq!(value.body().entries.len(), 1);
                    assert!(refs.contains(4));
                }
                PropertyValueObjectStream10_1::XmlFilterDef(value) => {
                    assert!(!collection);
                    assert_eq!(value.body().strings, ["", ""]);
                }
                other => panic!("过滤定义分派错误：{other:?}"),
            }
            assert!(refs.contains(5));
            assert_eq!(reader.position(), end);

            let mut reader = Reader::new(&bytes[..end - 1]);
            let mut refs = RefTable10_1::new();
            assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn variant_filter_reads_body_and_leaves_following_byte() {
        use crate::objects::property_set::{
            PropertyValueObjectStream10_1, read_padded_variant_object_stream_10_1,
        };

        for class_id in [QUERY_FILTER_CLASS_ID, SPATIAL_FILTER_CLASS_ID] {
            let spatial = class_id == SPATIAL_FILTER_CLASS_ID;
            let mut bytes = 13_u16.to_le_bytes().to_vec();
            bytes.extend([0; 14]);
            append_header(&mut bytes, class_id, 19);
            if spatial {
                bytes.extend(1_u16.to_le_bytes());
            }
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([2, 0, 0, 0, 0, 0].repeat(3));
            bytes.extend([0; 16 + 8 + 16]);
            if spatial {
                bytes.extend([0; 8]);
                bytes.extend([2, 0, 0, 0, 0, 0]);
                bytes.extend([0; 16]);
                bytes.extend([2, 0, 0, 0, 0, 0]);
                bytes.extend(1_u16.to_le_bytes());
            }
            let end = bytes.len();
            bytes.push(0xaa);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let value =
                read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
            let PropertyValueObjectStream10_1::QueryFilter(value) = value else {
                panic!("缺少查询过滤器");
            };
            assert_eq!(value.class_id, class_id);
            assert_eq!(value.key, 19);
            assert_eq!(
                matches!(value.body(), QueryFilterBody10_1::Spatial { .. }),
                spatial
            );
            assert!(refs.contains(19));
            assert_eq!(reader.position(), end);

            let mut reader = Reader::new(&bytes[..end - 1]);
            let mut refs = RefTable10_1::new();
            assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn filter_definitions_register_xml_children_and_stop_at_unknown_ones() {
        let mut bytes = Vec::new();
        append_header(&mut bytes, QUERY_FILTER_CLASS_ID, 1);
        bytes.extend(4u16.to_le_bytes());
        for _ in 0..3 {
            bytes.extend([2, 0, 0, 0, 0, 0]); // 空 String，长度含终止零。
        }
        bytes.extend([0; 16 + 8]);
        append_header(&mut bytes, FID_SET_CLASS_ID, 2);
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(1u32.to_le_bytes());
        bytes.extend(99i32.to_le_bytes());
        bytes.extend(0u32.to_le_bytes()); // 空 BSTR。
        append_header(&mut bytes, FILTER_DEFS_CLASS_ID, 3);
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(2u32.to_le_bytes());
        append_header(&mut bytes, XML_FILTER_DEF_CLASS_ID, 4);
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(1u32.to_le_bytes());
        bytes.extend([b'X', 0, 0, 0]);
        bytes.extend(0u32.to_le_bytes());
        let last = bytes.len();
        append_header(&mut bytes, XML_FILTER_DEF_CLASS_ID, 4);
        bytes.extend(0u32.to_le_bytes()); // PrefixClause。

        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let mut stop = None;
        let result = read_query_filter_slot_10_1(&mut reader, 64, &mut table, &mut stop).unwrap();
        let QueryFilterObject10_1::Body { key: 1, body } = result else {
            panic!("应读到 QueryFilter");
        };
        let QueryFilterBody10_1::Query(core) = *body else {
            panic!("应读到普通 QueryFilter");
        };
        let Some(FilterDefinitions10_1::Collection(QueryFilterObject10_1::Body { key: 3, body })) =
            core.definitions.as_ref()
        else {
            panic!("应读到 FilterDefs");
        };
        let [
            QueryFilterObject10_1::Body { key: 4, body: xml },
            QueryFilterObject10_1::BackReference { key: 4, .. },
        ] = body.entries.as_slice()
        else {
            panic!("应读到 XMLFilterDef 及其回引");
        };
        assert_eq!(xml.strings, ["X", ""]);
        assert!(
            matches!(core.fid_set.as_ref(), Some(QueryFilterObject10_1::Body { key: 2, body }) if body.ids == [99])
        );
        assert_eq!(core.prefix_clause.as_deref(), Some(""));
        assert_eq!(reader.position(), bytes.len());
        assert_eq!(table.len(), 4);
        assert_eq!(stop, None);

        bytes[last..last + 16].fill(0x55);
        bytes[last + 16..last + 20].copy_from_slice(&5u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        read_query_filter_slot_10_1(&mut reader, 64, &mut table, &mut stop).unwrap();
        assert!(matches!(
            stop,
            Some(DatasetNameMoreStop10_1::Nested { key: 1, inner, .. }) if matches!(
                inner.as_ref(), DatasetNameMoreStop10_1::Nested { key: 3, inner, .. }
                    if matches!(inner.as_ref(), DatasetNameMoreStop10_1::SlotUnsupported { key: 5, .. })
            )
        ));
        assert_eq!(reader.position(), last);
        assert_eq!(table.len(), 4);
        assert!(!table.contains(5));
    }
}
