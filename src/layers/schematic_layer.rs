//! ArcMap 10.1 示意图图层。字段顺序见 docs/arcmap-10.1-dep-tail.md。

use super::layer_list::{LayerBodyPrefix10_1, read_complete_layer_body_10_1};
use super::raster_layer_tail::{RasterLayerEnvelopeSlot10_1, read_envelope_slot};
use crate::maps::layer_masking::{LayerMaskingSupport10_1, known_layer_masking_support_10_1};
use crate::objects::dataset_name_more::{DatasetNameMoreSlot10_1, read_overlay_name_slot_10_1};
use crate::objects::hyperlink::{HYPERLINK_CLASS_ID, HyperlinkBody10_1, read_hyperlink_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io::{self, ErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicLayer10_1 {
    pub version: i16,
    pub name: String,
    pub shorts: [i16; 2],
    pub values: [f64; 3],
    pub secondary_name: String,
    pub version_one: Option<i16>,
    pub dataset: Option<SchematicDataset10_1>,
    pub legacy: Option<SchematicLegacy10_1>,
    pub envelopes: Option<[RasterLayerEnvelopeSlot10_1; 2]>,
    pub associated: Option<SchematicAssociated10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicDataset10_1 {
    pub values: [f64; 2],
    pub name: DatasetNameMoreSlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicLegacy10_1 {
    pub first: String,
    pub count: i32,
    pub hyperlinks: Vec<(ObjectEntry10_1, Option<HyperlinkBody10_1>)>,
    pub second: String,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicAssociated10_1 {
    pub count: i32,
    pub layers: Vec<SchematicAssociatedLayer10_1>,
    pub short: i16,
    pub value: f64,
    pub tail_shorts: [i16; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicAssociatedLayer10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<LayerBodyPrefix10_1>,
    pub name: String,
}

fn read_count(r: &mut Reader<'_>, max: usize, min_width: usize) -> io::Result<i32> {
    let count = r.read_u32_le()? as i32;
    if count.max(0) as usize > r.remaining().min(max) / min_width {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "SchematicLayer 条目数超限",
        ));
    }
    Ok(count)
}

fn read_legacy(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<SchematicLegacy10_1> {
    let first = read_string_10_1(r, max)?;
    let count = read_count(r, max, 16)?;
    let mut hyperlinks = Vec::new();
    for _ in 0..count {
        let entry = read_object_entry_10_1(r, refs)?;
        let body = match entry {
            ObjectEntry10_1::Null => None,
            ObjectEntry10_1::BackReference { key, .. }
                if refs.registered_class(key) == Some(HYPERLINK_CLASS_ID) =>
            {
                None
            }
            ObjectEntry10_1::NewObject {
                class_id: HYPERLINK_CLASS_ID,
                key,
            } => Some(read_hyperlink_body_10_1(r, max, refs, key)?),
            _ => {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "SchematicLayer 的 IHyperlink 类未支持",
                ));
            }
        };
        hyperlinks.push((entry, body));
    }
    Ok(SchematicLegacy10_1 {
        first,
        count,
        hyperlinks,
        second: read_string_10_1(r, max)?,
        value: r.read_u32_le()? as i32,
    })
}

fn read_associated(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<SchematicAssociated10_1> {
    let count = read_count(r, max, 22)?;
    let mut layers = Vec::new();
    for _ in 0..count {
        let mut entry = read_object_entry_10_1(r, refs)?;
        let body = match &mut entry {
            ObjectEntry10_1::Null => None,
            ObjectEntry10_1::BackReference { class_id, key } => {
                let registered = refs
                    .registered_class(*key)
                    .filter(|id| {
                        known_layer_masking_support_10_1(id) != LayerMaskingSupport10_1::Unresolved
                    })
                    .ok_or_else(|| {
                        io::Error::new(
                            ErrorKind::Unsupported,
                            "SchematicLayer 的 ILayer 回引未确认",
                        )
                    })?;
                *class_id = registered;
                None
            }
            ObjectEntry10_1::NewObject { class_id, .. } => {
                Some(read_complete_layer_body_10_1(r, *class_id, max, refs)?)
            }
        };
        layers.push(SchematicAssociatedLayer10_1 {
            entry,
            body,
            name: read_string_10_1(r, max)?,
        });
    }
    Ok(SchematicAssociated10_1 {
        count,
        layers,
        short: r.read_u16_le()? as i16,
        value: r.read_f64_le()?,
        tail_shorts: [r.read_u16_le()? as i16, r.read_u16_le()? as i16],
    })
}

pub(super) fn read_schematic_layer_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<SchematicLayer10_1> {
    let mut r = reader.clone();
    let mut refs = table.clone();
    let version = r.read_u16_le()? as i16;
    if version > 7 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "SchematicLayer 版本超过 7",
        ));
    }
    let name = read_string_10_1(&mut r, max)?;
    let shorts = [r.read_u16_le()? as i16, r.read_u16_le()? as i16];
    let values = [r.read_f64_le()?, r.read_f64_le()?, r.read_f64_le()?];
    let secondary_name = read_string_10_1(&mut r, max)?;
    let version_one = (version == 1)
        .then(|| r.read_u16_le().map(|n| n as i16))
        .transpose()?;
    let dataset = if version >= 2 {
        let values = [r.read_f64_le()?, r.read_f64_le()?];
        let mut stop = None;
        let name = read_overlay_name_slot_10_1(&mut r, max, &mut refs, &mut stop)?;
        if let Some(stop) = stop {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("SchematicLayer 名称体未读完：{stop:?}"),
            ));
        }
        Some(SchematicDataset10_1 { values, name })
    } else {
        None
    };
    let legacy = (3..=5)
        .contains(&version)
        .then(|| read_legacy(&mut r, max, &mut refs))
        .transpose()?;
    let envelopes = (version >= 4)
        .then(|| {
            Ok::<_, io::Error>([
                read_envelope_slot(&mut r, max, &mut refs)?,
                read_envelope_slot(&mut r, max, &mut refs)?,
            ])
        })
        .transpose()?;
    let associated = (version >= 5)
        .then(|| read_associated(&mut r, max, &mut refs))
        .transpose()?;
    *reader = r;
    *table = refs;
    Ok(SchematicLayer10_1 {
        version,
        name,
        shorts,
        values,
        secondary_name,
        version_one,
        dataset,
        legacy,
        envelopes,
        associated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::graphics_3d::{
        GLOBE_GRAPHICS_LAYER_CLASS_ID, GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1,
    };
    use crate::maps::layer_masking::SCHEMATIC_LAYER_CLASS_ID;

    #[test]
    fn associated_schematic_layers_read_versioned_records_and_stop_before_successor() {
        let empty = [2, 0, 0, 0, 0, 0];
        for (class, shorts, tail) in [
            (GRAPHICS_LAYER_3D_CLASS_ID, 8, 0),
            (GLOBE_GRAPHICS_LAYER_CLASS_ID, 4, 8),
        ] {
            for version in -1_i16..=7 {
                let mut bytes = 1_u16.to_le_bytes().to_vec();
                bytes.extend(empty);
                bytes.resize(bytes.len() + shorts, 0);
                bytes.extend(SCHEMATIC_LAYER_CLASS_ID);
                bytes.extend(41_u32.to_le_bytes());
                bytes.extend(version.to_le_bytes());
                bytes.extend(empty);
                bytes.extend([0; 28]);
                bytes.extend(empty);
                if version == 1 {
                    bytes.extend((-7_i16).to_le_bytes());
                }
                if version >= 2 {
                    bytes.extend([0; 32]);
                }
                if (3..=5).contains(&version) {
                    bytes.extend(empty);
                    bytes.extend(1_i32.to_le_bytes());
                    bytes.extend(HYPERLINK_CLASS_ID);
                    bytes.extend(42_u32.to_le_bytes());
                    bytes.extend([0; 6]);
                    bytes.extend(empty);
                    bytes.extend([0; 4]);
                    bytes.extend(empty);
                    bytes.extend((-9_i32).to_le_bytes());
                }
                if version >= 4 {
                    bytes.extend([0; 32]);
                }
                if version >= 5 {
                    bytes.extend(1_i32.to_le_bytes());
                    bytes.extend(SCHEMATIC_LAYER_CLASS_ID);
                    bytes.extend(43_u32.to_le_bytes());
                    bytes.extend(0_i16.to_le_bytes());
                    bytes.extend(empty);
                    bytes.extend([0; 28]);
                    bytes.extend(empty);
                    bytes.extend(empty);
                    bytes.extend([0; 12]);
                    bytes.extend((-1_i16).to_le_bytes());
                }
                bytes.resize(bytes.len() + 20 + tail, 0);
                let end = bytes.len();
                bytes.extend([0xa5; 8]);
                let mut refs = RefTable10_1::new();
                for cut in 0..end {
                    let mut r = Reader::new(&bytes[..cut]);
                    assert!(read_graphics_3d_layer_10_1(&class, &mut r, 4096, &mut refs).is_err());
                    assert_eq!(r.position(), 0);
                    assert!(refs.is_empty());
                }
                let mut r = Reader::new(&bytes);
                let host = read_graphics_3d_layer_10_1(&class, &mut r, 4096, &mut refs).unwrap();
                let Some(LayerBodyPrefix10_1::SchematicLayer(body)) =
                    host.associated_layer_body.as_deref()
                else {
                    panic!("应读取 SchematicLayer");
                };
                assert_eq!(body.version, version);
                assert_eq!(body.version_one, (version == 1).then_some(-7));
                if let Some(legacy) = &body.legacy {
                    assert_eq!(legacy.value, -9);
                    assert_eq!(legacy.hyperlinks[0].1.as_ref().unwrap().key, 42);
                }
                if let Some(associated) = &body.associated {
                    assert_eq!(associated.tail_shorts, [0, -1]);
                    assert!(matches!(
                        associated.layers[0].body,
                        Some(LayerBodyPrefix10_1::SchematicLayer(_))
                    ));
                    assert_eq!(refs.registered_class(43), Some(SCHEMATIC_LAYER_CLASS_ID));
                }
                assert_eq!(refs.registered_class(41), Some(SCHEMATIC_LAYER_CLASS_ID));
                assert_eq!(r.position(), end);
            }
        }
    }
}
