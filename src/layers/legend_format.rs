//! ArcMap 10.1 LegendClassFormat（图例类格式）对象体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-legend-tails.md`（2026-11-15 本轮研究记录）。
//! 二进制证据为 Layer.dll 10.1.0.3035（temp/agent-fl-tail/ 既有基线，SHA-256
//! 与第六轮记录一致）的静态反编译：GetClassID `0x78664070` 写出 CLSID
//! `{7A3F91E6-B9E3-11D1-8756-0000F8751720}`（与 esri.clsid 声明的模块
//! Layer.dll、类名 LegendClassFormat 三方互证）；IPersistStream 虚表
//! `0x7866f8ac`（COL `0x78673440`，TD `0x7867a180` 名
//! `.?AVLegendClassFormat@@`）：+0x0c=GetClassID、+0x14=Load `0x78664b20`、
//! +0x18=Save `0x78664c40`，槽序与 IPersistStream 一致。
//!
//! ## 接入点（背景，本轮未接线）
//!
//! LegendClass::Load 第 ⑤ 步为 `LoadObject`（请求 IID
//! `{52AA5BC2-CB21-11D0-87F9-080009EC732A}` = ILegendClassFormat，第六轮已
//! 闭合，`docs/arcmap-10.1-featurelayer-tail.md`）：仓库参考样本该槽为 16
//! 字节空标记，本类对象体在样本中未出现。此外 LayerLib.dll
//! BaseLegendItem::Load（本轮闭合，见 [`crate::layers::legend_item`]）第 ⑩
//! 步同样 `LoadObject` ILegendClassFormat——两个接入点都在新对象处停止，
//! 接线属后续共享文件改动（[`crate::layers::feature_layer_tail`] 与
//! [`crate::maps::map_surround`] 归其他轮次，本轮不动）。
//!
//! ## 字段序（Load `0x78664b20` / Save `0x78664c40`，逐字段对称）
//!
//! 1. int16 版本：有符号 `2 < v` 拒绝 `0x80004005`（汇编 `CMP/JG`；负值
//!    位模式原生通过，本读取器一并拒绝，较原生更严）。Save 恒写静态常量
//!    2（`DAT_7866d968` = `02 00`）；
//! 2. LoadObject（请求 IID `{A80B5E91-7F9C-11D0-9410-080009EEBECB}` =
//!    ITextSymbol，类型库 esriDisplay）→ obj+0x08：标签符号候选；
//! 3. LoadObject（同上 IID）→ obj+0x0c：描述符号候选；
//! 4. LoadObject（`{BEB4EDB7-E3B4-11D2-B868-00600802E603}` = ILinePatch，
//!    类型库 esriCarto）→ obj+0x10；
//! 5. LoadObject（`{BEB4EDB6-E3B4-11D2-B868-00600802E603}` = IAreaPatch）
//!    → obj+0x14；
//! 6. 版本 >1：double ×2 → obj+0x1c/obj+0x24（语义未定，原样保留位
//!    模式）。Save 侧这两个 double **恒写**（无版本门）——与读侧版本 >1
//!    门在 10.1 一致（Save 恒写版本 2），旧文档（版本 1）读写不对称。
//!
//! ## 样本边界
//!
//! 仓库参考样本 `samples/测试底图.mxd` 的 /Maps 流与本文件提取副本中，
//! 本类 CLSID 均 0 命中（样本 LegendClass 的 ILegendClassFormat 槽为空
//! 标记，本轮已复核）；全部布局结论**无样本字节佐证**，仅有反编译一手
//! 证据，单元测试全部使用合成输入。
//!
//! 调用约定：与 [`crate::objects::dataset_name`] 一致——调用者先用
//! [`crate::storage::object_stream::read_header_10_1`] 读取对象头、比对
//! [`LEGEND_CLASS_FORMAT_CLASS_ID`] 并取得键，从对象头之后的位置传入
//! Reader；本模块不重复读取对象头。四个对象槽经
//! [`crate::storage::object_refs::read_object_entry_10_1`] 分类：空标记与
//! 回引消费后继续；**新对象**（ITextSymbol/ILinePatch/IAreaPatch 实现类
//! 体未闭合）为停止点——游标回退到该槽位起点、该键不登记，其后的槽位
//! 不可达。任何失败不推进调用者游标、不修改登记表。`max_bytes` 与本轮
//! 其他读取器同口径（`usize::MAX` 表示不额外设限）。
//!
//! 未解决问题（详见研究记录「开放项」）：两个 double 的业务语义未定；
//! 四个 String/符号槽的实现类分布未观察（样本无实例）；版本 0/1 分支无
//! 字节佐证（Save 恒写 2）；全部结论为静态观察，未经目标软件运行验证。

use crate::layers::patch::{
    AREA_PATCH_CLASS_ID, LINE_PATCH_CLASS_ID, PatchBody10_1, PatchStop10_1, read_patch_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// LegendClassFormat 类标识（CLSID）的 16 字节内存序（Data1-3 小端、
/// Data4 原序）。
///
/// GUID 字符串形式为 `{7A3F91E6-B9E3-11D1-8756-0000F8751720}`。证据：
/// esri.clsid.ecfg 声明（模块 Layer.dll，类名 LegendClassFormat）；
/// Layer.dll GetClassID `0x78664070` 写出同一 CLSID（IPersistStream 虚表
/// `0x7866f8ac` +0x0c）；RTTI TD `0x7867a180` 名 `.?AVLegendClassFormat@@`
/// 经 COL `0x78673440` 与该虚表相连。仓库参考样本 /Maps 流 0 命中。
pub const LEGEND_CLASS_FORMAT_CLASS_ID: [u8; 16] = [
    0xe6, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// LegendClassFormat::Load 接受的最大版本：汇编有符号 `2 < v` 拒绝
/// （0x80004005）；Save 恒写 2。
const MAX_BODY_VERSION: i16 = 2;

/// ITextSymbol / ILinePatch / IAreaPatch 槽位的分类结果。
///
/// C 轮接线起补丁两槽（ILinePatch/IAreaPatch）的新对象命中 LinePatch/
/// AreaPatch 类时整体消费其共享闭合体（layers::patch 模块）；TextSymbol
/// 与其余实现类维持停止点（游标回退到槽位起点）。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendClassFormatSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 LinePatch 对象（ILinePatch 槽；C 轮接线）：已读取其共享闭合体。
    LinePatch(Box<PatchBody10_1>),
    /// 新 AreaPatch 对象（IAreaPatch 槽；C 轮接线）：已读取其共享闭合体。
    AreaPatch(Box<PatchBody10_1>),
    /// 新对象但类标识非本读取器可闭合类：体未知且无外部长度，停止点，
    /// 游标回退到该对象头起点、该键不登记。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// LegendClassFormat 体读取的停止原因（体内在首个不可闭合处停止，非错误）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegendClassFormatStop10_1 {
    /// 第 `index` 个对象槽（0 起：0/1=ITextSymbol、2=ILinePatch、
    /// 3=IAreaPatch）出现新对象且类体未闭合；游标已回退到该槽位起点。
    SlotUnsupported {
        /// 槽位序号（0 起）。
        index: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 补丁槽（ILinePatch/IAreaPatch）的闭合体自身停止（几何槽未知类，
    /// 原因透传）；游标停在其内层停止点。
    PatchBody(PatchStop10_1),
}

/// LegendClassFormat（Layer.dll Load `0x78664b20`）对象的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct LegendClassFormatBody10_1 {
    /// 对象头键（由调用者在分类新对象时按「登记先于持久化」登记；本
    /// 读取器不重复登记，仅登记嵌套槽位的新键）。
    pub key: u32,
    /// 体首 int16 版本的无符号位模式。原生有符号比较 >2 拒绝（汇编
    /// 证实）；Save 恒写 2。样本无实例，接受域内位模式均可能。
    pub version: u16,
    /// 第 2 步 ITextSymbol 槽（→ obj+0x08，标签符号候选）。
    pub label_symbol: LegendClassFormatSlot10_1,
    /// 第 3 步 ITextSymbol 槽（→ obj+0x0c，描述符号候选）。
    pub description_symbol: LegendClassFormatSlot10_1,
    /// 第 4 步 ILinePatch 槽（→ obj+0x10）。
    pub line_patch: LegendClassFormatSlot10_1,
    /// 第 5 步 IAreaPatch 槽（→ obj+0x14）。
    pub area_patch: LegendClassFormatSlot10_1,
    /// 版本 >1 的两个 double 原样位模式（→ obj+0x1c/0x24，语义未定；
    /// Save 恒写、无版本门）。
    pub patch_doubles: Option<[f64; 2]>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<LegendClassFormatStop10_1>,
    /// 停止点之后剩余的字节数（停止是成功返回）。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 10.1 LegendClassFormat 对象体，失败时不推进
/// 游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`LEGEND_CLASS_FORMAT_CLASS_ID`] 并
/// 取得键 `key`。字段序与证据见模块文档。`max_bytes` 为资源限额（本轮
/// 字段无字符串；保留参数与同族读取器同口径，`usize::MAX` 不设限）。
/// 对象槽新对象为成功返回的停止点（游标回退到槽位起点）；截断、版本
/// 超限为错误，游标与登记表不动。
pub fn read_legend_class_format_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<LegendClassFormatBody10_1> {
    let _ = max_bytes; // 本轮字段无变长载荷；保留参数与同族读取器同口径
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("LegendClassFormat 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };

    // 1. 版本 short：原生有符号 `2 < v` 拒绝（负值位模式原生通过）。负值
    //    一并拒绝，较原生更严（与 dataset_name/feature_layer 读取器口径一致）。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_BODY_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "架构版本 {version} 超出 LegendClassFormat::Load 接受范围（有符号 >2 拒绝 \
             0x80004005；Save 恒写 2；负值位模式原生通过，此处一并拒绝，较原生更严）"
        )));
    }

    // 2..5. 四个对象槽（ITextSymbol ×2、ILinePatch、IAreaPatch）。全部在
    //    trial 游标与暂存登记表上读取，整体成功（含停止点）才提交；中途
    //    截断等错误向上传播，登记表保持不动。
    let mut staged_table = table.clone();
    let mut slots: [LegendClassFormatSlot10_1; 4] = [
        LegendClassFormatSlot10_1::Null,
        LegendClassFormatSlot10_1::Null,
        LegendClassFormatSlot10_1::Null,
        LegendClassFormatSlot10_1::Null,
    ];
    let mut stop = None;
    for index in 0..4usize {
        let slot_offset = trial.position();
        let snapshot = staged_table.clone();
        // C 轮接线：槽 2/3（ILinePatch/IAreaPatch）按类标识分派补丁共享
        // 读取器（体内停止透传）；其余类与 TextSymbol 两槽维持停止语义。
        enum PatchKind {
            Line,
            Area,
        }
        let expected_patch = match index {
            2 => Some((LINE_PATCH_CLASS_ID, PatchKind::Line)),
            3 => Some((AREA_PATCH_CLASS_ID, PatchKind::Area)),
            _ => None,
        };
        let mut patch_body = None;
        match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => slots[index] = LegendClassFormatSlot10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key } => {
                slots[index] = LegendClassFormatSlot10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key } if matches!(&expected_patch, Some((id, _)) if class_id == *id) =>
            {
                let body = read_patch_body_10_1(&mut trial, max_bytes, &mut staged_table, key)?;
                if let Some(inner) = body.stop {
                    stop = Some(LegendClassFormatStop10_1::PatchBody(inner));
                }
                patch_body = Some(body);
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                // 实现类体未闭合：停止并回退到槽位起点（回退停止不保留该
                // 键），其后槽位不可达。之前槽位已消费的键按原生语义保留。
                slots[index] = LegendClassFormatSlot10_1::Unsupported { class_id, key };
                stop = Some(LegendClassFormatStop10_1::SlotUnsupported {
                    index,
                    class_id,
                    key,
                });
                trial = reposition_to(reader, slot_offset);
                staged_table = snapshot;
                break;
            }
        }
        if let Some(body) = patch_body {
            slots[index] = match expected_patch.expect("补丁体必有期望类").1 {
                PatchKind::Line => LegendClassFormatSlot10_1::LinePatch(Box::new(body)),
                PatchKind::Area => LegendClassFormatSlot10_1::AreaPatch(Box::new(body)),
            };
        }
        if stop.is_some() {
            break;
        }
    }

    // 6. 版本 >1：double ×2（原样位模式；Save 恒写）。停止时不可达。
    let mut patch_doubles = None;
    if stop.is_none() && version > 1 {
        let mut pair = [0_f64; 2];
        for slot in &mut pair {
            *slot = trial.read_f64_le()?;
        }
        patch_doubles = Some(pair);
    }

    let result = LegendClassFormatBody10_1 {
        key,
        version,
        label_symbol: slots[0].clone(),
        description_symbol: slots[1].clone(),
        line_patch: slots[2].clone(),
        area_patch: slots[3].clone(),
        patch_doubles,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 把游标回退到入口窗口内的绝对偏移（Reader 无随机定位原语，用入口克隆
/// 前向读取实现；目标必在已读范围内）。
fn reposition_to<'a>(origin: &Reader<'a>, target: usize) -> Reader<'a> {
    let mut positioned = origin.clone();
    positioned
        .read_bytes(target - origin.position())
        .expect("回退目标在停止前的已读位置，必在窗口内");
    positioned
}
