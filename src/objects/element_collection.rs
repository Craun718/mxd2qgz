//! ArcMap 10.1 ElementCollection（元素集合，IElementCollection2 coclass）对象
//! 体的严格读取器。
//!
//! 证据来源：`docs/arcmap-10.1-element-collection.md`（本轮研究记录）。原生实
//! 现为 **Layer.dll 10.1.0.3035**（Setup1.cab 条目
//! `Layer.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`，196608 字节，SHA-256
//! `fe52bd99…e9efed8`，pefile FileVersion/ProductVersion 10.1.0.3035，
//! FileDescription "Layer DLL"、OriginalFilename "Layer.DLL"，PE x86，
//! ImageBase `0x78650000`；与安装配置 `esri.clsid.xml` 声明
//! `{CE8F3972-E9BE-11D1-A232-080009B6F22B}` progID `ElementCollection`、
//! 模块 Layer.dll，及 esriCarto 类型库 coclass `ElementCollection`
//! （接口 `IElementCollection {CE8F3971}` / `IElementCollection2
//! {B2BFC403}`）三方互证）。
//!
//! ## 定位链（RTTI→COL→虚表→GetClassID，10.1 静态映像 VA）
//!
//! RTTI `.?AVElementCollection@@`（TypeDescriptor `0x78679628`）→ 6 个子接
//! 口 COL（this-adjustor 0/4/16/20/24/28）→ 虚表 `0x7866da2c`/`0x7866da4c`/
//! `0x7866da58`/`0x7866da7c`/`0x7866daa0`/`0x7866dac4`。QueryInterface
//! `0x78659f20` 比对并按 adjustor 回答五个 IID：`{CE8F3971}`
//! IElementCollection→this+0x10、`{B2BFC403}` IElementCollection2→this+0x14、
//! `{00000109-0000-0000-C000-000000000046}` **IPersistStream**→this+0x18、
//! `{0000010C-…46}` IPersist、`{BB75DA81-…}` IDocumentVersionSupportGEN。
//! IPersistStream 虚表 = **`0x7866daa0`**（adjustor 24）：槽 +0x0c = GetClassID
//! `0x78659d80`（写出 `{CE8F3972-…}` 常量 `0x7866db18`，三方互证闭环）、
//! +0x10 = IsDirty `0x7865a320`（读 this+0x1c 脏标志）、**+0x14 = Load
//! `0x7865a340`**、+0x18 = Save `0x7865a4e0`。
//!
//! ## Load `0x7865a340` 字段序（版本 short 的有符号比较，>4 拒绝
//! `0x80004005`）
//!
//! 读流前先经 adjustor-16 子接口（IElementCollection）虚表 +0x18 槽做集合
//! 清理（候选 RemoveAll，不读流）。随后：
//!
//! 1. int16 版本：`(short)版本 > 4` 拒绝；版本 <4（含 0 与负值位模式）进入
//!    旧加载器 `FUN_7865a650`；
//! 2. 版本 ==4 主分支：int32 元素计数；循环计数次，每元素 =
//!    [LoadObject（请求 IID `{9B001319-9ED7-11D0-859E-0800091A2A72}` =
//!    IElement，esriCarto 类型库考证）+ int32 ×3]（逐元素交错），每元素经
//!    adjustor-20 子接口（IElementCollection2）虚表 +0x0c 槽加入集合；任
//!    一读取失败即中止返回；循环后写脏标志 0（this+0x34）并返回——**体在
//!    最后一个元素的第三个 int32 处终结，无尾随字段**；
//! 3. 旧加载器 `FUN_7865a650`（版本 0..3 同一代码路径）：int32 计数 → 第
//!    一循环逐元素 LoadObject(IElement) → 第二循环逐元素 int32 →（仅版本
//!    ≥3）第三循环逐元素 int32。三个循环**顺序分离**，不逐元素交错。
//!
//! 原生对计数 ≤0（含负数）不进循环、直接成功返回；本读取器对负计数显式拒
//! 绝（较原生更严，负数语义未知且写侧不产）。
//!
//! ## Save `0x7865a4e0`（写方向互证）
//!
//! 版本 short 经 `AfStreamGetSaveVersion(docVersion, 2, 表 0x7866d9c4`，表
//! 字节 `02 00 04 00`） 取值 → int32 计数（成员 +0x10）→
//! docVersion==0：两循环分离（先逐元素 SaveObject，再逐元素写一个 int32，
//! 与旧布局对应）；否则逐元素交错 [SaveObject + int32 ×3]（元素记录
//! {指针, i32, i32, i32}，+4/+8/+0xc 三槽）。仓库参考样本（10.1）全部写
//! 版本 4。
//!
//! ## 与 FDOGraphicsLayer::Load 的衔接
//!
//! AnnoLayer.dll `0x6b1955a0` 第 5 步以请求 IID
//! `{B2BFC403-6B4C-4490-8878-EA62C4F49720}`（即本类的 IElementCollection2
//! 接口，非 CLSID）LoadObject 到 this+0x11c。样本 `/Maps:0x1608` 该槽为
//! **新对象**（类标识 `{CE8F3972-…}`、键 86；非空标记、非回引），体
//! `0x161c..0x1622` = `04 00` + `00 00 00 00`（版本 4、计数 0，恰 6 字节，
//! 终结于委派 FeatureLayer 体起点 `0x1622`，与图层列表条目边界互证）。当
//! 前 `fdo_graphics_layer` 读取器在关联 FeatureLayer 体的停止点（渲染器链
//! 未闭合）之前无法到达该槽——本模块是体闭合后的消费端，不改其接线。
//!
//! 样本对位（辅助观察）：`/Maps` 流内 12 个 ElementCollection 对象头全部
//! 版本 4；计数 0 的实例（键 86 @`0x1608`、键 1554 @`0x17030` 等 7 个）为
//! 完整可消费体；计数 1..3 的实例（键 1612 @`0x18537`、键 1856 @`0x1dcec`
//! 等 5 个）首元素均为 TextElement `{204034D3-…}` 新对象（esri.clsid 归
//! 名），按本读取器语义在元素对象头起点停止。
//!
//! 这是仅覆盖上述分支的严格安全子集：要求完整输入，任何失败不推进调用者
//! 游标、不修改登记表。元素循环中的新对象按类标识分派（接线轮）：闭合元
//! 素类（TextElement/MarkerElement/LineElement）整体解析其体，未知元素类
//! 是停止点并回退到该元素对象头起点；闭合元素体的嵌套停止上抛为
//! [`ElementCollectionStop10_1::ElementBodyIncomplete`]（游标停在内层停止
//! 点，元素键保留）。未接入 MXD 解析主链，人工目视验收未做，不得声称已
//! 验证。
//!
//! 未解决问题（详见证据文档）：每元素 3 个 int32 的业务语义未定性（候选
//! 元素放置/绘制次序数据）；adjustor-20 接口虚表 +0x0c 的加集方法与
//! adjustor-16 接口 +0x0x18 的清理方法签名按调用侧推断；版本表
//! `0x7866d9c4` 的 `AfStreamGetSaveVersion` 内部映射未逐指令展开（样本形
//! 态写 4）；元素类（TextElement 等）体未闭合；负计数原生按空集合成功、
//! 本读取器拒绝（较原生更严）；全部结论未运行 ArcMap 验证。

use crate::objects::element_body::{
    ElementBody10_1, ElementBodyStop10_1, read_element_body_of_class_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// ElementCollection::Load 接受的最大版本：反编译 `4 < (short)版本` 有符号
/// 比较拒绝；写侧 10.1 恒写 4（样本 12/12 实例为 4）。
const MAX_ARCHITECTURE_VERSION: i16 = 4;

/// 旧布局（版本 <4）第三循环额外 int32 组的版本门：`FUN_7865a650` 中
/// `版本 < 3` 时第三组不读流（写 -1 默认值），版本 3 才逐元素读 int32。
const LEGACY_SECOND_GROUP_MIN_VERSION: u16 = 3;

/// 版本 4 主分支每个元素的最小流内字节数：IElement 空引用标记 16 字节 +
/// int32 ×3。用于计数 ×最小条目的资源限额预检。
const MIN_INTERLEAVED_ENTRY_BYTES: usize = 16 + 12;

/// 旧布局每个元素的最小流内字节数（第一循环空标记 16 + 第二循环 int32
/// 4；版本 3 的第三循环另计）。
const MIN_SEQUENTIAL_ENTRY_BYTES: usize = 16 + 4;

/// ElementCollection 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4
/// 原序）。
///
/// GUID 字符串形式为 `{CE8F3972-E9BE-11D1-A232-080009B6F22B}`。证据：
/// `esri.clsid.ecfg` 声明（模块 Layer.dll）；Layer.dll GetClassID
/// `0x78659d80` 写出 `.rdata` 常量 `0x7866db18` 的同一 CLSID；仓库参考样
/// 本 `/Maps:0x1608` 等 12 处对象头的类标识与本常量逐字节相同（辅助观
/// 察）。
pub const ELEMENT_COLLECTION_CLASS_ID: [u8; 16] = [
    0x72, 0x39, 0x8f, 0xce, 0xbe, 0xe9, 0xd1, 0x11, 0xa2, 0x32, 0x08, 0x00, 0x09, 0xb6, 0xf2, 0x2b,
];

/// 元素循环 LoadObject 的请求 IID `{9B001319-9ED7-11D0-859E-0800091A2A72}`
/// （= IElement，esriCarto 类型库考证）的 16 字节内存序。
///
/// 该 IID 不是类 CLSID：元素对象的类身份由流内 16 字节对象头决定（样本非
/// 空实例首元素为 TextElement `{204034D3-…}`）。
pub const ELEMENT_REQUEST_INTERFACE_ID: [u8; 16] = [
    0x19, 0x13, 0x00, 0x9b, 0xd7, 0x9e, 0xd0, 0x11, 0x85, 0x9e, 0x08, 0x00, 0x09, 0x1a, 0x2a, 0x72,
];

/// 已确认可继续的元素条目形态。
///
/// 新对象条目（未见过键 + 任意类标识）按流内类标识分派：接线轮起
/// TextElement/MarkerElement/LineElement（`element_body` 模块闭合）解析为
/// 对应体变体；其余类不是本枚举的变体——元素对象体布局未闭合且无外部长
/// 度字段，读取器在其对象头起点停止（见 [`ElementCollectionStop10_1`]）。
/// 回引条目 20 字节无体；空标记按对象头读取器的 S_FALSE 约定仅消耗 16 字
/// 节、无键。
#[derive(Debug, Clone, PartialEq)]
pub enum ElementEntry10_1 {
    /// 16 字节全零类标识：空引用标记（原生把空元素指针传入加集方法）。
    Null,
    /// 键已在本会话登记过：20 字节回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 TextElement 对象：已解析其体（体内嵌套停止经
    /// [`ElementCollectionStop10_1::ElementBodyIncomplete`] 上抛）。
    TextElement(Box<crate::objects::element_body::TextElementBody10_1>),
    /// 新 MarkerElement 对象：已解析其体。
    MarkerElement(Box<crate::objects::element_body::MarkerElementBody10_1>),
    /// 新 LineElement 对象：已解析其体。
    LineElement(Box<crate::objects::element_body::LineElementBody10_1>),
}

impl ElementEntry10_1 {
    /// 已解析元素体内的嵌套停止原因；`None` = 空标记/回引或体已完整消费。
    pub fn body_stop(&self) -> Option<&ElementBodyStop10_1> {
        match self {
            Self::Null | Self::BackReference { .. } => None,
            Self::TextElement(body) => body.stop.as_ref(),
            Self::MarkerElement(body) => body.stop.as_ref(),
            Self::LineElement(body) => body.stop.as_ref(),
        }
    }
}

/// 版本 4 主分支的单个元素槽：条目与其后交错的 3 个 int32（原样保留，语
/// 义未定性）。
#[derive(Debug, Clone, PartialEq)]
pub struct InterleavedElement10_1 {
    /// 元素条目（空标记或回引；新对象到达即停止，不出现在本结构中）。
    pub entry: ElementEntry10_1,
    /// 元素后逐条交错的 int32 ×3（加入集合方法的第 2..4 实参，语义未定
    /// 性，仅原样保留）。
    pub values: [i32; 3],
}

/// 元素循环之后的体内布局。
#[derive(Debug, Clone, PartialEq)]
pub enum ElementCollectionBody10_1 {
    /// 版本 4 主分支：计数个 [元素条目 + int32 ×3] 逐元素交错，体在最后一
    /// 个元素的第三个 int32 处终结（无尾随字段）。
    Interleaved {
        /// 已消费的元素槽（到停止点为止；完整消费时长度 == 计数）。
        slots: Vec<InterleavedElement10_1>,
    },
    /// 版本 ≤3 旧布局（`FUN_7865a650`）：三个顺序分离的循环——先全部元
    /// 素条目，再全部 int32，版本 3 再全部 int32。元素循环停止时两个
    /// int32 组为空（结构性未消费，停止原因中含元素下标）。
    Sequential {
        /// 已消费的元素条目（到停止点为止；完整消费时长度 == 计数）。
        elements: Vec<ElementEntry10_1>,
        /// 第二循环的逐元素 int32（完整消费时长度 == 计数）。
        group1: Vec<i32>,
        /// 版本 ≥3 的第三循环 int32；版本 <3 为 `None`（原生不读流、写 -1
        /// 默认值）。元素循环停止时同样为 `None`（未消费，以停止原因为
        /// 准）。
        group2: Option<Vec<i32>>,
    },
}

/// 已确认前缀的停止原因；`None` 表示对象体被完整消费（版本 4 或旧布局的
/// 全部循环到达终点）。
#[derive(Debug, Clone, PartialEq)]
pub enum ElementCollectionStop10_1 {
    /// 元素循环中的新对象条目（任意类标识，含 TextElement 等真实元素类）：
    /// 元素体布局未闭合且无外部长度，无法跳过。游标回退到该元素对象头起
    /// 点，该条目的键不进入登记表（回退语义）；此前元素条目已消费、其回
    /// 引判定不受影响。
    ElementBodyNotClosed {
        /// 停止元素在声明计数中的下标（0 起）。
        index: usize,
        /// 停止元素的类标识（原样保留）。
        class_id: [u8; 16],
        /// 停止元素的对象键（未登记）。
        key: u32,
    },
    /// 接线轮新增：停止元素为闭合元素类（TextElement/MarkerElement/
    /// LineElement）且其体内出现嵌套停止（原因透传，见
    /// [`ElementBodyStop10_1`]）。游标停在其内层停止点；该元素的对象键已按
    /// 原生语义登记（登记先于持久化），此前元素条目已消费。
    ElementBodyIncomplete {
        /// 停止元素在声明计数中的下标（0 起）。
        index: usize,
        /// 元素的类标识（原样保留）。
        class_id: [u8; 16],
        /// 元素的对象键（已登记）。
        key: u32,
        /// 元素体内的嵌套停止原因。
        reason: ElementBodyStop10_1,
    },
}

impl ElementCollectionStop10_1 {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(&self) -> String {
        match self {
            Self::ElementBodyNotClosed {
                index,
                class_id,
                key,
            } => format!(
                "元素 {index}（键 {key}，类标识 {}）为新对象：元素体布局未闭合且无外部长\
                 度，游标回退到该元素对象头起点",
                crate::storage::class_id_hex(class_id)
            ),
            Self::ElementBodyIncomplete {
                index,
                class_id,
                key,
                reason,
            } => format!(
                "元素 {index}（键 {key}，类标识 {}）体在嵌套停止点停止：{reason:?}；游标停\
                 在内层停止点，其后元素不可达",
                crate::storage::class_id_hex(class_id)
            ),
        }
    }
}

/// 10.1 ElementCollection::Load（`0x7865a340`）可读取的对象体。
#[derive(Debug, Clone, PartialEq)]
pub struct ElementCollection10_1 {
    /// 2 字节版本 short 的无符号位模式。
    ///
    /// 原生有符号比较 >4 拒绝（`0x80004005`）；负值位模式原生落入旧加载
    /// 器，本读取器一并拒绝（较原生更严：负值版本无业务意义，写侧仅产
    /// 2/4）。版本 0 与 1/2 走同一旧布局代码路径（`FUN_7865a650` 的
    /// `版本 < 3` 分支），证据覆盖，接受。
    pub version: u16,
    /// int32 元素计数的无符号值。原生对 ≤0 计数不进循环直接成功；本读取
    /// 器对负计数显式拒绝（较原生更严），0 表示空集合。
    pub count: u32,
    /// 到停止点为止已消费的体内布局。
    pub body: ElementCollectionBody10_1,
    /// 停止原因；`None` = 对象体完整消费。
    pub stop: Option<ElementCollectionStop10_1>,
    /// 停止点之后剩余的字节数（相对传入 Reader 窗口；完整消费时为窗口内
    /// 剩余全部字节）。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 10.1 ElementCollection 对象体的严格子集。
///
/// 起点约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
/// 头、比对 [`ELEMENT_COLLECTION_CLASS_ID`] 确认类身份，并从对象头之后的位
/// 置传入 Reader；本函数不重复读取对象头。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额：在任何元素条目读取之前预检「声明计数 × 每元素
/// 最小字节数」（版本 4 为 28、旧布局为 20、版本 3 再加每元素 4）不超过限
/// 额，防止按异常计数继续解析；`usize::MAX` 表示不额外设限。`table` 是跨
/// 流共享的键登记表：元素新键在条目分类时登记，若该元素体不可消费（停止
/// 点），该次登记随回退一并撤销；此前条目的登记随成功提交。
///
/// 事务性：全部字段在 trial 游标与 trial 登记表上读取，成功返回（含停止）
/// 才提交；任何失败（截断、版本超限、负计数、限额溢出）都不推进调用者游
/// 标、不修改登记表。这是严格安全子集，不代表完整原生 Load 行为，通过也
/// 不代表 MXD 格式已验证。
pub fn read_element_collection_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<ElementCollection10_1> {
    let start = reader.position();
    let origin = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("ElementCollection 对象体 offset 0x{start:x}: {reason}"),
        )
    };

    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. 版本 short：原生有符号比较（反编译 `4 < (short)版本`），>4 拒绝；
    //    负值位模式一并拒绝（较原生更严，见结构体字段注释）。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_ARCHITECTURE_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "架构版本 {version} 超出 ElementCollection::Load 接受范围（有符号 >4 拒绝；负值\
             位模式一并拒绝）"
        )));
    }

    // 2. int32 元素计数：原生仅正数进入循环（≤0 直接成功）；负数显式拒绝。
    let count_offset = trial.position();
    let count_raw = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if count_raw < 0 {
        return Err(invalid(format!(
            "元素计数 offset 0x{count_offset:x} 为负数（{count_raw}）：原生按 ≤0 不进循环直\
             接成功，负数语义未知，本读取器拒绝"
        )));
    }
    let count = usize::try_from(count_raw).expect("计数已确认为非负");

    // 3. 限额预检（在任何条目读取之前）：按布局的最小每元素字节数。
    let (min_entry, extra_per_element) = if version == 4 {
        (MIN_INTERLEAVED_ENTRY_BYTES, 0_usize)
    } else {
        let extra = if version >= LEGACY_SECOND_GROUP_MIN_VERSION {
            4
        } else {
            0
        };
        (MIN_SEQUENTIAL_ENTRY_BYTES, extra)
    };
    let min_total = count
        .checked_mul(min_entry + extra_per_element)
        .ok_or_else(|| invalid("元素计数 × 最小条目字节数在 usize 上溢出".to_string()))?;
    if min_total > max_bytes {
        return Err(invalid(format!(
            "声明计数 {count_raw} × 最小条目字节数 {min_entry}{} 超过资源限额 {max_bytes}，\
             在任何条目读取之前拒绝",
            if extra_per_element > 0 {
                format!(" + {extra_per_element}")
            } else {
                String::new()
            }
        )));
    }

    // 4. 元素循环 + 后继字段（按布局分支）。
    let mut stop = None;
    let body = if version == 4 {
        // 版本 4 主分支：每元素 [IElement 条目 + int32 ×3] 逐条交错。
        let mut slots = Vec::new();
        for index in 0..count {
            let entry = read_stoppable_entry(
                &mut trial,
                &mut trial_table,
                &origin,
                start,
                max_bytes,
                index,
                &mut stop,
            )?;
            let Some(entry) = entry else {
                // 新对象元素：游标与登记表已回退到该元素对象头起点。
                break;
            };
            let mut values = [0_i32; 3];
            for slot in &mut values {
                *slot = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
            }
            slots.push(InterleavedElement10_1 { entry, values });
        }
        ElementCollectionBody10_1::Interleaved { slots }
    } else {
        // 旧布局：三个顺序分离的循环。
        let mut elements = Vec::new();
        for index in 0..count {
            let entry = read_stoppable_entry(
                &mut trial,
                &mut trial_table,
                &origin,
                start,
                max_bytes,
                index,
                &mut stop,
            )?;
            let Some(entry) = entry else {
                break;
            };
            elements.push(entry);
        }
        let mut group1 = Vec::new();
        let group2 = if stop.is_none() {
            for _ in 0..count {
                group1.push(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
            }
            if version >= LEGACY_SECOND_GROUP_MIN_VERSION {
                let mut group2 = Vec::new();
                for _ in 0..count {
                    group2.push(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
                }
                Some(group2)
            } else {
                // 版本 <3：原生第三组不读流（内存写 -1 默认值），无流内字节。
                None
            }
        } else {
            // 元素循环停止：两个 int32 循环在停止点之后，结构性未消费
            // （group1/group2 为空；停止原因中已含元素下标）。
            None
        };
        ElementCollectionBody10_1::Sequential {
            elements,
            group1,
            group2,
        }
    };

    let result = ElementCollection10_1 {
        version,
        count: count_raw as u32,
        body,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 读取一个元素条目并分类；未知元素类的新对象条目是停止点。
///
/// 空标记与回引用条目消费后返回 `Some`；新对象条目按类标识分派：接线轮起
/// TextElement/MarkerElement/LineElement 经
/// [`read_element_body_of_class_10_1`] 解析其体（对象键已由条目分类登记，
/// 原生「登记先于持久化」语义），体内嵌套停止把
/// [`ElementCollectionStop10_1::ElementBodyIncomplete`] 写入 `stop` 并返回
/// `None`（游标停在内层停止点，集合循环终止）；其余类把
/// [`ElementCollectionStop10_1::ElementBodyNotClosed`] 写入 `stop`，trial 游
/// 标与 trial 登记表回退到该元素对象头起点（该键不登记），返回 `None`。
/// 体读取失败（截断等）原样向上传播（整体事务性由调用者的 trial 副本保证）。
fn read_stoppable_entry<'a>(
    trial: &mut Reader<'a>,
    trial_table: &mut RefTable10_1,
    origin: &Reader<'a>,
    start: usize,
    max_bytes: usize,
    index: usize,
    stop: &mut Option<ElementCollectionStop10_1>,
) -> io::Result<Option<ElementEntry10_1>> {
    let slot_offset = trial.position();
    let saved_table = trial_table.clone();
    let entry = read_object_entry_10_1(trial, trial_table)?;
    let (class_id, key) = match &entry {
        ObjectEntry10_1::Null => return Ok(Some(ElementEntry10_1::Null)),
        ObjectEntry10_1::BackReference { class_id, key } => {
            return Ok(Some(ElementEntry10_1::BackReference {
                class_id: *class_id,
                key: *key,
            }));
        }
        ObjectEntry10_1::NewObject {
            class_id: id,
            key: k,
        } => (*id, *k),
    };
    // 新对象：按类标识分派闭合元素体；其他类维持未知类停止回退语义。
    let parsed = read_element_body_of_class_10_1(&class_id, trial, max_bytes, trial_table);
    match parsed {
        Ok(ElementBody10_1::TextElement(body)) => {
            if let Some(reason) = body.stop.clone() {
                *stop = Some(ElementCollectionStop10_1::ElementBodyIncomplete {
                    index,
                    class_id,
                    key,
                    reason,
                });
                return Ok(None);
            }
            Ok(Some(ElementEntry10_1::TextElement(body)))
        }
        Ok(ElementBody10_1::MarkerElement(body)) => {
            if let Some(reason) = body.stop.clone() {
                *stop = Some(ElementCollectionStop10_1::ElementBodyIncomplete {
                    index,
                    class_id,
                    key,
                    reason,
                });
                return Ok(None);
            }
            Ok(Some(ElementEntry10_1::MarkerElement(body)))
        }
        Ok(ElementBody10_1::LineElement(body)) => {
            if let Some(reason) = body.stop.clone() {
                *stop = Some(ElementCollectionStop10_1::ElementBodyIncomplete {
                    index,
                    class_id,
                    key,
                    reason,
                });
                return Ok(None);
            }
            Ok(Some(ElementEntry10_1::LineElement(body)))
        }
        Err(error) if error.kind() == ErrorKind::Unsupported => {
            // 未知元素类：体布局未闭合（TextElement 等类无读取器）且无外部
            // 长度：停止并回退到该元素对象头起点，撤销本次键登记。
            *trial_table = saved_table;
            *trial = reposition(origin, start, slot_offset);
            *stop = Some(ElementCollectionStop10_1::ElementBodyNotClosed {
                index,
                class_id,
                key,
            });
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

/// 把游标重定位到「传入 Reader 窗口」的绝对偏移 `target`（入口之后的前向
/// 读取实现；目标必在窗口内，越界按内部错误处理）。
fn reposition<'a>(origin: &Reader<'a>, start: usize, target: usize) -> Reader<'a> {
    debug_assert!(target >= start);
    let mut positioned = origin.clone();
    positioned
        .read_bytes(target - start)
        .expect("回退目标在停止前的已读位置，必在窗口内");
    positioned
}
