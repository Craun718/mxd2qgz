//! ArcMap 10.1 FeatureLayer 尾段链第一环（渲染器 SimpleRenderer 及其直接
//! 后继）对象体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-featurelayer-tail.md`（2026-11-14 第七轮，
//! 本轮新研究记录）。链头是 `feature_layer` 渲染器槽（FeatureLayer::Load
//! `0x6f6e9940` 第 6 步，请求 IID `{40A9E884-…}`=IFeatureRenderer）处出现的
//! SimpleRenderer 新对象（样本 `temp/r6-maps.bin` 流内偏移 `0x3ed`，键 35）。
//! 本模块闭合以下五类体（字段序全部来自 10.1.0.3035 原生 DLL 反编译）：
//!
//! 1. **SimpleRenderer**（FeatureLayer.dll，IPersistStream 虚表 `0x6f7b0dfc`
//!    槽 +0x14 = 调整器 thunk `0x6f724cf0` → 实体 Load `0x6f722b70`，
//!    Save `0x6f722cf0`；GetClassID `0x6f722e00` 写出 CLSID
//!    `{F3435801-5779-11D0-98BF-00805F7CED21}`，与 esri.clsid 声明、样本
//!    `/Maps:0x3ed` 头三方互证）：
//!    ① int16 版本（`3 < (short)ver` 有符号比较，>3 拒绝；样本 3）→
//!    ② LoadObject(ISymbol `{F3435802-…}`)（样本 SimpleMarkerSymbol 键 36）→
//!    ③ LoadObject(ILegendGroup `{167C5E9F-…}`)（样本 LegendGroup 键 39）→
//!    ④ LoadObject(ILegendItem `{52AA5BC0-…}`)（样本 16 零空标记）→
//!    ⑤ int16 → obj+0x128（样本 0）→
//!    ⑥ LoadLegacyRotationFieldAndType(LayerLib.dll `0x786f93d0`，本类调用
//!    不带版本 short)：String 旋转字段 + int32 旋转类型（样本空串 + 0）→
//!    ⑦ String → obj+0x12c 标签（样本空串）→
//!    ⑧ 版本 >2：RotationRenderer::Load（LayerLib.dll `0x786f7e40`：short
//!    版本 ≤1 + int32 ×2 + String ×3 + double ×6）+ SizeRenderer::Load
//!    （LayerLib.dll `0x7870bbc0`：short 版本 ≤1 + int32 + String +
//!    double ×2）。
//!    Save `0x6f722cf0` 对称（写版本表 docVersion→3，样本 3）；docVersion<4
//!    时省略 ⑧（读侧按版本 >2 门，二者在 10.1 一致）。
//! 2. **SimpleMarkerSymbol**（DisplayCore.dll 10.1.0.3035，SHA-256
//!    `9b3e61f9…a926e4d2`，Setup1.cab 新提取；IPersistStream 虚表
//!    `0x6ec15724` 槽 +0x14 = thunk `0x6ebc0460` → 实体 Load `0x6ebbefa0`，
//!    Save `0x6ebbf190`；GetClassID `0x6ebbf310` 写出
//!    `{7914E5FE-C892-11D0-8BB6-080009EE4E41}`）：
//!    ① short 版本（>2 拒绝；样本 2）→ ② LoadObject(IColor
//!    `{20CD40B0-…}`) → ③ double（样本 6.0）→ ④ int32 ×3（样本 0/13/0，
//!    13 为 marker 样式候选）→ ⑤ double ×3（样本 0）→ ⑥ 1 字节（样本 1）→
//!    ⑦ double（样本 1.0）→ ⑧ LoadObject(IColor)（轮廓色）→
//!    ⑨ 版本 >1：short（样本 0xFFFF）。构造中的 `FUN_6eb406f0` 两处调用
//!    不读流（虚调用返回值），无流内字节。
//! 3. **RgbColor**（DisplayCore.dll，IPersistStream 虚表 `0x6ec0c70c` 槽
//!    +0x14 = thunk `0x6eb3dbf0` → 实体 Load `0x6eb3d8b0` → Color 核心
//!    `0x6eb35310`；Save `0x6eb3d8f0`/`0x6eb35470`；GetClassID `0x6eb3d930`
//!    写出 `{7EE9C496-D123-11D0-8383-080009B996CC}`）：
//!    ① short 版本 ≤1 → ② 第二个 short 版本 ≤1（Color 核心）→ ③ 1 字节
//!    分支标志：==0 → 3 个 double（经 Color 虚表 +0x34 装载，样本三实例
//!    flag 均 0）；!=0 → 4 字节 CMYK（虚表 +0x1c）→ ④ 1 字节 → ⑤ 1 字节
//!    （内存值按 `0xff - 流内字节` 取补）。flag==0 分支体长 31 字节，与样本
//!    三实例（键 37/38/154）逐字节吻合。
//! 4. **LegendGroup**（Layer.dll 10.1.0.3035，SHA-256 与既有基线一致，
//!    IPersistStream 虚表 `0x7866fa3c` 槽 +0x14 = Load `0x78665cc0`，Save
//!    `0x78665e20`；GetClassID `0x78665c80` 写出
//!    `{167C5EA2-AF20-11D1-8817-080009EC732A}`）：
//!    ① short 版本（>3 拒绝；样本 3）→ ② short → obj+0xc（样本 -1）→
//!    ③ 版本 ≥2：short → obj+0xe（<2 时内存默认 -1；样本 -1）→
//!    ④ String → obj+0x14（样本空串）→ ⑤ int32 计数 + 循环
//!    LoadObject(ILegendClass `{167C5E9E-…}`)（样本 1 → LegendClass 键 40）→
//!    ⑥ 版本 >2：short → obj+0x12（样本 0）。Save 写版本表 docVersion→3；
//!    docVersion>4 时才写 ⑥（读侧按版本 >2 门）。
//! 5. **LegendClass**（Layer.dll，IPersistStream 虚表 `0x7866f728` 槽 +0x14 =
//!    Load `0x78663560`，Save `0x78663640`；GetClassID `0x78662fe0` 写出
//!    `{167C5EA3-AF20-11D1-8817-080009EC732A}`）：
//!    ① short 版本（**反编译未见上限拒绝**，样本 2；本读取器仅按 ver>1 门
//!    分支，不设额外上限）→ ② LoadObject(ISymbol)（样本为对 SimpleMarker
//!    Symbol 键 36 的回引头，无体）→ ③ String 标签（样本空串）→
//!    ④ String 描述（样本空串）→ ⑤ LoadObject(ILegendClassFormat
//!    `{52AA5BC2-…}`)（样本 16 零空标记；新对象体未闭合 = 停止点）→
//!    ⑥ 版本 >1：int32 → obj+0x1c（样本 -1）。
//!
//! 样本对位（辅助观察，不作测试依赖）：`temp/r6-maps.bin` `0x3ed..0x602`
//! 逐字段吻合（见研究记录字段序表）；SimpleRenderer 体终结于 `0x5b0`，其
//! 后 FeatureLayer 的 `+0x144` GUID（16 零）、String「BSM」、double ×2、
//! short、IHyperlink 计数 0、IColor 空标记、String ×2、int32 依次对位，
//! 到 `0x602` 的 `{A2BAAE2C-…}` 计数循环（FieldInfo 族）之前——该循环及
//! 其后的类不在本模块范围。
//!
//! 调用约定：调用者先用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
//! 头并比对类常量，从对象头之后的位置调用各 `read_*_body_10_1`；本模块不
//! 重复读取对象头。嵌套条目用 [`crate::storage::object_refs::read_object_entry_10_1`]
//! 分类并登记（登记先于对象体解析是原生语义；为满足「失败不修改登记表」，
//! 全部登记发生在调用者登记表的副本上，成功后整体提交）。槽位新对象类标识
//! 未知时读取器在该槽位起点停止并回退（不保留该键，与 feature_layer 渲染
//! 器槽语义一致）；体内已闭合前缀的停止（如 SMS 的第二个 IColor 槽）传播
//! 为外层停止原因，游标停在内层停止点（与 feature_layer 的
//! `DataSource` 停止传播语义一致）。
//!
//! 未解决问题（详见研究记录「未解决事项」）：
//! - ILegendItem 槽（`{52AA5BC0-…}`）的 coclass 体未闭合（样本为空标记，
//! 无字节佐证），新对象即停止；
//! - ILegendClassFormat 槽（`{52AA5BC2-…}`）coclass 体未闭合（样本空标
//! 记），LegendClass 在此停止；
//! - ISymbol 槽只闭合 SimpleMarkerSymbol 一类；SimpleLineSymbol /
//!   SimpleFillSymbol / MultiLayerMarkerSymbol 等同族体未闭合（新对象即停
//!   止）；
//! - IColor 槽只闭合 RgbColor；CmykColor/HlsColor/HsvColor/GrayColor 同族
//!   体未闭合（新对象即停止）；
//! - RotationRenderer/SizeRenderer 的「3D 随机旋转上下限」double ×6 中前
//!   3 个样本为 0、后 3 个为 360.0，语义为候选（对称写侧 getter 未逐个定
//!   名）；各 int32/String 的业务命名（style 候选等）按内容推断；
//! - 第三轮研究文档 IID 表中 `{F3435802-…}` 的 Data2 误记为 5799，经本轮
//!   二进制字节与类型库双重复核应为 **5779**（与 SimpleRenderer CLSID 同
//!   Data2），研究记录已更正。

use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// SimpleRenderer::Load 接受的最大版本：`3 < (short)ver` 有符号比较；样本 3。
const MAX_SIMPLE_RENDERER_VERSION: i16 = 3;

/// SimpleMarkerSymbol::Load 接受的最大版本：`2 < (short)ver`；样本 2。
const MAX_SYMBOL_VERSION: i16 = 2;

/// RgbColor 两个版本 short 的上限：各 `1 < ver` 拒绝；样本 1/1。
const MAX_COLOR_VERSION: i16 = 1;

/// LegendGroup::Load 接受的最大版本：`3 < (short)ver`；样本 3。
const MAX_LEGEND_GROUP_VERSION: i16 = 3;

/// RotationRenderer::Load / SizeRenderer::Load 版本上限：各 `1 < ver`；样本 1/1。
const MAX_RENDERER_TAIL_VERSION: i16 = 1;

/// SimpleRenderer 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{F3435801-5779-11D0-98BF-00805F7CED21}`。证据：
/// esri.clsid.ecfg 声明（模块 FeatureLayer.dll）与 esriCarto 类型库
/// coclass #514；FeatureLayer.dll GetClassID `0x6f722e00` 写出同一 CLSID
/// （`0x11d05779f3435801`/`0x21ed7c5f8000bf98` 两个 8 字节立即数）；样本
/// `/Maps:0x3ed` 渲染器槽对象头逐字节相同（键 35）。
pub const SIMPLE_RENDERER_CLASS_ID: [u8; 16] = [
    0x01, 0x58, 0x43, 0xf3, 0x79, 0x57, 0xd0, 0x11, 0x98, 0xbf, 0x00, 0x80, 0x5f, 0x7c, 0xed, 0x21,
];

/// SimpleMarkerSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5FE-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid.ecfg 声明（模块 DisplayCore.dll）；DisplayCore.dll GetClassID
/// `0x6ebbf310` 写出；样本 `/Maps:0x403`（SimpleRenderer 体内 ISymbol 槽，
/// 键 36）与 LegendClass 体内的回引头逐字节相同。
pub const SIMPLE_MARKER_SYMBOL_CLASS_ID: [u8; 16] = [
    0xfe, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// RgbColor 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7EE9C496-D123-11D0-8383-080009B996CC}`。证据：
/// esri.clsid.ecfg 声明（模块 DisplayCore.dll）；DisplayCore.dll GetClassID
/// `0x6eb3d930` 写出；样本三实例（键 37 @`0x419`、38 @`0x481`、154 @
/// `0x2c0b`）对象头逐字节相同。
pub const RGB_COLOR_CLASS_ID: [u8; 16] = [
    0x96, 0xc4, 0xe9, 0x7e, 0x23, 0xd1, 0xd0, 0x11, 0x83, 0x83, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

/// LegendGroup 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{167C5EA2-AF20-11D1-8817-080009EC732A}`。证据：
/// esri.clsid.ecfg 声明（模块 Layer.dll）；Layer.dll GetClassID `0x78665c80`
/// 写出；样本 `/Maps:0x4b6`（SimpleRenderer 体内 ILegendGroup 槽，键 39）
/// 逐字节相同。
pub const LEGEND_GROUP_CLASS_ID: [u8; 16] = [
    0xa2, 0x5e, 0x7c, 0x16, 0x20, 0xaf, 0xd1, 0x11, 0x88, 0x17, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// LegendClass 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{167C5EA3-AF20-11D1-8817-080009EC732A}`。证据：
/// esri.clsid.ecfg 声明（模块 Layer.dll）；Layer.dll GetClassID `0x78662fe0`
/// 写出；样本 `/Maps:0x4da`（LegendGroup 体内嵌对象头，键 40）逐字节相同。
pub const LEGEND_CLASS_CLASS_ID: [u8; 16] = [
    0xa3, 0x5e, 0x7c, 0x16, 0x20, 0xaf, 0xd1, 0x11, 0x88, 0x17, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// 把游标回退到入口窗口内的绝对偏移（Reader 无随机定位原语，用入口克隆
/// 前向读取实现；目标必在已读范围内）。
fn reposition_to<'a>(origin: &Reader<'a>, target: usize) -> Reader<'a> {
    let mut positioned = origin.clone();
    positioned
        .read_bytes(target - origin.position())
        .expect("回退目标在停止前的已读位置，必在窗口内");
    positioned
}

/// 读取 int16 并按原生有符号比较校验版本上限；负值位模式一并拒绝
/// （负值版本无业务意义，较原生更严，与 feature_layer 的版本处理一致）。
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

/// RgbColor（DisplayCore.dll `0x6eb3d8b0` + `0x6eb35310`）体的已确认字段。
///
/// 体长取决于分支标志：flag==0 为 31 字节（short ×2 + byte + double ×3 +
/// byte ×2），flag!=0 为 13 字节（short ×2 + byte + int32 + byte ×2）。
#[derive(Debug, Clone, PartialEq)]
pub struct RgbColorBody10_1 {
    /// 对象头键（样本 37/38/154）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 实体 Load `0x6eb3d8b0` 首个 short 版本（样本 1）；>1 原生拒绝。
    pub version_outer: u16,
    /// Color 核心 `0x6eb35310` 的第二个 short 版本（样本 1）；>1 原生拒绝。
    pub version_inner: u16,
    /// 1 字节分支标志（样本 0）：==0 走 3 个 double 分支，!=0 走 4 字节
    /// CMYK 分支（原生经 Color 虚表 +0x1c 装载）。
    pub branch_flag: u8,
    /// flag==0 分支的 3 个 double（原生经 Color 虚表 +0x34 装载；样本
    /// 37 ≈ 96.68/-20.42/87.37，语义未定，原样保留）。
    pub doubles: Option<[f64; 3]>,
    /// flag!=0 分支的 4 字节 CMYK 位模式（样本无实例，分支已按反编译闭合）。
    pub cmyk_raw: Option<u32>,
    /// 倒数第 2 字节（→ 内存 this-8；样本 0），语义未定。
    pub trailing_byte: u8,
    /// 末字节的原样位模式（→ 内存 this-7，原生按 `0xff - 流内字节` 取补
    /// 装载；样本 0，即内存值 0xff）。语义未定（透明度候选）。
    pub complement_byte_raw: u8,
}

/// 从对象头之后的位置读取 10.1 RgbColor 对象体，失败时不推进游标、不修改
/// 登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`RGB_COLOR_CLASS_ID`] 并取得键
/// `key`。RgbColor 体全部为标量（无嵌套对象、无字符串），无停止点：要么
/// 完整消费，要么失败（截断 / 版本超限）。
pub fn read_rgb_color_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<RgbColorBody10_1> {
    let _ = table; // RgbColor 无嵌套对象，登记表仅在签名上保持一致
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("RgbColor 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };

    // ① 实体版本 short（`1 < ver` 拒绝）。
    let version_outer = read_version_short(&mut trial, MAX_COLOR_VERSION, "RgbColor", entry_offset)
        .map_err(|e| invalid(e.to_string()))?;
    // ② Color 核心版本 short（同上限）。
    let version_inner = read_version_short(&mut trial, MAX_COLOR_VERSION, "RgbColor", entry_offset)
        .map_err(|e| invalid(e.to_string()))?;
    // ③ 1 字节分支标志。
    let branch_flag = trial.read_bytes(1)?[0];
    // ④ 分支：==0 → 3 个 double；!=0 → 4 字节 CMYK。
    let (doubles, cmyk_raw) = if branch_flag == 0 {
        let mut values = [0_f64; 3];
        for slot in values.iter_mut() {
            *slot = trial.read_f64_le()?;
        }
        (Some(values), None)
    } else {
        (None, Some(trial.read_u32_le()?))
    };
    // ⑤⑥ 尾随两个 1 字节字段（后者原生按补码装载，读取器保留流内位模式）。
    let trailing_byte = trial.read_bytes(1)?[0];
    let complement_byte_raw = trial.read_bytes(1)?[0];

    let result = RgbColorBody10_1 {
        key,
        version_outer,
        version_inner,
        branch_flag,
        doubles,
        cmyk_raw,
        trailing_byte,
        complement_byte_raw,
    };
    *reader = trial;
    Ok(result)
}

/// IColor 槽位（请求 IID `{20CD40B0-…}`）条目的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum ColorSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验（回引头会重复
    /// 被引对象的类标识，见 object_refs 模块文档）。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 RgbColor 对象（类标识经 [`RGB_COLOR_CLASS_ID`] 校验）：已解析体。
    RgbColor(Box<RgbColorBody10_1>),
}

/// SimpleMarkerSymbol（DisplayCore.dll `0x6ebbefa0`）体的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleMarkerSymbolStop10_1 {
    /// 首个 IColor 槽的新对象类标识不是 RgbColor（同族 CmykColor 等体未闭
    /// 合）：游标回退到该槽位起点，其后的标量字段不可达。
    ColorUnsupported { class_id: [u8; 16], key: u32 },
    /// 第二个 IColor 槽（轮廓色）同类停止；首个颜色与标量段已消费。
    OutlineColorUnsupported { class_id: [u8; 16], key: u32 },
}

/// SimpleMarkerSymbol 体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleMarkerSymbolPrefix10_1 {
    /// 对象头键（样本 36）。
    pub key: u32,
    /// 体首 short 版本（样本 2）；有符号比较 >2 拒绝。
    pub version: u16,
    /// IColor 槽（主色）：空标记 / 回引 / RgbColor 体 / 不支持停止。
    pub color: ColorSlot10_1,
    /// double → obj-0x6c（样本 6.0；尺寸候选）。`None` = 首色槽停止。
    pub size: Option<f64>,
    /// int32 → obj-0x70（样本 0），语义未定。
    pub int32_a: Option<i32>,
    /// int32 → obj-0xcc（样本 13；marker 样式候选），语义未定。
    pub int32_b: Option<i32>,
    /// int32 → obj-0xc4（样本 0），语义未定。
    pub int32_c: Option<i32>,
    /// double ×3 → obj-100/-0x5c/-0x54（样本 0/0/0；X/Y 偏移候选）。
    pub doubles: Option<[f64; 3]>,
    /// 1 字节 → obj-0x44（样本 1），语义未定。
    pub byte_flag: Option<u8>,
    /// double → obj-0x4c（样本 1.0；比例候选）。
    pub double_ratio: Option<f64>,
    /// 第二个 IColor 槽（轮廓色）。`None` = 在其之前已停止。
    pub outline_color: Option<ColorSlot10_1>,
    /// 版本 >1 的 short → obj-0xb4（样本 0xFFFF）。`None` = 版本 ≤1 或前置
    /// 停止未读取。
    pub rotate_short: Option<i16>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SimpleMarkerSymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 SimpleMarkerSymbol 对象体，失败时不推进游
/// 标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`SIMPLE_MARKER_SYMBOL_CLASS_ID`] 并
/// 取得键 `key`。两个 IColor 槽的新对象类标识非 RgbColor 时是成功停止（停
/// 止原因携带类标识与键，游标回退到该槽位起点）；截断与版本超限为错误。
pub fn read_simple_marker_symbol_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<SimpleMarkerSymbolPrefix10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // 1. 版本 short（有符号 >2 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "SimpleMarkerSymbol",
        entry_offset,
    )?;

    // 2. 首个 IColor 槽（主色）。
    let slot_offset = trial.position();
    let saved_table = table.clone();
    let mut staged_table = table.clone();
    let color = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
        ObjectEntry10_1::Null => ColorSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            ColorSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == RGB_COLOR_CLASS_ID => {
            ColorSlot10_1::RgbColor(Box::new(read_rgb_color_body_10_1(
                &mut trial,
                &mut staged_table,
                key,
            )?))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            // 未知颜色类：回退到槽位起点并停止（不保留该键）。
            let stop = SimpleMarkerSymbolStop10_1::ColorUnsupported { class_id, key };
            trial = reposition_to(reader, slot_offset);
            staged_table = saved_table;
            let result = SimpleMarkerSymbolPrefix10_1 {
                key,
                version,
                color: ColorSlot10_1::Null,
                size: None,
                int32_a: None,
                int32_b: None,
                int32_c: None,
                doubles: None,
                byte_flag: None,
                double_ratio: None,
                outline_color: None,
                rotate_short: None,
                stop: Some(stop),
            };
            *table = staged_table;
            *reader = trial;
            return Ok(result);
        }
    };

    // 3-8. 标量段：double、int32 ×3、double ×3、byte、double。
    let size = trial.read_f64_le()?;
    let int32_a = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let int32_b = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let int32_c = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let mut doubles = [0_f64; 3];
    for slot in doubles.iter_mut() {
        *slot = trial.read_f64_le()?;
    }
    let byte_flag = trial.read_bytes(1)?[0];
    let double_ratio = trial.read_f64_le()?;

    // 9. 第二个 IColor 槽（轮廓色）。
    let outline_offset = trial.position();
    let pre_outline_table = staged_table.clone();
    let (outline_color, stop) = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
        ObjectEntry10_1::Null => (ColorSlot10_1::Null, None),
        ObjectEntry10_1::BackReference { class_id, key } => {
            (ColorSlot10_1::BackReference { class_id, key }, None)
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == RGB_COLOR_CLASS_ID => (
            ColorSlot10_1::RgbColor(Box::new(read_rgb_color_body_10_1(
                &mut trial,
                &mut staged_table,
                key,
            )?)),
            None,
        ),
        ObjectEntry10_1::NewObject { class_id, key } => {
            // 轮廓色槽停止：游标回退到该槽位起点（首色与标量段保留）。
            let stop = SimpleMarkerSymbolStop10_1::OutlineColorUnsupported { class_id, key };
            trial = reposition_to(reader, outline_offset);
            staged_table = pre_outline_table;
            (ColorSlot10_1::Null, Some(stop))
        }
    };

    // 10. 版本 >1 的 short。
    let rotate_short = if stop.is_none() && (version as i16) > 1 {
        Some(trial.read_u16_le()? as i16)
    } else {
        None
    };

    let result = SimpleMarkerSymbolPrefix10_1 {
        key,
        version,
        color,
        size: Some(size),
        int32_a: Some(int32_a),
        int32_b: Some(int32_b),
        int32_c: Some(int32_c),
        doubles: Some(doubles),
        byte_flag: Some(byte_flag),
        double_ratio: Some(double_ratio),
        outline_color: Some(outline_color),
        rotate_short,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// ISymbol 槽位（请求 IID `{F3435802-…}`）条目的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目（样本 LegendClass 体内对 SimpleMarkerSymbol 键 36
    /// 的回引头），类标识原样保留。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 SimpleMarkerSymbol 对象：已解析体（体内停止经 `stop` 传播）。
    SimpleMarkerSymbol(Box<SimpleMarkerSymbolPrefix10_1>),
}

/// RotationRenderer::Load（LayerLib.dll `0x786f7e40`）的尾段字段。
#[derive(Debug, Clone, PartialEq)]
pub struct RotationRendererTail10_1 {
    /// short 版本（>1 拒绝；样本 1）。
    pub version: u16,
    /// int32 ×2（→ obj+8/+0xc；样本 0/0），语义未定（3D 旋转标志候选）。
    pub flags: [i32; 2],
    /// String ×3（→ obj+0x10/+0x14/+0x18；样本全空），语义未定（3D 旋转表
    /// 达式候选）。
    pub expressions: [String; 3],
    /// double ×6（→ obj+0x20..+0x48；样本 0/0/0/360/360/360），语义未定
    /// （3D 随机旋转上下限候选）。
    pub doubles: [f64; 6],
}

/// SizeRenderer::Load（LayerLib.dll `0x7870bbc0`）的尾段字段。
#[derive(Debug, Clone, PartialEq)]
pub struct SizeRendererTail10_1 {
    /// short 版本（>1 拒绝；样本 1）。
    pub version: u16,
    /// int32（→ obj+4；样本 0），语义未定（标志候选）。
    pub flags: i32,
    /// String（→ obj+8；样本空），语义未定（尺寸表达式候选）。
    pub expression: String,
    /// double ×2（→ obj+0x10/+0x18；样本 0.1/100.0），语义未定（尺寸上下
    /// 限候选）。
    pub doubles: [f64; 2],
}

/// SimpleRenderer 体读取的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleRendererStop10_1 {
    /// ISymbol 槽的新对象类标识不是 SimpleMarkerSymbol（SimpleLineSymbol 等
    /// 同族体未闭合）：游标回退到该槽位起点。
    SymbolUnsupported { class_id: [u8; 16], key: u32 },
    /// SimpleMarkerSymbol 体内的停止（游标停在其停止点，已闭合前缀保留）。
    SymbolBody(SimpleMarkerSymbolStop10_1),
    /// ILegendGroup 槽的新对象类标识不是 LegendGroup：游标回退到该槽位起点。
    LegendGroupUnsupported { class_id: [u8; 16], key: u32 },
    /// LegendGroup 体内的停止（游标停在其停止点）。
    LegendGroupBody(LegendGroupStop10_1),
    /// ILegendItem 槽的新对象（coclass 体未闭合，样本为空标记无字节佐证）：
    /// 游标回退到该槽位起点，其后的标量与尾段不可达。
    LegendItemNewObject { class_id: [u8; 16], key: u32 },
}

/// SimpleRenderer（FeatureLayer.dll `0x6f722b70`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleRendererPrefix10_1 {
    /// 对象头键（样本 35 @`/Maps:0x3fd`）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 short 版本（样本 3）；有符号比较 >3 拒绝。
    pub version: u16,
    /// ISymbol 槽（请求 IID `{F3435802-…}`）结果。
    pub symbol: SymbolSlot10_1,
    /// ILegendGroup 槽（请求 IID `{167C5E9F-…}`）结果。`None` = 前置停止。
    pub legend_group: Option<LegendGroupSlot10_1>,
    /// ILegendItem 槽（请求 IID `{52AA5BC0-…}`）分类；新对象为停止点。
    /// `None` = 未读取。
    pub legend_item: Option<ObjectEntry10_1>,
    /// short → obj+0x128（样本 0），语义未定。`None` = 未读取。
    pub short_128: Option<i16>,
    /// LoadLegacyRotationFieldAndType 的 String 旋转字段（样本空串）。
    /// `None` = 未读取。本类调用不带版本 short（第三参 false）。
    pub rotation_field: Option<String>,
    /// LoadLegacyRotationFieldAndType 的 int32 旋转类型（样本 0）。
    pub rotation_type: Option<i32>,
    /// String → obj+0x12c 标签（样本空串）。`None` = 未读取。
    pub label: Option<String>,
    /// 版本 >2 时 RotationRenderer::Load 尾段（样本有，版本 3）。
    pub rotation_tail: Option<RotationRendererTail10_1>,
    /// 版本 >2 时 SizeRenderer::Load 尾段（样本有）。
    pub size_tail: Option<SizeRendererTail10_1>,
    /// 停止原因；`None` = 体已完整消费（样本形态）。
    pub stop: Option<SimpleRendererStop10_1>,
    /// 停止点之后剩余的字节数（相对传入 Reader 窗口）。
    pub remaining_bytes: usize,
}

/// ILegendGroup 槽位条目的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendGroupSlot10_1 {
    /// 空引用标记。
    Null,
    /// 回引条目（无体）。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 LegendGroup 对象：已解析体（体内停止经 `stop` 传播）。
    LegendGroup(Box<LegendGroupPrefix10_1>),
}

/// LegendGroup 体内 ILegendClass 循环条目的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendClassEntry10_1 {
    /// 空引用标记（原生 LoadObject 返回空对象、仍入集合）。
    Null,
    /// 回引条目：无体，类标识原样保留（样本对键 36 的 SimpleMarkerSymbol
    /// 回引发生在 LegendClass 体内，不在本层）。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 LegendClass 对象：已解析体。
    NewObject(Box<LegendClassPrefix10_1>),
}

/// LegendGroup 体读取的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegendGroupStop10_1 {
    /// 第 `index` 个 ILegendClass 循环条目的新对象类标识不是 LegendClass：
    /// 游标回退到该条目对象头起点，版本 >2 尾 short 不可达。
    ClassUnsupported {
        class_id: [u8; 16],
        key: u32,
        index: usize,
    },
    /// 某个 LegendClass 体内的停止（游标停在其停止点）。
    ClassBody(LegendClassStop10_1),
}

/// LegendGroup（Layer.dll `0x78665cc0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct LegendGroupPrefix10_1 {
    /// 对象头键（样本 39）。
    pub key: u32,
    /// 体首 short 版本（样本 3）；有符号比较 >3 拒绝。
    pub version: u16,
    /// short → obj+0xc（样本 -1），语义未定（可编辑标志候选）。
    pub heading_short: i16,
    /// 版本 ≥2 的 short → obj+0xe（<2 时原生内存默认 -1，不读流；样本 -1）。
    pub editable_short: Option<i16>,
    /// String → obj+0x14（样本空串；标题候选）。
    pub heading: String,
    /// int32 LegendClass 计数（样本 1）。负数原生不进循环，原样保留。
    pub class_count: i32,
    /// LegendClass 循环条目（按流内顺序）。
    pub classes: Vec<LegendClassEntry10_1>,
    /// 版本 >2 的 short → obj+0x12（样本 0）。`None` = 版本 ≤2 或前置停止。
    pub trailing_short: Option<i16>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<LegendGroupStop10_1>,
}

/// ISymbol 槽或 ILegendClassFormat 槽的新对象在 LegendClass 体内的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegendClassStop10_1 {
    /// ISymbol 槽的新对象类标识不是 SimpleMarkerSymbol（同族体未闭合）：
    /// 游标回退到该槽位起点。
    SymbolUnsupported { class_id: [u8; 16], key: u32 },
    /// ISymbol 槽 SimpleMarkerSymbol 体内的停止（游标停在其停止点）。
    SymbolBody(SimpleMarkerSymbolStop10_1),
    /// ILegendClassFormat 槽的新对象（coclass 体未闭合，样本为空标记）：
    /// 游标回退到该槽位起点，版本 >1 的 int32 不可达。
    FormatNewObject { class_id: [u8; 16], key: u32 },
}

/// LegendClass（Layer.dll `0x78663560`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct LegendClassPrefix10_1 {
    /// 对象头键（样本 40）。
    pub key: u32,
    /// 体首 short 版本（样本 2）。**反编译未见上限拒绝**，本读取器不设上
    /// 限，仅按 ver>1 门分支。
    pub version: u16,
    /// ISymbol 槽（请求 IID `{F3435802-…}`）结果。
    pub symbol: SymbolSlot10_1,
    /// String 标签（样本空串）。
    pub label: String,
    /// String 描述（样本空串）。
    pub description: String,
    /// ILegendClassFormat 槽（请求 IID `{52AA5BC2-…}`）分类；新对象为停止
    /// 点。
    pub format: ObjectEntry10_1,
    /// 版本 >1 的 int32 → obj+0x1c（样本 -1）。`None` = 版本 ≤1 或前置停止。
    pub int32_1c: Option<i32>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<LegendClassStop10_1>,
}

/// 从对象头之后的位置读取 10.1 LegendClass 对象体，失败时不推进游标、不
/// 修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`LEGEND_CLASS_CLASS_ID`] 并取得键。
/// ISymbol 槽新对象仅支持 SimpleMarkerSymbol；ILegendClassFormat 槽新对象为
/// 停止点（回退到槽位起点）；体内停止成功返回。
pub fn read_legend_class_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<LegendClassPrefix10_1> {
    let mut trial = reader.clone();

    // 1. 版本 short：反编译未见上限检查，原样保留（负值同样原样；门判断按
    //    有符号语义 ver > 1）。
    let version = trial.read_u16_le()?;

    // 2. ISymbol 槽（请求 IID {F3435802-…}）。
    let slot_offset = trial.position();
    let saved_table = table.clone();
    let mut staged_table = table.clone();
    let mut stop = None;
    let symbol = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
        ObjectEntry10_1::Null => SymbolSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            SymbolSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key }
            if class_id == SIMPLE_MARKER_SYMBOL_CLASS_ID =>
        {
            let sms = read_simple_marker_symbol_body_10_1(&mut trial, &mut staged_table, key)?;
            if let Some(inner) = sms.stop {
                stop = Some(LegendClassStop10_1::SymbolBody(inner));
            }
            SymbolSlot10_1::SimpleMarkerSymbol(Box::new(sms))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            stop = Some(LegendClassStop10_1::SymbolUnsupported { class_id, key });
            trial = reposition_to(reader, slot_offset);
            staged_table = saved_table.clone();
            SymbolSlot10_1::Null
        }
    };

    // 3-5. 标签、描述与 ILegendClassFormat 槽（前置停止时不读取）。
    let mut label = String::new();
    let mut description = String::new();
    let mut format = ObjectEntry10_1::Null;
    let mut int32_1c = None;
    if stop.is_none() {
        label = read_string_10_1(&mut trial, max_bytes)?;
        description = read_string_10_1(&mut trial, max_bytes)?;
        let format_offset = trial.position();
        let pre_format_table = staged_table.clone();
        format = read_object_entry_10_1(&mut trial, &mut staged_table)?;
        if let ObjectEntry10_1::NewObject { class_id, key } = format {
            stop = Some(LegendClassStop10_1::FormatNewObject { class_id, key });
            trial = reposition_to(reader, format_offset);
            staged_table = pre_format_table;
        }
        // 6. 版本 >1 的 int32。
        if stop.is_none() && (version as i16) > 1 {
            int32_1c = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        }
    }

    let result = LegendClassPrefix10_1 {
        key,
        version,
        symbol,
        label,
        description,
        format,
        int32_1c,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 LegendGroup 对象体，失败时不推进游标、不
/// 修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`LEGEND_GROUP_CLASS_ID`] 并取得键。
/// 循环内非 LegendClass 新对象为停止点（回退到该条目对象头起点）。
pub fn read_legend_group_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<LegendGroupPrefix10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // 1. 版本 short（有符号 >3 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_LEGEND_GROUP_VERSION,
        "LegendGroup",
        entry_offset,
    )?;
    // 2. short → obj+0xc。
    let heading_short = trial.read_u16_le()? as i16;
    // 3. 版本 ≥2：short → obj+0xe（<2 时内存默认 -1，不读流）。
    let editable_short = if (version as i16) >= 2 {
        Some(trial.read_u16_le()? as i16)
    } else {
        None
    };
    // 4. String → obj+0x14。
    let heading = read_string_10_1(&mut trial, max_bytes)?;

    // 登记表副本：循环内条目登记先发生在副本上，完整成功（含停止）才随
    // 外层提交；任何失败路径直接向上传播，调用者的登记表不动。
    let mut staged_table = table.clone();

    // 5. int32 计数 + 循环 LoadObject(ILegendClass)。全部登记发生在
    //    staged_table 副本上，整体成功才随外层提交。
    let class_count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let mut classes = Vec::new();
    let mut stop = None;
    for index in 0..class_count.max(0) as usize {
        let item_offset = trial.position();
        let pre_item_table = staged_table.clone();
        let entry = read_object_entry_10_1(&mut trial, &mut staged_table)?;
        match entry {
            ObjectEntry10_1::Null => classes.push(LegendClassEntry10_1::Null),
            ObjectEntry10_1::BackReference { class_id, key } => {
                classes.push(LegendClassEntry10_1::BackReference { class_id, key });
            }
            ObjectEntry10_1::NewObject { class_id, key } if class_id == LEGEND_CLASS_CLASS_ID => {
                let lc =
                    read_legend_class_body_10_1(&mut trial, max_bytes, &mut staged_table, key)?;
                if let Some(inner) = lc.stop.clone() {
                    stop = Some(LegendGroupStop10_1::ClassBody(inner));
                }
                classes.push(LegendClassEntry10_1::NewObject(Box::new(lc)));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(LegendGroupStop10_1::ClassUnsupported {
                    class_id,
                    key,
                    index,
                });
                trial = reposition_to(reader, item_offset);
                // 回退本条目已做的登记（其后的条目尚未登记）。
                staged_table = pre_item_table;
            }
        }
        if stop.is_some() {
            break;
        }
    }

    // 6. 版本 >2 的 short → obj+0x12（前置停止时不读取）。
    let trailing_short = if stop.is_none() && (version as i16) > 2 {
        Some(trial.read_u16_le()? as i16)
    } else {
        None
    };

    let result = LegendGroupPrefix10_1 {
        key,
        version,
        heading_short,
        editable_short,
        heading,
        class_count,
        classes,
        trailing_short,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 SimpleRenderer 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者（如 feature_layer 渲染器槽分派的后续接线）已读取对象
/// 头、比对 [`SIMPLE_RENDERER_CLASS_ID`] 并取得键 `key`。`max_bytes` 应用于
/// 全部字符串（含嵌套 LegendGroup/LegendClass 体内字符串）的声明长度。
///
/// ISymbol 槽新对象仅支持 SimpleMarkerSymbol，ILegendGroup 槽仅支持
/// LegendGroup，ILegendItem 槽新对象为停止点（三者均回退到槽位起点、不保
/// 留该键）；嵌套体内的停止传播为本读取器停止原因，游标停在内层停止点。
/// 截断、版本超限、字符串超限为错误（游标与登记表不动）。
pub fn read_simple_renderer_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<SimpleRendererPrefix10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // 1. 版本 short（有符号 >3 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SIMPLE_RENDERER_VERSION,
        "SimpleRenderer",
        entry_offset,
    )?;

    // 2. ISymbol 槽（请求 IID {F3435802-…}）。
    let symbol_offset = trial.position();
    let mut staged_table = table.clone();
    let mut stop = None;
    let symbol = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
        ObjectEntry10_1::Null => SymbolSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            SymbolSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key }
            if class_id == SIMPLE_MARKER_SYMBOL_CLASS_ID =>
        {
            let sms = read_simple_marker_symbol_body_10_1(&mut trial, &mut staged_table, key)?;
            if let Some(inner) = sms.stop {
                stop = Some(SimpleRendererStop10_1::SymbolBody(inner));
            }
            SymbolSlot10_1::SimpleMarkerSymbol(Box::new(sms))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            stop = Some(SimpleRendererStop10_1::SymbolUnsupported { class_id, key });
            trial = reposition_to(reader, symbol_offset);
            staged_table = table.clone();
            SymbolSlot10_1::Null
        }
    };

    // 3. ILegendGroup 槽（请求 IID {167C5E9F-…}）。
    let mut legend_group = None;
    if stop.is_none() {
        let group_offset = trial.position();
        let pre_group_table = staged_table.clone();
        match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => legend_group = Some(LegendGroupSlot10_1::Null),
            ObjectEntry10_1::BackReference { class_id, key } => {
                legend_group = Some(LegendGroupSlot10_1::BackReference { class_id, key });
            }
            ObjectEntry10_1::NewObject { class_id, key } if class_id == LEGEND_GROUP_CLASS_ID => {
                let lg =
                    read_legend_group_body_10_1(&mut trial, max_bytes, &mut staged_table, key)?;
                if let Some(inner) = lg.stop.clone() {
                    stop = Some(SimpleRendererStop10_1::LegendGroupBody(inner));
                }
                legend_group = Some(LegendGroupSlot10_1::LegendGroup(Box::new(lg)));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(SimpleRendererStop10_1::LegendGroupUnsupported { class_id, key });
                trial = reposition_to(reader, group_offset);
                staged_table = pre_group_table;
                legend_group = Some(LegendGroupSlot10_1::Null);
            }
        }
    }

    // 4. ILegendItem 槽（请求 IID {52AA5BC0-…}）：coclass 体未闭合，新对象
    //    即停止（回退到槽位起点）。
    let mut legend_item = None;
    if stop.is_none() {
        let item_offset = trial.position();
        let pre_item_table = staged_table.clone();
        let entry = read_object_entry_10_1(&mut trial, &mut staged_table)?;
        if let ObjectEntry10_1::NewObject { class_id, key } = entry {
            stop = Some(SimpleRendererStop10_1::LegendItemNewObject { class_id, key });
            trial = reposition_to(reader, item_offset);
            staged_table = pre_item_table;
        }
        legend_item = Some(entry);
    }

    // 5-7. short +0x128、LoadLegacyRotationFieldAndType（无版本 short）与
    //      String +0x12c。
    let mut short_128 = None;
    let mut rotation_field = None;
    let mut rotation_type = None;
    let mut label = None;
    if stop.is_none() {
        short_128 = Some(trial.read_u16_le()? as i16);
        rotation_field = Some(read_string_10_1(&mut trial, max_bytes)?);
        rotation_type = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        label = Some(read_string_10_1(&mut trial, max_bytes)?);
    }

    // 8. 版本 >2：RotationRenderer::Load + SizeRenderer::Load 尾段。
    let mut rotation_tail = None;
    let mut size_tail = None;
    if stop.is_none() && (version as i16) > 2 {
        let tail_offset = trial.position();
        let rv = read_version_short(
            &mut trial,
            MAX_RENDERER_TAIL_VERSION,
            "RotationRenderer",
            tail_offset,
        )?;
        let mut flags = [0_i32; 2];
        for slot in flags.iter_mut() {
            *slot = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        }
        let mut expressions: [String; 3] = Default::default();
        for slot in expressions.iter_mut() {
            *slot = read_string_10_1(&mut trial, max_bytes)?;
        }
        let mut doubles = [0_f64; 6];
        for slot in doubles.iter_mut() {
            *slot = trial.read_f64_le()?;
        }
        rotation_tail = Some(RotationRendererTail10_1 {
            version: rv,
            flags,
            expressions,
            doubles,
        });

        let size_offset = trial.position();
        let sv = read_version_short(
            &mut trial,
            MAX_RENDERER_TAIL_VERSION,
            "SizeRenderer",
            size_offset,
        )?;
        let size_flags = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        let expression = read_string_10_1(&mut trial, max_bytes)?;
        let mut size_doubles = [0_f64; 2];
        for slot in size_doubles.iter_mut() {
            *slot = trial.read_f64_le()?;
        }
        size_tail = Some(SizeRendererTail10_1 {
            version: sv,
            flags: size_flags,
            expression,
            doubles: size_doubles,
        });
    }

    let remaining_bytes = trial.remaining();
    let result = SimpleRendererPrefix10_1 {
        key,
        version,
        symbol,
        legend_group,
        legend_item,
        short_128,
        rotation_field,
        rotation_type,
        label,
        rotation_tail,
        size_tail,
        stop,
        remaining_bytes,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}
