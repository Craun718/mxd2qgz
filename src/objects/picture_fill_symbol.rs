//! ArcMap 10.1 DisplayCore.dll PictureFillSymbol（IPictureFillSymbol
//! `{D842B081-…}` 的实现类）对象体严格读取子集与槽位分派。
//!
//! 证据来源：`docs/arcmap-10.1-display-slots-round3.md`（G 轮研究记录）。宿主
//! 经「esri.clsid 声明模块 → RTTI TypeDescriptor → COL → IPersistStream 虚表
//! → GetClassID 读 `.rdata` CLSID 常量」链在 DisplayCore.dll 10.1.0.3035（
//! SHA-256 `9b3e61f9…6e4d2`，ImageBase `0x6eb30000`）中闭环：
//!
//! - **PictureFillSymbol** `{D842B082-330C-11D2-9168-0000F87808EE}`：IPersistStream
//!   虚表 `0x6ec0febc`（COL `0x6ec1dc94`），GetClassID `0x6eb710d0`（读常量
//!   `0x6ec0ffd4`）、Load `0x6eb71110`、Save `0x6eb71580`。
//!
//! ## 字段序（Load 反编译 + Save 对称互证；证据强度：确定的；样本 0 实例）
//!
//! 版本门：`8 < (short)ver` 拒绝 `0x80004005`；Save 版本表 `DAT_6ec0b2ac`
//! （6 项短整，逐项字节读核）= docVer 0..5 → 3,3,3,4,7,8——10.1（Current）
//! 恒写版本 8；版本 ≤7 为旧文档历史路径（Load 旧路径 `0x6eb74aa0`、Save 旧
//! 路径 `0x6eb74740`，字段序按反编译记录，无样本）。
//!
//! **版本 ==8（Load `0x6eb71110` 主路径）**：
//!
//! 1. int16 版本；
//! 2. int32 图像类型 → this−0x10（==1 文件路径 / ==2 内嵌位图块 / ==3
//!    IPicture 对象；越界值原生拒绝 `0x80004005`）；
//! 3. int32 → this−0x14；
//! 4. 类型分支：
//!    - 类型 1：String → this−0x20（路径候选）+ double ×2 → this−0x4c /
//!      −0x44（语义未定）；
//!    - 类型 2：int32 N +（N≠0 时 N 字节原样保留的位图块；Save 经
//!      `Ordinal_23` 取内存块尺寸对称写）；
//!    - 类型 3：LoadObject（请求 IID `DAT_6ec0b08c` = IPicture
//!      `{7BF80980-BF32-101A-8BBB-00AA00300CAB}`）→ 图像对象槽（Save 槽
//!      this−0xa4）：StdPicture `{0BE35204-…}`（OLE oleaut32 系统类，证据
//!      docs/arcmap-10.1-ole-stdfont.md）整体解析，其余实现类停止回退；
//!    - 其他：原生拒绝。
//! 5. LoadObject（IColor `DAT_6ec0b4f4`）×3 → Save 槽 this−0xa0 / −0x9c /
//!    −0x98（颜色候选）；
//! 6. LoadObject（ISymbol `DAT_6ec0ca2c`）×1 → Save 槽 this−0x94（轮廓符号
//!    候选）；
//! 7. double ×7 → this−0x8c / −0x84 / −0x7c / −0x74 / −0x6c / −0x64 / −0x5c
//!    + 2 个读后即弃的图片尺寸值（Save 初始化为零后调用尺寸函数，成功时
//!    写入计算结果；流内 16 字节原样保留，见 `docs/mxd-terra-host-color.md`）；
//! 8. int32 ×2 → this−0x114 / −0x10c；
//! 9. byte → this−0x38；
//! 10. short ×3 → this−0x26 / −0x24 / −0x22。
//!
//! **版本 ≤7（旧路径 `0x6eb74aa0`，字段序反编译记录）**：
//!
//! 1. 版本 <6：LoadObject（IPicture）→ 图像对象槽；版本 ≥6：String（路径候
//!    选），串空 → LoadObject（IPicture），串非空 → double ×2；
//! 2. LoadObject（IColor）×3 + LoadObject（ISymbol）×1；
//! 3. double ×7 + double ×2（读后即弃）；
//! 4. int32 ×2；
//! 5. 版本 >1：byte；版本 >3：short ×3；版本 >6：int32 N +（N≠0 时 N 字节
//!    位图块，Save 侧 `FUN_6eb74740` docVer<5 分支对称）。
//!
//! IPicture 槽实现类的全库负证据：IPicture IID `{7BF80980-…}` 在 DisplayCore
//! 仅两处 `.rdata` 常量（`0x6ec0b08c` = LoadObject 请求；`0x6ec10010` = 消费
//! 方 QI 助手引用，引用函数 `0x6eb755f0`/`0x6eb7deb0` 均为对传入对象查询该
//! IID 的消费者），esri.clsid 全表无实现 IPicture 的 Esri coclass，三元素/
//! 显示模块 `.text`/`.rdata` 无 StdPicture CLSID 常量——流内实现类即 OLE
//! StdPicture（已闭合），无 Esri 侧实现（见研究记录）。
//!
//! ## 槽位分派
//!
//! PictureLineSymbol 的 IPictureFillSymbol 和通用符号槽共用本体读取器。
//!
//! 事务性与限额约定同仓内其他读取器（trial 游标/登记表、版本负值位模式拒绝、
//! 失败不推进、`max_bytes` 约束字符串与循环计数/位图块）。样本 0 实例，全部
//! 结论无样本字节佐证，单元测试仅合成输入，不代表原生已验证。

use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, SymbolSlotFull10_1, SymbolStop10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::objects::ole_font::{
    STDPICTURE_CLASS_ID, StdPictureBody10_1, read_std_picture_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// PictureFillSymbol 类标识。证据：esri.clsid + GetClassID `0x6eb710d0` 读常
/// 量 `0x6ec0ffd4` 互证。
pub const PICTURE_FILL_SYMBOL_CLASS_ID: [u8; 16] = [
    0x82, 0xb0, 0x42, 0xd8, 0x0c, 0x33, 0xd2, 0x11, 0x91, 0x68, 0x00, 0x00, 0xf8, 0x78, 0x08, 0xee,
];

/// Load 接受的最大版本：`8 < (short)ver` 拒绝（Save 版本表 10.1 恒写 8）。
const MAX_PICTURE_FILL_VERSION: i16 = 8;

/// IPicture 图像对象槽（类型 3 / 旧路径）。
#[derive(Debug, Clone, PartialEq)]
pub enum PictureObjectSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 StdPicture 对象（体闭合；oleaut32，图像数据原样保留）。
    StdPicture(Box<StdPictureBody10_1>),
}

/// 读取一个 IPicture 类槽位条目，返回 `(槽位, 停止)`。
fn read_picture_object_slot_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    kind: DisplaySlotKind10_1,
) -> io::Result<(Option<PictureObjectSlot10_1>, Option<SymbolStop10_1>)> {
    let mut refs = staged.clone();
    let mut forward = trial.clone();
    let entry = read_object_entry_10_1(&mut forward, &mut refs)?;
    match entry {
        ObjectEntry10_1::Null => {
            *trial = forward;
            Ok((Some(PictureObjectSlot10_1::Null), None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *trial = forward;
            Ok((
                Some(PictureObjectSlot10_1::BackReference { class_id, key }),
                None,
            ))
        }
        ObjectEntry10_1::NewObject { class_id, key: _ } if class_id == STDPICTURE_CLASS_ID => {
            // StdPicture 体完整读取后才提交对象键。
            let body = read_std_picture_body_10_1(&mut forward, max_bytes)?;
            *trial = forward;
            *staged = refs;
            Ok((
                Some(PictureObjectSlot10_1::StdPicture(Box::new(body))),
                None,
            ))
        }
        ObjectEntry10_1::NewObject { class_id, key } => Ok((
            None,
            Some(SymbolStop10_1::Slot {
                kind,
                class_id,
                key,
            }),
        )),
    }
}

/// 读取 8 字节 LE u64（Reader 无 u64 原语，本模块用原样位模式保存 double）。
fn read_u64_le(trial: &mut Reader<'_>) -> io::Result<u64> {
    Ok(u64::from_le_bytes(trial.read_bytes(8)?.try_into().unwrap()))
}

/// 位图块尺寸（int32；负值与超过 `max_bytes` 拒绝——读取器自身限额）。
fn read_blob_size(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    entry_offset: usize,
) -> io::Result<u32> {
    let raw = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if raw < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "PictureFillSymbol 对象体 offset 0x{entry_offset:x}: 位图尺寸 {raw} 为负值位模式"
            ),
        ));
    }
    let size = raw as u32;
    if size as usize > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "PictureFillSymbol 对象体 offset 0x{entry_offset:x}: 位图尺寸 {size} 超过 \
                 读取器 max_bytes 限额 {max_bytes}（读取器自身限额，非原生限制）"
            ),
        ));
    }
    Ok(size)
}

/// 位图块数据：size==0 时 `None`；size>0 时消费 size 字节原样保留。
fn read_blob_data(trial: &mut Reader<'_>, size: u32) -> io::Result<Option<Vec<u8>>> {
    if size == 0 {
        return Ok(None);
    }
    Ok(Some(trial.read_bytes(size as usize)?.to_vec()))
}

/// PictureFillSymbol 图像载荷（类型分支的已解析形态）。
#[derive(Debug, Clone, PartialEq)]
pub enum PictureFillImage10_1 {
    /// 类型 1：路径 + double ×2（语义未定）。
    Path { path: String, doubles: [u64; 2] },
    /// 类型 2：内嵌位图块（N + N 字节；N==0 无块）。
    Blob(Option<Vec<u8>>),
    /// 类型 3：IPicture 对象槽。
    Picture(PictureObjectSlot10_1),
}

/// PictureFillSymbol 体（Load `0x6eb71110` ==8 主路径；槽位停止经 `stop`
/// 传播）。
#[derive(Debug, Clone, PartialEq)]
pub struct PictureFillSymbolBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 short 版本；>8 原生拒绝（≤7 走旧路径）。
    pub version: u16,
    /// int32 图像类型 → this−0x10（1/2/3）。
    pub image_type: i32,
    /// int32 → this−0x14。
    pub raw_int32: i32,
    /// 类型分支载荷。
    pub image: PictureFillImage10_1,
    /// v7 在旧式图像和公共尾字段之后附加的位图块。
    pub legacy_bitmap: Option<Vec<u8>>,
    /// 颜色槽 ×3（Save 槽 this−0xa0 / −0x9c / −0x98）。
    pub colors: [Option<ColorSlot10_1>; 3],
    /// 轮廓符号槽 ×1（Save 槽 this−0x94）。
    pub outline: Option<SymbolSlotFull10_1>,
    /// double ×7 → this−0x8c..−0x5c（语义未定）。
    pub doubles: [u64; 7],
    /// 读后即弃的图片尺寸 double ×2；按流内位模式原样保留。
    pub discard_doubles: [u64; 2],
    /// int32 ×2 → this−0x114 / −0x10c。
    pub tail_int32s: [i32; 2],
    /// byte → this−0x38。
    pub tail_byte: u8,
    /// short ×3 → this−0x26 / −0x24 / −0x22。
    pub tail_shorts: [u16; 3],
    /// 嵌套停止（槽位未知实现类时非 None；游标停在该槽起点）。
    pub stop: Option<SymbolStop10_1>,
}

/// 读取公共尾段。v2 起有一个字节字段，v4 起有三个 short。
fn read_picture_fill_tail_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    depth: usize,
    version: u16,
    mut stop: Option<SymbolStop10_1>,
) -> io::Result<(
    [Option<ColorSlot10_1>; 3],
    Option<SymbolSlotFull10_1>,
    [u64; 7],
    [u64; 2],
    [i32; 2],
    u8,
    [u16; 3],
    Option<SymbolStop10_1>,
)> {
    let mut colors = [None, None, None];
    for slot in colors.iter_mut() {
        if stop.is_some() {
            break;
        }
        match read_color_slot_10_1(trial, max_bytes, staged)? {
            ColorSlot10_1::Unsupported { class_id, key } => {
                stop = Some(SymbolStop10_1::Slot {
                    kind: DisplaySlotKind10_1::Color,
                    class_id,
                    key,
                });
                break;
            }
            ok => *slot = Some(ok),
        }
    }
    let mut outline = None;
    if stop.is_none() {
        let slot = read_symbol_slot_full_10_1(trial, max_bytes, staged, depth)?;
        if let Some(slot_stop) = slot_full_stop_10_1(&slot, DisplaySlotKind10_1::Symbol) {
            stop = Some(slot_stop);
        } else {
            outline = Some(slot);
        }
    }
    let mut doubles = [0u64; 7];
    let mut discard = [0u64; 2];
    if stop.is_none() {
        for slot in doubles.iter_mut() {
            *slot = read_u64_le(trial)?;
        }
        for slot in discard.iter_mut() {
            *slot = read_u64_le(trial)?;
        }
    }
    let mut tail_int32s = [0i32; 2];
    let mut tail_byte = 0u8;
    let mut tail_shorts = [0u16; 3];
    if stop.is_none() {
        for slot in tail_int32s.iter_mut() {
            *slot = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        }
        if version >= 2 {
            tail_byte = trial.read_bytes(1)?[0];
        }
        if version >= 4 {
            for slot in tail_shorts.iter_mut() {
                *slot = trial.read_u16_le()?;
            }
        }
    }
    Ok((
        colors,
        outline,
        doubles,
        discard,
        tail_int32s,
        tail_byte,
        tail_shorts,
        stop,
    ))
}

/// 从对象头之后的位置读取 10.1 PictureFillSymbol 体（版本 ≤8 全分支）。
pub fn read_picture_fill_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<PictureFillSymbolBody10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let entry_offset = trial.position();
    let raw_version = trial.read_u16_le()?;
    let signed = raw_version as i16;
    if signed < 0 || signed > MAX_PICTURE_FILL_VERSION {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "PictureFillSymbol 对象体 offset 0x{entry_offset:x}: 版本位模式 \
                 {raw_version}（0x{raw_version:04x}）超出原生接受范围 0..={MAX_PICTURE_FILL_VERSION}"
            ),
        ));
    }
    let version = raw_version;

    if version <= 7 {
        // 旧路径（Load `0x6eb74aa0`；对象基址偏移差异不影响流内字节序）。
        let image_type;
        let raw_int32 = 0;
        let mut image = PictureFillImage10_1::Blob(None);
        let mut stop = None;
        if version < 6 {
            image_type = 3;
            let (slot, slot_stop) = read_picture_object_slot_10_1(
                &mut trial,
                max_bytes,
                &mut trial_table,
                DisplaySlotKind10_1::Picture,
            )?;
            match slot {
                Some(slot) => image = PictureFillImage10_1::Picture(slot),
                None => stop = slot_stop,
            }
        } else {
            let path = read_string_10_1(&mut trial, max_bytes)?;
            if path.is_empty() {
                image_type = 3;
                let (slot, slot_stop) = read_picture_object_slot_10_1(
                    &mut trial,
                    max_bytes,
                    &mut trial_table,
                    DisplaySlotKind10_1::Picture,
                )?;
                match slot {
                    Some(slot) => image = PictureFillImage10_1::Picture(slot),
                    None => stop = slot_stop,
                }
            } else {
                image_type = 1;
                let d1 = read_u64_le(&mut trial)?;
                let d2 = read_u64_le(&mut trial)?;
                image = PictureFillImage10_1::Path {
                    path,
                    doubles: [d1, d2],
                };
            }
        }
        let (colors, outline, doubles, discard, tail_int32s, tail_byte, tail_shorts, stop) =
            read_picture_fill_tail_10_1(
                &mut trial,
                max_bytes,
                &mut trial_table,
                depth,
                version,
                stop,
            )?;
        let legacy_bitmap = if stop.is_none() && version == 7 {
            let size = read_blob_size(&mut trial, max_bytes, entry_offset)?;
            read_blob_data(&mut trial, size)?
        } else {
            None
        };
        let result = PictureFillSymbolBody10_1 {
            key,
            version,
            image_type,
            raw_int32,
            image,
            legacy_bitmap,
            colors,
            outline,
            doubles,
            discard_doubles: discard,
            tail_int32s,
            tail_byte,
            tail_shorts,
            stop,
        };
        *reader = trial;
        *table = trial_table;
        return Ok(result);
    }

    // ==8 主路径。
    let image_type = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let raw_int32 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let mut stop = None;
    let image = match image_type {
        1 => {
            let path = read_string_10_1(&mut trial, max_bytes)?;
            let d1 = read_u64_le(&mut trial)?;
            let d2 = read_u64_le(&mut trial)?;
            PictureFillImage10_1::Path {
                path,
                doubles: [d1, d2],
            }
        }
        2 => {
            let size = read_blob_size(&mut trial, max_bytes, entry_offset)?;
            let data = read_blob_data(&mut trial, size)?;
            PictureFillImage10_1::Blob(data)
        }
        3 => {
            let (slot, slot_stop) = read_picture_object_slot_10_1(
                &mut trial,
                max_bytes,
                &mut trial_table,
                DisplaySlotKind10_1::Picture,
            )?;
            match slot {
                Some(slot) => PictureFillImage10_1::Picture(slot),
                None => {
                    stop = slot_stop;
                    PictureFillImage10_1::Blob(None)
                }
            }
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!(
                    "PictureFillSymbol 对象体 offset 0x{entry_offset:x}: 图像类型 \
                     {image_type} 越界（原生仅接受 1/2/3，越界拒绝 0x80004005）"
                ),
            ));
        }
    };
    let (colors, outline, doubles, discard, tail_int32s, tail_byte, tail_shorts, stop) =
        read_picture_fill_tail_10_1(
            &mut trial,
            max_bytes,
            &mut trial_table,
            depth,
            version,
            stop,
        )?;
    let result = PictureFillSymbolBody10_1 {
        key,
        version,
        image_type,
        raw_int32,
        image,
        legacy_bitmap: None,
        colors,
        outline,
        doubles,
        discard_doubles: discard,
        tail_int32s,
        tail_byte,
        tail_shorts,
        stop,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// IPictureFillSymbol 类槽位条目（PictureLineSymbol 槽位；实现类
/// PictureFillSymbol）。
#[derive(Debug, Clone, PartialEq)]
pub enum PictureFillSymbolSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 PictureFillSymbol 对象（体闭合；`0x6eb71110`）。
    PictureFill(Box<PictureFillSymbolBody10_1>),
}

/// 读取一个 IPictureFillSymbol 类槽位条目，返回 `(槽位, 停止)`。
///
/// 空标记与回引可消费；新对象类标识为 PictureFillSymbol 时整体解析，其余实
/// 现类回退到槽位起点、登记表不保留该键，以停止原因返回。
pub fn read_picture_fill_symbol_slot_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    depth: usize,
) -> io::Result<(Option<PictureFillSymbolSlot10_1>, Option<SymbolStop10_1>)> {
    let mut refs = staged.clone();
    let mut forward = trial.clone();
    let entry = read_object_entry_10_1(&mut forward, &mut refs)?;
    match entry {
        ObjectEntry10_1::Null => {
            *trial = forward;
            Ok((Some(PictureFillSymbolSlot10_1::Null), None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *trial = forward;
            Ok((
                Some(PictureFillSymbolSlot10_1::BackReference { class_id, key }),
                None,
            ))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            if depth == 0 {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "IPictureFillSymbol 槽位：嵌套深度超过读取器限额 16（防栈溢出，非原生限制）",
                ));
            }
            if class_id == PICTURE_FILL_SYMBOL_CLASS_ID {
                let body = read_picture_fill_symbol_body_10_1(
                    &mut forward,
                    max_bytes,
                    &mut refs,
                    key,
                    depth - 1,
                )?;
                let stop = body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                    class_id: PICTURE_FILL_SYMBOL_CLASS_ID,
                    key,
                });
                *trial = forward;
                *staged = refs;
                Ok((
                    Some(PictureFillSymbolSlot10_1::PictureFill(Box::new(body))),
                    stop,
                ))
            } else {
                Ok((
                    None,
                    Some(SymbolStop10_1::Slot {
                        kind: DisplaySlotKind10_1::PictureFillSymbol,
                        class_id,
                        key,
                    }),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::display_color::RGB_COLOR_CLASS_ID;

    fn entry(class_id: &[u8; 16], key: u32) -> Vec<u8> {
        let mut bytes = class_id.to_vec();
        bytes.extend(key.to_le_bytes());
        bytes
    }

    fn string_bytes(text: &str) -> Vec<u8> {
        let mut bytes = ((text.encode_utf16().count() * 2 + 2) as u32)
            .to_le_bytes()
            .to_vec();
        for unit in text.encode_utf16() {
            bytes.extend(unit.to_le_bytes());
        }
        bytes.extend(0_u16.to_le_bytes());
        bytes
    }

    fn rgb_body() -> Vec<u8> {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0x01, 0x11, 0x22, 0x33, 0x00]);
        bytes.extend([0xff, 0x00]);
        bytes
    }

    /// 版本 8、类型 1（路径）全路径体。
    fn v8_path_body() -> Vec<u8> {
        let mut bytes = 8_u16.to_le_bytes().to_vec();
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(5_i32.to_le_bytes());
        bytes.extend(string_bytes("C:/pic.png"));
        bytes.extend(1.0_f64.to_le_bytes());
        bytes.extend(2.0_f64.to_le_bytes());
        bytes.extend(rgb_body_bytes(3));
        bytes.extend(rgb_body_bytes(4));
        bytes.extend(rgb_body_bytes(5));
        bytes.extend([0u8; 16]); // 轮廓符号槽空标记
        for v in [1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0] {
            bytes.extend(v.to_le_bytes());
        }
        bytes.extend(72.0_f64.to_le_bytes());
        bytes.extend(144.0_f64.to_le_bytes());
        bytes.extend(11_i32.to_le_bytes());
        bytes.extend(12_i32.to_le_bytes());
        bytes.extend([0x01]);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(3_u16.to_le_bytes());
        bytes
    }

    fn rgb_body_bytes(key: u32) -> Vec<u8> {
        let mut bytes = entry(&RGB_COLOR_CLASS_ID, key);
        bytes.extend(rgb_body());
        bytes
    }

    #[test]
    fn picture_fill_v8_path_full_body() {
        let bytes = v8_path_body();
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_picture_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 70, 8)
            .expect("版本 8 路径形态应整体消费");
        assert_eq!(body.version, 8);
        assert_eq!(body.image_type, 1);
        match body.image {
            PictureFillImage10_1::Path { path, doubles } => {
                assert_eq!(path, "C:/pic.png");
                assert_eq!(doubles[0], 1.0_f64.to_bits());
            }
            other => panic!("载荷形态错误 {other:?}"),
        }
        assert!(body.colors.iter().all(|c| c.is_some()));
        assert_eq!(body.outline, Some(SymbolSlotFull10_1::Null));
        assert_eq!(
            body.discard_doubles,
            [72.0_f64.to_bits(), 144.0_f64.to_bits()]
        );
        assert!(body.stop.is_none());
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn picture_fill_v8_blob_roundtrip() {
        let mut bytes = 8_u16.to_le_bytes().to_vec();
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(4_i32.to_le_bytes());
        bytes.extend([0xde, 0xad, 0xbe, 0xef]);
        bytes.extend(rgb_body_bytes(1));
        bytes.extend(rgb_body_bytes(2));
        bytes.extend(rgb_body_bytes(3));
        bytes.extend([0u8; 16]);
        for v in [0.0f64; 7] {
            bytes.extend(v.to_le_bytes());
        }
        bytes.extend([0u8; 16]);
        bytes.extend([0u8; 8]);
        bytes.extend([0u8; 1]);
        bytes.extend([0u8; 6]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_picture_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 71, 8)
            .expect("版本 8 位图块形态应整体消费");
        match body.image {
            PictureFillImage10_1::Blob(Some(data)) => {
                assert_eq!(data, vec![0xde, 0xad, 0xbe, 0xef]);
            }
            other => panic!("载荷形态错误 {other:?}"),
        }
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn picture_fill_v8_std_picture_slot() {
        // StdPicture 体：魔数 0x746c + 第二字 0 + 声明长度 + 图像数据。
        let mut pic = 0x746c_u16.to_le_bytes().to_vec();
        pic.extend(0_u16.to_le_bytes());
        let image: [u8; 4] = [1, 2, 3, 4];
        pic.extend((image.len() as u32).to_le_bytes());
        pic.extend(image);
        let mut bytes = 8_u16.to_le_bytes().to_vec();
        bytes.extend(3_i32.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(entry(&STDPICTURE_CLASS_ID, 80));
        bytes.extend(pic.clone());
        bytes.extend(rgb_body_bytes(1));
        bytes.extend(rgb_body_bytes(2));
        bytes.extend(rgb_body_bytes(3));
        bytes.extend([0u8; 16]);
        bytes.extend([0u8; 56]); // double ×7
        bytes.extend([0u8; 16]); // 读后即弃 double ×2
        bytes.extend([0u8; 8]);
        bytes.extend([0u8; 1]);
        bytes.extend([0u8; 6]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_picture_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 72, 8)
            .expect("版本 8 IPicture 形态应整体消费");
        match body.image {
            PictureFillImage10_1::Picture(PictureObjectSlot10_1::StdPicture(stored)) => {
                assert_eq!(stored.image_data, image.to_vec());
            }
            other => panic!("载荷形态错误 {other:?}"),
        }
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn picture_fill_v8_unknown_picture_impl_stops() {
        let mut bytes = 8_u16.to_le_bytes().to_vec();
        bytes.extend(3_i32.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        let picture_offset = bytes.len();
        // 已知颜色类放在 IPicture 槽时，不能把它误读为后面的颜色字段。
        bytes.extend(rgb_body_bytes(81));
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_picture_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 72, 8)
            .expect("嵌套停止属于成功返回");
        assert!(body.stop.is_some());
        assert_eq!(reader.position(), picture_offset);
        assert!(table.is_empty());
    }

    #[test]
    fn picture_fill_v8_image_type_out_of_range_rejected() {
        let mut bytes = 8_u16.to_le_bytes().to_vec();
        bytes.extend(4_i32.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend([0u8; 16]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        assert!(read_picture_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 1, 8).is_err());
    }

    #[test]
    fn picture_fill_legacy_versions() {
        // 版本 5（≥6 之下：<6 直接 IPicture 槽）。
        let mut pic = 0x746c_u16.to_le_bytes().to_vec();
        pic.extend(0_u16.to_le_bytes());
        pic.extend(0_u32.to_le_bytes()); // 空图
        let mut bytes = 5_u16.to_le_bytes().to_vec();
        bytes.extend(entry(&STDPICTURE_CLASS_ID, 90));
        bytes.extend(pic);
        bytes.extend(rgb_body_bytes(1));
        bytes.extend(rgb_body_bytes(2));
        bytes.extend(rgb_body_bytes(3));
        bytes.extend([0u8; 16]);
        bytes.extend([0u8; 72]); // double ×9（7 + 读后即弃 2）
        bytes.extend([0u8; 8]);
        bytes.extend([0u8; 1]);
        bytes.extend([0u8; 6]);
        // 版本 5：>1 byte、>3 shorts 已随上方 [0u8;1]/[0u8;6] 写入；>6 blob 无。
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_picture_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 73, 8)
            .expect("版本 5 旧路径应整体消费");
        assert_eq!(body.version, 5);
        assert!(matches!(body.image, PictureFillImage10_1::Picture(_)));
        assert_eq!(reader.remaining(), 0);
        for version in 0_u16..=4 {
            let omitted = if version < 2 {
                7
            } else if version < 4 {
                6
            } else {
                0
            };
            let mut old = bytes[..bytes.len() - omitted].to_vec();
            old[..2].copy_from_slice(&version.to_le_bytes());
            old.extend([0xaa; 8]);
            let mut reader = Reader::new(&old);
            let mut table = RefTable10_1::new();
            let body = read_picture_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 73, 8)
                .expect("旧版尾字段应按版本读取");
            assert_eq!(body.version, version);
            assert_eq!(reader.read_bytes(8).unwrap(), &[0xaa; 8]);
        }
        // 版本 6：String 非空 → double ×2（无 IPicture 槽）；版本 >6 无 blob
        // 之外的附加。
        let mut bytes6 = 6_u16.to_le_bytes().to_vec();
        bytes6.extend(string_bytes("p.png"));
        bytes6.extend(1.0_f64.to_le_bytes());
        bytes6.extend(2.0_f64.to_le_bytes());
        bytes6.extend(rgb_body_bytes(1));
        bytes6.extend(rgb_body_bytes(2));
        bytes6.extend(rgb_body_bytes(3));
        bytes6.extend([0u8; 16]);
        bytes6.extend([0u8; 72]);
        bytes6.extend([0u8; 8]);
        bytes6.extend([0u8; 1]);
        bytes6.extend([0u8; 6]);
        let mut reader = Reader::new(&bytes6);
        let mut table = RefTable10_1::new();
        let body = read_picture_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 74, 8)
            .expect("版本 6 旧路径应整体消费");
        match body.image {
            PictureFillImage10_1::Path { path, .. } => assert_eq!(path, "p.png"),
            other => panic!("载荷形态错误 {other:?}"),
        }
        assert_eq!(reader.remaining(), 0);
        // 版本 7：>6 blob 附加段。
        let mut bytes7 = 7_u16.to_le_bytes().to_vec();
        bytes7.extend(string_bytes(""));
        bytes7.extend(entry(&STDPICTURE_CLASS_ID, 91));
        let mut pic7 = 0x746c_u16.to_le_bytes().to_vec();
        pic7.extend(0_u16.to_le_bytes());
        pic7.extend(0_u32.to_le_bytes());
        bytes7.extend(pic7);
        bytes7.extend(rgb_body_bytes(1));
        bytes7.extend(rgb_body_bytes(2));
        bytes7.extend(rgb_body_bytes(3));
        bytes7.extend([0u8; 16]);
        bytes7.extend([0u8; 72]);
        bytes7.extend([0u8; 8]);
        bytes7.extend([0u8; 1]);
        bytes7.extend([0u8; 6]);
        bytes7.extend(2_i32.to_le_bytes());
        bytes7.extend([0xaa, 0xbb]);
        let mut reader = Reader::new(&bytes7);
        let mut table = RefTable10_1::new();
        let body = read_picture_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 75, 8)
            .expect("版本 7 旧路径应整体消费");
        assert!(matches!(body.image, PictureFillImage10_1::Picture(_)));
        assert_eq!(body.legacy_bitmap.as_deref(), Some(&[0xaa, 0xbb][..]));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn picture_fill_slot_dispatch_and_truncation() {
        // 槽分派：版本 8 体全消费。
        let mut bytes = entry(&PICTURE_FILL_SYMBOL_CLASS_ID, 85);
        bytes.extend(v8_path_body());
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let (slot, stop) =
            read_picture_fill_symbol_slot_10_1(&mut reader, 1024, &mut table, 8).unwrap();
        assert!(matches!(
            slot,
            Some(PictureFillSymbolSlot10_1::PictureFill(_))
        ));
        assert!(stop.is_none());
        assert_eq!(reader.remaining(), 0);
        for cut in [20, bytes.len() - 1] {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut table = RefTable10_1::new();
            assert!(read_picture_fill_symbol_slot_10_1(&mut reader, 1024, &mut table, 8).is_err());
            assert_eq!(reader.position(), 0);
            assert!(table.is_empty());
        }
        // 未知实现类停止回退。
        let unknown = [0x77_u8; 16];
        let bytes = entry(&unknown, 95);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let (slot, stop) =
            read_picture_fill_symbol_slot_10_1(&mut reader, 1024, &mut table, 8).unwrap();
        assert!(slot.is_none());
        match stop.expect("未知类应停止") {
            SymbolStop10_1::Slot {
                kind,
                class_id,
                key,
            } => {
                assert_eq!(kind, DisplaySlotKind10_1::PictureFillSymbol);
                assert_eq!(class_id, unknown);
                assert_eq!(key, 95);
            }
            other => panic!("停止原因形态错误 {other:?}"),
        }
        assert_eq!(reader.remaining(), bytes.len());
        assert!(table.is_empty());
        // 版本 9 拒绝。
        let mut bytes = 9_u16.to_le_bytes().to_vec();
        bytes.extend([0u8; 16]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        assert!(read_picture_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 1, 8).is_err());
        // 全体逐前缀截断。
        let full = v8_path_body();
        for cut in 0..full.len() {
            let mut reader = Reader::new(&full[..cut]);
            let mut table = RefTable10_1::new();
            assert!(
                read_picture_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 1, 8).is_err(),
                "截断到 {cut} 字节应失败"
            );
            assert!(table.is_empty());
        }
    }
}
