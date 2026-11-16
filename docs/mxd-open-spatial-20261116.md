# MXD 解析空间参考与 Geometry 未完成项（2026-11-16）

本报告对应并行处理分派中的第七组。TODO 行号按分派时的原始 TODO 快照记录；不改 TODO。范围包括原行 108 的 Map 字段归属部分、226–227、230–233、236、265–266、270、282–286。实现检查基于当前共享工作区；其他组正在并发修改，未回滚或整理其改动。

## 结论与逐项状态

| 原 TODO 行 | 状态 | 结论 |
| --- | --- | --- |
| 108（Map 部分） | 部分完成 | Map 双名称、嵌套空间参考和显示范围已有明确 Load/Save 字段链并进入 `DataFrame` 模型；Map+0x298 已确认实现 IEnvelope 接口，但 coclass 身份仍未闭合。Map 后缀其余未定标量/对象字段仍需逐项归属。 |
| 226 | 未闭合 | Geometry 记录块首 u32 是导入内核使用的内部 shape 类型值；可确认空记录值 0、Polyline 值 3、Polygon 值 5 的局部用法，但尚无 10.1 类型库/运行证据把完整值域映射到 `esriGeometryType`。不把内部记录标记直接等同公开枚举。 |
| 227 | 未闭合 | Geometry wrapper 静态读取记录缓冲区后，通过对象 `+0xc8` 内嵌指针虚表 `+0x1cc` 槽交给几何内核；没有目标运行时装配/调用观察，不能确定该指针何时、由何工厂或构造路径赋值。 |
| 230 | 未闭合 | `{7453C805-9D64-44A6-BF2F-73E477DCE18F}` 在既有 FDO 记录中仍未定名（FDO Load 的 `+0xc0` 请求 IID）。 |
| 231 | 未闭合 | `{DBCA59AC-6771-4408-8F48-C7D53389440C}` 在既有 FDO 记录中仍未定名（FDO Load 的 `+0x28` 请求 IID）。 |
| 232 | 未闭合 | `{97D5B914-FB22-11D1-94A2-080009EEBECB}` 为 FeatureLayer 版本 `>0x10` 的请求 IID；现有类型库 dump 未命中。 |
| 233 | 未闭合 | 当前资料未建立完整、封闭的其余未名 IID 清单。不能用搜索不到名称推断接口不存在；需针对 IID 常量、类型库和请求槽继续核验。 |
| 236 | 部分完成 | PCS/GCS/Unknown 的持久化实现类、类标识、读写版本分支及可读取字段已有 10.1 静态证据与严格读取器。剩余业务含义包括 PCS/GCS 域串、Unknown 四串、PCS/GCS ISRPersistStream slot 8 恒返 `0x38` 的成员名、GCS 对象 `+0xac` 非零守卫，以及 pe/sg 推导容差数值的完整来源链。 |
| 265 | 分支已实现；样本佐证缺失 | Geometry 版本 0/1 被 10.1 Load 门接受；版本 1 不读空间参考，版本 0/1 的已实现布局仅由静态分支及合成输入支持。参考样本没有这两种版本的实例，故字节佐证未闭合。 |
| 266 | 未实现、未闭合 | 10.1 Load 在 AfStreamRead 返回 `S_FALSE` 时走空流捷径，读取器目前不实现此行为；没有目标运行时观察或样本证据。 |
| 270 | 未闭合 | Envelope 读取器显式拒绝版本 0，因为现有静态证据只确认版本 1 与 2/3 的字段分支。保存侧恒写版本 3；没有版本 0 字节实例。 |
| 282 | 未闭合 | FeatureLayer `+0x1f4` 在版本 `≥0xf` 装载 IEnvelope，样本呈现视图相交范围和宽数据范围混合，业务成员名及绘制路径尚未证实。 |
| 283 | 部分取证 | FeatureLayer `+0x1b4` 在版本 `>8` 装载 IEnvelope，Save 对应 `SaveObject(+0x1b4)`；参考样本 11 个 FeatureLayer 实例均为空对象标记。该槽的非空行为和成员语义仍无样本或运行证据。 |
| 284 | 已完成（静态推定） | Map+0x298 已确认持有实现 IEnvelope 的对象，IEnvelope 请求 IID 与 +0xdc/+0xe0/+0x18 槽用途相符。 |
| 285 | 未闭合 | Map+0x298 的 Geometry Envelope coclass `{30707212-…}` 仍是候选；尚未把其运行时 vtable/QI 来源与 Envelope 实现作静态或运行时匹配。 |
| 286 | 代码已接入；真实软件验证未做 | 现有 `mxd_maps` 代码已经从 FeatureLayer 的 `successor.envelope_2`（`+0x1f4`）提取嵌套 CRS，并填入 `Layer.crs`。Map CRS 回引按键和类标识双重匹配。TODO 的“待接入”文字已滞后；当前缺少 ArcMap/QGIS 人工检查，且专门的整条 FeatureLayer→模型测试未能在本轮运行。 |

## 原生证据与版本边界

本轮复用仓库已有 REA/反编译和类型库记录，没有启动新的 Ghidra 导入。当前 REA 会话状态查询为 `open=false`、保留记录数为 0；不会重置或关闭该共享会话。已有空间参考研究将 Geometry.dll 绑定到 ArcMap 10.1 Desktop 10.1.0.3035，SHA-256 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`；FeatureLayer.dll 为 10.1.0.3035，SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`；Map.dll 为 10.1.0.3035，SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`。Geometry.dll 与 sg.dll / pe.dll 10.1.0.679 的组件版本对应关系仍按既有记录保留为未核事项。

复用证据入口：

- [PCS、GCS、Unknown 与 Geometry 静态研究](arcmap-10.1-pcs-geometry-slots.md)：Geometry.dll 10.1.0.3035 的反编译/汇编、类型库 IID、类标识与已有样本交叉核对；证据明确没有运行 ArcMap。
- [Geometry 记录体研究](arcmap-10.1-geometry.md)：四类 Geometry wrapper 与记录导入器的字段及版本分支、内部记录标记、样本覆盖范围和未定位的运行时移交链。
- [空间参考、Envelope 与 Map 范围研究](arcmap-10.1-spatial-reference.md)：Map.dll、FeatureLayer.dll、Geometry.dll 10.1.0.3035 静态证据及参考样本 `/Maps` 的字段定位；报告分别标出 REA 证据、静态推断与样本辅助观察。
- 已保存的 [Envelope 归因 Evidence](../temp/arcgis/arcmap101-envelope-attribution-evidence.json) 记录 10.1 x86 PE 的 SHA-256、样本槽分布与未定 IID；证据包含本机路径，分享前应脱敏。

现有分析产物来自 REA/Ghidra 12.2（bridge 对 12.1.4 验证）；结论限于上述 10.1.0.3035 二进制及文中注明的 10.1.0.679 辅助组件。类型库/PDB 名称作为静态命名证据；不表示运行时对象已装配。样本字节只是辅助对照，合成测试不是 ArcMap 行为验证。

## 现有实现与接线检查

本轮没有修改 Rust 源码：涉及的读取器和 CRS 接线已存在，且对未有充分证据的行为保留停止、拒绝或未知值。

- `src/objects/spatial_reference.rs` 解析 PCS：保存版本、WKT、域块、精度标志及 double、容差 double；这些字段已由 Map 前缀转换到 `CoordinateSystem` / `ProjectedCoordinateSystem`。
- `src/objects/spatial_reference_slots.rs` 解析 GCS 与 UnknownCoordinateSystem，并在 Envelope、Geometry 槽按流内类标识分派。Unknown 没有 WKT 时不生成可伪装成有效 CRS 的定义。
- `src/objects/geometry.rs` 的类型化入口支持 Polyline、Polygon、Point、Multipoint；其内部 shape 记录 u32 保留原值，不冒充 `esriGeometryType`。Polygon/Point/Multipoint 的参考样本实例数为 0。
- `src/objects/envelope.rs` 支持版本 1/2/3、Z/M 标志对及嵌套 PCS/GCS/Unknown；版本 0 明确拒绝。嵌套未知新类因体无外长而报 Unsupported。
- `src/maps/mxd_maps.rs` 的 `feature_layer_crs` 从 `FeatureLayerSuccessor10_1.envelope_2` 取得 Envelope 体，再由 `envelope_reference_to_model` 映射内嵌 CRS，或在 CRS 键及类标识都匹配 Map 空间参考时复用 Map CRS。该路径用于普通 FeatureLayer、FDO 关联 FeatureLayer 和递归构造的图层节点。`src/model.rs` 的 `Layer.crs` 已提供目标字段。
- `+0x1b4` Envelope 目前保存在 successor 的槽位结果中供诊断/后续使用，不把它当作图层 CRS；已确认的 CRS 宿主是 `+0x1f4` 的 Envelope 第四段。
- Map+0x298 在 `Map::Save/Load` 的范围读写路径中作为 IEnvelope 持有者出现。Map 空间参考和显示范围已有中间模型映射；它的具体 coclass 身份没有闭合。

因此原 TODO 286 的“接入”动作已由当前实现完成，建议之后由主代理按项目 TODO 规则更新状态时改写为“核验图层 CRS 回引/内嵌两分支并记录未做人工验收”，本组未直接修改 TODO。

## 验证

- 定向运行 `cargo test --test envelope`：13 项通过。
- 定向运行 `cargo test --test geometry`：24 项通过。
- 定向运行 `cargo test --test spatial_reference`：11 项通过。
- `cargo test --lib maps::mxd_maps::tests` 未能编译测试目标：共享工作区 `src/objects/property_set.rs` 测试代码第 336、348 行把 `[u8; 2]` 与 `[u8; 4]` 放进同一数组，触发 E0308。该文件不在本组范围且为并发改动，本组未修改。因该编译错误，本轮无法执行 `mxd_maps` 的单元测试；已有 CRS 回引单元用例源代码只覆盖精确键/类匹配与不匹配。
- Cargo 曾提示无法清理全局 registry cache 文件（权限拒绝）；构建及上述三个集成测试仍正常完成。
- 未运行全仓测试（主代理已有全局基线）、未执行 ArcMap 10.1/QGIS 人工打开检查；本报告不声称真实 MXD 转换或坐标系显示已验证。

## 本轮后仍需处理

1. 用类型库成员级证据或绘制路径取证确定 FeatureLayer `+0x1f4` Envelope 的业务语义；补充 `+0x1b4` 非空/空行为证据。
2. 获得 Map+0x298 对象的 vtable/QI 或运行观察，闭合 Envelope coclass。
3. 为 `+0x1f4` 内嵌 CRS、Map CRS 回引、类/键不匹配和无 CRS 分支建立可运行的 `mxd_maps` 接线测试；修复共享测试编译错误后再执行。
4. 使用 ArcMap 10.1 和 QGIS 人工核对转换工程的图层 CRS、范围与地图显示。

## 主代理整合复核

共享测试编译错误已修正。主代理在 `mxd_maps` 中补充内嵌 CRS、同键异类拒绝、空槽以及仓库参考 MXD 的 FDO 关联图层到 `Layer.crs` 的接线验证；该模块 17 项测试全部通过。随后整库 661 项测试通过。以上只证明已实现的解析接线和输入处理，不是 ArcMap/QGIS 中的人工验收。原 TODO 286 的接入动作据此勾选，原 282–285 的语义及身份缺口仍保留。
