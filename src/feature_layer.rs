//! ArcMap 10.1 FeatureLayer（要素图层）对象体已确认前缀的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-map-object.md` 的「图层树、可见性与数据源连
//! 接」「FeatureLayer::Load 请求 IID 字节表」「FeatureLayer::Load 中后段补
//! 全」「MapSurround/书签循环接口身份…第五轮」各节（2026-11-13/14），以及
//! `docs/arcmap-10.1-spatial-reference.md` 的「FeatureLayer::Save（0x6f6eb1b0，
//! 写侧新证据；归因仍开放）」节。字段序来自 FeatureLayer.dll 10.1.0.3035
//! （SHA-256 `2baf8706…98bb60`）的 `IPersistStream::Load`（RTTI
//! `.?AVFeatureLayer@@` → IPersistStream 子接口虚表 `0x6f7acfb0` 槽 +0x14
//! = `0x6f6e9940`）静态反编译；全部证据未运行目标软件。
//!
//! 已确认前缀的字段序（Load `0x6f6e9940` 头部段；上限版本 0x22=34，样本值
//! 34）：
//!
//! 1. int16 架构版本（>34 拒绝；比较符号性未闭合，本读取器按无符号解释，
//!    负值位模式一并拒绝，与 map_object/spatial_reference 的版本处理一致）；
//! 2. AfCore String 名称 → obj+0x120；
//! 3. AfCore String 第二名称 → obj+0x124；
//! 4. int16 ×3 → obj+0x128 可见性 VARIANT_BOOL（样本 0xFFFF=TRUE，证据
//!    「确定的」）/ obj+0x12a / obj+0x12c（后两个语义未定，原样保留）；
//! 5. 嵌套 LoadObject（数据源连接，请求 IID
//!    `{4BA33331-D55F-11D1-8882-0000F877762D}`=IName，第五轮修正）：16 零
//!    字节 → 无数据源；键已见 → 回引（无对象体）；新对象且类标识为
//!    FeatureClassName → 读取其已确认前缀（int16 + AfCore String 要素类
//!    名），随后停止；新对象且类标识非 FeatureClassName → 不支持（对象体
//!    无外部长度字段，无法跳过）。
//!
//! 其后（+0x144 原样 16 字节 GUID、String、double ×2、short、`{3036D35D}`
//! 计数循环等）因 FeatureClassName 体中段未闭合而不可达，一律保留为剩余字
//! 节（[`FeatureLayerPrefix10_1::remaining_bytes`]），本读取器不读取、不解释。
//!
//! 调用约定：调用者先用 [`crate::object_stream::read_header_10_1`] 读取对象
//! 头并比对 [`FEATURE_LAYER_CLASS_ID`]，从对象头之后的位置调用
//! [`read_feature_layer_prefix_10_1`]；本模块不重复读取对象头。数据源嵌套
//! 条目用 [`crate::object_refs::read_object_entry_10_1`] 分类：登记先于对象
//! 体解析是原生语义；为满足「失败不修改登记表」，分类与登记先发生在调用
//! 者登记表的副本上，全部成功后才整体提交。
//!
//! 样本核对（辅助观察，不作测试依赖）：`samples/测试底图.mxd` /Maps 流内
//! 首个 FeatureLayer——对象头 @`0x296` 类标识与本模块常量逐字节相同（键
//! 32）；体 @`0x2aa` 版本 34（`22 00`）；名称 @`0x2ac` 长度 32 =
//! 「龙安镇人民政府工作底图用界址点」；第二名称 @`0x2d0` 长度 28 =
//! 「Shapefile 要素类」；可见性 `ff ff` @`0x2f0`、+0x12a/+0x12c 均为
//! `00 00`；FeatureClassName 对象头 @`0x2f6`（键 33），体 @`0x30a` =
//! `02 00` + String 长度 32（与图层名字节同文），已确认前缀止于 `0x330`。
//! 与反编译字段序逐字段吻合；样本吻合为辅助观察。
//!
//! 未解决问题（详见证据文档「未解决事项」）：
//! - FeatureClassName 体中段字段序未闭合（体内嵌 WorkspaceName
//!   `{5A350011-…}`：short(01)+String 路径+int32+String 类型描述+16 零，但
//!   定位与中段未闭合，GdbCore 候选不对位），本读取器读到要素类名后停止，
//!   中段保留在剩余字节中，不得实现；
//! - 样本在 +0x144 GUID 之后为固定 7 字节 `15 00 00 00 00 00 00` 且紧跟
//!   SimpleRenderer 头，与 RTM 10.1.0.3035 反编译的 Save/Load 字段序（GUID
//!   后 String + double ×2 + short，合计 ≥22 字节）不可调和；工作假设样本
//!   由 10.1 某服务包保存（/Version 串不区分 SP），未验证；
//! - 可见性之外两个 short（obj+0x12a/+0x12c）语义未定，原样保留；
//! - 版本比较的符号性未闭合（本读取器按无符号解释，较可能的有符号原生更
//!   严）；
//! - 回引条目不校验类标识是否为 FeatureClassName：样本证据（如
//!   `/Maps:0x2d2c` 对键 32 的回引头重复 FeatureLayer 类标识）表明回引头
//!   会重复被引对象的类标识，类标识原样保留、由调用者比对；回引无对象体、
//!   不影响后续字段对位，本读取器不据此拒绝。

use crate::native_string::read_string_10_1;
use crate::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::reader::Reader;
use std::io::{self, ErrorKind};

/// FeatureLayer::Load 接受的最大架构版本：>0x22（34）直接拒绝；样本值 34。
const MAX_ARCHITECTURE_VERSION: u16 = 34;

/// FeatureLayer 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{E663A651-8AAD-11D0-BEC7-00805F7C4268}`（Data1=
/// 0xE663A651 → `51 a6 63 e6`，Data2=0x8AAD → `ad 8a`，Data3=0x11D0 →
/// `d0 11`，Data4 原序 `be c7 00 80 5f 7c 42 68`）。三重印证：安装配置
/// `esri.clsid.ecfg` 声明该 CLSID → 类名 FeatureLayer（模块
/// FeatureLayer.dll）；FeatureLayer.dll RTTI 存在 `.?AVFeatureLayer@@`（TD
/// `0x6f7d2a34`）；GetClassID（IPersistStream 虚表 `0x6f7acfb0` 槽 3 =
/// `0x6f6ec450`）写出同一 CLSID。仓库参考样本 `/Maps:0x296` 首个
/// FeatureLayer 对象头的类标识与本常量逐字节相同（键 32）。
pub const FEATURE_LAYER_CLASS_ID: [u8; 16] = [
    0x51, 0xa6, 0x63, 0xe6, 0xad, 0x8a, 0xd0, 0x11, 0xbe, 0xc7, 0x00, 0x80, 0x5f, 0x7c, 0x42, 0x68,
];

/// FeatureClassName 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4
/// 原序）。
///
/// GUID 字符串形式为 `{198846D0-CA42-11D1-AA7C-00C04FA33A15}`（Data1=
/// 0x198846D0 → `d0 46 88 19`，Data2=0xCA42 → `42 ca`，Data3=0x11D1 →
/// `d1 11`，Data4 原序 `aa 7c 00 c0 4f a3 3a 15`）。证据：安装配置
/// `esri.clsid.ecfg` 声明（模块 GdbCore.dll）；GdbCore/GdbCoreLib/MxTools
/// 字节命中；仓库参考样本 `/Maps:0x2f6`（FeatureLayer 体内嵌数据源对象
/// 头，键 33）与本常量逐字节相同。
pub const FEATURE_CLASS_NAME_CLASS_ID: [u8; 16] = [
    0xd0, 0x46, 0x88, 0x19, 0x42, 0xca, 0xd1, 0x11, 0xaa, 0x7c, 0x00, 0xc0, 0x4f, 0xa3, 0x3a, 0x15,
];

/// 可见性字段（obj+0x128）的 VARIANT_BOOL 解释辅助。
///
/// VARIANT_BOOL 的规范取值只有 VARIANT_TRUE（0xFFFF）与 VARIANT_FALSE
/// （0x0000）；规范之外的位模式不做「非零即真」假设（COM 惯例如此，但原生
/// 比较语义未取证），按非规范原样保留。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility10_1 {
    /// 0xFFFF = VARIANT_TRUE（样本值 @`/Maps:0x2f0`）。
    True,
    /// 0x0000 = VARIANT_FALSE。
    False,
    /// 规范取值之外的位模式，原样保留。
    NonCanonical(u16),
}

/// FeatureClassName 对象体的已确认前缀。
///
/// 体中段字段序未闭合（见模块文档「未解决问题」），本结构止于要素类名。
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureClassNamePrefix10_1 {
    /// 对象头键（样本 33 @`/Maps:0x306`）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表（登记先于对象体解
    /// 析，见 [`crate::object_refs`]）。
    pub key: u32,
    /// 体首 int16（样本 02 @`/Maps:0x30a`），语义未闭合，按无符号位模式
    /// 原样保留、不校验。
    pub leading_short: u16,
    /// 要素类名（AfCore String 布局；样本 @`/Maps:0x30c` 长度 32，与图层
    /// 名「龙安镇人民政府工作底图用界址点」字节同文）。
    pub feature_class_name: String,
}

/// 数据源嵌套条目（请求 IID `{4BA33331-D55F-11D1-8882-0000F877762D}`=
/// IName）的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum DataSource10_1 {
    /// 16 字节全零类标识：空引用标记，无键、无对象体。
    Null,
    /// 新 FeatureClassName 对象：已确认前缀 = int16 + AfCore String 要素类名。
    FeatureClassName(FeatureClassNamePrefix10_1),
    /// 键已在本会话登记过的回引条目：其后没有对象体。
    ///
    /// 类标识原样保留、不校验是否为
    /// [`FEATURE_CLASS_NAME_CLASS_ID`]：回引头会重复被引对象的类标识（样本
    /// 证据见模块文档），调用者可自行比对；回引无体、不影响后续字段对位，
    /// 本读取器不据此拒绝。
    BackReference { class_id: [u8; 16], key: u32 },
}

/// 10.1 FeatureLayer::Load（`0x6f6e9940`）已确认前缀的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureLayerPrefix10_1 {
    /// 2 字节架构版本 short 的无符号位模式，>34 的位模式一律拒绝。
    ///
    /// 证据：Load `0x6f6e9940` 开头读取后对 >0x22（34）直接拒绝，样本值 34
    /// （`/Maps:0x2aa`）。反编译比较的符号性未闭合：按无符号解释接受
    /// 0..=34；若原生为有符号比较，负值位模式（0x8000..）可能通过原生检
    /// 查，此处一并拒绝（较原生更严，与 map_object/spatial_reference 的版
    /// 本处理一致）。
    pub version: u16,
    /// 图层名称（Load 第 2 步，写入 obj+0x120；样本「龙安镇人民政府工作底
    /// 图用界址点」）。
    pub name: String,
    /// 第二名称（Load 第 3 步，写入 obj+0x124；样本「Shapefile 要素类」）。
    pub secondary_name: String,
    /// obj+0x128 可见性的原始 u16 位模式（VARIANT_BOOL；样本 0xFFFF=TRUE）。
    ///
    /// 可见性结论为「确定的」（代码偏移 + 样本 0xFFFF @`/Maps:0x2f0` 互
    /// 证）；解释辅助见 [`Self::visibility`] 与 [`Visibility10_1`]，规范之
    /// 外的位模式原样保留。
    pub visibility_raw: u16,
    /// obj+0x12a 的 short，语义未定，按无符号位模式原样保留（样本 0x0000）。
    pub unknown_short_a: u16,
    /// obj+0x12c 的 short，语义未定，按无符号位模式原样保留（样本 0x0000）。
    pub unknown_short_b: u16,
    /// Load 第 5 步嵌套数据源条目的分类结果（请求 IID=IName，见
    /// [`DataSource10_1`]）。
    pub data_source: DataSource10_1,
    /// 已确认前缀之后剩余的字节数。
    ///
    /// FeatureClassName 体中段（含体内嵌 WorkspaceName）、+0x144 原样 16
    /// 字节 GUID、其后的 String/double ×2/short/`{3036D35D}` 计数循环等字
    /// 段因中段未闭合而不可达，本读取器不读取、不解释，全部保留于此。
    pub remaining_bytes: usize,
}

impl FeatureLayerPrefix10_1 {
    /// 按 VARIANT_BOOL 语义解释可见性原始位模式。
    ///
    /// 0xFFFF → [`Visibility10_1::True`]、0x0000 →
    /// [`Visibility10_1::False`]、其余位模式 → [`Visibility10_1::NonCanonical`]
    /// 原样保留（见枚举文档）。
    pub fn visibility(&self) -> Visibility10_1 {
        match self.visibility_raw {
            0xFFFF => Visibility10_1::True,
            0x0000 => Visibility10_1::False,
            other => Visibility10_1::NonCanonical(other),
        }
    }
}

/// 从对象头之后的位置读取 10.1 FeatureLayer 对象体的已确认前缀，失败时不
/// 推进游标、不修改登记表。
///
/// 起点约定：调用者已用 [`crate::object_stream::read_header_10_1`] 读取对象
/// 头、比对 [`FEATURE_LAYER_CLASS_ID`] 确认类身份，并从对象头之后的位置传
/// 入 Reader；本函数不重复读取对象头。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额：应用于两个名称字符串与要素类名字符串的声明长度
/// （含终止零、不含 4 字节前缀），在任何读取与分配之前检查；`usize::MAX`
/// 表示不额外设限，此时超长流按短读失败。
///
/// 数据源条目经 [`crate::object_refs::read_object_entry_10_1`] 分类并登记：
/// 16 零字节为空引用；键已见为回引（无体）；新对象且类标识为
/// FeatureClassName 时读取其已确认前缀后停止；其他类标识返回
/// [`ErrorKind::Unsupported`]——嵌套对象体没有外部长度字段，本读取器无法
/// 跳过未知类继续解析，调用者不能把该流继续当作 FeatureLayer 前缀消费。
/// 分类依赖登记表状态：空表会把实际的回引分类为新对象（流内无法局部区分，
/// 与原生一致）。登记先于对象体解析是原生语义；为保证「失败不修改登记
/// 表」，登记先发生在调用者登记表的副本上，全部成功后才整体提交。
///
/// 全部字段在 trial 游标上读取，完整成功才提交；任何失败路径都不推进调用
/// 者游标。错误位置相对于传入 Reader，调用方需另外记录流和对象身份。这是
/// 严格安全子集，不代表完整原生 Load 行为，通过也不代表 MXD 格式已验证。
pub fn read_feature_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<FeatureLayerPrefix10_1> {
    // 全部字段在 trial 游标上读取，完整成功才提交；任何失败路径都不推进原游标。
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // 1. 2 字节架构版本：无符号解释，>34 直接拒绝（原生 >0x22 拒绝）。
    let version = trial.read_u16_le()?;
    if version > MAX_ARCHITECTURE_VERSION {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "FeatureLayer 对象体 offset 0x{entry_offset:x}: 架构版本 {version} 超出 \
                 FeatureLayer::Load 接受范围（>0x22/34 直接拒绝）"
            ),
        ));
    }

    // 2/3. 两个名称字符串：AfCore String 布局（证据见模块文档），各自受
    // max_bytes 限额约束。
    let name = read_string_10_1(&mut trial, max_bytes)?;
    let secondary_name = read_string_10_1(&mut trial, max_bytes)?;

    // 4. int16 ×3：+0x128 可见性（VARIANT_BOOL，解释辅助见 visibility()）、
    // +0x12a/+0x12c 语义未定，均按原始位模式保留、不校验。
    let visibility_raw = trial.read_u16_le()?;
    let unknown_short_a = trial.read_u16_le()?;
    let unknown_short_b = trial.read_u16_le()?;

    // 5. 嵌套数据源条目：object_refs 的分类状态机（Null/回引/新对象）。
    // 登记先于对象体解析是原生语义；为满足「失败不修改登记表」，分类与
    // 登记先发生在调用者登记表的副本上，全部成功后才整体提交。
    let nested_offset = trial.position();
    let mut staged_table = table.clone();
    let data_source = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
        ObjectEntry10_1::Null => DataSource10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            DataSource10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == FEATURE_CLASS_NAME_CLASS_ID => {
            // FeatureClassName 体已确认前缀：int16 + 要素类名字符串。中段
            // 字段序未闭合（见模块文档「未解决问题」），读到要素类名后停止，
            // 剩余字节不解释。
            let leading_short = trial.read_u16_le()?;
            let feature_class_name = read_string_10_1(&mut trial, max_bytes)?;
            DataSource10_1::FeatureClassName(FeatureClassNamePrefix10_1 {
                key,
                leading_short,
                feature_class_name,
            })
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            // 严格子集只认 FeatureClassName 数据源。新对象体没有外部长度字
            // 段，无法跳过未知类继续读取；显式报错，原游标与登记表不动
            // （键虽已被分类状态机登记进副本，副本随失败丢弃）。
            let id: String = class_id.iter().map(|byte| format!("{byte:02x}")).collect();
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "FeatureLayer 数据源对象 offset 0x{nested_offset:x}: 类标识 {id}（键 \
                     {key}）不是 FeatureClassName；本读取器仅支持 FeatureClassName 数据源，\
                     且对象体无外部长度字段，无法跳过未知类继续读取"
                ),
            ));
        }
    };

    let result = FeatureLayerPrefix10_1 {
        version,
        name,
        secondary_name,
        visibility_raw,
        unknown_short_a,
        unknown_short_b,
        data_source,
        remaining_bytes: trial.remaining(),
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}
