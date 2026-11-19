//! FunctionRasterDB.dll 10.1 的 RasterInfo 二进制持久化体。

use super::{RuleSlot10_1, slot, unsupported_class, version};
use crate::objects::envelope::{
    ENVELOPE_CLASS_ID, Envelope10_1, SpatialReferenceEntry10_1, read_envelope_10_1,
    read_spatial_reference_entry_10_1,
};
use crate::objects::geodata_xform::{GeodataXformSlot10_1, read_geodata_xform_slot_10_1};
use crate::objects::geometry::{POINT_CLASS_ID, PointBody10_1, read_point_body_10_1};
use crate::objects::property_set::{
    PropertyValueObjectStream10_1, read_padded_variant_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub(super) const CLASS_ID: [u8; 16] = [
    0xca, 0x5f, 0x1e, 0x4f, 0x34, 0x23, 0x80, 0x42, 0xb1, 0x09, 0x17, 0xb3, 0xca, 0xf9, 0x29, 0x0e,
];
const RASTER_STATISTICS: [u8; 16] = [
    0x83, 0xeb, 0x46, 0x84, 0x51, 0x32, 0x20, 0x4f, 0x8b, 0x79, 0x1d, 0x54, 0x21, 0xe3, 0x9d, 0x4a,
];
const RASTER_HISTOGRAM: [u8; 16] = [
    0xea, 0x93, 0x12, 0xb0, 0xd0, 0x88, 0x7d, 0x4e, 0xaf, 0x82, 0xa0, 0xb1, 0x0b, 0xac, 0x9b, 0x33,
];
pub(super) const RASTER_COLORMAP: [u8; 16] = [
    0x22, 0x57, 0x19, 0x92, 0x32, 0x57, 0xd2, 0x11, 0xaa, 0xad, 0x00, 0xc0, 0x4f, 0xa3, 0x75, 0xfb,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RasterInfo10_1 {
    pub version: u16,
    pub transform: GeodataXformSlot10_1,
    pub first_extent: RuleSlot10_1<Envelope10_1>,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub second_extent: RuleSlot10_1<Envelope10_1>,
    pub doubles: [f64; 2],
    pub initial_longs: [i32; 2],
    pub value: PropertyValueObjectStream10_1,
    pub longs: [i32; 4],
    pub flags: u16,
    pub point: Option<RuleSlot10_1<PointBody10_1>>,
    pub text: Option<String>,
    pub statistics: Vec<RuleSlot10_1<RasterStatistics10_1>>,
    pub histograms: Vec<RuleSlot10_1<RasterInfoHistogram10_1>>,
    pub colormap: Option<RuleSlot10_1<RasterInfoColormap10_1>>,
    pub v6_entry: Option<ObjectEntry10_1>,
    pub v7_long: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterStatistics10_1 {
    pub version: u16,
    pub doubles: [f64; 6],
    pub longs: [i32; 2],
    pub value: PropertyValueObjectStream10_1,
    pub v2_value: Option<PropertyValueObjectStream10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterInfoHistogram10_1 {
    pub version: u16,
    pub value: PropertyValueObjectStream10_1,
    pub doubles: [f64; 3],
    pub long: i32,
    pub bins: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterInfoColormap10_1 {
    pub version: u16,
    pub v2_value: Option<PropertyValueObjectStream10_1>,
    pub value: PropertyValueObjectStream10_1,
}

pub(super) fn read_slot(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<RasterInfo10_1>> {
    slot(r, refs, |r, refs, class_id, _| {
        if class_id != CLASS_ID {
            return Err(unsupported_class("IRasterInfo", &class_id));
        }
        read_body(r, max, refs)
    })
}

pub(super) fn read_body(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RasterInfo10_1> {
    let body_version = version(r, 7, "RasterInfo")?;
    let transform = read_geodata_xform_slot_10_1(r, max, refs)?;
    let first_extent = read_extent(r, refs, max)?;
    let spatial_reference = read_spatial_reference_entry_10_1(r, max, refs)?;
    let second_extent = read_extent(r, refs, max)?;
    let doubles = [r.read_f64_le()?, r.read_f64_le()?];
    let initial_longs = [read_long(r)?, read_long(r)?];
    let value = read_padded_variant_object_stream_10_1(r, max, refs)?;
    let longs = [read_long(r)?, read_long(r)?, read_long(r)?, read_long(r)?];
    let flags = r.read_u16_le()?;
    let point = (body_version >= 2)
        .then(|| {
            slot(r, refs, |r, refs, id, _| {
                if id != POINT_CLASS_ID {
                    return Err(unsupported_class("RasterInfo IPoint", &id));
                }
                read_point_body_10_1(r, max, refs)
            })
        })
        .transpose()?;
    let text = (body_version >= 3)
        .then(|| read_string_10_1(r, max))
        .transpose()?;
    let (statistics, histograms) = if body_version >= 4 {
        let statistics = (0..read_count(r, max, 16)?)
            .map(|_| {
                slot(r, refs, |r, refs, id, _| {
                    if id != RASTER_STATISTICS {
                        return Err(unsupported_class("IRasterStatistics", &id));
                    }
                    read_statistics(r, max, refs)
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        let histograms = (0..read_count(r, max, 16)?)
            .map(|_| {
                slot(r, refs, |r, refs, id, _| {
                    if id != RASTER_HISTOGRAM {
                        return Err(unsupported_class("IRasterHistogram", &id));
                    }
                    read_histogram(r, max, refs)
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        (statistics, histograms)
    } else {
        (Vec::new(), Vec::new())
    };
    let colormap = (body_version >= 5)
        .then(|| {
            slot(r, refs, |r, refs, id, _| {
                if id != RASTER_COLORMAP {
                    return Err(unsupported_class("RasterInfo IRasterColormap", &id));
                }
                let version = version(r, 2, "RasterColormap")?;
                let v2_value = (version == 2)
                    .then(|| read_padded_variant_object_stream_10_1(r, max, refs))
                    .transpose()?;
                let value = read_padded_variant_object_stream_10_1(r, max, refs)?;
                Ok(RasterInfoColormap10_1 {
                    version,
                    v2_value,
                    value,
                })
            })
        })
        .transpose()?;
    let v6_entry = (body_version >= 6)
        .then(|| read_pnt_slot(r, refs))
        .transpose()?;
    let v7_long = (body_version >= 7).then(|| read_long(r)).transpose()?;
    Ok(RasterInfo10_1 {
        version: body_version,
        transform,
        first_extent,
        spatial_reference,
        second_extent,
        doubles,
        initial_longs,
        value,
        longs,
        flags,
        point,
        text,
        statistics,
        histograms,
        colormap,
        v6_entry,
        v7_long,
    })
}

fn read_extent(
    r: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    max: usize,
) -> io::Result<RuleSlot10_1<Envelope10_1>> {
    slot(r, refs, |r, refs, id, _| {
        if id != ENVELOPE_CLASS_ID {
            return Err(unsupported_class("RasterInfo IEnvelope", &id));
        }
        read_envelope_10_1(r, max, refs)
    })
}

fn read_statistics(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RasterStatistics10_1> {
    let version = version(r, 2, "RasterStatistics")?;
    let mut doubles = [0.0; 6];
    for value in &mut doubles {
        *value = r.read_f64_le()?;
    }
    let longs = [read_long(r)?, read_long(r)?];
    let value = read_padded_variant_object_stream_10_1(r, max, refs)?;
    let v2_value = (version == 2)
        .then(|| read_padded_variant_object_stream_10_1(r, max, refs))
        .transpose()?;
    Ok(RasterStatistics10_1 {
        version,
        doubles,
        longs,
        value,
        v2_value,
    })
}

fn read_histogram(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RasterInfoHistogram10_1> {
    let version = version(r, 1, "RasterHistogram")?;
    let value = read_padded_variant_object_stream_10_1(r, max, refs)?;
    let doubles = [r.read_f64_le()?, r.read_f64_le()?, r.read_f64_le()?];
    let long = read_long(r)?;
    let bins = (0..read_count(r, max, 8)?)
        .map(|_| r.read_f64_le())
        .collect::<io::Result<_>>()?;
    Ok(RasterInfoHistogram10_1 {
        version,
        value,
        doubles,
        long,
        bins,
    })
}

pub(super) fn read_pnt_slot(
    r: &mut Reader<'_>,
    refs: &mut RefTable10_1,
) -> io::Result<ObjectEntry10_1> {
    slot(r, refs, |_, _, id, _| {
        Err::<(), _>(unsupported_class("IPnt", &id))
    })
    .map(|result| result.entry)
}

fn read_count(r: &mut Reader<'_>, max: usize, min_bytes: usize) -> io::Result<usize> {
    let count = read_long(r)?.max(0) as usize;
    if count > r.remaining().min(max) / min_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RasterInfo 元素数量超限",
        ));
    }
    Ok(count)
}

fn read_long(r: &mut Reader<'_>) -> io::Result<i32> {
    Ok(r.read_u32_le()? as i32)
}
