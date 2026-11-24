//! ArcMap 10.1 Geometry.dll 几何 coclass（esriGeometry）对象体的严格读取
//! 子集：Polyline / Polygon / Point / Multipoint。
//!
//! 证据来源：`docs/arcmap-10.1-geometry.md`（两轮研究记录；第二轮闭合
//! Polygon/Point/Multipoint 记录布局，并把 Path/Ring/Line/BezierCurve/
//! EllipticArc 的「流内持久化」从开放项改写为「不存在」的结论）。定位链在
//! Geometry.dll 10.1.0.3035 内三方互证闭环（全部静态反编译/汇编核对，未运
//! 行目标软件）：esri.clsid.xml 声明 Geometry.dll 模块类 → `.rdata` CLSID
//! 常量（各类常量地址见常量文档）→ `ESRI::*` RTTI TypeDescriptor（TD 名
//! 称串 `.?AV<类名>@ESRI@@`）→ COL → 持久化虚表 → GetClassID `movq` 复
//! 制 → Load/Save。
//!
//! ## Polyline/Polygon 共享包装字段序（Load `0x7192db20` 反编译逐行核对，
//! 确定的；Polygon 持久化虚表 `0x71a220d4` 的 Load/Save 槽与 Polyline 同值）
//!
//! 1. int16 版本 short：AfStreamRead 读 2 字节；`(short)版本 > 2` 拒绝
//!    `0x80004005`（经 `FUN_719f1570` 记录，有符号比较，负值位模式原生通
//!    过，本读取器一并拒绝，较原生更严）；AfStreamRead 返回 1（S_FALSE）
//!    时原生调用 IGeometry::SetEmpty 后返回 S_FALSE。受控流的零字节和
//!    一字节短读均触发该路径，字节读取器对短版本字段返回截断错误；
//! 2. int32 声明长度：AfStreamRead 读 4 字节；
//! 3. 该长度的记录字节块（原生 malloc 后 AfStreamRead 读入，无长度上限，
//!    分配失败记 `0x8007000e`；本读取器以 `max_bytes` 设限，较原生更严）
//!    交给内嵌接口（对象 `+0x24` 槽，持久化 this−0x8 视角）`+0x38` 方法
//!    导入，导入标志恒 `0x100`（对象类目 4 = Polygon 时导入入口改构造
//!    `ESRI::ShapePolygon`，标记 5；两者的 `+0x94` 解析槽为同一函数）；
//! 4. 版本 > 1：LoadObject 请求 IID `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`
//!    （ISpatialReference，与 Map::Load 第 4 步、Envelope 版本 3 槽同值），
//!    装入对象 `+0x10`（持久化 this−0x1c 视角）`+0x18` 槽。
//!
//! Save（`0x7192dcb0`，反编译核对）：`AfStreamGetDocumentVersion` → 写 2 字
//! 节**固定常量 `02 00`**（`.rdata 0x71a11e50`，不走版本表，10.1 恒写版本
//! 2）→ 导出记录（尺寸查询 + 填充两次调 `+0x10`/`+0x34` 槽，填充标志 =
//! docVer > 1 ? `0x400` : 0）→ 写 int32 长度 → 写记录字节 → SaveObject 空
//! 间参考。与 Load 版本 2 路径逐字段对称。
//!
//! ## Polyline/Polygon 记录字节块内部布局（共享解析器 `0x7197fc50`；证据：
//! 确定的）
//!
//! `ESRI::ShapePolyline::vftable`（基址 `0x71a245e4`）与
//! `ESRI::ShapePolygon::vftable`（基址 `0x71a244ac`）的 `+0x94` 槽为**同
//! 一函数** `0x7197fc50`（本轮直接读取两虚表槽位证实），按 shape 对象自身
//! 类型标记（Polyline 3 / Polygon 5）区分空几何回填值，字段布局完全一致。
//! 导入标志经 `& 0xfffffeff` 清除 `0x100` 位后恒为 0，因此 Z/M/段描述等可
//! 选段在读路径上不可达。布局（小端）：
//!
//! | 偏移 | 字段 | 说明 |
//! | --- | --- | --- |
//! | 0 | int32 类型 | 内部 shape 类别；Polyline 样本 16/16 = 3、Polygon 语义 5，0 = 空几何 |
//! | 4 | 4×f64 包络 | `WKSEnvelope::PutCoords(XMin, YMin, XMax, YMax)` 调用序 |
//! | 36 | int32 路径（环）数 | < 0 原生拒 `0x80040253` |
//! | 40 | int32 点数 | < 0 拒 `0x80040253`；== 1 拒 `0x80040221`（共享解析器对两类形状同一检查） |
//! | 44 | A×int32 | 路径（环）起始点序号数组（Polyline 样本恒 `[0]`；多环 Polygon 即 A>1 + 每环首点序号） |
//! | 44+4×A | B×(f64,f64) | 二维点坐标，包络由写侧重算（读侧不校验） |
//!
//! 类型 0（空几何）记录仅 4 字节、无任何后续字段（Polygon 分支回填标记 5、
//! Polyline 分支回填 3，读取器不关心该回填）。原生解析不校验类型值、也不校
//! 验计数与声明长度的一致性（越界即未定义行为）；本读取器对两者显式校验并
//! 拒绝，较原生更严。可选段（Load 标志恒 0 故不可达，仅反编译记录）：标志
//! `0x80000000` → Z 域 min/max 对 + 每点 Z；`0x40000000` → M 域对 + 每点
//! M；`0x20000000` → 段描述数组（段类型 1 = Line、4 = CircularArc、
//! 5 = BezierCurve，仅描述条目，非流内对象）；`0x10000000` → 每点 4 字节附
//! 加数组。坐标 < −1e38 时原生钳位为 `DAT_71a66f20` 占位值，本读取器原样保
//! 留位模式不钳位（与 Envelope 读取器同约定）。
//!
//! ## Point 记录字节块内部布局（Load `0x7191c1a0` → 内嵌导入 `0x7191bd30`；
//! 反编译逐行核对，确定的；样本 0 实例）
//!
//! wrapper 变体（与 Polyline/Polygon 同骨架）：版本门同上；int32 声明长度
//! 超过 `0x50` 直接拒 `0x80004005`（80 字节栈缓冲，无符号比较，无负值通
//! 过）；记录读入栈缓冲后经内嵌接口（对象 `+0x40` 槽，持久化 this−0x8 视角，虚表 `0x71a20fbc`）`+0x18` 槽转发（`0x71906090`，补写标志 0）至
//! `+0x38` 槽（`0x7191bd30`）解析；版本 > 1 的 LoadObject ISpatialReference
//! 段与 Polyline 相同。Save `0x7191c2b0` 写版本常量 `0x71a11e54`（`02 00`）
//! 与导出对称。布局（小端，坐标 < −1e38 原生钳位，读取器原样保留）：
//!
//! | 类型字 | 声明长度 | 字段 |
//! | --- | --- | --- |
//! | 恰 0 | 4 | 空点（原生 `*consumed = 4`，无坐标） |
//! | 恰 1 | 20 | f64 X（偏移 4）+ f64 Y（偏移 12） |
//! | 恰 9 | 28 | X + Y + f64 Z（偏移 20） |
//! | 恰 0xb | 36 | X + Y + f64 Z（偏移 20）+ f64 M（偏移 28） |
//! | 恰 0x15 | 28 | X + Y + f64 M（偏移 20） |
//! | 低字节 0x34 且 `& 0x2f000000` == 0 | 20 + 8·Z位 + 8·M位 + 4·ID位 | X + Y + 可选 Z（`0x80000000` 位，偏移 20）+ 可选 M（`0x40000000` 位，偏移 28）+ 可选 ID（`0x10000000` 位，u32） |
//!
//! 类型判别说明：恰值 1/9/0xb/0x15 按**全字**匹配（switch 精确比较，带高
//! 位的组合落默认分支）；低字节 0x34 的变体接受高标志位，但 `0x2f000000`
//! 掩码位（= 0x01000000/0x02000000/0x04000000/0x08000000/0x20000000）置位
//! 即原生拒 `0x8004020a`——该掩码**不含** `0x10000000` ID 位，ID 段可达。
//! 其余类型字原生拒绝，本读取器同拒。原生不校验声明长度与消费长度的关系
//! （wrapper 不回退）。本读取器拒绝短于记录布局的声明长度，保留已声明区间内的尾随字节。
//!
//! ## Multipoint 记录字节块内部布局（Load `0x718ff9c0` → 内嵌导入
//! `0x71906590`；反编译逐行核对，确定的；样本 0 实例）
//!
//! wrapper 变体（malloc 版，原生无长度上限，本读取器以 `max_bytes` 设限较
//! 原生更严）：内嵌接口在对象 `+0x3c` 槽（持久化 this−0x20 视角，虚表
//! `0x71a1ffa4`），`+0x18` → `+0x38` 槽（`0x71906590`）解析；版本门与
//! LoadObject ISpatialReference 段同 Polyline；Save `0x718ffb50` 写版本常量
//! `0x71a11e58`（`02 00`）。导入按**类型字低字节** switch（高标志位按全字
//! 判别；坐标 < −1e38 原生钳位，读取器原样保留）：
//!
//! | 低字节 | 声明长度 | 字段 |
//! | --- | --- | --- |
//! | 0 | 4 | 空点集（原生完全不读内容、consumed 不回写；读取器要求恰 4 字节较原生更严） |
//! | 1/9/0xb/0x15/0x34 | 20 + 8·Z + 8·M + 4·ID | f64 X（偏移 4）+ f64 Y（偏移 12）；Z 段 ⇔ 类型字恰为 9/0xb 或 `0x80000000` 位（偏移 20）；M 段 ⇔ 恰为 0x15/0xb 或 `0x40000000` 位；ID（u32）⇔ `0x10000000` 位；均按全字判别 |
//! | 8/0x12/0x14/0x1c/0x35 | 40 + 16N + (16+8N)·Z + (16+8N)·M + 4N·ID | 4×f64 包络（偏移 4）+ int32 点数 N（偏移 36）+ N×(f64,f64) 点集（偏移 40）+ Z 段（16 字节 min/max 对 + 每点 f64）⇔ 恰为 0x14/0x12 或 `0x80000000` 位 + M 段同构 ⇔ 恰为 0x1c/0x12 或 `0x40000000` 位 + 每点 u32 ID ⇔ `0x10000000` 位；段序 Z → M → ID |
//!
//! 其余低字节原生拒 `0x8004020a`，本读取器同拒。多点型点数 < 0 时原生静默
//! 跳过建点继续读尾随段（无显式错误），本读取器拒绝，较原生更严（与折线
//! 解析器的 `0x80040253` 对齐）。原生不校验声明长度一致性，本读取器要求严
//! 格相等，较原生更严。
//!
//! ## Path/Ring/Line/BezierCurve/EllipticArc/CircularArc：流内持久化不存在
//! （本轮结论，非缺口）
//!
//! 上轮表中这几类的「Load」（`0x719089e0` 等）经反编译为 **GetClassID 形
//! 态函数**（判空 `0x80004003` 后 movq 双 8 字节复制本类 CLSID 常量、返
//! 回 S_OK，不读流）；「Save 空桩 `0x71973ab0`」实为类身份虚表尾槽——两
//! 者所在的 `0x71a203bc`（Path）等虚表形状为「类身份虚表」：`[QI、共享
//! AddRef `0x719f1550`、共享 Release `0x718de060`、类名 getter（返回
//! L"Path"/L"Ring"/L"Line"…）@+0x0c、私有槽 @+0x10、GetClassID movq
//! @+0x14、空桩 @+0x18、…]`，与 Polyline 主虚表 `0x71a228c4` 同族（该族
//! GetClassID 同在 +0x14 非标准位、同一空桩在 +0x18）。本轮补证：
//!
//! 1. 全镜像（.rdata + .data）4 字节扫描：五类各自的 movq-GetClassID 函
//!    数**各仅出现 1 次**，位置即上述类身份虚表 +0x14 槽——不存在把它们的
//!    movq 放在 +0x0c（持久化形状 GetClassID 位）的第二个虚表，即不存在
//!    持久化形状虚表；
//! 2. 构造器逐指令核对（Path `0x719091f0` 一族 5 处、Ring `0x71961d70`、
//!    Line `0x718da350`）：对象初始化只安装类身份虚表 + 能力接口虚表
//!    （Path 17 个、Ring 20 个、Line 23 个写点，含两阶段覆写），**没有任何
//!    持久化虚表安装点**；Polyline/Polygon 共享的 `ESRI::PolyBase` 持久化
//!    虚表 `0x71a2173c`（TD `.?AVPolyBase@ESRI@@`，GetClassID = purecall
//!    `0x719f36fe`，Load/Save = `0x7192db20`/`0x7192dcb0`——这解释了「两
//!    类共用 Load/Save」：派生类覆写 GetClassID 而继承基类持久化实现）从
//!    未被这三族构造器安装；
//! 3. RTTI TD 名称串（独立字节扫描）：`.?AVPath@ESRI@@`（0x71a6a380）、
//!    `.?AVRing@ESRI@@`、`.?AVLine@ESRI@@`、`.?AVBezierCurve@ESRI@@`、
//!    `.?AVEllipticArc@ESRI@@` 均唯一命中并锚定上述类身份虚表；CircleArc
//!    —— `.?AVCircularArc@ESRI@@` 与 `.?AVCircularArc@@` 两种命名形态均
//!    **未命中**（维持上轮「虚表未定位」）。
//!
//! 结论：这五个类**不实现 IPersistStream**（构造性不存在，而非「Save 为空
//! 桩」）；它们从不独立出现在 MXD 流内（样本全流 CLSID 扫描 0 实例，两轮
//! 一致），段仅以「段描述数组条目」形态存在于折线/多边形记录的标志
//! `0x20000000` 变体中，而该标志在 Load 路径恒不可达（见上节）。上轮表
//! 中「Load/Save」列的归属为标注错误，本轮更正；「Path/segment 的
//! IPersistStream 体在样本中无事可做」按构造性不存在闭合。
//!
//! ## 分派与兼容入口
//!
//! - [`read_geometry_body_10_1`]：类型化分派入口（返回
//!   [`GeometryBody10_1`] 四类枚举），覆盖 Polyline/Polygon/Point/
//!   Multipoint；未知类 Unsupported（游标不推进、登记表不修改）。
//! - [`read_geometry_body_of_class_10_1`]：初版兼容入口（返回
//!   [`PolylineBody10_1`]）。IGeometry 槽整合接线（`element_body.rs`）当前
//!   仅按 Polyline 类标识调用本入口并取具体类型；签名保持不变，其余几何类
//!   仍返回 Unsupported 并提示改用类型化入口——切换工作属整合轮（见研究文
//!   档接线建议），本模块不越界改动共享文件。
//!
//! ## 样本对位（辅助观察）
//!
//! 仓库参考样本 /Maps 流（152,941 字节）全流 Geometry.dll 模块 CLSID 扫描
//! 两轮一致：Polyline 16 实例（对象头 @`0x18724`、`0x1dc0c`、`0x1ded9`、
//! `0x1e13a`、`0x1e39b`、`0x1fb2b`、`0x1fd90`、`0x1fff5`、`0x2155f`、
//! `0x217c4`、`0x22f8e`、`0x231f3`、`0x23458`、`0x249c2`、`0x24c27`、
//! `0x24e8c`）、Envelope 25 实例（envelope 轮闭合）、Polygon/Point/
//! Multipoint/Path/Ring/Line/BezierCurve/EllipticArc/CircularArc/MultiPatch
//! 均 **0 实例**（本轮扫描复核）。因此本轮新增读取器全部以合成测试验证，
//! 不声称样本验证；Polyline 16/16 实例逐字节走查吻合（版本 2、80 字节记录、
//! 16 字节全零空标记）为既有辅助观察。
//!
//! ## 调用约定与事务性
//!
//! 调用者已用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
//! 头并按类标识分派，从对象头之后的位置传入 Reader；本模块不重复读取对象
//! 头。全部字段在 trial 游标与 trial 登记表上读取，完整成功才提交；任何失
//! 败不推进调用者游标、不修改登记表。`max_bytes` 约束记录声明长度（先检查
//! 后读取），并原样转发给嵌套投影坐标系对象体；`usize::MAX` 表示不额外设
//! 限。这是仅覆盖上述布局的严格安全子集：不代表完整原生 Load 行为，通过
//! 也不代表 MXD 格式已验证。未接入 MXD 解析主链的部分（Polygon/Point/
//! Multipoint 分派）由整合轮接线，人工目视验收未做，不得声称已验证。
//!
//! ## 未解决问题（开放项单列）
//!
//! 1. 10.1 类型库区分记录的 esriShapeType 与公开 esriGeometryType。
//!    例如记录的 5 为 Polygon，公开类型的 Polygon 为 4；0x34/0x35 为
//!    GeneralPoint/GeneralMultipoint。读取器仍原样保留；尚未核对所有
//!    导入、导出分支是否接受每个变体。见第三轮 Display / Spatial 记录。
//! 2. 版本 0/1 的记录边界、无空间参考分支及 S_FALSE 短读已通过原生函数
//!    验证，仍缺对应旧 MXD 实档。读到完整版本字段后才解析几何记录。
//! 3. 可选段（Z/M/段描述/每点附加数组）在折线/多边形 Load 路径恒不可达
//!    （标志恒 0），仅反编译记录；Save 侧标志 `0x400`（docVer > 1）的导
//!    出差异未展开。Point/Multipoint 类型字高标志位的 Z/M/ID 段已按反编
//!    译闭合（合成测试覆盖），但同样无样本佐证。
//! 4. CircularArc 的持久化虚表与 RTTI 名称未定位（两种命名形态均未命中），
//!    维持上轮状态；其 CLSID 常量与 movq 站点（`0x717e8d73` 一带）仅候选。
//! 5. 几何内核移交链（对象 `+0xc8` 内嵌指针虚表 `+0x1cc` 槽）运行期装配，
//!    静态未定位；内核侧语义（路径/环起始的合法性校验等）未核。
//! 6. < −1e38 坐标的原生钳位（`DAT_71a66f20`）未在读取器中实现（原样保
//!    留）。Point/Multipoint 导入内的钳位分支同理。
//! 7. 嵌套空间参考仅支持投影坐标系；GeographicCoordinateSystem、
//!    UnknownCoordinateSystem 等无样本与分派证据。
//! 8. Polygon/Point/Multipoint 记录布局证据为静态反编译（确定的），但样本
//!    0 实例：全部由合成测试覆盖，未运行 ArcMap 10.1，未在 QGIS 中核对，
//!    不声称已验证。

use crate::objects::envelope::{
    ENVELOPE_CLASS_ID, Envelope10_1, SpatialReferenceEntry10_1, read_envelope_10_1,
    read_spatial_reference_entry_10_1,
};
use crate::objects::multi_patch::{MultiPatchRecord10_1, decode_multi_patch_record};
use crate::objects::spatial_reference::{
    PROJECTED_COORDINATE_SYSTEM_CLASS_ID, read_projected_coordinate_system_10_1,
};
use crate::objects::spatial_reference_slots::{
    GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID, UNKNOWN_COORDINATE_SYSTEM_CLASS_ID,
    read_geographic_coordinate_system_10_1, read_unknown_coordinate_system_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// 四类几何 Load 接受的版本上限：`(short)版本 > 2` 拒绝 `0x80004005`（有符
/// 号）。Polyline/Polygon 共用 Load `0x7192db20`，Point `0x7191c1a0`、
/// Multipoint `0x718ff9c0` 同门；Save 侧写固定常量 2（`0x71a11e50`/`0x71a11e54`/
/// `0x71a11e58` 三处常量字节均为 `02 00`，本轮逐一读取证实）。
const MAX_GEOMETRY_VERSION: i16 = 2;

/// 记录空几何布局的固定长度：仅 4 字节类型字段（共享解析器 `0x7197fc50`
/// 对类型 0 提前返回，`*consumed = 4`；Point 导入 `0x7191bd30` 的类型 0 同
/// 为 4 字节）。
const EMPTY_RECORD_LEN: usize = 4;

/// Polyline/Polygon 记录非空布局的固定头部：4 字节类型 + 32 字节包络 +
/// 2×int32 计数。
const FIXED_RECORD_HEADER: usize = 44;

/// Point 记录声明长度的原生上限：`Load 0x7191c1a0` 对 80 字节栈缓冲的长度
/// 门 `0x50 < 声明长度` 直接拒绝 `0x80004005`（无符号比较）。
const MAX_POINT_RECORD_LEN: u32 = 0x50;

/// Polyline 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{30707210-52D5-11D0-A8F2-00608C85EDE5}`。证据：
/// esri.clsid.xml 声明（模块 Geometry.dll，progID `Polyline`）；Geometry.dll
/// `.rdata` 常量 `0x71a1187c`（本轮独立字节扫描复核唯一命中）与 GetClassID
/// `0x719402d0` 的 movq 复制；仓库参考样本 /Maps 流 16 处对象头与本常量逐
/// 字节相同（辅助观察）。
pub const POLYLINE_CLASS_ID: [u8; 16] = [
    0x10, 0x72, 0x70, 0x30, 0xd5, 0x52, 0xd0, 0x11, 0xa8, 0xf2, 0x00, 0x60, 0x8c, 0x85, 0xed, 0xe5,
];

/// Polygon 类标识的 16 字节内存序，`{00A5CB42-52DA-11D0-A8F2-00608C85EDE5}`。
///
/// 证据：esri.clsid.xml 声明（progID `Polygon`）；`.rdata` 常量 `0x71a125f8`
/// （独立字节扫描唯一命中）；Polygon 持久化虚表 `0x71a220d4`（COL 锚定，
/// TD 名称串 `.?AVPolygon@ESRI@@`）Load/Save 槽与 Polyline 同函数；样本 /Maps
/// 流 0 实例（本轮全流扫描复核）。
pub const POLYGON_CLASS_ID: [u8; 16] = [
    0x42, 0xcb, 0xa5, 0x00, 0xda, 0x52, 0xd0, 0x11, 0xa8, 0xf2, 0x00, 0x60, 0x8c, 0x85, 0xed, 0xe5,
];

/// Point 类标识的 16 字节内存序，`{00A5CB41-52DA-11D0-A8F2-00608C85EDE5}`。
///
/// 证据：esri.clsid.xml 声明（progID `Point`）；`.rdata` 常量 `0x71a13a78`
/// （独立字节扫描唯一命中）；Point 持久化虚表 `0x71a21010`（TD
/// `.?AVPoint@ESRI@@`）Load `0x7191c1a0` / Save `0x7191c2b0`；样本 0 实例。
pub const POINT_CLASS_ID: [u8; 16] = [
    0x41, 0xcb, 0xa5, 0x00, 0xda, 0x52, 0xd0, 0x11, 0xa8, 0xf2, 0x00, 0x60, 0x8c, 0x85, 0xed, 0xe5,
];

/// Multipoint 类标识的 16 字节内存序，`{00A5CB40-52DA-11D0-A8F2-00608C85EDE5}`。
///
/// 证据：esri.clsid.xml 声明（progID `Multipoint`）；`.rdata` 常量
/// `0x71a1186c`（独立字节扫描唯一命中）；Multipoint 持久化虚表
/// `0x71a2010c`（TD `.?AVMultipoint@ESRI@@`）Load `0x718ff9c0` / Save
/// `0x718ffb50`；样本 0 实例。
pub const MULTIPOINT_CLASS_ID: [u8; 16] = [
    0x40, 0xcb, 0xa5, 0x00, 0xda, 0x52, 0xd0, 0x11, 0xa8, 0xf2, 0x00, 0x60, 0x8c, 0x85, 0xed, 0xe5,
];

/// 记录字节块的解析结果（Polyline/Polygon 共享解析器 `0x7197fc50` 标志 0
/// 布局；两类布局完全一致，仅空几何回填标记与语义名称不同）。
///
/// 兼容别名：[`PolylineRecord10_1`]（初版 Polyline 专用命名，保留供既有调
/// 用方使用）。
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeRecord10_1 {
    /// 记录首 4 字节类型，原样保留。
    ///
    /// 候选：内部 shape 类别（`ShapePolyline` 自身标记 3、`ShapePolygon` 5、
    /// 0 = 空几何），与 esriGeometryType 枚举的映射未闭合；Polyline 样本
    /// 16/16 为 3。原生不校验该值，本读取器同样不校验。
    pub kind: u32,
    /// 4 个小端 double，`WKSEnvelope::PutCoords(XMin, YMin, XMax, YMax)` 的
    /// 出参序（与 [`crate::objects::envelope`] 的字段序同源）。仅原样保留
    /// 位模式，不做 NaN、次序或范围判断（原生读侧不校验，样本 16/16 满足
    /// d0 ≤ d2、d1 ≤ d3 且各点在包络内）。
    pub envelope: [f64; 4],
    /// 路径（Polygon 语义：环）起始点序号数组（路径数在流内先于点数声明；
    /// Polyline 样本恒 `[0]`；多环 Polygon 即数组长度 > 1）。原生负值拒
    /// `0x80040253`，本读取器一并拒绝（u32 无负值位模式）。
    pub path_starts: Vec<u32>,
    /// 二维点坐标（点数来自记录计数；Polyline 样本恒 2）。原生拒
    /// 绝单点（`0x80040221`，共享解析器对折线与多边形同一检查），本读取器
    /// 同拒；坐标原样保留位模式、不钳位。
    pub points: Vec<[f64; 2]>,
    /// 记录声明区间内，原生二维布局未解释的尾随字节。
    pub trailing_bytes: Vec<u8>,
}

/// 兼容别名：Polyline 记录即共享 shape 记录（本轮起 Polygon 同构）。
pub type PolylineRecord10_1 = ShapeRecord10_1;

/// 10.1 Polyline 对象体（`IPersistStream::Load` 版本 2 布局）。
#[derive(Debug, Clone, PartialEq)]
pub struct PolylineBody10_1 {
    /// 2 字节版本 short 的无符号位模式。
    ///
    /// 原生有符号 `(short)版本 > 2` 拒绝 `0x80004005`；负值位模式原生通过
    /// （有符号比较），本读取器一并拒绝，较原生更严。Save 侧写固定常量
    /// 2；版本 0/1 原生只读到记录末尾，受控流已核对两个版本的对象边界。
    pub version: u16,
    /// 记录字节块的解析结果。类型 0 空几何时 [`ShapeRecord10_1`] 各集合字
    /// 段为空。
    pub record: ShapeRecord10_1,
    /// 版本 > 1 的嵌套空间参考条目；版本 0/1 无该段（`None`）。
    ///
    /// `Some([`SpatialReferenceEntry10_1::Null`])` 是版本 2 流内 16 字节全
    /// 零空引用标记（样本 16/16），`None` 表示该版本分支没有此段。
    pub spatial_reference: Option<SpatialReferenceEntry10_1>,
}

/// 10.1 Polygon 对象体（与 Polyline 共用 Load `0x7192db20` / Save
/// `0x7192dcb0` / 解析器 `0x7197fc50`，字段序完全一致）。
#[derive(Debug, Clone, PartialEq)]
pub struct PolygonBody10_1 {
    /// 2 字节版本 short 的无符号位模式（同 [`PolylineBody10_1::version`]）。
    pub version: u16,
    /// 记录字节块的解析结果（环语义：`path_starts` 为每环首点序号）。
    pub record: ShapeRecord10_1,
    /// 版本 > 1 的嵌套空间参考条目（语义循 Polyline，复用 envelope 读取器
    /// 的 [`SpatialReferenceEntry10_1`]）。
    pub spatial_reference: Option<SpatialReferenceEntry10_1>,
}

/// 10.1 Point 记录字节块的解析结果（导入函数 `0x7191bd30` 反编译核对）。
///
/// Point 无包络、无路径概念：X/Y 直接读自记录。类型字恰为 0 表示空点（仅
/// 4 字节）；恰为 1/9/0xb/0x15 的固定长度变体之后是「低字节 0x34 的带属性
/// 变体」（高标志位组合，见 [`POINT_CLASS_ID`] 模块文档表）；其余类型原生
/// 拒 `0x8004020a`。坐标原样保留位模式、不钳位（原生对 < −1e38 钳位）。
#[derive(Debug, Clone, PartialEq)]
pub struct PointRecord10_1 {
    /// 记录首 4 字节类型字，原样保留（接受清单见模块文档；语义标记为反编
    /// 译观察候选：1 = XY、9 = XYZ、0xb = XYZM、0x15 = XYM、低字节 0x34 =
    /// 带属性变体，0 = 空点）。
    pub kind: u32,
    /// X 坐标（记录偏移 4 的 f64）；空点为 `None`。
    pub x: Option<f64>,
    /// Y 坐标（记录偏移 12 的 f64）；空点为 `None`。
    pub y: Option<f64>,
    /// Z 坐标（类型 9/0xb 或 0x80000000 位时的下一字段）；无该段为 `None`。
    pub z: Option<f64>,
    /// M 坐标（类型 0x15/0xb 或 0x40000000 位时的下一字段）；无该段为
    /// `None`。
    pub m: Option<f64>,
    /// 点 ID（低字节 0x34 变体且置 `0x10000000` 位时位于末尾的 u32；恰值
    /// 1/9/0xb/0x15 的分支不读 ID）；无该段为 `None`。
    pub id: Option<u32>,
}

/// 10.1 Point 对象体（Load `0x7191c1a0`：声明长度 > 0x50 拒 `0x80004005`
/// 的 80 字节栈缓冲变体；内嵌导入 `0x71906090` → `0x7191bd30`，标志恒 0）。
#[derive(Debug, Clone, PartialEq)]
pub struct PointBody10_1 {
    /// 2 字节版本 short 的无符号位模式（同 [`PolylineBody10_1::version`]）。
    pub version: u16,
    /// 记录字节块的解析结果。
    pub record: PointRecord10_1,
    /// 版本 > 1 的嵌套空间参考条目（语义循 Polyline）。
    pub spatial_reference: Option<SpatialReferenceEntry10_1>,
}

/// 10.1 Multipoint 记录字节块的解析结果（导入函数 `0x71906590` 反编译核
/// 对；类型字按低字节分派，高标志位按全字判别）。
#[derive(Debug, Clone, PartialEq)]
pub enum MultipointRecord10_1 {
    /// 空点集：类型字低字节为 0（原生对该分支完全不读内容，`*consumed` 不
    /// 回写）。本读取器要求声明长度恰为 4 字节，较原生更严；类型字原样保
    /// 留（原生只看低字节）。
    Empty {
        /// 记录首 4 字节类型字原样保留（仅低字节参与原生判别）。
        kind: u32,
    },
    /// 单点型（低字节 1/9/0xb/0x15/0x34）：退化包络 `(x, y, x, y)`，无点
    /// 数组。Z 段存在当且仅当类型字恰为 9/0xb 或置 `0x80000000` 位；M 段
    /// 当且仅当恰为 0x15/0xb 或置 `0x40000000` 位；ID 段当且仅当置
    /// `0x10000000` 位（判别均按全字，原生如此）。
    Single {
        /// 记录首 4 字节类型字，原样保留。
        kind: u32,
        /// X 坐标（记录偏移 4）。
        x: f64,
        /// Y 坐标（记录偏移 12）。
        y: f64,
        /// 单点 Z（存在时位于记录偏移 20 的 f64）。
        z: Option<f64>,
        /// 单点 M（存在时位于 Z 之后的 f64，无 Z 则偏移 20）。
        m: Option<f64>,
        /// 单点 ID（存在时位于末尾的 u32）。
        id: Option<u32>,
    },
    /// 多点型（低字节 8/0x12/0x14/0x1c/0x35）：完整包络 + 点数 + 点集 +
    /// 可选尾随段。Z 段（min/max 对 + 每点 Z）当且仅当类型字恰为 0x14/0x12
    /// 或置 `0x80000000` 位；M 段当且仅当恰为 0x1c/0x12 或置 `0x40000000`
    /// 位；ID 段（每点 u32）当且仅当置 `0x10000000` 位。段序 Z → M → ID。
    Multi {
        /// 记录首 4 字节类型字，原样保留。
        kind: u32,
        /// 4 个小端 double 包络（`WKSEnvelope::PutCoords` 四参序）。
        envelope: [f64; 4],
        /// 点集（点数在记录偏移 36 声明；每点 16 字节 X/Y）。原生对负点数
        /// 静默跳过建点（无显式错误），本读取器拒绝，较原生更严。
        points: Vec<[f64; 2]>,
        /// Z 域 min/max 对（存在时为两个 f64）。
        z_range: Option<[f64; 2]>,
        /// 每点 Z（与 `points` 等长）。
        point_z: Vec<f64>,
        /// M 域 min/max 对（存在时为两个 f64）。
        m_range: Option<[f64; 2]>,
        /// 每点 M（与 `points` 等长）。
        point_m: Vec<f64>,
        /// 每点 ID（与 `points` 等长）。
        point_id: Vec<u32>,
    },
}

/// 10.1 Multipoint 对象体（Load `0x718ff9c0`：malloc 变体、原生无长度上限；
/// 内嵌导入 `0x71906090` → `0x71906590`，标志恒 0）。
#[derive(Debug, Clone, PartialEq)]
pub struct MultipointBody10_1 {
    /// 2 字节版本 short 的无符号位模式（同 [`PolylineBody10_1::version`]）。
    pub version: u16,
    /// 记录字节块的解析结果。
    pub record: MultipointRecord10_1,
    /// 版本 > 1 的嵌套空间参考条目（语义循 Polyline）。
    pub spatial_reference: Option<SpatialReferenceEntry10_1>,
}

/// Geometry.dll 的 GeometryBag 与 MultiPatch 类标识。
pub const GEOMETRY_BAG_CLASS_ID: [u8; 16] = [
    0xc0, 0xf5, 0xb5, 0x10, 0x81, 0x37, 0xd2, 0x11, 0xbc, 0xc5, 0, 0, 0xf8, 0x75, 0xbc, 0xce,
];
pub const MULTIPATCH_CLASS_ID: [u8; 16] = [
    0xc6, 0x41, 0xc0, 0xf3, 0x4d, 0xae, 0xd2, 0x11, 0x9c, 0x93, 0, 0xc0, 0x4f, 0xb1, 0x78, 0x38,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GeometryBagSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<GeometryBody10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeometryBag10_1 {
    pub version: u16,
    /// 子几何循环前的 IGeometry 缓存槽。
    pub cached_geometry: GeometryBagSlot10_1,
    pub members: Vec<GeometryBagSlot10_1>,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub flags_byte: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultiPatchBody10_1 {
    pub version: u16,
    /// 原生导入器接收的完整限定区间，包含可能存在的 v1 兼容包装。
    pub record_bytes: Vec<u8>,
    pub record: Option<MultiPatchRecord10_1>,
    pub spatial_reference: SpatialReferenceEntry10_1,
}

/// 按类标识分派的几何体读取结果（[`read_geometry_body_10_1`] 的返回）。
#[derive(Debug, Clone, PartialEq)]
pub enum GeometryBody10_1 {
    /// Polyline（样本 16/16 实例佐证）。
    Polyline(PolylineBody10_1),
    /// Polygon（与 Polyline 共享 Load/Save/解析器；样本 0 实例，合成测试覆盖）。
    Polygon(PolygonBody10_1),
    /// Point（样本 0 实例，合成测试覆盖）。
    Point(PointBody10_1),
    /// Multipoint（样本 0 实例，合成测试覆盖）。
    Multipoint(MultipointBody10_1),
    Envelope(Envelope10_1),
    GeometryBag(Box<GeometryBag10_1>),
    MultiPatch(Box<MultiPatchBody10_1>),
}

/// 初版兼容入口：按调用者已读取的对象头类标识分派几何体读取，仅闭合
/// Polyline（返回具体类型 [`PolylineBody10_1`]）。
///
/// `class_id` 必须是调用方从对象头取得的 16 字节类标识；起点为对象头之后
/// 的第一字节。Polygon/Point/Multipoint 的闭合读取走类型化入口
/// [`read_geometry_body_10_1`]；本入口对其余几何类（含未闭合的 Path/Ring/
/// Line/BezierCurve/EllipticArc/CircularArc——它们的流内持久化经反编译证
/// 实不存在，见模块文档结论节）返回 [`ErrorKind::Unsupported`]，游标不推
/// 进、登记表不修改。保持签名不变的原因：IGeometry 槽整合接线
/// （`element_body.rs`）按 Polyline 类标识调用本入口并取具体类型，签名变
/// 更属整合轮工作（研究文档接线建议）。
pub fn read_geometry_body_of_class_10_1(
    class_id: &[u8; 16],
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<PolylineBody10_1> {
    if *class_id == POLYLINE_CLASS_ID {
        read_polyline_body_10_1(reader, max_bytes, table)
    } else {
        let id: String = class_id.iter().map(|byte| format!("{byte:02x}")).collect();
        Err(io::Error::new(
            ErrorKind::Unsupported,
            format!(
                "几何类 {id} 不在兼容入口的闭合清单（仅 Polyline）；Polygon/Point/Multipoint \
                 及其他已支持类请改用 read_geometry_body_10_1 类型化分派"
            ),
        ))
    }
}

/// 类型化分派入口：从对象头之后读取可识别的 10.1 IGeometry 实现。
/// 未知类返回 `Unsupported`，失败不推进游标或修改登记表。
pub fn read_geometry_body_10_1(
    class_id: &[u8; 16],
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<GeometryBody10_1> {
    read_geometry_body(class_id, reader, max_bytes, table, 0)
}

pub(crate) fn read_keyed_geometry_body_10_1(
    class_id: &[u8; 16],
    key: u32,
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<GeometryBody10_1> {
    let body = read_geometry_body_10_1(class_id, reader, max_bytes, table)?;
    remember_geometry_marker_string(&body, key, table);
    Ok(body)
}

fn remember_geometry_marker_string(body: &GeometryBody10_1, key: u32, table: &mut RefTable10_1) {
    if let GeometryBody10_1::MultiPatch(body) = body
        && let Some(MultiPatchRecord10_1::Mesh(mesh)) = &body.record
        && mesh.xy.len() == 10
        && mesh.point_ids.first() == Some(&0x00b8_342d)
        && let Some(ids) = mesh.point_ids.get(1..9)
    {
        // v9 把点 ID 1–8 的大端字节依次扩展为 UTF-16 码元。
        let value = ids
            .iter()
            .flat_map(|id| id.to_be_bytes())
            .take_while(|byte| *byte != 0)
            .map(char::from)
            .collect();
        table.remember_multi_patch_marker_string(key, value);
    }
}

fn read_geometry_body(
    class_id: &[u8; 16],
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: u32,
) -> io::Result<GeometryBody10_1> {
    Ok(match *class_id {
        POLYLINE_CLASS_ID => {
            GeometryBody10_1::Polyline(read_polyline_body_10_1(reader, max_bytes, table)?)
        }
        POLYGON_CLASS_ID => {
            GeometryBody10_1::Polygon(read_polygon_body_10_1(reader, max_bytes, table)?)
        }
        POINT_CLASS_ID => GeometryBody10_1::Point(read_point_body_10_1(reader, max_bytes, table)?),
        MULTIPOINT_CLASS_ID => {
            GeometryBody10_1::Multipoint(read_multipoint_body_10_1(reader, max_bytes, table)?)
        }
        ENVELOPE_CLASS_ID => {
            GeometryBody10_1::Envelope(read_envelope_10_1(reader, max_bytes, table)?)
        }
        GEOMETRY_BAG_CLASS_ID => GeometryBody10_1::GeometryBag(Box::new(read_geometry_bag(
            reader, max_bytes, table, depth,
        )?)),
        MULTIPATCH_CLASS_ID => {
            GeometryBody10_1::MultiPatch(Box::new(read_multi_patch(reader, max_bytes, table)?))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("尚不支持几何类 {}", crate::storage::class_id_hex(class_id)),
            ));
        }
    })
}

/// GeometryBag::Load 0x71873be0 和 Save 0x71873dc0。
fn read_geometry_bag(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: u32,
) -> io::Result<GeometryBag10_1> {
    if depth >= 32 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "GeometryBag 嵌套超过限额",
        ));
    }
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let version = trial.read_u16_le()?;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的 GeometryBag 版本",
        ));
    }
    let count = trial.read_u32_le()? as i32;
    if count < 0 || count as usize > max_bytes.min(trial.remaining()) / 16 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "GeometryBag 几何数量超出范围",
        ));
    }
    let mut read_slot = || {
        let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
        let body = match entry {
            ObjectEntry10_1::NewObject { class_id, key } => {
                let body =
                    read_geometry_body(&class_id, &mut trial, max_bytes, &mut staged, depth + 1)?;
                remember_geometry_marker_string(&body, key, &mut staged);
                Some(Box::new(body))
            }
            _ => None,
        };
        Ok::<_, io::Error>(GeometryBagSlot10_1 { entry, body })
    };
    let cached_geometry = read_slot()?;
    let mut members = Vec::new();
    for _ in 0..count {
        members.push(read_slot()?);
    }
    let spatial_reference = read_spatial_reference_entry_10_1(&mut trial, max_bytes, &mut staged)?;
    let flags_byte = trial.read_bytes(1)?[0];
    *reader = trial;
    *table = staged;
    Ok(GeometryBag10_1 {
        version,
        cached_geometry,
        members,
        spatial_reference,
        flags_byte,
    })
}

/// MultiPatch::Load 0x718e4610。可识别的面片从限定区间解码。
fn read_multi_patch(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<MultiPatchBody10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let version = trial.read_u16_le()?;
    if !(1..=2).contains(&version) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的 MultiPatch 版本",
        ));
    }
    let size = read_and_gate_declared_length_10_1(&mut trial, start, "MultiPatch", max_bytes)?;
    let minimum = if version == 1 { 12 } else { 4 };
    if size < minimum {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "MultiPatch 记录短于其版本所需前导",
        ));
    }
    let record_bytes = trial.read_bytes(record_len(size))?.to_vec();
    let record = decode_multi_patch_record(version, &record_bytes, max_bytes)?;
    let spatial_reference = read_spatial_reference_entry_10_1(&mut trial, max_bytes, &mut staged)?;
    *reader = trial;
    *table = staged;
    Ok(MultiPatchBody10_1 {
        version,
        record_bytes,
        record,
        spatial_reference,
    })
}

/// 从对象头之后的位置读取 10.1 Polyline 对象体的严格子集。
///
/// 字段序与证据见模块文档。`max_bytes` 约束记录声明长度（在任何读取与分配
/// 之前检查）并原样转发给嵌套投影坐标系对象体；原生无此限额（malloc 不设
/// 上限），本读取器设限较原生更严。任何失败不推进游标、不修改登记表；错
/// 误位置相对于传入 Reader，调用方需另外记录流和对象身份。
///
/// 声明区间短于布局、点数为 1、路径数或点数为负位模式时返回
/// [`ErrorKind::InvalidData`]（原生或拒绝同型输入、或存在越界未定义行为，
/// 本读取器一律显式拒绝）。这是严格安全子集，通过不代表 MXD 格式已验证。
pub fn read_polyline_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<PolylineBody10_1> {
    let (version, record, spatial_reference) =
        read_polycurve_parts_10_1(reader, max_bytes, table, "Polyline", "路径")?;
    Ok(PolylineBody10_1 {
        version,
        record,
        spatial_reference,
    })
}

/// 从对象头之后的位置读取 10.1 Polygon 对象体的严格子集。
///
/// 与 [`read_polyline_body_10_1`] 共用同一 wrapper 与记录布局（Polyline 与
/// Polygon 共享 Load `0x7192db20` / Save `0x7192dcb0` / 解析器 `0x7197fc50`，
/// 证据见模块文档）；差异仅在语义名称（路径 → 环）与错误文案。样本 0 实
/// 例，全部结论为静态反编译、合成测试覆盖。
pub fn read_polygon_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<PolygonBody10_1> {
    let (version, record, spatial_reference) =
        read_polycurve_parts_10_1(reader, max_bytes, table, "Polygon", "环")?;
    Ok(PolygonBody10_1 {
        version,
        record,
        spatial_reference,
    })
}

/// 从对象头之后的位置读取 10.1 Point 对象体的严格子集。
///
/// wrapper 为 80 字节栈缓冲变体：声明长度超过原生上限 [`MAX_POINT_RECORD_LEN`]
/// 拒绝（Load `0x7191c1a0` 原生拒 `0x80004005`，同型输入同拒）；其余约定
/// （事务性、max_bytes、嵌套空间参考语义）循 [`read_polyline_body_10_1`]。
/// 样本 0 实例，全部结论为静态反编译、合成测试覆盖。
pub fn read_point_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<PointBody10_1> {
    let label = "Point";
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("{label} 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    let version = read_and_gate_version_10_1(&mut trial, start, label)?;
    let declared = read_and_gate_declared_length_10_1(&mut trial, start, label, max_bytes)?;
    if declared > MAX_POINT_RECORD_LEN {
        return Err(invalid(format!(
            "记录声明长度 {declared} 超过 Point::Load 原生上限 0x50（Load 0x7191c1a0 的 \
             80 字节栈缓冲长度门拒 0x80004005），本读取器同拒"
        )));
    }
    let record_offset = trial.position();
    let record = {
        let mut window = trial
            .read_region(record_len(declared))
            .map_err(|error| invalid(format!("记录声明长度 {declared} 超出剩余字节：{error}")))?;
        read_point_record_10_1(&mut window, record_len(declared)).map_err(invalid)?
    };
    debug_assert_eq!(trial.position(), record_offset + record_len(declared));

    let spatial_reference = read_nested_spatial_reference_10_1(
        &mut trial,
        &mut trial_table,
        max_bytes,
        version,
        start,
        label,
    )?;
    *reader = trial;
    *table = trial_table;
    Ok(PointBody10_1 {
        version,
        record,
        spatial_reference,
    })
}

/// 从对象头之后的位置读取 10.1 Multipoint 对象体的严格子集。
///
/// wrapper 为 malloc 变体：原生对声明长度无上限（`0x718ff9c0` 直接
/// malloc），本读取器以 `max_bytes` 设限较原生更严；其余约定循
/// [`read_polyline_body_10_1`]。样本 0 实例，全部结论为静态反编译、合成
/// 测试覆盖。
pub fn read_multipoint_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<MultipointBody10_1> {
    let label = "Multipoint";
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("{label} 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    let version = read_and_gate_version_10_1(&mut trial, start, label)?;
    let declared = read_and_gate_declared_length_10_1(&mut trial, start, label, max_bytes)?;
    let record_offset = trial.position();
    let record = {
        let mut window = trial
            .read_region(record_len(declared))
            .map_err(|error| invalid(format!("记录声明长度 {declared} 超出剩余字节：{error}")))?;
        read_multipoint_record_10_1(&mut window, record_len(declared)).map_err(invalid)?
    };
    debug_assert_eq!(trial.position(), record_offset + record_len(declared));

    let spatial_reference = read_nested_spatial_reference_10_1(
        &mut trial,
        &mut trial_table,
        max_bytes,
        version,
        start,
        label,
    )?;
    *reader = trial;
    *table = trial_table;
    Ok(MultipointBody10_1 {
        version,
        record,
        spatial_reference,
    })
}

/// u32 声明长度转 usize（32 位平台上位模式为负的声明不会走到这里——读侧
/// 以 u32 承载，无负值；转 usize 在 16 位平台不可行，本项目目标平台 ≥32 位）。
fn record_len(declared: u32) -> usize {
    declared as usize
}

/// 步骤 1（四类共享）：int16 版本 short + 原生有符号 >2 拒绝；负值位模式
/// 一并拒绝（较原生更严）。
fn read_and_gate_version_10_1(
    trial: &mut Reader<'_>,
    start: usize,
    label: &str,
) -> io::Result<u16> {
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_GEOMETRY_VERSION || (version as i16) < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{label} 对象体 offset 0x{start:x}: 版本位模式 {version}（0x{version:04x}）超出\
                 原生接受范围 0..=2（Load 0x7192db20 / 0x7191c1a0 / 0x718ff9c0 的有符号比较 \
                 >2 拒绝 0x80004005；负值位模式原生通过，本读取器一并拒绝）"
            ),
        ));
    }
    Ok(version)
}

/// 步骤 2（四类共享）：int32 声明长度；先限额后读取（原生 malloc 无上限，
/// 本读取器设限较原生更严）。
fn read_and_gate_declared_length_10_1(
    trial: &mut Reader<'_>,
    start: usize,
    label: &str,
    max_bytes: usize,
) -> io::Result<u32> {
    let declared = trial.read_u32_le()?;
    if declared as usize > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{label} 对象体 offset 0x{start:x}: 记录声明长度 {declared} 超过资源限额 \
                 {max_bytes}（原生无长度上限，本读取器设限较原生更严）"
            ),
        ));
    }
    Ok(declared)
}

/// 步骤 4（四类共享）：版本 > 1 的嵌套空间参考条目（类身份完全由流内对象
/// 头决定；语义循 envelope 读取器：Null/回引原样返回，新键按 F 轮三分派
/// 认投影/地理/未知坐标系，其余类 Unsupported——嵌套对象体无外部长度字段，
/// 无法跳过未知类继续读取）。版本 ≤ 1 原生按 `jl` 跳过该段（无样本，合成
/// 测试覆盖），返回 `None`。
fn read_nested_spatial_reference_10_1(
    trial: &mut Reader<'_>,
    trial_table: &mut RefTable10_1,
    max_bytes: usize,
    version: u16,
    start: usize,
    label: &str,
) -> io::Result<Option<SpatialReferenceEntry10_1>> {
    if version <= 1 {
        return Ok(None);
    }
    let nested_offset = trial.position();
    match read_object_entry_10_1(trial, trial_table)? {
        ObjectEntry10_1::Null => Ok(Some(SpatialReferenceEntry10_1::Null)),
        ObjectEntry10_1::BackReference { class_id, key } => {
            Ok(Some(SpatialReferenceEntry10_1::BackReference {
                class_id,
                key,
            }))
        }
        ObjectEntry10_1::NewObject { class_id, key }
            if class_id == PROJECTED_COORDINATE_SYSTEM_CLASS_ID =>
        {
            let body = read_projected_coordinate_system_10_1(trial, max_bytes)?;
            Ok(Some(SpatialReferenceEntry10_1::Projected { key, body }))
        }
        ObjectEntry10_1::NewObject { class_id, key }
            if class_id == GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID =>
        {
            // 地理坐标系（F 轮接线三分派）。
            let body = read_geographic_coordinate_system_10_1(trial, max_bytes)?;
            Ok(Some(SpatialReferenceEntry10_1::Geographic { key, body }))
        }
        ObjectEntry10_1::NewObject { class_id, key }
            if class_id == UNKNOWN_COORDINATE_SYSTEM_CLASS_ID =>
        {
            // 未知坐标系（F 轮接线三分派）。
            let body = read_unknown_coordinate_system_10_1(trial, max_bytes)?;
            Ok(Some(SpatialReferenceEntry10_1::UnknownCoordinateSystem {
                key,
                body,
            }))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            let id: String = class_id.iter().map(|byte| format!("{byte:02x}")).collect();
            Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "{label} 对象体嵌套空间参考 offset 0x{nested_offset:x}（对象体 offset \
                     0x{start:x}）: 类标识 {id}（键 {key}）不是已闭合的空间参考类；本读取器\
                     仅支持投影/地理/未知坐标系嵌套（F 轮三分派），且对象体无外部长度字段，\
                     无法跳过未知类继续读取"
                ),
            ))
        }
    }
}

/// Polyline/Polygon 共享的 wrapper 内部实现（Load `0x7192db20`）：返回
/// （版本、记录解析结果、嵌套空间参考）。`path_term` 为错误文案中的计数名
/// 称（"路径" / "环"）。事务性：trial 游标 + trial 登记表，完整成功才提交。
fn read_polycurve_parts_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    label: &str,
    path_term: &str,
) -> io::Result<(u16, ShapeRecord10_1, Option<SpatialReferenceEntry10_1>)> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("{label} 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    // trial 游标 + trial 登记表：嵌套空间参考条目在其对象体之前登记新键，
    // 失败必须整体回滚（与 envelope 读取器同约定）。
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    let version = read_and_gate_version_10_1(&mut trial, start, label)?;
    let declared = read_and_gate_declared_length_10_1(&mut trial, start, label, max_bytes)?;

    // 步骤 3：记录字节块在 read_region 有界子窗口上解析（长度受声明长度约
    // 束，read_region 失败即截断错误）。布局见模块文档；原生不校验计数与长
    // 度的一致性，本读取器显式校验（较原生更严）。
    let record_len = declared as usize;
    let record_offset = trial.position();
    let record = {
        let mut window = trial
            .read_region(record_len)
            .map_err(|error| invalid(format!("记录声明长度 {record_len} 超出剩余字节：{error}")))?;
        read_shape_record_10_1(&mut window, record_len, path_term).map_err(invalid)?
    };
    debug_assert_eq!(trial.position(), record_offset + record_len);

    let spatial_reference = read_nested_spatial_reference_10_1(
        &mut trial,
        &mut trial_table,
        max_bytes,
        version,
        start,
        label,
    )?;

    *reader = trial;
    *table = trial_table;
    Ok((version, record, spatial_reference))
}

/// 布局长度严格相等校验（记录解析器共用）。原生不校验声明长度与消费长度的
/// 关系（wrapper 不回退，声明尾随字节成为死空间）；本读取器要求严格相等，
/// 较原生更严。
fn require_exact_len(record_len: usize, expected: usize, what: &str) -> Result<(), String> {
    if record_len != expected {
        return Err(format!(
            "记录长度 {record_len} 与布局常量（{what} = {expected}）不一致；原生不校验声明\
             长度与消费长度的关系（wrapper 不回退，声明尾随字节成为死空间），本读取器要求\
             严格相等，较原生更严"
        ));
    }
    Ok(())
}

/// 在有界窗口上解析 Polyline/Polygon 记录字节块（共享解析器 `0x7197fc50`
/// 标志 0 布局）。
///
/// `record_len` 为包装声明的长度；空几何（类型 0）消费类型字段，非空布局
/// 至少需要 `44 + 路径数×4 + 点数×16` 字节。`path_term` 为错误文案中的
/// 计数名称（"路径" / "环"）。返回 `Err(String)` 表示布局违规（由调用方包
/// 装为 InvalidData）。
fn read_shape_record_10_1(
    window: &mut Reader<'_>,
    record_len: usize,
    path_term: &str,
) -> Result<ShapeRecord10_1, String> {
    let read_u32 =
        |window: &mut Reader<'_>| window.read_u32_le().map_err(|error| error.to_string());
    let read_f64 =
        |window: &mut Reader<'_>| window.read_f64_le().map_err(|error| error.to_string());
    let kind = read_u32(window)?;
    if kind == 0 {
        return Ok(ShapeRecord10_1 {
            kind,
            envelope: [0.0; 4],
            path_starts: Vec::new(),
            points: Vec::new(),
            trailing_bytes: window
                .read_bytes(window.remaining())
                .map_err(|e| e.to_string())?
                .to_vec(),
        });
    }
    if record_len < FIXED_RECORD_HEADER {
        return Err(format!(
            "记录声明长度 {record_len} 不足以容纳非空布局头部 44 字节"
        ));
    }
    let mut envelope = [0.0; 4];
    for slot in &mut envelope {
        *slot = read_f64(window)?;
    }
    let path_count = read_u32(window)?;
    let point_count = read_u32(window)?;
    let expected =
        FIXED_RECORD_HEADER as u64 + u64::from(path_count) * 4 + u64::from(point_count) * 16;
    if expected > record_len as u64 || path_count > i32::MAX as u32 || point_count > i32::MAX as u32
    {
        return Err(format!(
            "记录长度 {record_len} 或计数无效（44 + {path_term}数 {path_count}×4 + 点数 {point_count}×16 = {expected}）"
        ));
    }
    if point_count == 1 {
        return Err(format!(
            "点数 1：原生共享解析器（ShapePolyline::Parse == ShapePolygon::Parse，0x7197fc50）\
             对任何形状类别拒绝（0x80040221，至少两点），本读取器同拒"
        ));
    }
    let mut path_starts = Vec::new();
    for _ in 0..path_count {
        path_starts.push(read_u32(window)?);
    }
    let mut points = Vec::new();
    for _ in 0..point_count {
        let x = read_f64(window)?;
        let y = read_f64(window)?;
        points.push([x, y]);
    }
    Ok(ShapeRecord10_1 {
        kind,
        envelope,
        path_starts,
        points,
        trailing_bytes: window
            .read_bytes(window.remaining())
            .map_err(|e| e.to_string())?
            .to_vec(),
    })
}

/// 在有界窗口上解析 Point 记录字节块（导入函数 `0x7191bd30`，标志恒 0）。
///
/// 类型字恰为 0 的空点要求恰 4 字节；恰值 1/9/0xb/0x15 与低字节 0x34 的带
/// 属性变体的期望长度见模块文档表。坐标原样保留位模式、不钳位。
fn read_point_record_10_1(
    window: &mut Reader<'_>,
    record_len: usize,
) -> Result<PointRecord10_1, String> {
    let read_u32 =
        |window: &mut Reader<'_>| window.read_u32_le().map_err(|error| error.to_string());
    let read_f64 =
        |window: &mut Reader<'_>| window.read_f64_le().map_err(|error| error.to_string());
    if record_len == EMPTY_RECORD_LEN {
        let kind = read_u32(window)?;
        if kind != 0 {
            return Err(format!(
                "空点记录（长度 4）的类型位模式 {kind} 非 0；原生对类型 0 之外的值会继续读取\
                 X/Y（越过声明缓冲，未定义行为），本读取器拒绝"
            ));
        }
        return Ok(PointRecord10_1 {
            kind,
            x: None,
            y: None,
            z: None,
            m: None,
            id: None,
        });
    }
    if record_len < 20 {
        return Err(format!(
            "记录声明长度 {record_len} 不足以容纳非空布局头部 20 字节（类型字 + X + Y）"
        ));
    }
    let kind = read_u32(window)?;
    let x = read_f64(window)?;
    let y = read_f64(window)?;
    // Z/M/ID 存在性与期望长度按全字类型位模式判定（原生语义，见模块文档表）。
    let (z, m, id) = match kind {
        1 => {
            require_exact_len(record_len, 20, "类型字 1（X + Y）")?;
            (None, None, None)
        }
        9 => {
            require_exact_len(record_len, 28, "类型字 9（X + Y + Z）")?;
            (Some(read_f64(window)?), None, None)
        }
        0xb => {
            require_exact_len(record_len, 36, "类型字 0xb（X + Y + Z + M）")?;
            (Some(read_f64(window)?), Some(read_f64(window)?), None)
        }
        0x15 => {
            require_exact_len(record_len, 28, "类型字 0x15（X + Y + M）")?;
            (None, Some(read_f64(window)?), None)
        }
        _ => {
            // 低字节 0x34 的带属性变体（原生默认分支）：0x2f000000 掩码位
            // （0x01000000/0x02000000/0x04000000/0x08000000/0x20000000）全零
            // 才接受；`0x10000000` ID 位不在该掩码内，可读（u32，位于末尾）。
            if kind & 0xff != 0x34 {
                return Err(format!(
                    "类型位模式 0x{kind:08x} 不在 Point::Load 接受清单（恰值 0、1、9、0xb、\
                     0x15 与低字节 0x34 的带属性变体；原生其余值拒绝 0x8004020a），本读取器同拒"
                ));
            }
            if kind & 0x2f000000 != 0 {
                return Err(format!(
                    "类型位模式 0x{kind:08x} 含 0x2f000000 掩码位；原生默认分支直接拒绝 \
                     0x8004020a（该掩码 = 0x01000000/0x02000000/0x04000000/0x08000000/\
                     0x20000000，不含 0x10000000 ID 位），本读取器同拒"
                ));
            }
            let has_z = kind & 0x80000000 != 0;
            let has_m = kind & 0x40000000 != 0;
            let has_id = kind & 0x10000000 != 0;
            let expected = 20
                + if has_z { 8 } else { 0 }
                + if has_m { 8 } else { 0 }
                + if has_id { 4 } else { 0 };
            require_exact_len(
                record_len,
                expected,
                &format!("类型字 0x{kind:08x}（X + Y + 可选 Z/M/ID）"),
            )?;
            let z = if has_z { Some(read_f64(window)?) } else { None };
            let m = if has_m { Some(read_f64(window)?) } else { None };
            let id = if has_id {
                Some(read_u32(window)?)
            } else {
                None
            };
            (z, m, id)
        }
    };
    debug_assert_eq!(window.remaining(), 0);
    Ok(PointRecord10_1 {
        kind,
        x: Some(x),
        y: Some(y),
        z,
        m,
        id,
    })
}

/// 在有界窗口上解析 Multipoint 记录字节块（导入函数 `0x71906590`，标志恒
/// 0）。类型字按低字节分派（空点集低字节 0、单点型 1/9/0xb/0x15/0x34、多
/// 点型 8/0x12/0x14/0x1c/0x35），高标志位（Z/M/ID 段）按全字判别（原生语
/// 义）。多点型点数负位模式拒绝（原生静默跳过，较原生更严）；期望长度用
/// u64 算术防溢出（点数为 u32 全域时乘积仍远小于 u64 上界）。
fn read_multipoint_record_10_1(
    window: &mut Reader<'_>,
    record_len: usize,
) -> Result<MultipointRecord10_1, String> {
    let read_u32 =
        |window: &mut Reader<'_>| window.read_u32_le().map_err(|error| error.to_string());
    let read_f64 =
        |window: &mut Reader<'_>| window.read_f64_le().map_err(|error| error.to_string());
    if record_len < EMPTY_RECORD_LEN {
        return Err(format!(
            "记录声明长度 {record_len} 不足以容纳类型字（4 字节）"
        ));
    }
    let kind = read_u32(window)?;
    if record_len == EMPTY_RECORD_LEN {
        // 空点集：原生只看低字节（switch(uVar1 & 0xff) case 0），内容完全不
        // 读。读取器要求恰 4 字节（死空间拒绝较原生更严），类型字原样保留。
        if kind & 0xff != 0 {
            return Err(format!(
                "类型位模式 0x{kind:08x} 搭配 4 字节记录无法容纳坐标；原生会越过声明缓冲\
                 继续（未定义行为），本读取器拒绝"
            ));
        }
        return Ok(MultipointRecord10_1::Empty { kind });
    }
    if kind & 0xff == 0 {
        return Err(format!(
            "空点集记录（类型字低字节 0）的声明长度 {record_len} ≠ 4；原生对该分支完全不读\
             取内容（wrapper 按声明长度推进），本读取器要求恰 4 字节，较原生更严"
        ));
    }
    if record_len < 20 {
        return Err(format!(
            "记录声明长度 {record_len} 不足以容纳单点布局头部 20 字节（类型字 + X + Y）"
        ));
    }
    let low = kind & 0xff;
    if matches!(low, 0x01 | 0x09 | 0x0b | 0x15 | 0x34) {
        // 单点型：退化包络 (x, y, x, y)，无点数组。
        let x = read_f64(window)?;
        let y = read_f64(window)?;
        let has_z = kind == 9 || kind == 0xb || kind & 0x80000000 != 0;
        let has_m = kind == 0x15 || kind == 0xb || kind & 0x40000000 != 0;
        let has_id = kind & 0x10000000 != 0;
        let expected =
            20 + if has_z { 8 } else { 0 } + if has_m { 8 } else { 0 } + if has_id { 4 } else { 0 };
        require_exact_len(record_len, expected, &format!("单点类型字 0x{kind:08x}"))?;
        let z = if has_z { Some(read_f64(window)?) } else { None };
        let m = if has_m { Some(read_f64(window)?) } else { None };
        let id = if has_id {
            Some(read_u32(window)?)
        } else {
            None
        };
        debug_assert_eq!(window.remaining(), 0);
        return Ok(MultipointRecord10_1::Single {
            kind,
            x,
            y,
            z,
            m,
            id,
        });
    }
    if !matches!(low, 0x08 | 0x12 | 0x14 | 0x1c | 0x35) {
        return Err(format!(
            "类型字低字节 0x{low:02x}（全字 0x{kind:08x}）不在 Multipoint::Load 接受清单\
             （空点集低字节 0、单点型 1/9/0xb/0x15/0x34、多点型 8/0x12/0x14/0x1c/0x35；\
             原生其余值拒绝 0x8004020a），本读取器同拒"
        ));
    }
    // 多点型：完整包络 + 点数 + 点集 + 可选尾随段（段序 Z → M → ID）。
    if record_len < 40 {
        return Err(format!(
            "记录声明长度 {record_len} 不足以容纳多点布局头部 40 字节（类型字 + 包络 + 点数）"
        ));
    }
    let mut envelope = [0.0; 4];
    for slot in &mut envelope {
        *slot = read_f64(window)?;
    }
    let count = read_u32(window)?;
    if (count as i32) < 0 {
        return Err(format!(
            "点数 {count}（负位模式）：原生对负点数静默跳过建点继续读尾随段（无显式错误），\
             本读取器拒绝，较原生更严（与折线解析器 0x80040253 对齐）"
        ));
    }
    let has_z = kind == 0x14 || kind == 0x12 || kind & 0x80000000 != 0;
    let has_m = kind == 0x1c || kind == 0x12 || kind & 0x40000000 != 0;
    let has_id = kind & 0x10000000 != 0;
    let count64 = count as u64;
    let mut expected = 40u64 + 16 * count64;
    if has_z {
        expected += 16 + 8 * count64;
    }
    if has_m {
        expected += 16 + 8 * count64;
    }
    if has_id {
        expected += 4 * count64;
    }
    require_exact_len(
        record_len,
        expected as usize,
        &format!("多点类型字 0x{kind:08x}（40 + 16×点数 {count} + 尾随段）",),
    )?;
    let mut points = Vec::new();
    for _ in 0..count {
        let x = read_f64(window)?;
        let y = read_f64(window)?;
        points.push([x, y]);
    }
    let (z_range, point_z) = if has_z {
        let z_min = read_f64(window)?;
        let z_max = read_f64(window)?;
        let mut values = Vec::new();
        for _ in 0..count {
            values.push(read_f64(window)?);
        }
        (Some([z_min, z_max]), values)
    } else {
        (None, Vec::new())
    };
    let (m_range, point_m) = if has_m {
        let m_min = read_f64(window)?;
        let m_max = read_f64(window)?;
        let mut values = Vec::new();
        for _ in 0..count {
            values.push(read_f64(window)?);
        }
        (Some([m_min, m_max]), values)
    } else {
        (None, Vec::new())
    };
    let point_id = if has_id {
        let mut values = Vec::new();
        for _ in 0..count {
            values.push(read_u32(window)?);
        }
        values
    } else {
        Vec::new()
    };
    debug_assert_eq!(window.remaining(), 0);
    Ok(MultipointRecord10_1::Multi {
        kind,
        envelope,
        points,
        z_range,
        point_z,
        m_range,
        point_m,
        point_id,
    })
}
