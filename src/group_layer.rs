//! ArcMap 10.1 GroupLayer（图层组）对象体已确认前缀的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-map-object.md`「字段序闭合补充（2026-11-13
//! 第二轮）」的 GroupLayer::Load 完整字段表、「Map::Save 方向、ExtraSafe
//! 前缀与图层列表样本闭合（2026-11-14 第四轮）」的 ExtraSafeReadObject
//! 节、「图层树、可见性与数据源连接」节及各轮「未解决事项」。字段序与
//! 分支条件以 FeatureLayer.dll 10.1.0.3035 的 `GroupLayer::Load`
//! （`0x6f711270`，IPersistStream 虚表 `0x6f7af244` +0x14；RTTI
//! `.?AVGroupLayer@@`，TD `0x6f7d3458`）反编译为准（研究产物
//! `/tmp/r3/grouplayer_load.c`，见证据文档「证据与复现」节）；本模块实现
//! 时逐行复核了该反编译，与文档字段表不同的分支条件在下方逐处标注。
//!
//! 已确认字段序（编号 = 流内顺序）：
//!
//! 1. int16 版本：原生以有符号 short 比较 `0xc < ver` 拒绝（0x80004005），
//!    上限 12。反编译的符号性未在文档闭合，本读取器按无符号解释，负值位
//!    模式（0x8000..）一并拒绝，较原生更严（与 map_object、
//!    spatial_reference 的版本处理一致）；
//! 2. AfCore String 名称 → obj+0x9c（[`crate::native_string::read_string_10_1`]）；
//! 3. int16 ×3 → +0xa0/+0xa2/+0xa4（+0xa0 为可见性「候选」，语义未定，
//!    原样保留）；
//! 4. double ×2 → +0xa8/+0xb0；
//! 5. int32 子图层计数（原生按有符号比较，<=0 不迭代）+ 每项一条：
//!    版本 <11 为普通嵌套对象（16 字节类标识 + 4 字节键 + 体）；版本 >=11
//!    每项先读 8 字节 ExtraSafe 前缀 `[A:int32][B:int32]`（AfCore
//!    `ExtraSafeReadObject 0x68a7be60` 反编译）：`A==0 && B==0` 为空条目，
//!    否则仍是普通嵌套对象布局，B!=0 时原生 Seek 到「前缀后位置 + B」强
//!    同步。读侧反编译未见原生消费 A；写侧观察值 = 对象头 20 字节 + 体的
//!    总体积（`ExtraSafeWriteObject 0x68a7bd70`，/Maps 图层列表逐条吻合，
//!    第四轮），本读取器据此把 A 用作新对象体的有界跳过；
//! 6. int16 可见性 → +0xbc（VARIANT_BOOL，与 FeatureLayer +0x128 同构，
//!    「确定的」）：版本 <2 不读取（原生默认 0xffff=可见），>=2 读取；
//! 7. 版本 >2：嵌套对象（请求 IID {1B077914-38C5-11D0-92D2-00805F7C28B0}，
//!    第五轮类型库考证为 IEnvelope）。[文档字段表标注 >=2，反编译为
//!    `2 < ver`，以反编译为准]；
//! 8. 版本 >=4：double → +0xc8；
//! 9. 版本 >=5：`ExtensionHelper::LoadExtensions`（参数含流对象，流消费
//!    布局未展开——不可实现，本读取器在此停止）；
//! 10. 反编译可见、但本读取器不可达的字段：>=6 String→+0xdc、==6 再读
//!     short、>=7 short→+0xd6、>7 double×2→+0xe0/+0xe8 [文档字段表标注
//!     >=7，反编译为 `7 < ver`]、>=9 short→+0xd8 [文档字段表漏记]、>9
//!     short×3（local、+0xf2、+0xf4）、>=11 嵌套对象（IID
//!     {19F85377-1387-11D3-B89C-00600802E603}，第五轮考证为
//!     IDisplayFilter）、尾部 LayerMasking 条件循环。步骤 9 在版本 >=5
//!     时先于这些字段出现且必须停止，版本 <5 时它们的版本条件又不满足，
//!     因此对任何版本都不可达，不进入解析结果。
//!
//! 停止语义（与 [`crate::map_object`] 的设计差异及理由）：map_object 对
//! 未知嵌套类直接报错，因为其后没有同序字段可继续；本模块面对的嵌套对象
//! （子图层、IEnvelope）之后仍有已确认字段，而新对象体没有外部长度字段、
//! 无法跳过——报错会丢弃已解析前缀。因此本读取器是「已确认前缀」语义：
//! 解析到首个不可闭合字段（新对象体 / ExtensionHelper / 尾段）为止，返回
//! 已解析字段 + 剩余字节数 + 停止原因 [`GroupLayerStop10_1`]，不因未知嵌
//! 套类报错中断。ExtraSafe 条目例外：A 前缀给出有界体积，A>=20 时把对象
//! 头后 A-20 字节按未知体范围记录并继续；A 与 B 的同步位置冲突时以反编译
//! 证实的 B 同步为准。
//!
//! 调用约定：调用者先用 [`crate::object_stream::read_header_10_1`] 读取对象
//! 头并比对 [`GROUP_LAYER_CLASS_ID`]，从对象头之后的位置调用
//! [`read_group_layer_prefix_10_1`]；本模块不重复读取对象头。
//!
//! 样本对照：仓库参考样本 `samples/测试底图.mxd` 无组图层，本模块字段序只
//! 有反编译证据、无样本字节佐证（文档「未解决事项」2）；ExtraSafe 前缀的
//! `[A][B]` 布局与「A=对象总体积」有 /Maps 图层列表（Map 的图层循环，非
//! GroupLayer 子图层）的逐条目吻合（第四轮），仅作布局佐证。人工目视验
//! 收未做，不声称已验证。
//!
//! 未解决问题：三个 short（+0xa0/+0xa2/+0xa4）语义未定性；版本 short 的
//! 符号性未闭合（本读取器按无符号更严处理）；A 前缀仅是写侧观察，读侧原
//! 生不消费 A，A<20 的输入按本读取器报错（原生可能经 B 同步接受）；B 为
//! 负值位模式时的原生 Seek 行为未证实，本读取器直接拒绝；>=5 的
//! ExtensionHelper 与尾部 LayerMasking/`FUN_6f6c8100` 未展开（后者反编译
//! 无参数、按不消费流处理，未验证）。

use crate::native_string::read_string_10_1;
use crate::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::reader::Reader;
use std::io::{self, ErrorKind};

/// GroupLayer::Load 接受的最大架构版本：>12（0x0c）原生拒绝 0x80004005。
const MAX_ARCHITECTURE_VERSION: u16 = 12;

/// 嵌套对象头体积：16 字节类标识 + 4 字节键。
const OBJECT_HEADER_BYTES: u32 = 20;

/// GroupLayer 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{EDAD6647-1810-11D1-86AE-0000F8751720}`。证据：
/// 安装配置 `esri.clsid.ecfg` 声明该 CLSID → 类名 GroupLayer（模块
/// FeatureLayer.dll）；FeatureLayer.dll RTTI 存在 `.?AVGroupLayer@@`
/// （TD `0x6f7d3458`），其 IPersistStream::Load 即 `0x6f711270`（文档类清
/// 单「确定的」）。参考样本无组图层，无样本头佐证。
pub const GROUP_LAYER_CLASS_ID: [u8; 16] = [
    0x47, 0x66, 0xad, 0xed, 0x10, 0x18, 0xd1, 0x11, 0x86, 0xae, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// 已消费的嵌套对象条目（新对象是停止点，不出现在记录中）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumedEntry10_1 {
    /// 16 字节全零类标识的空引用：无键、无对象体。
    Null,
    /// 键已在此前登记：回引条目，其后没有对象体。
    BackReference { class_id: [u8; 16], key: u32 },
}

/// 子图层条目的分类记录（字段序第 5 步的循环）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupChildEntry10_1 {
    /// ExtraSafe 前缀 `[A=0][B=0]` 的空条目（仅版本 >=11 出现）：原生
    /// `ExtraSafeReadObject` 返回 1，调用方跳过。
    Empty,
    /// 16 字节全零类标识的空引用。
    Null,
    /// 回引条目：无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 版本 >=11 ExtraSafe 条目内的新对象：头已分类，头后 `A-20` 字节按未
    /// 知体范围跳过并记录（读侧原生不消费 A，A 的体积语义是写侧观察，见
    /// 模块文档）。
    SkippedNewObject {
        class_id: [u8; 16],
        key: u32,
        /// A 前缀的原始值（写侧观察 = 头 20 字节 + 体的总体积）。
        declared_total: u32,
        /// 未知体范围在传入 Reader 窗口内的起始偏移。
        body_offset: usize,
        /// 未知体范围的字节长度（= A - 20）。
        body_length: usize,
    },
}

/// 已确认前缀的停止原因；本读取器对任何成功输入都返回停止原因（见模块文档）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupLayerStop10_1 {
    /// 版本 <11 的子图层循环中出现新对象：其体没有外部长度字段，本读取器
    /// 无法消费。`index` 为条目序号（0 起）；游标回退到该对象头起点。
    ChildNewObject {
        index: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 版本 >2 的嵌套对象（IEnvelope 候选）是新对象：停止并回退到其头起点。
    NestedNewObject { class_id: [u8; 16], key: u32 },
    /// 版本 >=5 的 `ExtensionHelper::LoadExtensions` 未展开：其流消费布局
    /// 未知，无法继续；其后的版本分支字段因此全部不可达（模块文档第 10 条）。
    ExtensionHelperNotExpanded,
    /// 版本 <5 时全部版本条件字段之后的尾段（`FUN_6f6c8100` 与 LayerMasking
    /// 条件循环）未逐项展开：反编译未见其中的流读取，但未经验证，保守停止。
    TailNotExpanded,
}

impl GroupLayerStop10_1 {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(&self) -> String {
        match self {
            Self::ChildNewObject { index, .. } => format!(
                "子图层条目 {index} 为新对象：其体没有外部长度字段，无法消费，游标\
                 回退到该对象头起点"
            ),
            Self::NestedNewObject { .. } => {
                "版本 >2 的嵌套对象（IEnvelope 候选）为新对象：停止并回退到其头起点".to_string()
            }
            Self::ExtensionHelperNotExpanded => {
                "版本 >=5 的 ExtensionHelper::LoadExtensions 未展开：其后的版本分支\
                 字段全部不可达"
                    .to_string()
            }
            Self::TailNotExpanded => {
                "版本 <5 的尾段（LayerMasking 条件循环等）未逐项展开：保守停止".to_string()
            }
        }
    }
}

/// 10.1 GroupLayer::Load（`0x6f711270`）已确认前缀的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct GroupLayerPrefix10_1 {
    /// 2 字节版本 short 的无符号位模式，>12 一律拒绝。
    pub version: u16,
    /// 名称（→ obj+0x9c）。
    pub name: String,
    /// 名称后的三个 short（→ +0xa0/+0xa2/+0xa4），语义未定性、原样保留；
    /// +0xa0 是可见性「候选」。
    pub shorts_after_name: [u16; 3],
    /// 两个 double（→ +0xa8/+0xb0），原样保留。
    pub doubles: [f64; 2],
    /// 子图层计数的原始位模式；原生按有符号比较，<=0 不迭代。
    pub child_count: u32,
    /// 已成功分类的子图层条目；触发 [`GroupLayerStop10_1::ChildNewObject`]
    /// 停止的条目不记录。
    pub children: Vec<GroupChildEntry10_1>,
    /// 可见性（→ +0xbc，VARIANT_BOOL 位模式）。`None` = 版本 <2 未读取
    /// （原生默认 0xffff=可见），或在可见性之前已停止。
    pub visibility: Option<u16>,
    /// 版本 >2 的嵌套对象分类（请求 IID {1B077914-…}=IEnvelope 候选）。
    /// `None` = 版本条件未满足，或该对象是新对象（停止原因
    /// [`GroupLayerStop10_1::NestedNewObject`]）。
    pub nested_after_visibility: Option<ConsumedEntry10_1>,
    /// 版本 >=4 的 double（→ +0xc8）。`None` = 版本条件未满足或已停止。
    pub double_after_nested: Option<f64>,
    /// 停止原因；成功输入也总有停止点（新对象 / ExtensionHelper / 尾段）。
    pub stop: GroupLayerStop10_1,
    /// 停止位置之后的剩余字节数（相对传入 Reader 窗口）。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 10.1 GroupLayer 对象体的已确认前缀。
///
/// 起点约定：调用者已用 [`crate::object_stream::read_header_10_1`] 读取对象
/// 头、比对 [`GROUP_LAYER_CLASS_ID`] 确认类身份，并从对象头之后的位置传入
/// Reader；本函数不重复读取对象头。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额，应用于名称字符串的声明长度（含终止零、不含
/// 4 字节前缀），在任何读取与分配之前检查；`usize::MAX` 表示不额外设限。
/// `table` 是跨流共享的键登记表：新对象键在消费其体（或按 ExtraSafe 的
/// A 前缀跳过其体）前登记；回引判定只依赖登记表状态。
///
/// 事务性：全部字段在 trial 游标与 trial 登记表上读取，成功返回才提交；
/// 任何失败都不推进调用者游标、不修改登记表。触发停止的返回是成功：游标
/// 推进到停止点（新对象停止时回退到该对象头起点，调用者可把剩余字节交给
/// 对应的对象体读取器），已消费条目登记进调用者登记表。错误位置相对于传
/// 入 Reader，调用方需另外记录流和对象身份。这是严格安全子集，不代表完
/// 整原生 Load 行为，通过也不代表 MXD 格式已验证。
pub fn read_group_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<GroupLayerPrefix10_1> {
    let entry_offset = reader.position();
    // 入口克隆：Reader 没有随机定位原语，B 同步与停止回退都用「入口克隆 +
    // 前向读取」把游标重定位到窗口内绝对偏移；所有目标都在入口之后。
    let origin = reader.clone();
    let limit = entry_offset + origin.remaining();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("GroupLayer 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };
    // 全部字段在 trial 游标与 trial 登记表上读取，成功才提交；任何失败路径
    // 都不推进原游标、不改原登记表。
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. 2 字节版本：无符号解释，>12 拒绝（原生 0xc < ver 记 0x80004005）。
    let version = trial.read_u16_le()?;
    if version > MAX_ARCHITECTURE_VERSION {
        return Err(invalid(format!(
            "架构版本 {version} 超出 GroupLayer::Load 接受范围（>12 直接拒绝）"
        )));
    }

    // 2. 名称：AfCore String 布局，受 max_bytes 限额约束。
    let name = read_string_10_1(&mut trial, max_bytes)?;

    // 3. 三个 short：语义未定性，原样保留。
    let mut shorts_after_name = [0_u16; 3];
    for slot in &mut shorts_after_name {
        *slot = trial.read_u16_le()?;
    }

    // 4. 两个 double：原样保留。
    let mut doubles = [0.0; 2];
    for slot in &mut doubles {
        *slot = trial.read_f64_le()?;
    }

    // 5. 子图层计数 + 循环。原生按有符号比较，<=0 不迭代；计数位模式原样保留。
    let child_count = trial.read_u32_le()?;
    let mut children = Vec::new();
    let mut stop: Option<GroupLayerStop10_1> = None;
    if (child_count as i32) > 0 {
        'children: for index in 0..child_count as i32 as usize {
            if version >= 11 {
                // 版本 >=11：ExtraSafeReadObject 条目，先读 8 字节 [A][B] 前缀。
                let prefix_offset = trial.position();
                let a = trial.read_u32_le()?;
                let b = trial.read_u32_le()?;
                if a == 0 && b == 0 {
                    // 空条目：原生返回 1、调用方跳过，无对象头。
                    children.push(GroupChildEntry10_1::Empty);
                    continue;
                }
                let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
                match entry {
                    ObjectEntry10_1::Null => {
                        children.push(GroupChildEntry10_1::Null);
                    }
                    ObjectEntry10_1::BackReference { class_id, key } => {
                        children.push(GroupChildEntry10_1::BackReference { class_id, key });
                    }
                    ObjectEntry10_1::NewObject { class_id, key } => {
                        // A 的体积语义是写侧观察（读侧原生不消费 A）：A>=20 时
                        // 头后 A-20 字节按未知体范围记录并继续；A<20 与写侧不
                        // 变量矛盾，按本读取器报错。
                        if a < OBJECT_HEADER_BYTES {
                            return Err(invalid(format!(
                                "ExtraSafe 子图层条目 offset 0x{prefix_offset:x}: 前缀 A={a} \
                                 小于对象头 {OBJECT_HEADER_BYTES} 字节，无法用作有界跳过"
                            )));
                        }
                        let body_length = (a - OBJECT_HEADER_BYTES) as usize;
                        let body_offset = trial.position();
                        // 未知体范围必须完整落在输入内（否则按缺短读失败）。
                        trial.read_bytes(body_length)?;
                        children.push(GroupChildEntry10_1::SkippedNewObject {
                            class_id,
                            key,
                            declared_total: a,
                            body_offset,
                            body_length,
                        });
                    }
                }
                if b != 0 {
                    // B!=0 强同步：原生 Seek 到「前缀后位置 + B」。与 A 跳过的
                    // 终点冲突时以 B 同步为准（反编译证据）；负值位模式的回退
                    // Seek 行为未证实，直接拒绝。
                    if (b as i32) < 0 {
                        return Err(invalid(format!(
                            "ExtraSafe 子图层条目 offset 0x{prefix_offset:x}: 前缀 B={b} 为\
                             负值位模式，原生行为未证实"
                        )));
                    }
                    let target = prefix_offset
                        .checked_add(8)
                        .and_then(|position| position.checked_add(b as usize))
                        .ok_or_else(|| {
                            invalid(format!(
                                "ExtraSafe 子图层条目 offset 0x{prefix_offset:x}: 前缀 B={b} \
                                 同步目标溢出"
                            ))
                        })?;
                    trial = reposition(&origin, entry_offset, target, limit).map_err(|_| {
                        invalid(format!(
                            "ExtraSafe 子图层条目 offset 0x{prefix_offset:x}: 前缀 B={b} 同步\
                             目标 0x{target:x} 超出输入末尾"
                        ))
                    })?;
                }
            } else {
                // 版本 <11：普通 LoadObject 条目，无前缀。
                let item_offset = trial.position();
                let saved_table = trial_table.clone();
                let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
                match entry {
                    ObjectEntry10_1::Null => {
                        children.push(GroupChildEntry10_1::Null);
                    }
                    ObjectEntry10_1::BackReference { class_id, key } => {
                        children.push(GroupChildEntry10_1::BackReference { class_id, key });
                    }
                    ObjectEntry10_1::NewObject { class_id, key } => {
                        // 新对象体没有外部长度字段，本读取器无法消费：回退登记
                        // 与游标到该对象头起点，在此停止，不报错（见模块文档的
                        // 停止语义）。
                        trial_table = saved_table;
                        trial = reposition(&origin, entry_offset, item_offset, limit)
                            .expect("回退目标在停止前的已读位置，必在窗口内");
                        stop = Some(GroupLayerStop10_1::ChildNewObject {
                            index,
                            class_id,
                            key,
                        });
                        break 'children;
                    }
                }
            }
        }
    }

    // 6. 可见性（→ +0xbc）：版本 <2 不读（原生默认 0xffff），>=2 读取。
    let mut visibility = None;
    if stop.is_none() && version >= 2 {
        visibility = Some(trial.read_u16_le()?);
    }

    // 7. 版本 >2（反编译 `2 < ver`，文档表格标注 >=2，以反编译为准）：
    //    嵌套对象（IEnvelope 候选）。
    let mut nested_after_visibility = None;
    if stop.is_none() && version > 2 {
        let header_offset = trial.position();
        let saved_table = trial_table.clone();
        let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
        match entry {
            ObjectEntry10_1::Null => {
                nested_after_visibility = Some(ConsumedEntry10_1::Null);
            }
            ObjectEntry10_1::BackReference { class_id, key } => {
                nested_after_visibility = Some(ConsumedEntry10_1::BackReference { class_id, key });
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                trial_table = saved_table;
                trial = reposition(&origin, entry_offset, header_offset, limit)
                    .expect("回退目标在停止前的已读位置，必在窗口内");
                stop = Some(GroupLayerStop10_1::NestedNewObject { class_id, key });
            }
        }
    }

    // 8. 版本 >=4：double（→ +0xc8）。
    let mut double_after_nested = None;
    if stop.is_none() && version >= 4 {
        double_after_nested = Some(trial.read_f64_le()?);
    }

    // 9/10. 停止点：版本 >=5 是未展开的 ExtensionHelper；否则（版本 <5）全部
    // 版本条件字段之后是未展开的尾段（反编译未见其中的流读取，保守停止）。
    let stop = match stop {
        Some(stop) => stop,
        None if version >= 5 => GroupLayerStop10_1::ExtensionHelperNotExpanded,
        None => GroupLayerStop10_1::TailNotExpanded,
    };

    let result = GroupLayerPrefix10_1 {
        version,
        name,
        shorts_after_name,
        doubles,
        child_count,
        children,
        visibility,
        nested_after_visibility,
        double_after_nested,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
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
