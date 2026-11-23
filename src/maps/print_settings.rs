//! 10.1 分流文档中位于 PageLayout 对象体之后的纸张和打印机设置。

use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::{read_bstr_string_10_1, read_string_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const EMF_PRINTER_CLASS_ID: [u8; 16] = [
    0x01, 0x4d, 0x06, 0xae, 0xce, 0xd6, 0xd0, 0x11, 0x86, 0x7a, 0, 0, 0xf8, 0x75, 0x17, 0x20,
];
pub const PS_PRINTER_CLASS_ID: [u8; 16] = [
    0x61, 0x4e, 0x8c, 0xe2, 0x55, 0x3f, 0xd1, 0x11, 0x88, 0x5e, 0, 0, 0xf8, 0x78, 0x08, 0xee,
];
pub const ARC_PRESS_PRINTER_CLASS_ID: [u8; 16] = [
    0x53, 0x51, 0x1b, 0x9e, 0xa8, 0x1c, 0xb6, 0x42, 0xa7, 0xe0, 0x4b, 0x21, 0x5d, 0xae, 0xa9, 0x56,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PrintSaveFormat10_1 {
    Default,
    ArcGis83,
    ArcGis90,
    LegacyShortDevMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintSettings10_1 {
    pub paper: Paper10_1,
    pub printer: ObjectEntry10_1,
    pub printer_body: Option<PrinterBody10_1>,
    pub end_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paper10_1 {
    pub version: u16,
    pub system_language: Option<u16>,
    pub printer_description: Option<String>,
    pub dev_names: Vec<u8>,
    pub declared_dev_mode_size: u32,
    pub dev_mode: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrinterBody10_1 {
    Emf { version: u16 },
    PostScript { version: u16 },
    ArcPress(ArcPressPrinter10_1),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcPressPrinter10_1 {
    pub version: u16,
    pub name: String,
    pub short: u16,
    pub first_flag: u8,
    pub driver_class: String,
    pub driver_data: Option<Vec<u8>>,
    pub longs: [i32; 5],
    pub last_flag: u8,
}

pub(super) fn read_print_settings_10_1(
    reader: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
    format: PrintSaveFormat10_1,
) -> io::Result<PrintSettings10_1> {
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    let paper = read_paper(&mut trial, limit, format)
        .map_err(|error| io::Error::new(error.kind(), format!("IPaper：{error}")))?;
    let mut printer = read_object_entry_10_1(&mut trial, &mut staged)
        .map_err(|error| io::Error::new(error.kind(), format!("IPrinter 对象头：{error}")))?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut printer
        && let Some(registered) = staged.registered_class(*key)
    {
        *class_id = registered;
    }
    let printer_body = match printer {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, .. }
            if matches!(
                class_id,
                EMF_PRINTER_CLASS_ID | PS_PRINTER_CLASS_ID | ARC_PRESS_PRINTER_CLASS_ID
            ) =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: EMF_PRINTER_CLASS_ID,
            ..
        } => Some(PrinterBody10_1::Emf {
            version: version(&mut trial, 1, "EmfPrinter")?,
        }),
        ObjectEntry10_1::NewObject {
            class_id: PS_PRINTER_CLASS_ID,
            ..
        } => Some(PrinterBody10_1::PostScript {
            version: version(&mut trial, 1, "PsPrinter")?,
        }),
        ObjectEntry10_1::NewObject {
            class_id: ARC_PRESS_PRINTER_CLASS_ID,
            ..
        } => Some(PrinterBody10_1::ArcPress(read_arc_press(
            &mut trial, limit, format,
        )?)),
        _ => return Err(super::map_frame::unsupported("IPrinter", printer)),
    };
    if format == PrintSaveFormat10_1::LegacyShortDevMode
        && (!matches!(printer_body, Some(PrinterBody10_1::Emf { version: 1 }))
            || trial.remaining() != 0)
    {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "旧 DEVMODE 后没有完整的流尾 EmfPrinter v1",
        ));
    }
    *refs = staged;
    *reader = trial;
    Ok(PrintSettings10_1 {
        paper,
        printer,
        printer_body,
        end_offset: reader.position(),
    })
}

fn read_paper(
    reader: &mut Reader<'_>,
    limit: usize,
    format: PrintSaveFormat10_1,
) -> io::Result<Paper10_1> {
    let version = version(reader, 3, "Paper")?;
    if format == PrintSaveFormat10_1::LegacyShortDevMode && version != 2 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "旧 DEVMODE 需要 Paper v2",
        ));
    }
    let system_language = (version >= 2).then(|| reader.read_u16_le()).transpose()?;
    let printer_description = (version >= 2)
        .then(|| read_string_10_1(reader, limit))
        .transpose()?;
    let dev_names_size = reader.read_u32_le()?;
    let dev_names = blob(reader, dev_names_size, limit)?.to_vec();
    let declared_dev_mode_size = reader.read_u32_le()?;
    let mut dev_mode = Vec::new();
    if declared_dev_mode_size != 0 && version == 2 && format == PrintSaveFormat10_1::ArcGis83 {
        // 0x7bcd71c3 按分配长度直接写内存块。
        dev_mode = blob(reader, declared_dev_mode_size, limit)?.to_vec();
    } else if declared_dev_mode_size != 0 {
        // 10.1 默认读序固定体为 220 字节。Yellowstone 的旧写法仅存 212 字节。
        // 驱动私有字节在流中紧跟固定成员，在内存中却写到 dmSize 所指的偏移。
        let fixed_size = if format == PrintSaveFormat10_1::LegacyShortDevMode {
            212
        } else {
            220
        };
        let mut probe = reader.clone();
        let fixed = probe.read_bytes(fixed_size)?;
        let dm_size = usize::from(u16::from_le_bytes([fixed[0x44], fixed[0x45]]));
        let extra = usize::from(u16::from_le_bytes([fixed[0x46], fixed[0x47]]));
        let stored = fixed_size + extra;
        let declared = u64::from(declared_dev_mode_size);
        if declared < fixed_size.max(dm_size + extra) as u64
            || declared > limit as u64
            || stored > limit
        {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "DEVMODE 长度与声明或限额不符",
            ));
        }
        dev_mode = reader.read_bytes(stored)?.to_vec();
    }
    Ok(Paper10_1 {
        version,
        system_language,
        printer_description,
        dev_names,
        declared_dev_mode_size,
        dev_mode,
    })
}

fn read_arc_press(
    reader: &mut Reader<'_>,
    limit: usize,
    format: PrintSaveFormat10_1,
) -> io::Result<ArcPressPrinter10_1> {
    let version = version(reader, 2, "ArcPressPrinter")?;
    let name = read_string_10_1(reader, limit)?;
    let short = reader.read_u16_le()?;
    let first_flag = reader.read_bytes(1)?[0];
    let driver_class = read_bstr_string_10_1(reader, limit)?;
    // 旧写侧省去整个驱动段，打印机是 /PageLayout 的最后一项。
    let omit_driver = matches!(
        format,
        PrintSaveFormat10_1::ArcGis83 | PrintSaveFormat10_1::ArcGis90
    ) && reader.remaining() == 5 * 4 + 1;
    let driver_data = if version >= 2 && !omit_driver {
        let length = reader.read_u32_le()?;
        Some(blob(reader, length, limit)?.to_vec())
    } else {
        None
    };
    let mut longs = [0; 5];
    for value in &mut longs {
        *value = reader.read_u32_le()? as i32;
    }
    let last_flag = reader.read_bytes(1)?[0];
    Ok(ArcPressPrinter10_1 {
        version,
        name,
        short,
        first_flag,
        driver_class,
        driver_data,
        longs,
        last_flag,
    })
}

fn version(reader: &mut Reader<'_>, max: u16, name: &str) -> io::Result<u16> {
    let version = reader.read_u16_le()?;
    if version > max {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("不支持的 {name} 版本 {version}"),
        ));
    }
    Ok(version)
}

fn blob<'a>(reader: &mut Reader<'a>, size: u32, limit: usize) -> io::Result<&'a [u8]> {
    let size = usize::try_from(size)
        .ok()
        .filter(|&size| size <= limit)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "打印设置数据长度超过限额"))?;
    reader.read_bytes(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printer_reference_uses_registered_class() {
        let mut bytes = vec![0; 10]; // Paper v0，空 DEVNAMES 和 DEVMODE。
        bytes.extend(EMF_PRINTER_CLASS_ID);
        bytes.extend(7_u32.to_le_bytes());
        bytes.push(0x42);

        for registered in [PS_PRINTER_CLASS_ID, super::super::map_object::MAP_CLASS_ID] {
            let mut definition = registered.to_vec();
            definition.extend(7_u32.to_le_bytes());
            let mut refs = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&definition), &mut refs).unwrap();
            let mut reader = Reader::new(&bytes);
            let result = read_print_settings_10_1(
                &mut reader,
                4096,
                &mut refs,
                PrintSaveFormat10_1::Default,
            );
            if registered == PS_PRINTER_CLASS_ID {
                let settings = result.unwrap();
                assert_eq!(
                    settings.printer,
                    ObjectEntry10_1::BackReference {
                        class_id: registered,
                        key: 7,
                    }
                );
                assert!(settings.printer_body.is_none());
                assert_eq!(reader.read_bytes(1).unwrap(), &[0x42]);
            } else {
                assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
                assert_eq!(reader.position(), 0);
            }
            assert_eq!(refs.registered_class(7), Some(registered));
        }
    }
}
