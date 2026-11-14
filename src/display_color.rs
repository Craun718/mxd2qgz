//! ArcMap 10.1 DisplayCore.dll 颜色族（CmykColor/HsvColor/GrayColor/HlsColor，
//! 含 RgbColor 同构核心）对象体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-display-tails.md`（2026-11-15 本轮研究记录）。
//! 全部字段序来自 DisplayCore.dll 10.1.0.3035（SHA-256 `9b3e61f9…a926e4d2`，
//! 复用前轮提取件 temp/agent-fl-tail/）的静态反编译；接口身份经 RTTI 类型
//! 描述符 → 完整对象定位符（COL）→ 虚表 → GetClassID 立即数逐类确认。
//!
//! **重要边界：仓库参考样本 `samples/测试底图.mxd` 只含 RgbColor 实例
//! （键 37/38/154，前轮已闭合于 `feature_layer_tail`），本模块新增的四类
//! 颜色无样本字节佐证**；只能用合成输入测试，不得声称样本验证。
//!
//! ## 共享 Color 核心（HsvColor/GrayColor/HlsColor 与 RgbColor 同构）
//!
//! 四类的 `IPersistStream::Load` 槽经同一调整器 thunk `0x6eb3dbf0`（对象内
//! IPersistStream 子对象偏移同为 44）进入同一实体 Load `0x6eb3d8b0`，再转入
//! 共享核心 `0x6eb35310`。流内布局对四类完全一致（每类对象内的虚槽只影响
//! 字段语义、不影响字节数）：
//!
//! 1. short 版本（实体 `0x6eb3d8b0`，`1 < (short)ver` 有符号拒绝，错误
//!    `0xA0000005`）；
//! 2. short 版本（核心 `0x6eb35310`，同上限）；
//! 3. 1 字节分支标志：==0 → 3 个 double（经 `(this−0x20)` 接口虚表 +0x34
//!    装载：HsvColor `0x6eb36d30`、其余三类共享 `0x6eb34170`→`0x6eb35130`；
//!    两实现均做范围校验（第一值 ∈ [−0.0001, 100.0001]、后两值 ∈
//!    [−120.0001, 120.0001]，越界返回 `0x80070057`），语义候选 HSV/HLS 分量）；
//!    !=0 → 4 字节（虚表 +0x1c：rgb 直拆 `0x6eb3d690`、`RgbToHsv`
//!    `0x6eb36c10`、`RgbToGray` `0x6eb35d30`、`RgbToHls` `0x6eb36470`——
//!    流内 4 字节语义候选为 packed RGB）；
//! 4. 1 字节 → `this−8`；
//! 5. 1 字节 → `this−7`，装载时按 `0xff − 流内字节` 取补（读取器保留流内
//!    位模式）。
//!
//! Save 侧对称：各实体 Save（HsvColor `0x6eb36f70`/GrayColor `0x6eb35e50`/
//! HlsColor `0x6eb365d0`/RgbColor `0x6eb3d8f0`）恒写版本 1（`DAT_6ec0b330`/
//! `0b338`/`0b334`/`0b318`）后进入共享核心 Save `0x6eb35470`（版本常量
//! `DAT_6ec0b33c`=1），经虚表 +0x38/+0x20 取值对称写出。RgbColor 的字段
//! 归属由前轮记录闭合（`feature_layer_tail` 模块）；本模块的
//! [`read_core_color_body_10_1`] 覆盖同构的 HsvColor/GrayColor/HlsColor。
//!
//! ## CmykColor（版本分支与四类不同）
//!
//! IPersistStream 虚表 `0x6ec0b82c`（COL 偏移 60），Load thunk `0x6eb34ee0`
//! → 实体 `0x6eb34850`；Save thunk `0x6eb34f90` → 实体 `0x6eb34970`（版本
//! 常量 `DAT_6ec0b340`=**4**）：
//!
//! - short 版本（`4 < (short)ver` 有符号拒绝）；
//! - 版本 ≤1：直接转入共享核心 `0x6eb35310`（`this−0x1c`），此后无本类字段；
//! - 版本 ≥2：1 字节（`this−0x10`）→ 1 字节标志（`this−0xf`）→ 标志非零时
//!   String（`this−0xc`）+ short（`this−0x8`）→
//!   - 版本 ==2：转入共享核心（同上）；
//!   - 版本 ≥3：4 字节 CMYK 位模式（`this−0x14`）；
//!     - 版本 ≥4：1 字节（`this−0x24`）+ 1 字节（`this−0x23`，装载按
//!       `0xff − 流内字节` 取补）。
//!
//! Save 恒写版本 4，与版本 4 读分支逐字段对称；版本 ≤3 分支在本二进制无
//! 写侧来源（历史版本兼容分支）。GetClassID 写出 `{7EE9C497-…}`
//! （`0x6eb34a70`，经槽 `0x6eb34f10` thunk 调用）。
//!
//! ## 槽位分派（建议接入点）
//!
//! [`read_color_slot_10_1`] 把一个对象条目（20 字节头）按流内类标识分派到
//! 上述五类（RgbColor 委托 `feature_layer_tail::read_rgb_color_body_10_1`）。
//! 这是 `feature_layer_tail` 中 SimpleMarkerSymbol 两个 IColor 槽
//! （`ColorUnsupported` 停止点）的替换接入点：该文件本轮不动，由其所有者
//! 决定是否接线。未知颜色类在此回退到槽位起点并返回
//! [`ColorSlot10_1::Unsupported`]（登记表不保留该键）。
//!
//! 调用约定：调用者先用 [`crate::storage::object_stream::read_header_10_1`]
//! 读取对象头并比对类常量，从对象头之后的位置调用各读取器；本模块不重复
//! 读取对象头。全部读取在 trial 游标与 trial 登记表副本上进行，完整成功才
//! 一并提交；任何错误既不推进游标、也不修改登记表。版本 short 按原生有符号
//! 比较设上限，负值位模式一并拒绝（较原生更严，与既有读取器一致）。
//! `max_bytes` 约束 CmykColor 标志分支内 String 的声明长度。
//!
//! 未解决问题（详见研究记录开放项）：共享核心两个版本 short 的独立语义、
//! 3 个 double 的确切颜色分量归属、4 字节分支的 packed RGB 位序（含 bit25
//! 可见性标志候选）、`this−8`/`this−7` 两字节语义、CmykColor 版本 ≥2 的
//! byte/String/short 字段语义，均为候选；全部结论未经目标软件运行验证。

/// RgbColor 类标识重导出（前轮闭合于 `feature_layer_tail`，槽位分派共用）。
pub use crate::layers::feature_layer_tail::RGB_COLOR_CLASS_ID;
use crate::layers::feature_layer_tail::{RgbColorBody10_1, read_rgb_color_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// 共享 Color 核心两个版本 short 的上限：各 `1 < ver` 拒绝；四类样本均 1。
const MAX_CORE_COLOR_VERSION: i16 = 1;

/// CmykColor 实体版本上限：`4 < ver` 拒绝；Save 恒写 4（`DAT_6ec0b340`）。
const MAX_CMYK_VERSION: i16 = 4;

/// CmykColor 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7EE9C497-D123-11D0-8383-080009B996CC}`。证据：
/// esri.clsid 声明（模块 DisplayCore.dll，progID esriDisplay.CmykColor）；
/// DisplayCore.dll GetClassID `0x6eb34a70` 写出（经 IPersistStream 虚表
/// `0x6ec0b82c` 槽 +0x0c 的 thunk `0x6eb34f10` 调用）。样本中无实例，
/// 无字节佐证。
pub const CMYK_COLOR_CLASS_ID: [u8; 16] = [
    0x97, 0xc4, 0xe9, 0x7e, 0x23, 0xd1, 0xd0, 0x11, 0x83, 0x83, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

/// HsvColor 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7EE9C492-D123-11D0-8383-080009B996CC}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6eb36fb0` 写出（虚表
/// `0x6ec0be5c` 槽 +0x0c thunk `0x6eb37290`）。样本中无实例，无字节佐证。
pub const HSV_COLOR_CLASS_ID: [u8; 16] = [
    0x92, 0xc4, 0xe9, 0x7e, 0x23, 0xd1, 0xd0, 0x11, 0x83, 0x83, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

/// GrayColor 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7EE9C495-D123-11D0-8383-080009B996CC}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6eb35e90` 写出（虚表
/// `0x6ec0bb1c` 槽 +0x0c thunk `0x6eb36110`）。样本中无实例，无字节佐证。
pub const GRAY_COLOR_CLASS_ID: [u8; 16] = [
    0x95, 0xc4, 0xe9, 0x7e, 0x23, 0xd1, 0xd0, 0x11, 0x83, 0x83, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

/// HlsColor 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7EE9C493-D123-11D0-8383-080009B996CC}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6eb36610` 写出（虚表
/// `0x6ec0bcbc` 槽 +0x0c thunk `0x6eb368c0`）。样本中无实例，无字节佐证。
pub const HLS_COLOR_CLASS_ID: [u8; 16] = [
    0x93, 0xc4, 0xe9, 0x7e, 0x23, 0xd1, 0xd0, 0x11, 0x83, 0x83, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

/// 读取 int16 版本并按原生有符号比较校验上限；负值位模式一并拒绝
/// （负值版本无业务意义，较原生更严，与 feature_layer_tail 一致）。
fn read_version_short(
    trial: &mut Reader<'_>,
    max: i16,
    what: &str,
    entry_offset: usize,
) -> io::Result<u16> {
    let version = trial.read_u16_le()?;
    let signed = version as i16;
    if signed > max || signed < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{what} 对象体 offset 0x{entry_offset:x}: 版本 {version} 超出 \
                 {what}::Load 接受范围（有符号比较 >{max} 拒绝；负值位模式一并拒绝）"
            ),
        ));
    }
    Ok(version)
}

/// HsvColor/GrayColor/HlsColor（及同构 RgbColor）共享 Color 核心的已确认
/// 字段。体长固定：flag==0 为 29 字节（short ×2 + byte + double ×3 +
/// byte ×2），flag!=0 为 9 字节（short ×2 + byte + int32 + byte ×2）。
#[derive(Debug, Clone, PartialEq)]
pub struct CoreColorBody10_1 {
    /// 对象头键（样本中无实例；RgbColor 样本键为 37/38/154，见前轮记录）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 实体 Load `0x6eb3d8b0` 首个 short 版本（写侧恒 1）；>1 原生拒绝。
    pub version_outer: u16,
    /// 共享核心 `0x6eb35310` 的第二个 short 版本（写侧恒 1）；>1 原生拒绝。
    pub version_inner: u16,
    /// 1 字节分支标志：==0 走 3 个 double 分支（经虚表 +0x34 装载），
    /// !=0 走 4 字节分支（经虚表 +0x1c 装载）。
    pub branch_flag: u8,
    /// flag==0 分支的 3 个 double（语义候选 HSV/HLS 分量；原生按范围
    /// [−0.0001, 100.0001]/[−120.0001, 120.0001] 校验，越界 `0x80070057`）。
    pub doubles: Option<[f64; 3]>,
    /// flag!=0 分支的 4 字节位模式（各色类经 `RgbToHsv`/`RgbToGray`/
    /// `RgbToHls` 解释，语义候选 packed RGB）。样本无实例，分支按反编译闭合。
    pub packed_rgb_raw: Option<u32>,
    /// 倒数第 2 字节（→ 内存 this−8），语义未定。
    pub trailing_byte: u8,
    /// 末字节的原样位模式（→ 内存 this−7，装载按 `0xff − 流内字节` 取补）。
    /// 语义未定。
    pub complement_byte_raw: u8,
}

/// 读取 HsvColor/GrayColor/HlsColor（以及同构的 RgbColor 布局）对象体。
///
/// 起点约定：调用者已读取对象头、比对 [`HSV_COLOR_CLASS_ID`] /
/// [`GRAY_COLOR_CLASS_ID`] / [`HLS_COLOR_CLASS_ID`] 并取得键 `key`；`what`
/// 为错误消息中的类名。体全部为标量（无嵌套对象、无字符串），要么完整
/// 消费，要么失败（截断 / 版本超限）。RgbColor 优先用
/// `feature_layer_tail::read_rgb_color_body_10_1`（字段归属更完整），二者
/// 流内布局一致。
pub fn read_core_color_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    key: u32,
    what: &str,
) -> io::Result<CoreColorBody10_1> {
    let _ = table; // 共享核心体无嵌套对象，登记表仅在签名上保持一致
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("{what} 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };

    // ①② 两个版本 short（实体 + 共享核心，各有符号 >1 拒绝）。
    let version_outer = read_version_short(&mut trial, MAX_CORE_COLOR_VERSION, what, entry_offset)
        .map_err(|e| invalid(e.to_string()))?;
    let version_inner = read_version_short(&mut trial, MAX_CORE_COLOR_VERSION, what, entry_offset)
        .map_err(|e| invalid(e.to_string()))?;
    // ③ 1 字节分支标志。
    let branch_flag = trial.read_bytes(1)?[0];
    // 分支：==0 → 3 个 double；!=0 → 4 字节。
    let (doubles, packed_rgb_raw) = if branch_flag == 0 {
        let mut values = [0_f64; 3];
        for slot in values.iter_mut() {
            *slot = trial.read_f64_le()?;
        }
        (Some(values), None)
    } else {
        (None, Some(trial.read_u32_le()?))
    };
    // ④⑤ 尾随两个 1 字节字段（后者装载时取补，读取器保留流内位模式）。
    let trailing_byte = trial.read_bytes(1)?[0];
    let complement_byte_raw = trial.read_bytes(1)?[0];

    let result = CoreColorBody10_1 {
        key,
        version_outer,
        version_inner,
        branch_flag,
        doubles,
        packed_rgb_raw,
        trailing_byte,
        complement_byte_raw,
    };
    *reader = trial;
    Ok(result)
}

/// CmykColor（DisplayCore.dll `0x6eb34850`）体的已确认字段。
///
/// 版本 ≤2 时以共享 Color 核心段（[`CoreColorBody10_1`]）收尾；版本 ≥3 时
/// 颜色值为 4 字节 CMYK 位模式且无核心段。
#[derive(Debug, Clone, PartialEq)]
pub struct CmykColorBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 实体版本 short；>4 原生拒绝（Save 恒写 4）。
    pub version: u16,
    /// 版本 ≥2 的 1 字节（→ this−0x10），语义未定。`None` = 版本 ≤1。
    pub byte_10: Option<u8>,
    /// 版本 ≥2 的 1 字节标志（→ this−0xf）：非零时其后为 String + short。
    /// `None` = 版本 ≤1。
    pub string_flag: Option<u8>,
    /// string_flag 非零时的 String（→ this−0xc），语义未定。
    pub string: Option<String>,
    /// string_flag 非零时的 short（→ this−0x8），语义未定。
    pub string_short: Option<i16>,
    /// 版本 ≥3 的 4 字节 CMYK 位模式（→ this−0x14）。`None` = 版本 ≤2。
    pub cmyk_raw: Option<u32>,
    /// 版本 ≥4 的 1 字节（→ this−0x24）。`None` = 版本 ≤3。
    pub byte_24: Option<u8>,
    /// 版本 ≥4 末字节的原样位模式（→ this−0x23，装载按 `0xff − 流内字节`
    /// 取补）。`None` = 版本 ≤3。
    pub complement_byte_raw: Option<u8>,
    /// 版本 ≤2 的共享 Color 核心段（版本 ≤1 时前面无本类字段）。
    pub core: Option<CoreColorBody10_1>,
}

/// 从对象头之后的位置读取 10.1 CmykColor 对象体，失败时不推进游标、不修改
/// 登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`CMYK_COLOR_CLASS_ID`] 并取得键
/// `key`。版本分支按反编译如实覆盖（0..=4；版本 ≤1/==2 以共享核心段收尾，
/// ==3 无末尾两字节，==4 补齐）。`max_bytes` 约束标志分支内 String 声明长度。
pub fn read_cmyk_color_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<CmykColorBody10_1> {
    let _ = table;
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("CmykColor 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };

    // ① short 版本（>4 有符号拒绝）。
    let version = read_version_short(&mut trial, MAX_CMYK_VERSION, "CmykColor", entry_offset)
        .map_err(|e| invalid(e.to_string()))?;
    // ②③ 版本 ≥2：byte + 标志 byte +（标志非零）String + short。
    let mut byte_10 = None;
    let mut string_flag = None;
    let mut string = None;
    let mut string_short = None;
    if version as i16 >= 2 {
        byte_10 = Some(trial.read_bytes(1)?[0]);
        let flag = trial.read_bytes(1)?[0];
        string_flag = Some(flag);
        if flag != 0 {
            string = Some(read_string_10_1(&mut trial, max_bytes)?);
            string_short = Some(trial.read_u16_le()? as i16);
        }
    }
    // ④⑤ 版本分支：≤2 → 共享核心；≥3 → 4 字节 CMYK（≥4 再补 2 字节）。
    let mut cmyk_raw = None;
    let mut byte_24 = None;
    let mut complement_byte_raw = None;
    let mut core = None;
    if version as i16 >= 3 {
        cmyk_raw = Some(trial.read_u32_le()?);
        if version as i16 >= 4 {
            byte_24 = Some(trial.read_bytes(1)?[0]);
            complement_byte_raw = Some(trial.read_bytes(1)?[0]);
        }
    } else {
        core = Some(read_core_color_body_10_1(
            &mut trial,
            table,
            key,
            "CmykColor",
        )?);
    }

    let result = CmykColorBody10_1 {
        key,
        version,
        byte_10,
        string_flag,
        string,
        string_short,
        cmyk_raw,
        byte_24,
        complement_byte_raw,
        core,
    };
    *reader = trial;
    Ok(result)
}

/// 颜色族对象体的分类结果（RgbColor 委托前轮读取器）。
#[derive(Debug, Clone, PartialEq)]
pub enum ColorBody10_1 {
    /// RgbColor（样本三实例，前轮闭合于 `feature_layer_tail`）。
    Rgb(Box<RgbColorBody10_1>),
    /// HsvColor/GrayColor/HlsColor 共享核心布局；`class_id` 区分具体类
    /// （流内布局一致，仅虚槽语义不同）。
    Core {
        class_id: [u8; 16],
        body: Box<CoreColorBody10_1>,
    },
    /// CmykColor（版本分支独立）。
    Cmyk(Box<CmykColorBody10_1>),
}

/// IColor 槽位条目的分类结果（五类颜色全部闭合，未知类回退后为停止）。
#[derive(Debug, Clone, PartialEq)]
pub enum ColorSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新对象且类标识属于闭合的五类颜色：已解析体。
    Color {
        class_id: [u8; 16],
        body: ColorBody10_1,
    },
    /// 新对象但类标识不在闭合集合：游标已回退到该槽位起点、登记表未保留
    /// 该键（本读取器内部完成回退，调用者据此停止）。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// 从当前位置读取一个 IColor 槽位对象条目并按类标识分派。
///
/// 条目为空引用 / 回引时原样返回；新对象类标识属于五类闭合颜色时解析其体；
/// 其余类回退到槽位起点并返回 [`ColorSlot10_1::Unsupported`]。全部过程在
/// trial 游标与 trial 登记表上进行，任何失败不推进调用者游标、不修改登记
/// 表。`max_bytes` 供 CmykColor 字符串分支使用。
pub fn read_color_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<ColorSlot10_1> {
    let origin = reader.clone();
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
    let result = match entry {
        ObjectEntry10_1::Null => ColorSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            ColorSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            let body = if class_id == RGB_COLOR_CLASS_ID {
                ColorBody10_1::Rgb(Box::new(read_rgb_color_body_10_1(
                    &mut trial,
                    &mut staged,
                    key,
                )?))
            } else if class_id == HSV_COLOR_CLASS_ID {
                ColorBody10_1::Core {
                    class_id,
                    body: Box::new(read_core_color_body_10_1(
                        &mut trial,
                        &mut staged,
                        key,
                        "HsvColor",
                    )?),
                }
            } else if class_id == GRAY_COLOR_CLASS_ID {
                ColorBody10_1::Core {
                    class_id,
                    body: Box::new(read_core_color_body_10_1(
                        &mut trial,
                        &mut staged,
                        key,
                        "GrayColor",
                    )?),
                }
            } else if class_id == HLS_COLOR_CLASS_ID {
                ColorBody10_1::Core {
                    class_id,
                    body: Box::new(read_core_color_body_10_1(
                        &mut trial,
                        &mut staged,
                        key,
                        "HlsColor",
                    )?),
                }
            } else if class_id == CMYK_COLOR_CLASS_ID {
                ColorBody10_1::Cmyk(Box::new(read_cmyk_color_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                )?))
            } else {
                // 未知颜色类：回退到槽位起点，不保留该键。
                *reader = origin;
                return Ok(ColorSlot10_1::Unsupported { class_id, key });
            };
            ColorSlot10_1::Color { class_id, body }
        }
    };
    *reader = trial;
    *table = staged;
    Ok(result)
}
