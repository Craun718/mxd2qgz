//! ArcMap 10.1 ElementCollection 元素类（IElement coclass）对象体的严格读取
//! 子集与类分派。
//!
//! 证据来源：`docs/arcmap-10.1-element-bodies.md`（本轮研究记录）。三个元素
//! 类的宿主经「esri.clsid 声明模块 → DLL 内 RTTI TypeDescriptor → COL → 虚
//! 表 → GetClassID 写出」链在 10.1 二进制中印证：
//!
//! - **TextElement** `{204034D3-F6EA-11D0-83AD-080009B996CC}`：esri.clsid 声
//!   明 CartoXProvider.dll（本轮提取，10.1.0.3035，SHA-256 `1c4a6d36…6f3cf5`，
//!   ImageBase `0x6ce00000`）。GetClassID `0x6ce70e10` 写出该 CLSID；
//!   IPersistStream 虚表 `0x6cee6f40`（GetClassID 数据引用 `0x6cee6f4c`，
//!   +0x14 = Load `0x6ce70e50`、+0x18 = Save `0x6ce711c0`）。
//! - **MarkerElement** `{530FD712-EF0C-11D0-83A0-080009B996CC}` 与
//!   **LineElement** `{8AB7FBE1-D871-11D0-8389-080009B996CC}`：esri.clsid 声
//!   明 GraphicElements.dll（本轮提取，10.1.0.3035，SHA-256
//!   `f24451c8…f77ae9`，ImageBase `0x75030000`）。MarkerElement GetClassID
//!   `0x7504ea20`（CLSID 以立即数写出），IPersistStream 虚表 `0x750715d0`
//!   （+0x0c 数据引用 `0x750715dc`，Load `0x7504ea50`、Save `0x7504eb70`）；
//!   LineElement GetClassID `0x7504a090`（写常量 `0x75071088`），虚表
//!   `0x750712d8`（Load `0x7504a100`、Save `0x7504a230`）。两者共享基类实现
//!   （QI/AddRef/Release/IsDirty/GetSizeMax 同函数），Load/Save 各自覆写。
//! - **ElementProperties 共同前缀**：所有元素类 Load 的第一步调用
//!   `?Load@ElementProperties@@UAGJPAUIStream@@@Z`（LayerLib.dll `0x786b97f0`
//!   导出；Save `0x786b9940`）。TextElement 经 CartoXProvider IAT
//!   `0x6ced53f8` 导入；GraphicElements 同构。
//! - **TextSymbol** `{B65A3E74-2993-11D1-9A43-0080C7EC5C96}`（TextElement 符
//!   号槽的流内类）：esri.clsid 声明 DisplayCore.dll（本轮复制复核既有提取
//!   物，SHA-256 `9b3e61f9…6e4d2`，ImageBase `0x6eb30000`）；RTTI
//!   `.?AVTextSymbol@@` TD 基址 `0x6ec3e5e0`；GetClassID `0x6ebc9540`（读常
//!   量 `0x6ec167a8`）与 `0x6ebd08e0`（立即数写出）；Load =
//!   **`0x6ebd0a60`**（IColor 请求 IID 常量 `0x6ec0b4f4` 的引用函数族中，
//!   紧邻 GetClassID `0x6ebd08e0`，反编译字段序与样本逐字节互证）。
//!
//! ## ElementProperties::Load 前缀字段序（LayerLib `0x786b97f0`，确定的）
//!
//! 1. int16 版本：`6 < (short)ver` 拒绝 `0x80004005`（有符号；Save 版本表
//!    `DAT_7871ee10`，样本写 6）；负值位模式原生通过，本读取器一并拒绝（较
//!    原生更严）；
//! 2. String → this+8（名称候选，样本空串）；
//! 3. String → this+0xc（类型描述候选，样本 `"文本"`）；
//! 4. 版本 <2：int32 读后丢弃（旧版遗留段，仅原样位模式）；
//! 5. 版本 <4：LoadObject（请求 IID `{0000010C-…}` IPersist，常量
//!    `0x7871e78c`）→ 包裹为 Variant 存 this+0x14（CustomProperty 候选；仅
//!    分类，新对象 = 停止点）；版本 ≥4：类方法 `Variant::Read`（与
//!    [`crate::objects::property_set::read_variant_10_1`] 同一序列化器，样本
//!    为 VT_EMPTY）；
//! 6. 版本 >2：int16 → this+0x10（样本 `0xFFFF`）；
//! 7. 版本 >4：double → this+0x2c（参考比例候选）；
//! 8. 版本 >5：int32 → this+0x34（样本 6）。
//!
//! ## TextElement::Load `0x6ce70e50` 字段序（确定的）
//!
//! 1. int16 版本：`==9` 置「尾加块」标志后按 8 处理；`>8` 拒绝
//!    `0x80004005`（有符号）；负值一并拒绝（较原生更严）；
//! 2. ElementProperties 前缀；
//! 3. String → this+0x21（文本内容，样本 `"T35"`）；
//! 4. int32 → 布尔 this+0x22（样本 1，语义未定性）；
//! 5. LoadObject（IID `{A80B5E91-7F9C-11D0-9410-080009EEBECB}` = ITextSymbol）
//!    → this+0x1d：流内为 TextSymbol `{B65A3E74}` 新对象（样本 16/16），经类
//!    分派进入 [`read_text_symbol_body_10_1`]；
//! 6. LoadObject（IID `{1B07790D-38C5-11D0-92D2-00805F7C28B0}` = IGeometry）
//!    → 文本几何槽：仅分类，新对象 = 停止点（样本该槽为 Polyline
//!    `{30707210}` 新对象，体未闭合）；
//! 7. 版本 >2：int32 → this+0x2a；版本 >3：int32 → this+0x2b（Save 恒写
//!    -1 常量）；
//! 8. 版本 ==7 精确块：LoadObject（`{E4240307-…}` IBorder）+ LoadObject
//!    （`{E4240309-…}` IBackground）+ LoadObject（`{A9258348-1753-4B99-820A-
//!    FCF74429ABC1}` IShadow；第三轮接线起三槽经 element_decor 分派：
//!    SymbolBorder/SymbolBackground/SymbolShadow 整体解析，其余实现类 = 停
//!    止回退）+ double + int32 + double；
//! 9. 版本 ==9 尾加块：LoadObject（`{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`
//!    ISpatialReference；F 轮接线起三类闭合空间参考分派，其余实现类 = 停止
//!    回退）+ int32 + 32 字节原样。
//!
//! Save（`0x6ce711c0`）版本表 `DAT_6ced7ef8`：10.1 写 8——EP 前缀 + String +
//! int32 + SaveObject(符号) + SaveObject(几何) + int32 + int32 恒 -1，与
//! Load 版本 8 路径对称。
//!
//! ## TextSymbol::Load `0x6ebd0a60` 字段序（确定的；样本逐字节互证）
//!
//! 1. int16 版本：`4 < ver` 拒绝（样本 4）；
//! 2. LoadObject（IID `{20CD40B0-8F3D-11D0-8590-0800091A2A72}` = IColor）→
//!    this-0x480：颜色槽 #1（样本 RgbColor 键 1615，体经 feature_layer_tail
//!    的 RgbColor 读取器整体消费）；
//! 3. int32 → this-0x478（样本 -1）；int32 → this-0x470（样本 3）；int32 →
//!    this-0x474（样本 1）；
//! 4. byte → this-0x468；byte → this-0x467（样本 0、0）；
//! 5. double → this-0x45c / -0x44c / -0x444（样本 0.0 ×3）；
//! 6. int32 → this-0x4fc（样本 13）；int32 → this-0x4f4（样本 0）；
//! 7. LoadObject（IColor）→ 颜色槽 #2（样本 RgbColor 键 1616，全 0）；
//! 8. double → this-0x37c / -0x374（样本 0.0、0.0）；int32 → this-0x364 /
//!    -0x360（样本 0、0）；double → this-0x35c / -0x354 / -0x34c（样本
//!    0.0、100.0、100.0）；byte → this-0x344（样本 1）；double → this-0x33c
//!    （样本 0.0）；int32 → this-0x330（样本 0）；double → this-0x32c（样本
//!    88.0）；byte → this-0x320（样本 1）；
//! 9. ITextPath 的实现类 CLSID，由原生实例化，无对象头或键；
//! 10. LoadObject（IID `{8FEB6611-2A0D-11D1-9A44-0080C7EC5C96}` = ITextBackground）
//!     → Background getter 存储，解析已取证的四种背景对象；
//! 11. LoadObject（IID `{F3435802-5779-11D0-98BF-00805F7CED21}` = ISymbol）
//!     → 符号槽 #1（FillSymbol getter 底层存储）；
//! 12. String → this-0x238（样本 `"T35"`，与外层文本相同）；
//! 13. double → this-0x1ec（样本 8.0，读后作虚调用实参）；int32 →
//!     this-0x210（样本 1）；double → this-0x204（样本 1.5）；
//! 14. LoadObject（ISymbol）→ 符号槽 #2（仅分类；样本为 `{7914E603-C892-
//!     11D0-8BB6-080009EE4E41}` 新对象键 1593，体未闭合 = 停止点）；
//! 15. LoadObject（IID `{BEF6E002-A874-101A-8BBA-00AA00300CAB}` = IFontDisp）
//!     → 字体槽（本轮起 StdFont `{0BE35203-8F91-11CE-9DE3-00AA004BB851}`
//!     新对象整体解析——oleaut32 2.40.4275 OleFontImpl 持久化格式 + 样本 37
//!     处逐字节互证，证据 docs/arcmap-10.1-ole-stdfont.md；其余实现类维持
//!     停止回退）；
//! 16. 版本 ≥2：RotateSymbolsOnConvert 对应的 short；版本 ≥3：ITextParser
//!     实现类 CLSID（零 GUID 表示清除解析器）；版本 ≥4：CJKCharactersRotation
//!     的原始 short。见 docs/mxd-terra-round9-text.md。
//!
//! 第五轮「TextSymbol 体内键位恒 0 的内嵌 SimpleTextPath」现象在本轮重新归
//! 因：字段 9 的 16 字节是**原样 GUID 字段**（非对象头、无键），样本值为
//! TextSymbol 自身 CLSID；其后 32 个零字节是字段 10/11 两个空标记（16+16），
//! 不是「键 0 的对象头」。内嵌链消失，TextSymbol 体在 ISymbol 槽 #2 处以常
//! 规对象分派停止。
//!
//! ## MarkerElement::Load `0x7504ea50` / LineElement::Load `0x7504a100`
//! （确定的）
//!
//! 共同结构：int16 版本（`3 < (short)ver` 拒绝；负值本读取器一并拒绝）→
//! ElementProperties 前缀 → LoadObject（符号槽）→ LoadObject（几何槽）→ 版
//! 本门尾段。差异：
//!
//! | | 符号槽请求 IID | 几何槽请求 IID | >1 | >2 |
//! | --- | --- | --- | --- | --- |
//! | MarkerElement | `{E6BDAA7C-4D35-11D0-98BE-00805F7CED21}` IMarkerSymbol → this+0x1d | `{1B077915-38C5-11D0-92D2-00805F7C28B0}` IPoint → this+0x1c | int32 → +0x26 | 32 字节原样 → +0x2b |
//! | LineElement | `{E6BDAA7D-4D35-11D0-98BE-00805F7CED21}` ILineSymbol → this+0x1c | `{1B07790D-…}` IGeometry → this+0x1a | int32 → +0x23 | int16 → +0x24 |
//!
//! Save 对称（MarkerElement 版本表 `DAT_7506e7d8`；LineElement 版本常量
//! `DAT_7506e7dc`）。符号/几何槽新对象体均未闭合（实现类众多），仅分类空标
//! 记与回引，新对象 = 停止点。
//!
//! ## 样本对位（辅助观察）
//!
//! /Maps 全流 CLSID 扫描（GraphicElements.dll + CartoXProvider.dll 两模块全
//! 部 coclass）：**仅 TextElement `{204034D3}` 16 实例**；MarkerElement/
//! LineElement 与其余元素 coclass 0 实例。首例 TextElement（键 1613
//! @`0x18551`）体逐字段对位：
//!
//! - 版本 8 @`0x18565`；EP 前缀（版本 6、空名称、`"文本"`、VT_EMPTY、
//!   int16 0xFFFF、double ≈5540.06、int32 6）@`0x18567..0x18589`；文本
//!   `"T35"` @`0x18589`；int32 1 @`0x18595`；
//! - TextSymbol `{B65A3E74}` 键 1614 @`0x18599`，体 @`0x185ad` 起按上节字段
//!   序推进到符号槽 #2（`{7914E603}` 键 1593 @`0x186d9`）为止逐字节吻合——
//!   覆盖两 RgbColor（键 1615/1616）、全部标量组、原样 GUID 字段（TextSymbol
//!   自身 CLSID）、ITextBackground/ISymbol #1 双空标记、`"T35"` String 与 8.0/1/1.5
//!   标量组；停止于符号槽 #2 新对象（体未闭合）。
//! - 第二实例（键 1850 @`0x1da39`，文本 `"T12"`）同构互证；16 实例的版本 8 +
//!   EP 版本 6 头部全部一致。
//!
//! 参考样本无 MarkerElement/LineElement；安装介质模板的局部候选现可读至
//! 几何与尾段。局部探针假定此前键已登记，不能替代整流或目标软件验证。
//!
//! ## 与 element_collection 的分派衔接（接线轮已接入）
//!
//! [`read_element_body_of_class_10_1`] 按类标识分派：命中 TextElement/
//! MarkerElement/LineElement 时读取对应体；其他类标识返回
//! [`ErrorKind::Unsupported`]，`element_collection` 的元素循环对该错误维持
//! 既有停止点语义（游标回退到该元素对象头起点、键不登记）。闭合元素体的
//! 嵌套停止经返回值透传（游标停在该嵌套对象头起点、该键不登记，已消费字
//! 段与登记保持提交），元素循环以
//! [`crate::objects::element_collection::ElementCollectionStop10_1::
//! ElementBodyIncomplete`] 停止并携带原因。接线轮起：符号槽 #2 经
//! [`crate::objects::display_symbol::read_symbol_slot_full_10_1`] 全闭合集
//! 分派、IColor 槽经 [`crate::objects::display_color`] 分派、TextElement 与
//! LineElement 的 IGeometry 槽经 [`crate::objects::geometry::
//! read_geometry_body_10_1`] 四类类型化分派（Polyline/Polygon/Point/
//! Multipoint 体整体解析，样本 16 实例原全部停在 TextElement 该槽；证据：
//! docs/arcmap-10.1-geometry.md「接线建议」与「第二轮落位」）、MarkerElement
//! 的 IPoint 槽经 [`read_point_slot`] Point 分派；点/线符号槽分别限制
//! 已知对应类后调用通用符号读取器，嵌套停止透传；TextSymbol 分派已取证的
//! ITextBackground、ISymbol 和 IFontDisp 类；TextSymbol 体内的嵌套停止上抛为
//! TextElement 的停止原因（原实现未透传，见 [`read_text_element_body_10_1`]
//! 内注释）。`fdo_graphics_layer` 的 ElementCollection 槽已在前轮接入
//! element_collection。
//!
//! 调用约定与事务性：调用者已用 [`crate::storage::object_stream::read_header_10_1`]
//! 读取对象头并分类（新对象已登记），从对象头之后的位置传入 Reader；本模块
//! 不重复读取对象头。全部字段在 trial 游标与 trial 登记表上读取，成功返回
//! （含停止点）才提交；任何失败不推进调用者游标、不修改登记表。`max_bytes`
//! 约束全部字符串声明长度；`usize::MAX` 表示不额外设限。这是仅覆盖上述字段
//! 的严格安全子集：不代表完整原生 Load 行为，通过也不代表 MXD 格式已验证。
//! 未接入 MXD 解析主链，人工目视验收未做，不得声称已验证。
//!
//! 未解决问题（详见研究文档「开放项」）：部分标量字段业务语义未定性；
//! IGeometry/IMarkerSymbol/ILineSymbol/ISymbol 槽的其余实现类体未闭
//! 合（IFontDisp 槽的 StdFont 实现类本轮已闭合，证据
//! docs/arcmap-10.1-ole-stdfont.md；该槽其余实现类仍为停止回退）；
//! TextSymbol 版本门尾段（>1 short、≥3 GUID、≥4 short）StdFont 体闭合后样
//! 本可达、位模式原样保留；MarkerElement 尾块虽有模板
//! 局部候选，仍未确定语义；TextElement 版本 7/9 精确块无样本佐证；
//! 全部结论未运行目标软件验证。

use crate::objects::callout_background::{CalloutSlot10_1, read_callout_slot_10_1};
use crate::objects::display_color::read_color_slot_10_1;
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, SymbolStop10_1,
    read_symbol_slot_full_10_1, slot_full_stop_10_1,
};
use crate::objects::geometry::{
    GEOMETRY_BAG_CLASS_ID, GeometryBody10_1, MULTIPATCH_CLASS_ID, MULTIPOINT_CLASS_ID,
    POINT_CLASS_ID, POLYGON_CLASS_ID, POLYLINE_CLASS_ID, PointBody10_1, PolylineBody10_1,
    read_geometry_body_10_1, read_point_body_10_1,
};
use crate::objects::ole_font::{STDFONT_CLASS_ID, StdFontBody10_1, read_std_font_body_10_1};
use crate::objects::property_set::{PropertyValue10_1, read_variant_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// TextElement::Load 接受的最大版本：`>8` 拒绝（有符号）；`==9` 按「8 + 尾
/// 加块」处理；Save（10.1 版本表）恒写 8；样本 16/16 实例为 8。
const MAX_TEXT_ELEMENT_VERSION: i16 = 9;

/// MarkerElement / LineElement::Load 接受的最大版本：`3 < (short)ver` 拒绝
/// （有符号）；负值位模式原生通过，本读取器一并拒绝。
const MAX_SHAPE_ELEMENT_VERSION: i16 = 3;

/// ElementProperties::Load 接受的最大版本：`6 < (short)ver` 拒绝（有符号）；
/// 样本写 6。
const MAX_ELEMENT_PROPERTIES_VERSION: i16 = 6;

/// TextSymbol::Load 接受的最大版本：`4 < ver` 拒绝；样本 4。
const MAX_TEXT_SYMBOL_VERSION: i16 = 4;

/// TextElement 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{204034D3-F6EA-11D0-83AD-080009B996CC}`。证据：
/// esri.clsid.xml 声明（模块 CartoXProvider.dll）；CartoXProvider.dll
/// GetClassID `0x6ce70e10` 写出同一 CLSID；仓库参考样本 /Maps 流 16 处对象
/// 头与本常量逐字节相同（辅助观察）。
pub const TEXT_ELEMENT_CLASS_ID: [u8; 16] = [
    0xd3, 0x34, 0x40, 0x20, 0xea, 0xf6, 0xd0, 0x11, 0x83, 0xad, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

/// MarkerElement 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{530FD712-EF0C-11D0-83A0-080009B996CC}`。证据：
/// esri.clsid.xml 声明（模块 GraphicElements.dll）；GraphicElements.dll
/// GetClassID `0x7504ea20` 以立即数写出同一 CLSID。样本 0 实例，无字节佐证。
pub const MARKER_ELEMENT_CLASS_ID: [u8; 16] = [
    0x12, 0xd7, 0x0f, 0x53, 0x0c, 0xef, 0xd0, 0x11, 0x83, 0xa0, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

/// LineElement 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{8AB7FBE1-D871-11D0-8389-080009B996CC}`。证据：
/// esri.clsid.xml 声明（模块 GraphicElements.dll）；GraphicElements.dll
/// GetClassID `0x7504a090` 写出常量 `0x75071088` 的同一 CLSID。样本 0 实例，
/// 无字节佐证。
pub const LINE_ELEMENT_CLASS_ID: [u8; 16] = [
    0xe1, 0xfb, 0xb7, 0x8a, 0x71, 0xd8, 0xd0, 0x11, 0x83, 0x89, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

/// GraphicElements.dll PolygonElement 的 CLSID。
pub const POLYGON_ELEMENT_CLASS_ID: [u8; 16] = [
    0xc7, 0x67, 0x97, 0x3a, 0x53, 0xf2, 0xd0, 0x11, 0x83, 0xa4, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

pub const MULTIPATCH_ELEMENT_CLASS_ID: [u8; 16] = [
    0xc9, 0xe5, 0x1a, 0xe9, 0x16, 0x2c, 0xd4, 0x11, 0x80, 0xe2, 0x00, 0xc0, 0x4f, 0xa0, 0xad, 0xf8,
];

/// TextSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{B65A3E74-2993-11D1-9A43-0080C7EC5C96}`（esri.clsid
/// 声明 DisplayCore.dll）。样本 TextElement 的符号槽 16/16 为该类新对象。
pub const TEXT_SYMBOL_CLASS_ID: [u8; 16] = [
    0x74, 0x3e, 0x5a, 0xb6, 0x93, 0x29, 0xd1, 0x11, 0x9a, 0x43, 0x00, 0x80, 0xc7, 0xec, 0x5c, 0x96,
];

/// 类标识的十六进制描述（诊断用）。
fn class_id_hex(class_id: &[u8; 16]) -> String {
    class_id.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// 嵌套对象槽位的分类结果。
///
/// 新对象不是本枚举的变体：新对象体在未知类处产生停止点（见
/// [`ElementBodyStop10_1`]），该槽的键不登记（回退语义）。回引条目 20 字节
/// 无体；空标记仅消耗 16 字节、无键。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestedEntry10_1 {
    /// 16 字节全零类标识：空引用标记。
    Null,
    /// 键已在本会话登记过：20 字节回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
}

/// IGeometry 类槽位条目的分类结果（接线轮起经几何类型化分派整体解析
/// Polyline/Polygon/Point/Multipoint 四类，证据：
/// docs/arcmap-10.1-geometry.md「接线建议」与「第二轮落位」）。
#[derive(Debug, Clone, PartialEq)]
pub enum GeometrySlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference {
        class_id: [u8; 16],
        key: u32,
    },
    /// 新 Polyline 对象（类标识经 [`crate::objects::geometry::
    /// POLYLINE_CLASS_ID`] 校验）：已解析体。体读取要么完整成功、要么错误
    /// 上抛（无体内停止点，见 geometry 模块；样本 16 实例佐证）。
    Polyline(Box<PolylineBody10_1>),
    /// 新 Polygon 对象（与 Polyline 共享解析器；样本 0 实例，合成测试覆盖）。
    Polygon(Box<crate::objects::geometry::PolygonBody10_1>),
    /// 新 Point 对象（样本 0 实例，合成测试覆盖）。
    Point(Box<PointBody10_1>),
    /// 新 Multipoint 对象（样本 0 实例，合成测试覆盖）。
    Multipoint(Box<crate::objects::geometry::MultipointBody10_1>),
    Other(Box<GeometryBody10_1>),
}

/// IPoint 类槽位条目的分类结果（MarkerElement 几何槽；接线轮起 Point 新对
/// 象整体解析——该槽请求 IID `{1B077915-38C5-11D0-92D2-…}`，流内实现类为
/// Point；其余几何类维持停止回退。证据：docs/arcmap-10.1-geometry.md「接线
/// 建议」2 的 Point 体无停止点、错误上抛语义与 Polyline 一致）。
#[derive(Debug, Clone, PartialEq)]
pub enum PointSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 Point 对象（类标识经 [`POINT_CLASS_ID`] 校验）：已解析体。体读取
    /// 要么完整成功、要么错误上抛（无体内停止点）。其余实现类不落入本变体。
    Point(Box<PointBody10_1>),
}

/// IFontDisp 类槽位条目的分类结果（本轮起 StdFont 经
/// [`crate::objects::ole_font::read_std_font_body_10_1`] 整体解析，证据：
/// docs/arcmap-10.1-ole-stdfont.md；其余实现类维持停止回退，字段保持
/// `Null`）。
#[derive(Debug, Clone, PartialEq)]
pub enum FontSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 StdFont 对象（类标识经 [`STDFONT_CLASS_ID`] 校验）：已解析体。
    /// 体读取要么完整成功、要么错误上抛（无体内停止点：体不含嵌套对象条
    /// 目，见 ole_font 模块）。其余实现类不落入本变体。
    StdFont(Box<StdFontBody10_1>),
}

/// 元素体内的嵌套新对象停止点（非错误：已消费部分有效）。
#[derive(Debug, Clone, PartialEq)]
pub enum ElementBodyStop10_1 {
    /// 某个 LoadObject 槽位为新对象且类标识无读取器：游标回退到该对象头起
    /// 点，该键不登记。
    NestedObjectNotClosed {
        /// 槽位说明（诊断用）。
        slot: &'static str,
        /// 槽位对象头起点偏移（相对最外层传入 Reader 窗口）。
        offset: usize,
        /// 嵌套对象的类标识（原样保留）。
        class_id: [u8; 16],
        /// 嵌套对象的键（未登记）。
        key: u32,
    },
    /// TextSymbol 体内 RgbColor 之外的槽位新对象（透传）：游标停在该嵌套对
    /// 象头起点，键不登记。
    TextSymbolSlot {
        offset: usize,
        class_id: [u8; 16],
        key: u32,
        /// 槽位说明（ITextBackground/ISymbol/IFontDisp）。
        slot: &'static str,
    },
    /// TextSymbol 符号槽 #2 为本轮闭合的 DisplayCore 同族符号且其体内出现
    /// 嵌套停止（原因透传）：游标停在内层停止点，已消费字段保持提交。
    SymbolSlotNested {
        /// 槽位说明（ISymbol 符号槽 #2）。
        slot: &'static str,
        /// 内层符号体的停止原因。
        reason: SymbolStop10_1,
    },
    /// ElementProperties 前缀 CustomProperty 槽的新对象（透传）。
    PropertiesCustomProperty {
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
}

impl ElementBodyStop10_1 {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(&self) -> String {
        match self {
            Self::NestedObjectNotClosed {
                slot,
                offset,
                class_id,
                key,
            } => format!(
                "{slot}（offset 0x{offset:x}）为新对象（类 {}，键 {key}）：该类体未闭合且无外\
                 部长度，游标回退到该对象头起点",
                class_id_hex(class_id)
            ),
            Self::TextSymbolSlot {
                offset,
                class_id,
                key,
                slot,
            } => format!(
                "TextSymbol 体 {slot}（offset 0x{offset:x}）为新对象（类 {}，键 {key}）：该类\
                 体未闭合，游标回退到该对象头起点",
                class_id_hex(class_id)
            ),
            Self::SymbolSlotNested { slot, reason } => format!(
                "{slot} 的已闭合符号体内出现嵌套停止：{reason:?}；游标停在\
                 内层停止点，已消费字段保持提交"
            ),
            Self::PropertiesCustomProperty {
                offset,
                class_id,
                key,
            } => format!(
                "ElementProperties CustomProperty 槽（offset 0x{offset:x}）为新对象（类 {}，\
                 键 {key}）：该类体未闭合，游标回退到该对象头起点",
                class_id_hex(class_id)
            ),
        }
    }
}

/// ElementProperties 的 CustomProperty 值（版本 <4 的槽位分类或版本 ≥4 的
/// Variant 值）。
#[derive(Debug, Clone, PartialEq)]
pub enum CustomProperty10_1 {
    /// 版本 ≥4：`Variant::Read` 值。
    Variant(PropertyValue10_1),
    /// 版本 <4：LoadObject 槽（请求 IID IPersist）的空标记/回引分类。
    ObjectSlot(NestedEntry10_1),
}

/// ElementProperties::Load（LayerLib `0x786b97f0`）共同前缀的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct ElementPropertiesPrefix10_1 {
    /// int16 版本的无符号位模式（原生有符号 `>6` 拒绝；本读取器接受 0..=6，
    /// 负值一并拒绝较原生更严；样本 6）。
    pub version: u16,
    /// String → this+8；与 `Name` 访问器对位，见 docs/mxd-terra-chain-element.md。
    pub name: String,
    /// String → this+0xc；与 `Type` 访问器对位。
    pub type_description: String,
    /// 版本 <2 的 int32（读后丢弃的旧版遗留段，原样位模式）。
    pub legacy_int32: Option<i32>,
    /// 第 5 字段（版本 ≥4 经 Variant 序列化，旧版为对象槽）；与
    /// `CustomProperty` 访问器对位。
    pub custom_property: CustomProperty10_1,
    /// 版本 >2 的 int16 原样位模式（this+0x10；AutoTransform 存储）。
    pub extra_short: Option<u16>,
    /// 版本 >4 的 double（this+0x2c；ReferenceScale 存储）。
    pub reference_scale: Option<f64>,
    /// 版本 >5 的 int32 原样位模式（this+0x34；AnchorPoint 存储）。
    pub tail_int32: Option<i32>,
}

/// esriCarto 10.1 的 esriAnchorPointEnum；未知整数保留在流字段中。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementAnchorPoint10_1 {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl ElementPropertiesPrefix10_1 {
    pub fn anchor_point(&self) -> Option<ElementAnchorPoint10_1> {
        use ElementAnchorPoint10_1::*;
        match self.tail_int32? {
            0 => Some(TopLeft),
            1 => Some(Top),
            2 => Some(TopRight),
            3 => Some(Left),
            4 => Some(Center),
            5 => Some(Right),
            6 => Some(BottomLeft),
            7 => Some(Bottom),
            8 => Some(BottomRight),
            _ => None,
        }
    }
}

/// 把游标重定位到传入 Reader 窗口的绝对偏移 `target`（回退实现；目标必在
/// 窗口内）。
fn rewind<'a>(origin: &Reader<'a>, start: usize, target: usize) -> Reader<'a> {
    debug_assert!(target >= start);
    let mut positioned = origin.clone();
    positioned
        .read_bytes(target - start)
        .expect("回退目标在停止前的已读位置，必在窗口内");
    positioned
}

/// 从对象头之后的位置读取 ElementProperties 共同前缀。
///
/// 起点约定：调用者已确认这是某元素类体的 EP 前缀起点（各类 Load 第一跳）。
/// CustomProperty 槽（版本 <4）新对象时返回停止原因，游标回退到该槽对象头
/// 起点、该键不登记（登记表恢复快照）；其余失败不推进游标、不修改登记表。
pub(crate) fn read_element_properties_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<PrefixOutcome10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("ElementProperties 前缀 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();

    // 1. int16 版本：原生有符号 >6 拒绝；负值一并拒绝（较原生更严）。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_ELEMENT_PROPERTIES_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出原生接受范围 0..=6（Load 0x786b97f0 \
             有符号比较 >6 拒绝 0x80004005；负值位模式原生通过，本读取器一并拒绝）"
        )));
    }

    // 2/3. 双 String（this+8 / this+0xc）。
    let name = read_string_10_1(&mut trial, max_bytes)?;
    let type_description = read_string_10_1(&mut trial, max_bytes)?;

    // 4. 版本 <2：int32 读后丢弃（原样位模式）。
    let legacy_int32 = if version < 2 {
        Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
    } else {
        None
    };

    // 5. CustomProperty：版本 <4 的 LoadObject 槽（IPersist）仅分类；版本 ≥4
    //    的 Variant::Read。
    let custom_property;
    if version < 4 {
        let slot_offset = trial.position();
        let saved_table = table.clone();
        match read_object_entry_10_1(&mut trial, table)? {
            ObjectEntry10_1::Null => {
                custom_property = CustomProperty10_1::ObjectSlot(NestedEntry10_1::Null)
            }
            ObjectEntry10_1::BackReference { class_id, key } => {
                custom_property =
                    CustomProperty10_1::ObjectSlot(NestedEntry10_1::BackReference { class_id, key })
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                // 新对象体无读取器：回退到该对象头起点，撤销本次键登记，停
                // 止（已消费字段保持提交，由调用方的 trial 机制落地）。
                *table = saved_table;
                let rewound = rewind(reader, start, slot_offset);
                *reader = rewound;
                return Ok(PrefixOutcome10_1::Stopped(
                    ElementBodyStop10_1::PropertiesCustomProperty {
                        offset: slot_offset,
                        class_id,
                        key,
                    },
                ));
            }
        }
    } else {
        custom_property =
            CustomProperty10_1::Variant(match read_variant_10_1(&mut trial, max_bytes) {
                Ok(value) => value,
                Err(error) => {
                    return Err(invalid(format!("CustomProperty Variant 读取失败：{error}")));
                }
            });
    }

    // 6. 版本 >2：int16（this+0x10）。
    let extra_short = if version > 2 {
        Some(trial.read_u16_le()?)
    } else {
        None
    };
    // 7. 版本 >4：double（this+0x2c）。
    let reference_scale = if version > 4 {
        Some(trial.read_f64_le()?)
    } else {
        None
    };
    // 8. 版本 >5：int32（this+0x34）。
    let tail_int32 = if version > 5 {
        Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
    } else {
        None
    };

    *reader = trial;
    Ok(PrefixOutcome10_1::Complete(ElementPropertiesPrefix10_1 {
        version,
        name,
        type_description,
        legacy_int32,
        custom_property,
        extra_short,
        reference_scale,
        tail_int32,
    }))
}

/// EP 前缀读取结果：完整前缀或 CustomProperty 槽停止。
pub(crate) enum PrefixOutcome10_1 {
    Complete(ElementPropertiesPrefix10_1),
    Stopped(ElementBodyStop10_1),
}

/// TextSymbol 体内一个 IColor 槽的分类结果：接线轮起为对象层
/// [`crate::objects::display_color::ColorSlot10_1`] 的等价重导出（五类闭合
/// 颜色经 [`read_color_slot_10_1`] 整体解析，未知颜色类为 `Unsupported`
/// 变体、由本模块映射为停止原因）。
pub use crate::objects::display_color::ColorSlot10_1;

/// 10.1 TextSymbol（DisplayCore `0x6ebd0a60`）的已确认前缀（TextElement 符
/// 号槽的流内体）。
#[derive(Debug, Clone, PartialEq)]
pub struct TextSymbolBody10_1 {
    /// 2 字节版本原始位模式。原生按有符号值判断上限与尾段字段。
    pub version: u16,
    /// 颜色槽 #1（IColor；Color getter 在 FillSymbol 底层槽为空时使用）。
    pub color1: ColorSlot10_1,
    /// 字段 3 组：BreakCharacter、VerticalAlignment、HorizontalAlignment，
    /// 原样保留（this-0x478/-0x470/-0x474；样本 -1、3、1）。
    pub ints_1: [i32; 3],
    /// 字段 4 组：Clip、RightToLeft 原始 byte
    /// （this-0x468/-0x467；样本 0、0）。
    pub bytes_1: [u8; 2],
    /// 字段 5 组：Angle、以排版点为单位的 X/Y 偏移输入
    /// （this-0x45c/-0x44c/-0x444；样本均为 0）。
    pub doubles_1: [f64; 3],
    /// 字段 6 组：ROP2 和 IMapLevel.MapLevel 原值（this-0x4fc/-0x4f4）。
    pub ints_2: [i32; 2],
    /// 颜色槽 #2（IColor；与 ShadowColor 访问器存储对位）。
    pub color2: ColorSlot10_1,
    /// 字段 8 前组：ShadowXOffset、ShadowYOffset（double），Position、Case
    /// （int32）；访问器与 Load 偏移对位见 docs/mxd-terra-integrate-text.md。
    /// 样本依次为 0、0、0、0；保留流内原始值。
    pub group_8a: ([f64; 2], [i32; 2]),
    /// CharacterSpacing、CharacterWidth、WordSpacing（double ×3；样本 0、100、100）。
    pub doubles_2: [f64; 3],
    /// Kerning 的原始 byte（this-0x344；样本 1）。
    pub byte_2: u8,
    /// Leading（double，this-0x33c；样本 0.0）。
    pub double_3: f64,
    /// Direction（int32，this-0x330；样本 0）。
    pub int_3: i32,
    /// FlipAngle（double，this-0x32c；样本 88.0）。
    pub double_4: f64,
    /// TypeSetting 的原始 byte（this-0x320；样本 1）。
    pub byte_3: u8,
    /// ITextPath 实现类的原样 CLSID。样本为 TextSymbol 自身 CLSID。
    pub raw_path_clsid: [u8; 16],
    /// ITextBackground 槽的原有空值或回引分类。新对象体见 `background`。
    pub text_path: NestedEntry10_1,
    /// ITextBackground 槽。未到达或遇到未知 CLSID 时为空。
    pub background: Option<CalloutSlot10_1>,
    /// ISymbol 槽 #1 的原有空值或回引分类。新对象体见 `fill_symbol`。
    pub symbol1: NestedEntry10_1,
    /// FillSymbol 对应的 ISymbol 槽。
    pub fill_symbol: SymbolSlotFull10_1,
    /// String（this-0x238；与 Text getter 存储对位）。
    pub name_string: String,
    /// Size 相关的 double（this-0x1ec；样本 8.0；Load 读取后还作虚调用）。
    pub double_5: f64,
    /// IMask::MaskStyle 的原始 int32（this-0x210；样本 1）。
    pub int_4: i32,
    /// IMask::MaskSize 的原始 double（this-0x204；样本 1.5）。
    pub double_6: f64,
    /// IMask::MaskSymbol 底层 ISymbol 符号槽 #2（经 [`read_display_symbol_slot_10_1`] 分派：
    /// SimpleMarkerSymbol 与本轮闭合的 DisplayCore 同族符号解析为对应体变
    /// 体；样本为 {7914E603} SimpleFillSymbol 新对象；未知符号类不出现在本
    /// 字段——映射为停止原因，字段保持 `Null`）。
    pub symbol2: SymbolSlotFull10_1,
    /// IFontDisp 字体槽（与 Font getter 存储对位；经 [`read_font_slot`] 分派：StdFont 新对
    /// 象解析为 [`FontSlot10_1::StdFont`] 体变体（样本 37 处 StdFont 全为该
    /// 类，证据 docs/arcmap-10.1-ole-stdfont.md）；未知字体类不出现在本字
    /// 段——映射为停止原因，字段保持 `Null`）。
    pub font: FontSlot10_1,
    /// 版本 ≥2 的 RotateSymbolsOnConvert 原始位模式（this-0x4e4）。
    pub tail_short_1: Option<u16>,
    /// 版本 ≥3 的 ITextParser 类标识，零 GUID 表示原生清除解析器。
    pub tail_path_clsid: Option<[u8; 16]>,
    /// 版本 ≥4 的 CJKCharactersRotation 原始位模式（this-0x384）。
    pub tail_short_2: Option<u16>,
    /// 停止原因；`None` = 本读取器覆盖的字段全部消费（版本门尾段读毕）。
    pub stop: Option<ElementBodyStop10_1>,
    /// 停止点之后剩余的字节数（完整消费时为窗口内剩余全部字节）。
    pub remaining_bytes: usize,
}

impl TextSymbolBody10_1 {
    /// 完整读取时返回断行字符的原始 int32，包括原生缺省值 -1。
    pub fn break_character(&self) -> Option<i32> {
        self.stop.is_none().then_some(self.ints_1[0])
    }

    /// 完整读取时返回裁剪开关的原始字节。
    pub fn clip_raw(&self) -> Option<u8> {
        self.stop.is_none().then_some(self.bytes_1[0])
    }

    /// 完整读取时返回存盘的 X/Y 偏移（1/72 英寸），保留流内方向。
    pub fn offsets_in_points(&self) -> Option<[f64; 2]> {
        self.stop
            .is_none()
            .then_some([self.doubles_1[1], self.doubles_1[2]])
    }

    /// 完整读取时返回 Symbol.ROP2 的原始枚举值。
    pub fn rop2(&self) -> Option<i32> {
        self.stop.is_none().then_some(self.ints_2[0])
    }

    /// 完整读取时返回 IMapLevel.MapLevel 的存盘值。
    pub fn map_level(&self) -> Option<i32> {
        self.stop.is_none().then_some(self.ints_2[1])
    }

    /// 返回流内的 CJKCharactersRotation 位模式。旧版由原生字体回填。
    pub fn cjk_characters_rotation_raw(&self) -> Option<u16> {
        self.tail_short_2
    }

    /// 完整读取时返回流内 MaskStyle 原始值，不转换未知枚举值。
    pub fn mask_style(&self) -> Option<i32> {
        self.stop.is_none().then_some(self.int_4)
    }

    /// 完整读取时返回流内 MaskSize 原始值，不套用 setter 的限制。
    pub fn mask_size(&self) -> Option<f64> {
        self.stop.is_none().then_some(self.double_6)
    }

    /// 完整读取时返回 MaskSymbol 的底层槽；不模拟原生 getter 的接口查询。
    pub fn mask_symbol(&self) -> Option<&SymbolSlotFull10_1> {
        self.stop.is_none().then_some(&self.symbol2)
    }
}

/// 10.1 TextElement::Load（`0x6ce70e50`）可读取的对象体。
#[derive(Debug, Clone, PartialEq)]
pub struct TextElementBody10_1 {
    /// 2 字节版本的无符号位模式（原生有符号 `>8` 拒绝、`==9` 按「8 + 尾加
    /// 块」；负值一并拒绝较原生更严；样本 16/16 为 8）。
    pub version: u16,
    /// ElementProperties 共同前缀（第 2 字段；CustomProperty 槽停止时不完整
    /// ——见 [`ElementBodyStop10_1::PropertiesCustomProperty`]）。
    pub properties: Option<ElementPropertiesPrefix10_1>,
    /// 文本内容 String（this+0x21；样本 `"T35"`）。
    pub text: String,
    /// 第 4 字段 int32（布尔 this+0x22；样本 1，语义未定性，原样保留）。
    pub raw_bool_int32: i32,
    /// ITextSymbol 符号槽（第 5 字段）：TextSymbol 新对象的体（类分派）。
    pub symbol: Option<TextSymbolBody10_1>,
    /// IGeometry 文本几何槽（第 6 字段）：接线轮起经
    /// [`crate::objects::geometry::read_geometry_body_10_1`] 四类类型化分派
    /// ——Polyline/Polygon/Point/Multipoint 新对象整体解析其体（记录定长、
    /// 空间参考段可闭合，无停止点；本轮自 Polyline 专用扩展，样本 16 实例
    /// 全为 Polyline，其余三类合成测试覆盖）；其余几何类维持既有停止语义
    /// （见 `stop`）。
    pub geometry: GeometrySlot10_1,
    /// 版本 >2 的 int32（this+0x2a；Save 恒写 -1）。
    pub tail_int32_1: Option<i32>,
    /// 版本 >3 的 int32（this+0x2b；Save 恒写 -1）。
    pub tail_int32_2: Option<i32>,
    /// 版本 ==7 精确块（IBorder/IBackground/IShadow 三槽 + double + int32 +
    /// double；样本版本 8 不含）。
    pub decoration_block: Option<TextDecorationBlock10_1>,
    /// 版本 ==9 尾加块（ISpatialReference 槽 + int32 + 32 字节原样）。
    pub version9_block: Option<Version9Block10_1>,
    /// 停止原因；`None` = 本读取器覆盖的字段全部消费（版本门尾段读毕）。
    pub stop: Option<ElementBodyStop10_1>,
    /// 停止点之后剩余的字节数（完整消费时为窗口内剩余全部字节）。
    pub remaining_bytes: usize,
}

/// TextElement 版本 ==7 精确块（装饰物三槽 + 标量组；样本版本 8 不含）。
///
/// 第三轮接线起三装饰槽经 [`crate::objects::element_decor`] 分派：
/// SymbolBorder/SymbolBackground/SymbolShadow 新对象整体解析（样本 0 实例，
/// 合成测试），其余实现类维持停止回退（该槽为 `Null`、停止原因记入宿主体）。
#[derive(Debug, Clone, PartialEq)]
pub struct TextDecorationBlock10_1 {
    /// IBorder 槽（SymbolBorder 分派）。
    pub border: crate::objects::element_decor::DecorSlot10_1,
    /// IBackground 槽（SymbolBackground 分派）。
    pub background: crate::objects::element_decor::DecorSlot10_1,
    /// IShadow 槽（SymbolShadow 分派）。
    pub shadow: crate::objects::element_decor::DecorSlot10_1,
    /// 第一个 double 的原样位模式。
    pub double_1: f64,
    /// int32 的原样位模式。
    pub raw_int32: i32,
    /// 第二个 double 的原样位模式。
    pub double_2: f64,
}

/// 两个原生 Load 路径共用相同的 IBorder / IBackground / IShadow 和标量组。
fn read_text_decoration_block_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    trial_table: &mut RefTable10_1,
) -> io::Result<(Option<TextDecorationBlock10_1>, Option<ElementBodyStop10_1>)> {
    let mut stop = None;
    let mut block = TextDecorationBlock10_1 {
        border: crate::objects::element_decor::DecorSlot10_1::Null,
        background: crate::objects::element_decor::DecorSlot10_1::Null,
        shadow: crate::objects::element_decor::DecorSlot10_1::Null,
        double_1: 0.0,
        raw_int32: 0,
        double_2: 0.0,
    };
    // 槽序固定：border → background → shadow，按序落位（空标记同样
    // 消费该槽位，不改变后续槽的归属）。
    let decor_slots: [(
        &'static str,
        fn(
            &mut Reader<'_>,
            usize,
            &mut RefTable10_1,
            usize,
        ) -> io::Result<(
            Option<crate::objects::element_decor::DecorSlot10_1>,
            Option<crate::objects::display_symbol::SymbolStop10_1>,
        )>,
    ); 3] = [
        (
            "IBorder 槽（文本装饰块）",
            crate::objects::element_decor::read_border_slot_10_1,
        ),
        (
            "IBackground 槽（文本装饰块）",
            crate::objects::element_decor::read_background_slot_10_1,
        ),
        (
            "IShadow 槽（文本装饰块）",
            crate::objects::element_decor::read_shadow_slot_10_1,
        ),
    ];
    for (slot_index, (label, read_slot)) in decor_slots.iter().enumerate() {
        let slot_offset = trial.position();
        let (slot, inner) = read_slot(trial, max_bytes, trial_table, MAX_SYMBOL_NESTING_DEPTH)?;
        let entry = match slot {
            Some(entry) => entry,
            None => {
                // 未知实现类：分派器已回退游标与登记表，停止原因
                //（Slot，含类标识与键）随返回值带出；映射为既有
                // NestedObjectNotClosed（游标回退到槽位起点）。
                if let Some(crate::objects::display_symbol::SymbolStop10_1::Slot {
                    class_id,
                    key,
                    ..
                }) = inner
                {
                    stop = Some(ElementBodyStop10_1::NestedObjectNotClosed {
                        slot: label,
                        offset: slot_offset,
                        class_id,
                        key,
                    });
                }
                break;
            }
        };
        match slot_index {
            0 => block.border = entry,
            1 => block.background = entry,
            _ => block.shadow = entry,
        }
        if let Some(inner) = inner {
            // 已解析装饰体内嵌套停止：复用 SymbolSlotNested 透传
            //（游标停在内层停止点）。
            stop = Some(ElementBodyStop10_1::SymbolSlotNested {
                slot: label,
                reason: inner,
            });
            break;
        }
    }
    if stop.is_some() {
        return Ok((None, stop));
    }
    block.double_1 = trial.read_f64_le()?;
    block.raw_int32 = trial.read_u32_le()? as i32;
    block.double_2 = trial.read_f64_le()?;
    Ok((Some(block), None))
}

/// CartoXProvider.dll 0x6ce52050：先读装饰块，再委派 TextElement::Load。
pub const PARAGRAPH_TEXT_ELEMENT_CLASS_ID: [u8; 16] = [
    0xb1, 0x98, 0x45, 0xc8, 0xbe, 0xc4, 0x03, 0x42, 0x91, 0x32, 0xad, 0xa2, 0xbe, 0x57, 0xf3, 0x0c,
];

#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphTextElement10_1 {
    pub version: u16,
    pub decoration: Option<TextDecorationBlock10_1>,
    pub text_element: Option<TextElementBody10_1>,
    pub stop: Option<ElementBodyStop10_1>,
}

pub fn read_paragraph_text_element_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<ParagraphTextElement10_1> {
    let mut trial = reader.clone();
    let mut refs = table.clone();
    let version = trial.read_u16_le()?;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的 ParagraphTextElement 版本",
        ));
    }
    let (decoration, mut stop) = read_text_decoration_block_10_1(&mut trial, max_bytes, &mut refs)?;
    let text_element = if stop.is_none() {
        let body = read_text_element_body_10_1(&mut trial, max_bytes, &mut refs)?;
        stop = body.stop.clone();
        Some(body)
    } else {
        None
    };
    *reader = trial;
    *table = refs;
    Ok(ParagraphTextElement10_1 {
        version,
        decoration,
        text_element,
        stop,
    })
}

/// TextElement 版本 ==9 尾加块的 ISpatialReference 槽分类结果（F 轮接线：
/// 三类闭合空间参考分派，其余实现类 = 停止回退；样本版本 8 不含本块）。
#[derive(Debug, Clone, PartialEq)]
pub enum SpatialRefSlot10_1 {
    /// 16 字节全零类标识：空引用标记。
    Null,
    /// 键已在本会话登记：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 ProjectedCoordinateSystem 对象：已读取其闭合体。
    Projected(crate::objects::spatial_reference::ProjectedCoordinateSystem10_1),
    /// 新 GeographicCoordinateSystem 对象：已读取其闭合体。
    Geographic(crate::objects::spatial_reference_slots::GeographicCoordinateSystem10_1),
    /// 新 UnknownCoordinateSystem 对象：已读取其闭合体。
    Unknown(crate::objects::spatial_reference_slots::UnknownCoordinateSystem10_1),
}

/// TextElement 版本 ==9 尾加块。
#[derive(Debug, Clone, PartialEq)]
pub struct Version9Block10_1 {
    /// ISpatialReference 槽（三类闭合空间参考分派）。
    pub spatial_reference: SpatialRefSlot10_1,
    /// int32 的原样位模式。
    pub raw_int32: i32,
    /// 32 字节原样读取（WKSEnvelope 候选）。
    pub raw_envelope: [u8; 32],
}

/// 10.1 MarkerElement::Load（`0x7504ea50`）可读取的对象体。
#[derive(Debug, Clone, PartialEq)]
pub struct MarkerElementBody10_1 {
    /// 2 字节版本的无符号位模式（原生有符号 `>3` 拒绝；负值一并拒绝；无样
    /// 本佐证）。
    pub version: u16,
    /// ElementProperties 共同前缀（CustomProperty 槽停止时 `None`）。
    pub properties: Option<ElementPropertiesPrefix10_1>,
    /// IMarkerSymbol 槽：仅对已知点符号类分派，未知类停在槽头。
    pub symbol: SymbolSlotFull10_1,
    /// IPoint 几何槽（接线轮起经 [`read_point_slot`] 类分派：Point 新对象
    /// 整体解析其体（无停止点）；其余实现类映射为停止原因，字段保持
    /// `Null`。参考样本无实例；安装介质有局部候选）。
    pub geometry: PointSlot10_1,
    /// 版本 >1 的 int32（this+0x26）。
    pub tail_int32: Option<i32>,
    /// 版本 >2 的 32 字节原样读取（this+0x2b；WKSEnvelope 候选）。
    pub raw_envelope: Option<[u8; 32]>,
    /// 停止原因；`None` = 覆盖字段全部消费。
    pub stop: Option<ElementBodyStop10_1>,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

/// LineElement 和 PolygonElement 的共同持久化字段。
#[derive(Debug, Clone, PartialEq)]
pub struct LineOrPolygonElementBody10_1 {
    /// 2 字节版本的无符号位模式（原生有符号 `>3` 拒绝；负值一并拒绝）。
    pub version: u16,
    /// ElementProperties 共同前缀（CustomProperty 槽停止时 `None`）。
    pub properties: Option<ElementPropertiesPrefix10_1>,
    /// 符号槽按 ILineSymbol 或 IFillSymbol 分派，未知类停在槽头。
    pub symbol: SymbolSlotFull10_1,
    /// IGeometry 几何槽（接线轮起与 TextElement 同经四类类型化分派：
    /// [`GeometrySlot10_1`]；其余几何类映射为停止原因）。
    pub geometry: GeometrySlot10_1,
    /// 版本 >1 的 int32（this+0x23）。
    pub tail_int32: Option<i32>,
    /// 版本 >2 的 int16（this+0x24）。
    pub tail_short: Option<u16>,
    /// 停止原因；`None` = 覆盖字段全部消费。
    pub stop: Option<ElementBodyStop10_1>,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

pub use LineOrPolygonElementBody10_1 as LineElementBody10_1;

#[derive(Debug, Clone, PartialEq)]
pub struct MultiPatchElement10_1 {
    pub version: u16,
    pub properties: ElementPropertiesPrefix10_1,
    pub fill: SymbolSlotFull10_1,
    pub geometry: crate::maps::map_frame::FrameGeometrySlot10_1,
    pub tail_long: i32,
    pub tail_short: u16,
}

/// 元素类对象体分派结果。
#[derive(Debug, Clone, PartialEq)]
pub enum ElementBody10_1 {
    TextElement(Box<TextElementBody10_1>),
    ParagraphTextElement(Box<ParagraphTextElement10_1>),
    MarkerElement(Box<MarkerElementBody10_1>),
    LineElement(Box<LineElementBody10_1>),
    PolygonElement(Box<LineOrPolygonElementBody10_1>),
    FrameElement(Box<crate::maps::map_frame::BaseFrameElement10_1>),
    MultiPatchElement(Box<MultiPatchElement10_1>),
    PictureElement(Box<crate::maps::picture_element::PictureElement10_1>),
}

impl ElementBody10_1 {
    pub fn stop(&self) -> Option<&ElementBodyStop10_1> {
        match self {
            Self::TextElement(body) => body.stop.as_ref(),
            Self::ParagraphTextElement(body) => body.stop.as_ref(),
            Self::MarkerElement(body) => body.stop.as_ref(),
            Self::LineElement(body) | Self::PolygonElement(body) => body.stop.as_ref(),
            Self::PictureElement(_) | Self::FrameElement(_) | Self::MultiPatchElement(_) => None,
        }
    }
}

/// 按调用者已读取的对象头类标识分派元素体读取。
///
/// `class_id` 必须是调用方从对象头取得的 16 字节类标识；起点为对象头之后的
/// 第一字节。未闭合的元素类返回 [`ErrorKind::Unsupported`]，游标不推进、登
/// 记表不修改。
pub fn read_element_body_of_class_10_1(
    class_id: &[u8; 16],
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<ElementBody10_1> {
    use crate::maps::picture_element::{PictureElementKind10_1, read_picture_element_10_1};

    if *class_id == TEXT_ELEMENT_CLASS_ID {
        Ok(ElementBody10_1::TextElement(Box::new(
            read_text_element_body_10_1(reader, max_bytes, table)?,
        )))
    } else if *class_id == PARAGRAPH_TEXT_ELEMENT_CLASS_ID {
        Ok(ElementBody10_1::ParagraphTextElement(Box::new(
            read_paragraph_text_element_10_1(reader, max_bytes, table)?,
        )))
    } else if *class_id == MARKER_ELEMENT_CLASS_ID {
        Ok(ElementBody10_1::MarkerElement(Box::new(
            read_marker_element_body_10_1(reader, max_bytes, table)?,
        )))
    } else if *class_id == LINE_ELEMENT_CLASS_ID {
        Ok(ElementBody10_1::LineElement(Box::new(
            read_line_element_body_10_1(reader, max_bytes, table)?,
        )))
    } else if *class_id == POLYGON_ELEMENT_CLASS_ID {
        Ok(ElementBody10_1::PolygonElement(Box::new(
            read_line_or_polygon_element(reader, max_bytes, table, ShapeSymbol::Fill)?,
        )))
    } else if *class_id == MULTIPATCH_ELEMENT_CLASS_ID {
        Ok(ElementBody10_1::MultiPatchElement(Box::new(
            read_multi_patch_element(reader, max_bytes, table)?,
        )))
    } else if *class_id == crate::maps::page_layout_prefix::FRAME_ELEMENT_CLASS_ID {
        let mut candidate = reader.clone();
        let mut refs = table.clone();
        let body = crate::maps::map_frame::base_frame(&mut candidate, max_bytes, &mut refs)?;
        *reader = candidate;
        *table = refs;
        Ok(ElementBody10_1::FrameElement(Box::new(body)))
    } else if let Some(kind) = PictureElementKind10_1::from_class(class_id) {
        Ok(ElementBody10_1::PictureElement(Box::new(
            read_picture_element_10_1(reader, max_bytes, table, kind)?,
        )))
    } else {
        Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("尚不支持元素类 {}", class_id_hex(class_id)),
        ))
    }
}

fn read_multi_patch_element(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<MultiPatchElement10_1> {
    let mut trial = reader.clone();
    let mut refs = table.clone();
    let version = trial.read_u16_le()?;
    if version as i16 > 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的 MultiPatchElement 版本",
        ));
    }
    let properties = match read_element_properties_prefix_10_1(&mut trial, max_bytes, &mut refs)? {
        PrefixOutcome10_1::Complete(properties) => properties,
        PrefixOutcome10_1::Stopped(reason) => {
            return Err(io::Error::new(ErrorKind::Unsupported, reason.describe()));
        }
    };
    let mut stop = None;
    let fill = read_shape_symbol_slot(
        &mut trial,
        max_bytes,
        &mut refs,
        ShapeSymbol::Fill,
        &mut stop,
    )?;
    if let Some(reason) = stop {
        return Err(io::Error::new(ErrorKind::Unsupported, reason.describe()));
    }
    let geometry = crate::maps::map_frame::geometry_slot(&mut trial, max_bytes, &mut refs)?;
    let tail_long = trial.read_u32_le()? as i32;
    let tail_short = trial.read_u16_le()?;
    *reader = trial;
    *table = refs;
    Ok(MultiPatchElement10_1 {
        version,
        properties,
        fill,
        geometry,
        tail_long,
        tail_short,
    })
}

/// 读取一个 IFontDisp 类槽位（TextSymbol 第 15 字段）：本轮起经类分派——
/// StdFont 新对象（`{0BE35203-…}`）由 [`read_std_font_body_10_1`] 整体解析
/// 为 [`FontSlot10_1::StdFont`]（体不含嵌套对象条目，无停止点；体自身错误
/// 上抛即整体失败，与 Polyline 几何体语义一致）；其余实现类新对象 = 停止点
/// （回退游标、登记表不保留该键、字段保持 `Null`）。证据：
/// docs/arcmap-10.1-ole-stdfont.md。
fn read_font_slot<'a>(
    trial: &mut Reader<'a>,
    max_bytes: usize,
    trial_table: &mut RefTable10_1,
    origin: &Reader<'a>,
    start: usize,
    slot: &'static str,
    stop: &mut Option<ElementBodyStop10_1>,
) -> io::Result<FontSlot10_1> {
    let slot_offset = trial.position();
    let saved_table = trial_table.clone();
    match read_object_entry_10_1(trial, trial_table)? {
        ObjectEntry10_1::Null => Ok(FontSlot10_1::Null),
        ObjectEntry10_1::BackReference { class_id, key } => {
            Ok(FontSlot10_1::BackReference { class_id, key })
        }
        ObjectEntry10_1::NewObject { class_id, key: _ } if class_id == STDFONT_CLASS_ID => {
            // 键已由条目分类按原生「登记先于持久化」语义登记；体无停止点，
            // 完整成功才提交（失败不推进游标、登记表由调用方 trial 语义保
            // 护；错误上抛即整体失败）。
            let body = read_std_font_body_10_1(trial, max_bytes)?;
            Ok(FontSlot10_1::StdFont(Box::new(body)))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            *trial_table = saved_table;
            *trial = rewind(origin, start, slot_offset);
            *stop = Some(ElementBodyStop10_1::NestedObjectNotClosed {
                slot,
                offset: slot_offset,
                class_id,
                key,
            });
            Ok(FontSlot10_1::Null)
        }
    }
}

/// 读取一个 IColor 槽：经 [`read_color_slot_10_1`] 分派（五类闭合颜色整体
/// 解析）；未知颜色类新对象 = 停止点（分派内部已回退游标、登记表不保留该
/// 键，本函数映射为停止原因并返回 `None`）。
fn read_color_slot<'a>(
    trial: &mut Reader<'a>,
    max_bytes: usize,
    trial_table: &mut RefTable10_1,
    stop: &mut Option<ElementBodyStop10_1>,
) -> io::Result<Option<ColorSlot10_1>> {
    let slot_offset = trial.position();
    match read_color_slot_10_1(trial, max_bytes, trial_table)? {
        // 空标记/回引/五类闭合颜色体：分派已提交，原样返回。
        slot @ (ColorSlot10_1::Null
        | ColorSlot10_1::BackReference { .. }
        | ColorSlot10_1::Color { .. }) => Ok(Some(slot)),
        ColorSlot10_1::Unsupported { class_id, key } => {
            *stop = Some(ElementBodyStop10_1::NestedObjectNotClosed {
                slot: "IColor 颜色槽",
                offset: slot_offset,
                class_id,
                key,
            });
            Ok(None)
        }
    }
}

/// 读取一个 ISymbol 类槽位（TextSymbol 符号槽 #2）：接线轮起经
/// [`read_symbol_slot_full_10_1`] 全闭合集分派（第一轮 Simple 族 + 第二轮
/// MultiLayer/装饰等 17 类整体解析，证据：docs/arcmap-10.1-display-symbols-
/// round2.md 第五节宿主切换）；未知符号类新对象 = 停止点；闭合符号体内的
/// 嵌套停止透传为 [`ElementBodyStop10_1::SymbolSlotNested`]。
fn read_symbol_slot<'a>(
    trial: &mut Reader<'a>,
    max_bytes: usize,
    trial_table: &mut RefTable10_1,
    slot: &'static str,
    stop: &mut Option<ElementBodyStop10_1>,
) -> io::Result<SymbolSlotFull10_1> {
    let slot_offset = trial.position();
    match read_symbol_slot_full_10_1(trial, max_bytes, trial_table, MAX_SYMBOL_NESTING_DEPTH)? {
        SymbolSlotFull10_1::Unsupported { class_id, key } => {
            // 未知符号类：分派内部已回退到槽位起点、登记表不保留该键。
            *stop = Some(ElementBodyStop10_1::NestedObjectNotClosed {
                slot,
                offset: slot_offset,
                class_id,
                key,
            });
            Ok(SymbolSlotFull10_1::Null)
        }
        body => {
            if let Some(reason) = slot_full_stop_10_1(&body, DisplaySlotKind10_1::Symbol) {
                *stop = Some(ElementBodyStop10_1::SymbolSlotNested { slot, reason });
            }
            Ok(body)
        }
    }
}

#[derive(Clone, Copy)]
enum ShapeSymbol {
    Marker,
    Line,
    Fill,
}

/// 在调用通用符号读取器前按槽位接口限制新对象类，避免把填充符号、
/// 辅助对象或另一类符号误当成目标接口实现。空值和回引维持既有语义。
fn read_shape_symbol_slot<'a>(
    trial: &mut Reader<'a>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    kind: ShapeSymbol,
    stop: &mut Option<ElementBodyStop10_1>,
) -> io::Result<SymbolSlotFull10_1> {
    use crate::objects::display_symbol as symbols;
    let slot = match kind {
        ShapeSymbol::Marker => "IMarkerSymbol 符号槽",
        ShapeSymbol::Line => "ILineSymbol 符号槽",
        ShapeSymbol::Fill => "IFillSymbol 符号槽",
    };
    let offset = trial.position();
    let mut probe = trial.clone();
    let mut probe_table = table.clone();
    let (class_id, key) = match read_object_entry_10_1(&mut probe, &mut probe_table)? {
        ObjectEntry10_1::Null => {
            *trial = probe;
            return Ok(SymbolSlotFull10_1::Null);
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *trial = probe;
            return Ok(SymbolSlotFull10_1::BackReference { class_id, key });
        }
        ObjectEntry10_1::NewObject { class_id, key } => (class_id, key),
    };
    let allowed = match kind {
        ShapeSymbol::Marker => symbols::MARKER_SYMBOL_CLASSES.contains(&class_id),
        ShapeSymbol::Line => symbols::LINE_SYMBOL_CLASSES.contains(&class_id),
        ShapeSymbol::Fill => symbols::FILL_SYMBOL_CLASSES.contains(&class_id),
    };
    if !allowed {
        *stop = Some(ElementBodyStop10_1::NestedObjectNotClosed {
            slot,
            offset,
            class_id,
            key,
        });
        return Ok(SymbolSlotFull10_1::Null);
    }
    read_symbol_slot(trial, max_bytes, table, slot, stop)
}

/// 读取一个 IGeometry 类槽位：接线轮起对闭合四类经
/// [`crate::objects::geometry::read_geometry_body_10_1`] 类型化分派整体解
/// 析（Polyline/Polygon/Point/Multipoint，形态仿
/// [`read_color_slot`]/[`read_symbol_slot`]；整合轮自 Polyline 专用扩展，
/// 证据：docs/arcmap-10.1-geometry.md「第二轮落位」接线建议 1）；清单之外
/// 的几何类新对象维持既有停止语义（映射为停止原因并回退游标、登记表不留
/// 该键）；四类体自身错误上抛（含体内空间参考未知类，与 envelope 嵌套语义
/// 一致，见 geometry 研究文档「接线建议」3）。
fn read_geometry_slot<'a>(
    trial: &mut Reader<'a>,
    max_bytes: usize,
    trial_table: &mut RefTable10_1,
    origin: &Reader<'a>,
    start: usize,
    slot: &'static str,
    stop: &mut Option<ElementBodyStop10_1>,
) -> io::Result<Option<GeometrySlot10_1>> {
    let slot_offset = trial.position();
    let saved_table = trial_table.clone();
    match read_object_entry_10_1(trial, trial_table)? {
        ObjectEntry10_1::Null => Ok(Some(GeometrySlot10_1::Null)),
        ObjectEntry10_1::BackReference { class_id, key } => {
            Ok(Some(GeometrySlot10_1::BackReference { class_id, key }))
        }
        ObjectEntry10_1::NewObject { class_id, key: _ }
            if class_id == POLYLINE_CLASS_ID
                || class_id == POLYGON_CLASS_ID
                || class_id == POINT_CLASS_ID
                || class_id == MULTIPOINT_CLASS_ID
                || class_id == crate::objects::envelope::ENVELOPE_CLASS_ID
                || class_id == GEOMETRY_BAG_CLASS_ID
                || class_id == MULTIPATCH_CLASS_ID =>
        {
            // 键已由条目分类登记（登记先于持久化）；已支持的几何体无中途停止点，
            // 完整成功才提交（失败不推进游标、登记表由调用方 trial 语义保
            // 护；错误上抛即整体失败）。
            let body = read_geometry_body_10_1(&class_id, trial, max_bytes, trial_table)?;
            Ok(Some(match body {
                GeometryBody10_1::Polyline(body) => GeometrySlot10_1::Polyline(Box::new(body)),
                GeometryBody10_1::Polygon(body) => GeometrySlot10_1::Polygon(Box::new(body)),
                GeometryBody10_1::Point(body) => GeometrySlot10_1::Point(Box::new(body)),
                GeometryBody10_1::Multipoint(body) => GeometrySlot10_1::Multipoint(Box::new(body)),
                body => GeometrySlot10_1::Other(Box::new(body)),
            }))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            *trial_table = saved_table;
            *trial = rewind(origin, start, slot_offset);
            *stop = Some(ElementBodyStop10_1::NestedObjectNotClosed {
                slot,
                offset: slot_offset,
                class_id,
                key,
            });
            Ok(None)
        }
    }
}

/// 读取一个 IPoint 类槽位（MarkerElement 第 4 字段）：接线轮起经
/// [`read_point_body_10_1`] 类分派——Point 新对象整体解析（体无停止点、错
/// 误上抛语义与 Polyline 一致）；其余实现类新对象 = 停止点（回退游标、登记
/// 表不保留该键、字段保持 `Null`）。证据：
/// docs/arcmap-10.1-geometry.md「第二轮落位」接线建议 2（样本 0 实例，接线
/// 契约由合成测试固定）。
fn read_point_slot<'a>(
    trial: &mut Reader<'a>,
    max_bytes: usize,
    trial_table: &mut RefTable10_1,
    origin: &Reader<'a>,
    start: usize,
    slot: &'static str,
    stop: &mut Option<ElementBodyStop10_1>,
) -> io::Result<PointSlot10_1> {
    let slot_offset = trial.position();
    let saved_table = trial_table.clone();
    match read_object_entry_10_1(trial, trial_table)? {
        ObjectEntry10_1::Null => Ok(PointSlot10_1::Null),
        ObjectEntry10_1::BackReference { class_id, key } => {
            Ok(PointSlot10_1::BackReference { class_id, key })
        }
        ObjectEntry10_1::NewObject { class_id, key: _ } if class_id == POINT_CLASS_ID => {
            // 键已由条目分类登记（登记先于持久化）；Point 体无停止点，完整
            // 成功才提交（失败不推进游标、登记表由调用方 trial 语义保护）。
            let body = read_point_body_10_1(trial, max_bytes, trial_table)?;
            Ok(PointSlot10_1::Point(Box::new(body)))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            *trial_table = saved_table;
            *trial = rewind(origin, start, slot_offset);
            *stop = Some(ElementBodyStop10_1::NestedObjectNotClosed {
                slot,
                offset: slot_offset,
                class_id,
                key,
            });
            Ok(PointSlot10_1::Null)
        }
    }
}

/// 从对象头之后的位置读取 10.1 TextSymbol 体（DisplayCore `0x6ebd0a60`）。
///
/// 字段序见模块文档；按已取证的类分派背景、符号、颜色和字体。`max_bytes` 约束 String 字段；失败不推进游标、不修改登记表。
pub fn read_text_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<TextSymbolBody10_1> {
    table.with_nested_graphics(|table| read_text_symbol_body(reader, max_bytes, table))
}

fn read_text_symbol_body(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<TextSymbolBody10_1> {
    let start = reader.position();
    let origin = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("TextSymbol 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. 保留原始位模式，按原生有符号比较判断版本和尾段。
    let version = trial.read_u16_le()?;
    let layout_version = version as i16;
    if layout_version > MAX_TEXT_SYMBOL_VERSION {
        return Err(invalid(format!(
            "版本 {layout_version}（0x{version:04x}）超过原生上限 {MAX_TEXT_SYMBOL_VERSION}"
        )));
    }

    let mut stop = None;

    // 2. 颜色槽 #1（五类闭合颜色经对象层颜色槽位分派）。
    let mut color1 = ColorSlot10_1::Null;
    if stop.is_none() {
        match read_color_slot(&mut trial, max_bytes, &mut trial_table, &mut stop)? {
            Some(slot) => color1 = slot,
            None => {}
        }
    }

    // 3..6. 标量组（任一读取失败原样向上传播）。
    let mut ints_1 = [0_i32; 3];
    let mut bytes_1 = [0_u8; 2];
    let mut doubles_1 = [0_f64; 3];
    let mut ints_2 = [0_i32; 2];
    let mut color2 = ColorSlot10_1::Null;
    let mut group_8a = ([0_f64; 2], [0_i32; 2]);
    let mut doubles_2 = [0_f64; 3];
    let mut byte_2 = 0_u8;
    let mut double_3 = 0_f64;
    let mut int_3 = 0_i32;
    let mut double_4 = 0_f64;
    let mut byte_3 = 0_u8;
    let mut raw_path_clsid = [0_u8; 16];
    let mut background = None;
    let mut fill_symbol = SymbolSlotFull10_1::Null;
    let mut symbol2 = SymbolSlotFull10_1::Null;
    let mut font = FontSlot10_1::Null;
    let mut tail_short_1 = None;
    let mut tail_path_clsid = None;
    let mut tail_short_2 = None;
    let mut name_string = String::new();
    let mut double_5 = 0_f64;
    let mut int_4 = 0_i32;
    let mut double_6 = 0_f64;

    if stop.is_none() {
        for slot in &mut ints_1 {
            *slot = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        }
        for slot in &mut bytes_1 {
            *slot = trial.read_bytes(1)?[0];
        }
        for slot in &mut doubles_1 {
            *slot = trial.read_f64_le()?;
        }
        for slot in &mut ints_2 {
            *slot = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        }
        // 7. 颜色槽 #2（同一颜色槽位分派）。
        match read_color_slot(&mut trial, max_bytes, &mut trial_table, &mut stop)? {
            Some(slot) => color2 = slot,
            None => {}
        }
    }

    if stop.is_none() {
        // 8. 标量组。
        for slot in &mut group_8a.0 {
            *slot = trial.read_f64_le()?;
        }
        for slot in &mut group_8a.1 {
            *slot = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        }
        for slot in &mut doubles_2 {
            *slot = trial.read_f64_le()?;
        }
        byte_2 = trial.read_bytes(1)?[0];
        double_3 = trial.read_f64_le()?;
        int_3 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        double_4 = trial.read_f64_le()?;
        byte_3 = trial.read_bytes(1)?[0];
        // 9. 独立的 ITextPath 实现类标识。
        raw_path_clsid = trial.read_bytes(16)?.try_into().expect("固定 16 字节");
        // 10. 带键的 ITextBackground 槽。
        const SLOT: &str = "ITextBackground 文字背景槽";
        let offset = trial.position();
        let (body, reason) = read_callout_slot_10_1(
            &mut trial,
            max_bytes,
            &mut trial_table,
            MAX_SYMBOL_NESTING_DEPTH,
            true,
            DisplaySlotKind10_1::TextBackground,
        )?;
        background = body;
        stop = reason.map(|reason| match reason {
            SymbolStop10_1::Slot { class_id, key, .. } => {
                ElementBodyStop10_1::NestedObjectNotClosed {
                    slot: SLOT,
                    offset,
                    class_id,
                    key,
                }
            }
            reason => ElementBodyStop10_1::SymbolSlotNested { slot: SLOT, reason },
        });
    }

    if stop.is_none() {
        // 11. ISymbol 符号槽 #1。
        fill_symbol = read_symbol_slot(
            &mut trial,
            max_bytes,
            &mut trial_table,
            "ISymbol 填充符号槽",
            &mut stop,
        )?;
    }

    if stop.is_none() {
        // 12. String。
        name_string = read_string_10_1(&mut trial, max_bytes)?;
        // 13. 标量组。
        double_5 = trial.read_f64_le()?;
        int_4 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        double_6 = trial.read_f64_le()?;
        // 14. ISymbol 符号槽 #2：经对象层符号槽位分派（接线；样本
        //     {7914E603} = SimpleFillSymbol，体已闭合并整体消费；未知符号类
        //     维持停止回退语义）。
        symbol2 = read_symbol_slot(
            &mut trial,
            max_bytes,
            &mut trial_table,
            "ISymbol 符号槽 #2",
            &mut stop,
        )?;
    }

    // 15/16. IFontDisp 槽与版本门尾段（#2 停止时结构性未达）。
    if stop.is_none() {
        match read_font_slot(
            &mut trial,
            max_bytes,
            &mut trial_table,
            &origin,
            start,
            "IFontDisp 字体槽",
            &mut stop,
        ) {
            Ok(slot) => font = slot,
            Err(error) => {
                return Err(invalid(format!("IFontDisp 字体槽读取失败：{error}")));
            }
        }
    }
    if stop.is_none() {
        // 16. 版本 >1：int16；版本 ≥3：16 字节原样 GUID；版本 ≥4：int16。
        if layout_version >= 2 {
            tail_short_1 = Some(trial.read_u16_le()?);
        }
        if layout_version >= 3 {
            tail_path_clsid = Some(trial.read_bytes(16)?.try_into().expect("固定 16 字节"));
        }
        if layout_version >= 4 {
            tail_short_2 = Some(trial.read_u16_le()?);
        }
    }

    let result = TextSymbolBody10_1 {
        version,
        color1,
        ints_1,
        bytes_1,
        doubles_1,
        ints_2,
        color2,
        group_8a,
        doubles_2,
        byte_2,
        double_3,
        int_3,
        double_4,
        byte_3,
        raw_path_clsid,
        text_path: match &background {
            Some(CalloutSlot10_1::BackReference { class_id, key }) => {
                NestedEntry10_1::BackReference {
                    class_id: *class_id,
                    key: *key,
                }
            }
            _ => NestedEntry10_1::Null,
        },
        background,
        symbol1: match &fill_symbol {
            SymbolSlotFull10_1::BackReference { class_id, key } => NestedEntry10_1::BackReference {
                class_id: *class_id,
                key: *key,
            },
            _ => NestedEntry10_1::Null,
        },
        fill_symbol,
        name_string,
        double_5,
        int_4,
        double_6,
        symbol2,
        font,
        tail_short_1,
        tail_path_clsid,
        tail_short_2,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 TextElement 对象体。
///
/// 字段序见模块文档；TextSymbol 符号槽经类分派递归。`max_bytes` 约束 EP 前
///缀与文本字符串；失败不推进游标、不修改登记表。
pub fn read_text_element_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<TextElementBody10_1> {
    let start = reader.position();
    let origin = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("TextElement 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. int16 版本：原生有符号 `>8` 拒绝（==9 特判为「8 + 尾加块」）；负值
    //    一并拒绝（较原生更严）。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_TEXT_ELEMENT_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出原生接受范围 0..=9（Load 0x6ce70e50 \
             有符号比较 >8 拒绝 0x80004005、==9 按 8+尾加块处理；负值位模式原生通过，\
             本读取器一并拒绝）"
        )));
    }

    // 2. ElementProperties 共同前缀（CustomProperty 槽停止时透传）。
    let (properties, mut stop) =
        match read_element_properties_prefix_10_1(&mut trial, max_bytes, &mut trial_table) {
            Ok(PrefixOutcome10_1::Complete(prefix)) => (Some(prefix), None),
            Ok(PrefixOutcome10_1::Stopped(reason)) => (None, Some(reason)),
            Err(error) => return Err(invalid(format!("ElementProperties 前缀读取失败：{error}"))),
        };

    // 3..9. 其余字段。
    let mut text = String::new();
    let mut raw_bool_int32 = 0;
    let mut symbol = None;
    let mut geometry = GeometrySlot10_1::Null;
    let mut tail_int32_1 = None;
    let mut tail_int32_2 = None;
    let mut decoration_block = None;
    let mut version9_block = None;

    if stop.is_none() {
        // 3. String 文本；4. int32 布尔。
        text = read_string_10_1(&mut trial, max_bytes)?;
        raw_bool_int32 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());

        // 5. LoadObject(ITextSymbol)：TextSymbol 新对象经类分派递归。
        let symbol_offset = trial.position();
        let saved_table = trial_table.clone();
        match read_object_entry_10_1(&mut trial, &mut trial_table)? {
            ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => {
                // 空标记/回引：无 TextSymbol 体（symbol 保持 None）。
            }
            ObjectEntry10_1::NewObject { class_id, key: _ } if class_id == TEXT_SYMBOL_CLASS_ID => {
                let body = match read_text_symbol_body_10_1(&mut trial, max_bytes, &mut trial_table)
                {
                    Ok(body) => body,
                    Err(error) => {
                        return Err(invalid(format!("TextSymbol 符号体读取失败：{error}")));
                    }
                };
                // 符号体内的嵌套停止上抛为 TextElement 的停止原因（游标已停
                // 在其内层停止点）。原实现未透传：样本恰在符号槽 #2 的 20 字
                // 节对象头停止，外层重读同一头恰好对位；接线轮闭合符号 #2 体
                // 后该头不再停留，必须传播否则外层会在符号体剩余字节上错位
                // 解析。
                if stop.is_none() {
                    if let Some(reason) = body.stop.clone() {
                        stop = Some(reason);
                    }
                }
                symbol = Some(body);
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                trial_table = saved_table;
                trial = rewind(&origin, start, symbol_offset);
                stop = Some(ElementBodyStop10_1::NestedObjectNotClosed {
                    slot: "ITextSymbol 符号槽",
                    offset: symbol_offset,
                    class_id,
                    key,
                });
            }
        }
    }

    if stop.is_none() {
        // 6. LoadObject(IGeometry)：接线轮起 Polyline 经几何类分派整体解析
        //    （样本 16 实例全为 Polyline 新对象，原停止点全部消除）；其余几
        //    何类维持既有停止语义。
        match read_geometry_slot(
            &mut trial,
            max_bytes,
            &mut trial_table,
            &origin,
            start,
            "IGeometry 文本几何槽",
            &mut stop,
        )? {
            Some(entry) => geometry = entry,
            None => {}
        }
    }

    if stop.is_none() {
        // 7. 版本 >2 int32；版本 >3 int32。
        if version > 2 {
            tail_int32_1 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        }
        if version > 3 {
            tail_int32_2 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        }
        // 8. 版本 ==7 精确块（第三轮接线：三装饰槽接 element_decor 分派；
        // 深度预算与符号槽同源，未知实现类维持停止回退）。
        if version == 7 {
            (decoration_block, stop) =
                read_text_decoration_block_10_1(&mut trial, max_bytes, &mut trial_table)?;
        }
        // 9. 版本 ==9 尾加块（F 轮接线：ISpatialReference 槽接三类闭合空间
        // 参考分派；其余实现类维持停止回退，语义与仅分类时一致）。
        if version == 9 && stop.is_none() {
            let mut block = Version9Block10_1 {
                spatial_reference: SpatialRefSlot10_1::Null,
                raw_int32: 0,
                raw_envelope: [0; 32],
            };
            let sr_offset = trial.position();
            let sr_snapshot = trial_table.clone();
            let slot = match read_object_entry_10_1(&mut trial, &mut trial_table)? {
                ObjectEntry10_1::Null => SpatialRefSlot10_1::Null,
                ObjectEntry10_1::BackReference { class_id, key } => {
                    SpatialRefSlot10_1::BackReference { class_id, key }
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    // F 轮接线：三类闭合空间参考（PCS/GCS/Unknown）三分派；
                    // 其余实现类回退到槽位起点、该键不保留，映射为停止。
                    use crate::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
                    use crate::objects::spatial_reference_slots::{
                        GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID, UNKNOWN_COORDINATE_SYSTEM_CLASS_ID,
                    };
                    if class_id == PROJECTED_COORDINATE_SYSTEM_CLASS_ID {
                        SpatialRefSlot10_1::Projected(
                            crate::objects::spatial_reference::read_projected_coordinate_system_10_1(
                                &mut trial,
                                max_bytes,
                            )?,
                        )
                    } else if class_id == GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID {
                        SpatialRefSlot10_1::Geographic(
                            crate::objects::spatial_reference_slots::read_geographic_coordinate_system_10_1(
                                &mut trial,
                                max_bytes,
                            )?,
                        )
                    } else if class_id == UNKNOWN_COORDINATE_SYSTEM_CLASS_ID {
                        SpatialRefSlot10_1::Unknown(
                            crate::objects::spatial_reference_slots::read_unknown_coordinate_system_10_1(
                                &mut trial,
                                max_bytes,
                            )?,
                        )
                    } else {
                        trial_table = sr_snapshot;
                        trial = rewind(&origin, start, sr_offset);
                        stop = Some(ElementBodyStop10_1::NestedObjectNotClosed {
                            slot: "ISpatialReference 槽（版本 9 尾加块）",
                            offset: sr_offset,
                            class_id,
                            key,
                        });
                        SpatialRefSlot10_1::Null
                    }
                }
            };
            block.spatial_reference = slot;
            if stop.is_none() {
                block.raw_int32 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
                block.raw_envelope = trial.read_bytes(32)?.try_into().expect("固定 32 字节");
                version9_block = Some(block);
            }
        }
    }

    let result = TextElementBody10_1 {
        version,
        properties,
        text,
        raw_bool_int32,
        symbol,
        geometry,
        tail_int32_1,
        tail_int32_2,
        decoration_block,
        version9_block,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 MarkerElement 对象体。
pub fn read_marker_element_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<MarkerElementBody10_1> {
    let start = reader.position();
    let origin = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("MarkerElement 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. int16 版本：原生有符号 >3 拒绝；负值一并拒绝。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_SHAPE_ELEMENT_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出原生接受范围 0..=3（Load 0x7504ea50 \
             有符号比较 >3 拒绝 0x80004005；负值位模式原生通过，本读取器一并拒绝）"
        )));
    }

    // 2. ElementProperties 前缀。
    let (properties, mut stop) =
        match read_element_properties_prefix_10_1(&mut trial, max_bytes, &mut trial_table) {
            Ok(PrefixOutcome10_1::Complete(prefix)) => (Some(prefix), None),
            Ok(PrefixOutcome10_1::Stopped(reason)) => (None, Some(reason)),
            Err(error) => return Err(invalid(format!("ElementProperties 前缀读取失败：{error}"))),
        };

    // 3. IMarkerSymbol 符号槽；4. IPoint 几何槽；5. 版本门尾段。
    let mut symbol = SymbolSlotFull10_1::Null;
    let mut geometry = PointSlot10_1::Null;
    let mut tail_int32 = None;
    let mut raw_envelope = None;

    if stop.is_none() {
        symbol = read_shape_symbol_slot(
            &mut trial,
            max_bytes,
            &mut trial_table,
            ShapeSymbol::Marker,
            &mut stop,
        )?;
    }
    if stop.is_none() {
        geometry = read_point_slot(
            &mut trial,
            max_bytes,
            &mut trial_table,
            &origin,
            start,
            "IPoint 几何槽",
            &mut stop,
        )?;
    }
    if stop.is_none() {
        if version > 1 {
            tail_int32 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        }
        if version > 2 {
            raw_envelope = Some(trial.read_bytes(32)?.try_into().expect("固定 32 字节"));
        }
    }

    let result = MarkerElementBody10_1 {
        version,
        properties,
        symbol,
        geometry,
        tail_int32,
        raw_envelope,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 LineElement 对象体。
pub fn read_line_element_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<LineElementBody10_1> {
    read_line_or_polygon_element(reader, max_bytes, table, ShapeSymbol::Line)
}

fn read_line_or_polygon_element(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    kind: ShapeSymbol,
) -> io::Result<LineOrPolygonElementBody10_1> {
    let class_name = match kind {
        ShapeSymbol::Fill => "PolygonElement",
        _ => "LineElement",
    };
    let start = reader.position();
    let origin = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("{class_name} 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. int16 版本：原生有符号 >3 拒绝；负值一并拒绝。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_SHAPE_ELEMENT_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出读取范围 0..=3"
        )));
    }

    // 2. ElementProperties 前缀。
    let (properties, mut stop) =
        match read_element_properties_prefix_10_1(&mut trial, max_bytes, &mut trial_table) {
            Ok(PrefixOutcome10_1::Complete(prefix)) => (Some(prefix), None),
            Ok(PrefixOutcome10_1::Stopped(reason)) => (None, Some(reason)),
            Err(error) => return Err(invalid(format!("ElementProperties 前缀读取失败：{error}"))),
        };

    // 依次读取符号槽、IGeometry 槽和版本条件尾段。
    let mut symbol = SymbolSlotFull10_1::Null;
    let mut geometry = GeometrySlot10_1::Null;
    let mut tail_int32 = None;
    let mut tail_short = None;

    if stop.is_none() {
        symbol = read_shape_symbol_slot(&mut trial, max_bytes, &mut trial_table, kind, &mut stop)?;
    }
    if stop.is_none() {
        match read_geometry_slot(
            &mut trial,
            max_bytes,
            &mut trial_table,
            &origin,
            start,
            "IGeometry 几何槽",
            &mut stop,
        )? {
            Some(slot) => geometry = slot,
            None => {}
        }
    }
    if stop.is_none() {
        if version > 1 {
            tail_int32 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        }
        if version > 2 {
            tail_short = Some(trial.read_u16_le()?);
        }
    }

    let result = LineElementBody10_1 {
        version,
        properties,
        symbol,
        geometry,
        tail_int32,
        tail_short,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}
