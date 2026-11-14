//! ArcMap 10.1 CompositeGraphicsLayer（复合图形层）对象体已确认前缀的严格
//! 读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-map-object.md`「CompositeGraphicsLayer::Load
//! （0x6f6b2100）」节（2026-11-14 第四轮）、「未解决事项（第五轮新增）」
//! 与「字段序闭合补充（2026-11-13 第二轮）」。字段序以 FeatureLayer.dll
//! 10.1.0.3035 的 `CompositeGraphicsLayer::Load`（`0x6f6b2100`，RTTI
//! `.?AVCompositeGraphicsLayer@@` → COL `0x6f7b4624` → 虚表 `0x6f7aa318`
//! +0x14）反编译为准（研究产物 `/tmp/r3/annolayer_work/cgl_load_6f6b2100.c`，
//! 见证据文档「证据与复现」节）；本模块实现时逐行复核了该反编译，与文档
//! 节摘要有差异的字段在下方逐处标注。
//!
//! 已确认字段序（编号 = 流内顺序）：
//!
//! 1. int16 版本：原生以有符号 short 比较 `ver < 7` 接受（>6 拒绝）。符
//!    号性未在文档闭合，本读取器按无符号解释，负值位模式一并拒绝，较原
//!    生更严；
//! 2. int32 元素组计数（原生按有符号比较，<=0 不迭代）+ 每组
//!    [int32 元素计数（同样有符号比较）+ 每元素一个嵌套对象（请求 IID
//!    `DAT_6f7aa4fc`，字节未解码）]；
//! 3. AfCore String 名称 → obj+0x20；
//! 4. int16 ×3 → +0x34/+0x36/+0x38；
//! 5. double ×2 → +0x24/+0x2c；
//! 6. int32 ×3 → +0x4c、+0x50、读入局部变量。[文档节摘要有误：反编译为
//!    连续三个 int32，第三个读入局部变量后未见消费；样本 `/Maps:0x17024`
//!    起的 `3,1,1` 与其后对象头逐字节印证这一布局]；
//! 7. 嵌套对象 ×1（请求 IID `DAT_6f7aa58c`，字节未解码）→ +0x1c。样本中
//!    该对象是 ElementCollection `{CE8F3972-E9BE-11D1-A232-080009B6F22B}`；
//! 8. 版本 >1：double → +0x54 + int32（读入局部变量）；
//! 9. [反编译此处调用 `FUN_6f6b3100()`，无参数——流对象是本函数局部变量，
//!    按不消费流处理，未验证] int32 子图层计数（有符号比较，<=0 不迭代）
//!    + 每项：AfCore String 条目名 + 嵌套对象（请求 IID
//!    {34C20002-4D3C-11D0-92D8-00805F7C28B0}=ILayer，第五轮候选转确认）
//!    + 条目尾段 short ×3（→ 新记录 +0x60/+0x62/+0x64）、double ×2
//!    （→ +0x68/+0x70）、int32 ×2（→ +0x7c/+0x80）、嵌套对象
//!    （`DAT_6f7aa58c` → +0x94）、版本 >1 再读 double（→ +0x88）。
//!    [条目尾段在文档节摘要中未列出，第五轮「未解决事项」3 已记载
//!    「short×3+double×2+int×2+LoadObject+double，CGL::Load 已有」]；
//! 10. 尾段未逐项展开（不可实现，本读取器在此停止）：反编译可见循环后的
//!     int32（→ +0x50）、嵌套对象（ILayer 族 → +0x48）、版本 >2 的嵌套对
//!     象（`DAT_6f7aa48c` → +0x3c）、版本 >=4 的 String（→ +0xb4）、==4
//!     再读 short、版本 >=5 的 double ×2（→ +0xbc/+0xc4）；这些字段本读
//!     取器不消费，保留为未知尾部。
//!
//! 停止语义（与 [`crate::map_object`] 的设计差异及理由）：map_object 对未
//! 知嵌套类直接报错，因为其后没有同序字段可继续；本模块面对的嵌套对象
//! （元素、条目 ILayer、ElementCollection 等）之后仍有已确认字段，而新对
//! 象体没有外部长度字段、无法跳过——报错会丢弃已解析前缀。因此本读取器
//! 是「已确认前缀」语义：解析到首个不可闭合字段（新对象体 / 尾段）为止，
//! 返回已解析字段 + 剩余字节数 + 停止原因
//! [`CompositeGraphicsLayerStop10_1`]，不因未知嵌套类报错中断。
//!
//! 调用约定：调用者先用 [`crate::object_stream::read_header_10_1`] 读取对象
//! 头并比对 [`COMPOSITE_GRAPHICS_LAYER_CLASS_ID`]，从对象头之后的位置调用
//! [`read_composite_graphics_layer_prefix_10_1`]；本模块不重复读取对象头。
//!
//! 样本核对（辅助观察，非测试依赖）：`samples/测试底图.mxd` 的 `/Maps`
//! 导出中，对象头 @`0x16fba` 类标识与本常量逐字节相同、键 1553；体
//! @`0x16fce` 起逐段吻合：版本 6、元素组计数 11 且组内计数全 0（44 字节
//! 零）、名称 `<默认>`（长度 10，@`0x17000`）、short
//! `0xffff/0xffff/0x0000`（@`0x1700e`）、double `0.0/0.0`（@`0x17014`）、
//! 三个 int32 `3,1,1`（@`0x17024`）、第 7 步对象头 @`0x17030` 为
//! ElementCollection 键 1554（新对象，本读取器在此停止）。停止点之后的
//! 人工观察：子图层计数存在 `0x17052`=1 与 `0x17056`=10 二解未裁决（第五
//! 轮「未解决事项」3；按 `0x17056`=10 读时，`0x1705a` 起 String
//! `JZD1 注记` 恰好终结于 FeatureLayer 对象头 @`0x1706e` 键 1555、体版本
//! 34、名称 `JZD1`，链条自洽；按 `0x17052`=1 读则该 String 非法——本读取
//! 器按反编译字段序消费、计数原样保留，不做裁决）；文档第四节转录的条目
//! 名「MZD1 」与当前样本字节不符（实际为 `JZD1 注记`）。本次核对基于
//! 2026-11-14 的样本导出（与第四轮研究副本在 CGL 区域外有 157 字节字符
//! 串差异；CGL 区域内逐字节一致）。样本吻合为辅助观察，未在目标软件中
//! 验证。
//!
//! 未解决问题：`DAT_6f7aa4fc`/`DAT_6f7aa58c`/`DAT_6f7aa48c` 的请求 IID
//! 字节未解码；子图层计数二解未裁决（见上）；第 6 步第三个 int32、第 8
//! 步 int32、条目尾段字段的语义未定性；`FUN_6f6b3100` 与尾段未展开；版
//! 本 short 符号性未闭合（本读取器按无符号更严处理）；条目尾段与单元
//! FeatureLayer 体结束边界的逐字节对位未完成（第五轮「未解决事项」3）。
//! 人工目视验收未做，不声称已验证。

use crate::native_string::read_string_10_1;
use crate::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::reader::Reader;
use std::io::{self, ErrorKind};

/// CompositeGraphicsLayer::Load 接受的最大版本：原生 `ver < 7`（有符号），
/// 即 >6 拒绝；样本值 6。
const MAX_ARCHITECTURE_VERSION: u16 = 6;

/// CompositeGraphicsLayer 类标识（CLSID）的 16 字节内存序（Data1-3 小端、
/// Data4 原序）。
///
/// GUID 字符串形式为 `{9646BB83-9512-11D2-A2F6-080009B6F22B}`。证据：
/// 安装配置 `esri.clsid.ecfg` 声明该 CLSID → 类名 CompositeGraphicsLayer
/// （模块 FeatureLayer.dll）；FeatureLayer.dll RTTI 存在
/// `.?AVCompositeGraphicsLayer@@`，其 IPersistStream::Load 即 `0x6f6b2100`；
/// 参考样本 `/Maps:0x16fba` 的对象头类标识与本常量逐字节相同（键 1553）。
pub const COMPOSITE_GRAPHICS_LAYER_CLASS_ID: [u8; 16] = [
    0x83, 0xbb, 0x46, 0x96, 0x12, 0x95, 0xd2, 0x11, 0xa2, 0xf6, 0x08, 0x00, 0x09, 0xb6, 0xf2, 0x2b,
];

/// 已消费的嵌套对象条目（新对象是停止点，不出现在记录中）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumedEntry10_1 {
    /// 16 字节全零类标识的空引用：无键、无对象体。
    Null,
    /// 键已在此前登记：回引条目，其后没有对象体。
    BackReference { class_id: [u8; 16], key: u32 },
}

/// 一个元素组（字段序第 2 步循环）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementGroup10_1 {
    /// 组内元素计数的原始位模式；原生按有符号比较，<=0 不迭代。
    pub declared_count: u32,
    /// 已成功分类的元素条目；触发
    /// [`CompositeGraphicsLayerStop10_1::ElementNewObject`] 停止的元素不
    /// 记录，该组整体不记录。
    pub elements: Vec<ConsumedEntry10_1>,
}

/// 一个子图层条目（字段序第 9 步循环）。
#[derive(Debug, Clone, PartialEq)]
pub struct SubLayerEntry10_1 {
    /// 条目名称（AfCore String，样本 `JZD1 注记`）。
    pub name: String,
    /// 条目的 ILayer 对象（请求 IID {34C20002-…}）分类。
    pub layer: ConsumedEntry10_1,
    /// 条目尾段 short ×3（→ 新记录 +0x60/+0x62/+0x64），语义未定性。
    pub tail_shorts: [u16; 3],
    /// 条目尾段 double ×2（→ +0x68/+0x70），原样保留。
    pub tail_doubles: [f64; 2],
    /// 条目尾段 int32 ×2（→ +0x7c/+0x80），语义未定性。
    pub tail_int32s: [u32; 2],
    /// 条目尾段嵌套对象（`DAT_6f7aa58c` → +0x94）。`None` 表示本读取器在
    /// 该对象处停止（见停止原因
    /// [`CompositeGraphicsLayerStop10_1::SubLayerTailNewObject`]），此时
    /// `tail_double` 亦为 `None`。
    pub tail_nested: Option<ConsumedEntry10_1>,
    /// 版本 >1 的尾随 double（→ +0x88）。`None` = 版本 <=1，或本条目在尾
    /// 段嵌套对象处停止。
    pub tail_double: Option<f64>,
}

/// 已确认前缀的停止原因；本读取器对任何成功输入都返回停止原因（见模块文档）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeGraphicsLayerStop10_1 {
    /// 元素组循环（第 2 步）中的新元素对象：其体没有外部长度字段，本读取
    /// 器无法消费。`group`/`index` 为组序号与组内序号（0 起）；游标回退到
    /// 该对象头起点。
    ElementNewObject {
        group: usize,
        index: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 第 7 步嵌套对象（`DAT_6f7aa58c`；样本为 ElementCollection）是新对象：
    /// 停止并回退到其头起点。
    NestedNewObject { class_id: [u8; 16], key: u32 },
    /// 子图层条目的 ILayer 新对象：停止并回退到其头起点；`name` 为已消费的
    /// 条目字符串（该条目不进入 [`CompositeGraphicsLayerPrefix10_1::sublayers`]）。
    SubLayerNewObject {
        index: usize,
        name: String,
        class_id: [u8; 16],
        key: u32,
    },
    /// 子图层条目尾段嵌套对象（`DAT_6f7aa58c`）是新对象：该条目以部分消费
    /// 状态记录（`tail_nested`/`tail_double` 为 `None`），游标回退到其头起点。
    SubLayerTailNewObject {
        index: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 子图层循环之后的尾段（第 10 条）未逐项展开：反编译可见其后的 int32、
    /// ILayer 族嵌套对象与版本分支 String/double，本读取器不消费。
    TailNotExpanded,
}

impl CompositeGraphicsLayerStop10_1 {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(&self) -> String {
        match self {
            Self::ElementNewObject { group, index, .. } => format!(
                "元素组 {group} 的元素条目 {index} 为新对象：其体没有外部长度字段，\
                 无法消费，游标回退到该对象头起点"
            ),
            Self::NestedNewObject { .. } => {
                "第 7 步嵌套对象（DAT_6f7aa58c；样本为 ElementCollection）为新对象：\
                 停止并回退到其头起点"
                    .to_string()
            }
            Self::SubLayerNewObject { index, .. } => format!(
                "子图层条目 {index} 的 ILayer 为新对象：停止并回退到其头起点（该条目\
                 的名称字符串已消费但不入列表）"
            ),
            Self::SubLayerTailNewObject { index, .. } => format!(
                "子图层条目 {index} 的尾段嵌套对象为新对象：条目以部分消费状态记录，\
                 游标回退到其头起点"
            ),
            Self::TailNotExpanded => {
                "子图层循环之后的尾段未逐项展开：本读取器不消费，保守停止".to_string()
            }
        }
    }
}

/// 10.1 CompositeGraphicsLayer::Load（`0x6f6b2100`）已确认前缀的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct CompositeGraphicsLayerPrefix10_1 {
    /// 2 字节版本 short 的无符号位模式，>6 一律拒绝；样本值 6。
    pub version: u16,
    /// 元素组计数的原始位模式；原生按有符号比较，<=0 不迭代。
    pub group_count: u32,
    /// 已成功处理的元素组；停止组不记录。
    pub element_groups: Vec<ElementGroup10_1>,
    /// 名称（→ obj+0x20；样本 `<默认>`）。
    pub name: String,
    /// 三个 short（→ +0x34/+0x36/+0x38），语义未定性、原样保留；样本
    /// `0xffff/0xffff/0x0000`。
    pub shorts: [u16; 3],
    /// 两个 double（→ +0x24/+0x2c），原样保留；样本 `0.0/0.0`。
    pub doubles: [f64; 2],
    /// 第 6 步三个 int32 之一（→ +0x4c）；样本 3。语义未定性。
    pub int32_4c: u32,
    /// 第 6 步三个 int32 之一（→ +0x50）；样本 1。语义未定性。
    pub int32_50: u32,
    /// 第 6 步第三个 int32：反编译读入局部变量后未见消费；样本 1。
    pub int32_local: u32,
    /// 第 7 步嵌套对象（`DAT_6f7aa58c` → +0x1c）分类。`None` 表示该对象是
    /// 新对象（停止原因
    /// [`CompositeGraphicsLayerStop10_1::NestedNewObject`]）。
    pub nested_1c: Option<ConsumedEntry10_1>,
    /// 版本 >1 的 double（→ +0x54）与 int32（局部变量）。
    pub after_version1: Option<(f64, u32)>,
    /// 子图层计数的原始位模式；原生按有符号比较，<=0 不迭代。样本中存在
    /// 两种读法（模块文档「样本核对」），本读取器按反编译字段序消费、不裁
    /// 决。停止发生在读取该计数之前时保持 0（默认值）。
    pub sublayer_count: u32,
    /// 已成功处理的子图层条目。
    pub sublayers: Vec<SubLayerEntry10_1>,
    /// 停止原因；成功输入也总有停止点（新对象 / 尾段）。
    pub stop: CompositeGraphicsLayerStop10_1,
    /// 停止位置之后的剩余字节数（相对传入 Reader 窗口）。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 10.1 CompositeGraphicsLayer 对象体的已确认前缀。
///
/// 起点约定：调用者已用 [`crate::object_stream::read_header_10_1`] 读取对象
/// 头、比对 [`COMPOSITE_GRAPHICS_LAYER_CLASS_ID`] 确认类身份，并从对象头之
/// 后的位置传入 Reader；本函数不重复读取对象头。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额，应用于图形层名称与每个子图层条目名称的声明长
/// 度（含终止零、不含 4 字节前缀），在任何读取与分配之前检查；
/// `usize::MAX` 表示不额外设限。`table` 是跨流共享的键登记表：新对象键在
/// 登记后若本读取器无法消费其体，则停止时不保留该登记（见各停止原因）。
///
/// 事务性：全部字段在 trial 游标与 trial 登记表上读取，成功返回才提交；
/// 任何失败都不推进调用者游标、不修改登记表。触发停止的返回是成功：游标
/// 推进到停止点（新对象停止时回退到该对象头起点，调用者可把剩余字节交给
/// 对应的对象体读取器），已消费条目登记进调用者登记表。错误位置相对于传
/// 入 Reader，调用方需另外记录流和对象身份。这是严格安全子集，不代表完
/// 整原生 Load 行为，通过也不代表 MXD 格式已验证。
pub fn read_composite_graphics_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<CompositeGraphicsLayerPrefix10_1> {
    let entry_offset = reader.position();
    // 入口克隆：Reader 没有随机定位原语，停止回退都用「入口克隆 + 前向读
    // 取」把游标重定位到窗口内绝对偏移；所有目标都在入口之后。
    let origin = reader.clone();
    let limit = entry_offset + origin.remaining();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("CompositeGraphicsLayer 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };
    // 全部字段在 trial 游标与 trial 登记表上读取，成功才提交；任何失败路径
    // 都不推进原游标、不改原登记表。
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let mut stop: Option<CompositeGraphicsLayerStop10_1> = None;

    // 1. 2 字节版本：无符号解释，>6 拒绝（原生 ver < 7 有符号比较）。
    let version = trial.read_u16_le()?;
    if version > MAX_ARCHITECTURE_VERSION {
        return Err(invalid(format!(
            "架构版本 {version} 超出 CompositeGraphicsLayer::Load 接受范围（>6 拒绝）"
        )));
    }

    // 2. 元素组计数 + 每组 [元素计数 + 每元素一个嵌套对象]。
    let group_count = trial.read_u32_le()?;
    let mut element_groups = Vec::new();
    if (group_count as i32) > 0 {
        'groups: for group in 0..group_count as i32 as usize {
            let declared_count = trial.read_u32_le()?;
            let mut elements = Vec::new();
            if (declared_count as i32) > 0 {
                for index in 0..declared_count as i32 as usize {
                    let header_offset = trial.position();
                    let saved_table = trial_table.clone();
                    let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
                    match entry {
                        ObjectEntry10_1::Null => elements.push(ConsumedEntry10_1::Null),
                        ObjectEntry10_1::BackReference { class_id, key } => {
                            elements.push(ConsumedEntry10_1::BackReference { class_id, key });
                        }
                        ObjectEntry10_1::NewObject { class_id, key } => {
                            // 新元素体没有外部长度字段，本读取器无法消费：回退
                            // 登记与游标到该对象头起点，在此停止，不报错。
                            trial_table = saved_table;
                            trial = reposition(&origin, entry_offset, header_offset, limit)
                                .expect("回退目标在停止前的已读位置，必在窗口内");
                            stop = Some(CompositeGraphicsLayerStop10_1::ElementNewObject {
                                group,
                                index,
                                class_id,
                                key,
                            });
                            break 'groups;
                        }
                    }
                }
            }
            element_groups.push(ElementGroup10_1 {
                declared_count,
                elements,
            });
        }
    }

    // 元素循环若已停止，以下字段全部不属于已确认前缀（游标已回退到停止点），
    // 一律保持默认值。
    let mut name = String::new();
    let mut shorts = [0_u16; 3];
    let mut doubles = [0.0; 2];
    let mut int32_4c = 0;
    let mut int32_50 = 0;
    let mut int32_local = 0;
    let mut nested_1c = None;
    let mut after_version1 = None;
    let mut sublayer_count = 0;
    let mut sublayers = Vec::new();
    if stop.is_none() {
        // 3. 名称：AfCore String 布局，受 max_bytes 限额约束。
        name = read_string_10_1(&mut trial, max_bytes)?;

        // 4. 三个 short：语义未定性，原样保留。
        for slot in &mut shorts {
            *slot = trial.read_u16_le()?;
        }

        // 5. 两个 double：原样保留。
        for slot in &mut doubles {
            *slot = trial.read_f64_le()?;
        }

        // 6. 连续三个 int32（文档节摘要记为一个，反编译与样本均为三个）。
        int32_4c = trial.read_u32_le()?;
        int32_50 = trial.read_u32_le()?;
        int32_local = trial.read_u32_le()?;

        // 7. 嵌套对象（DAT_6f7aa58c → +0x1c；样本为 ElementCollection）。
        let header_offset = trial.position();
        let saved_table = trial_table.clone();
        let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
        match entry {
            ObjectEntry10_1::Null => nested_1c = Some(ConsumedEntry10_1::Null),
            ObjectEntry10_1::BackReference { class_id, key } => {
                nested_1c = Some(ConsumedEntry10_1::BackReference { class_id, key });
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                trial_table = saved_table;
                trial = reposition(&origin, entry_offset, header_offset, limit)
                    .expect("回退目标在停止前的已读位置，必在窗口内");
                stop = Some(CompositeGraphicsLayerStop10_1::NestedNewObject { class_id, key });
            }
        }

        // 8. 版本 >1：double（→ +0x54）+ int32（局部变量）。
        if stop.is_none() && version > 1 {
            let double = trial.read_f64_le()?;
            let int32 = trial.read_u32_le()?;
            after_version1 = Some((double, int32));
        }

        // 9. [反编译此处调用无参数的 FUN_6f6b3100，按不消费流处理（见模块
        //    文档）] 子图层计数 + 每项 [String + ILayer + 条目尾段]。
        if stop.is_none() {
            sublayer_count = trial.read_u32_le()?;
            if (sublayer_count as i32) > 0 {
                'sublayers: for index in 0..sublayer_count as i32 as usize {
                    let item_name = read_string_10_1(&mut trial, max_bytes)?;
                    // 条目的 ILayer 对象。
                    let layer = {
                        let header_offset = trial.position();
                        let saved_table = trial_table.clone();
                        let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
                        match entry {
                            ObjectEntry10_1::Null => ConsumedEntry10_1::Null,
                            ObjectEntry10_1::BackReference { class_id, key } => {
                                ConsumedEntry10_1::BackReference { class_id, key }
                            }
                            ObjectEntry10_1::NewObject { class_id, key } => {
                                // 新对象体不可消费：回退并停止；已消费的条目字
                                // 符串保存在停止原因里，不丢弃。
                                trial_table = saved_table;
                                trial = reposition(&origin, entry_offset, header_offset, limit)
                                    .expect("回退目标在停止前的已读位置，必在窗口内");
                                stop = Some(CompositeGraphicsLayerStop10_1::SubLayerNewObject {
                                    index,
                                    name: item_name,
                                    class_id,
                                    key,
                                });
                                break 'sublayers;
                            }
                        }
                    };
                    // 条目尾段 short ×3 + double ×2 + int32 ×2。
                    let mut tail_shorts = [0_u16; 3];
                    for slot in &mut tail_shorts {
                        *slot = trial.read_u16_le()?;
                    }
                    let mut tail_doubles = [0.0; 2];
                    for slot in &mut tail_doubles {
                        *slot = trial.read_f64_le()?;
                    }
                    let mut tail_int32s = [0_u32; 2];
                    for slot in &mut tail_int32s {
                        *slot = trial.read_u32_le()?;
                    }
                    // 条目尾段嵌套对象（DAT_6f7aa58c → +0x94）。
                    let tail_header_offset = trial.position();
                    let saved_table = trial_table.clone();
                    let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
                    let (tail_nested, tail_double) = match entry {
                        ObjectEntry10_1::Null => {
                            let tail_double = read_tail_double(&mut trial, version)?;
                            (Some(ConsumedEntry10_1::Null), tail_double)
                        }
                        ObjectEntry10_1::BackReference { class_id, key } => {
                            let tail_double = read_tail_double(&mut trial, version)?;
                            (
                                Some(ConsumedEntry10_1::BackReference { class_id, key }),
                                tail_double,
                            )
                        }
                        ObjectEntry10_1::NewObject { class_id, key } => {
                            // 新对象体不可消费：条目以部分消费状态记录，回退并
                            // 停止。
                            trial_table = saved_table;
                            trial = reposition(&origin, entry_offset, tail_header_offset, limit)
                                .expect("回退目标在停止前的已读位置，必在窗口内");
                            stop = Some(CompositeGraphicsLayerStop10_1::SubLayerTailNewObject {
                                index,
                                class_id,
                                key,
                            });
                            sublayers.push(SubLayerEntry10_1 {
                                name: item_name,
                                layer,
                                tail_shorts,
                                tail_doubles,
                                tail_int32s,
                                tail_nested: None,
                                tail_double: None,
                            });
                            break 'sublayers;
                        }
                    };
                    sublayers.push(SubLayerEntry10_1 {
                        name: item_name,
                        layer,
                        tail_shorts,
                        tail_doubles,
                        tail_int32s,
                        tail_nested,
                        tail_double,
                    });
                }
            }
        }
    }

    // 10. 尾段未逐项展开：默认停止点（若此前未因新对象停止）。
    let stop = stop.unwrap_or(CompositeGraphicsLayerStop10_1::TailNotExpanded);

    let result = CompositeGraphicsLayerPrefix10_1 {
        version,
        group_count,
        element_groups,
        name,
        shorts,
        doubles,
        int32_4c,
        int32_50,
        int32_local,
        nested_1c,
        after_version1,
        sublayer_count,
        sublayers,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 读取子图层条目版本 >1 的尾随 double（→ 新记录 +0x88）。
fn read_tail_double(trial: &mut Reader<'_>, version: u16) -> io::Result<Option<f64>> {
    if version > 1 {
        Ok(Some(trial.read_f64_le()?))
    } else {
        Ok(None)
    }
}

/// 把游标重定位到「传入 Reader 窗口」的绝对偏移 `target`。
///
/// Reader 没有随机定位原语；`origin` 是函数入口时的游标克隆（位置 =
/// `entry_offset`），这里用入口克隆前向读取实现重定位。`target` 必须在
/// `entry_offset` 与 `limit`（窗口末尾）之间，越界返回错误。
fn reposition<'a>(
    origin: &Reader<'a>,
    entry_offset: usize,
    target: usize,
    limit: usize,
) -> io::Result<Reader<'a>> {
    if !(entry_offset..=limit).contains(&target) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("重定位目标 0x{target:x} 超出输入窗口 [0x{entry_offset:x}, 0x{limit:x}]"),
        ));
    }
    let mut positioned = origin.clone();
    // 长度已由上界校验限制在窗口内，前向读取不会失败。
    positioned
        .read_bytes(target - entry_offset)
        .expect("目标位置已校验在窗口内");
    Ok(positioned)
}
