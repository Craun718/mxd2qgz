# ArcMap 10.1 坐标变换类工厂与共同对象体（2026-11-17）

接续 [Map 坐标变换记录](mxd-terra-chain-map.md)。本文只讨论静态证据和独立读取器；没有编辑 Map、TODO，也没有将新类接入宿主。

## 文件身份与取证方式

本轮重算 `temp/arcgis101-bin/geometrydll/Geometry.dll.F84366F9_C0B7_4267_A16E_D48D9E6DD261` SHA-256 为 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`。PE machine `0x14c`、ImageBase `0x717e0000`，版本资源 FileVersion/ProductVersion `10.1.0.3035`、InternalName `Geometry`、OriginalFilename `Geometry.DLL`，与既有基线相符；版本自述与字节摘要不证明签名。以下地址均为静态 VA。

前轮独立 Ghidra 桥接工具 6.9.0 CLI 使用 JDK 25 时 Ghidra 12.2 的 Unix socket 绑定失败（`Operation not permitted`），见前述记录；本轮未切换共享 Ghidra 桥接工具。将前轮 Geometry 工程复制到 `temp/terra-host-geo/project`，以隔离 pyghidra/Ghidra 12.2、JDK 25 和仓库内独立临时/配置/缓存目录查询虚表及反编译，并以原 PE 的 objdump 指令核对构造。伪代码不是原始源码，私有分析产物在忽略目录，分享前需检查路径。

## 类工厂、构造和持久化槽

| 证据 | GeoTransformation | NADCONTransformation |
| --- | --- | --- |
| RTTI 类工厂虚表 | `0x71a1e06c` | `0x71a1dcfc` |
| 工厂 `+0x0c` 槽 | `0x718c7ff0` | `0x718c4c90` |
| 工厂创建路径构造 | `0x719c61e0` | `0x719c9020` |
| 持久化虚表 | `0x71a274f4` | `0x71a2894c` |
| GetClassID `+0x0c` | `0x719c40b0` | `0x719c4480` |
| Load `+0x14` | `0x719c5d40` | `0x719c5d40` |
| Save `+0x18` | `0x719c5ef0` | `0x719c5ef0` |

新反编译显示两个工厂各分配 `0x60` 字节并分别调用上表构造函数；GeoTransformation 构造 `0x719c626f` 明确将 `0x71a274f4` 写到对象 `+0x18`。新核对虚表指针，以及 `0x719c40b0` 写出的 16 字节，得到 GeoTransformation CLSID `{843C4D40-BD7E-4542-A126-24AEBA43AF20}`；NADCON 的 CLSID 为 `{D661941C-DA8A-11D3-9F60-00C04F6BDD7F}`。两个不同类的持久化 Load/Save 槽为同一函数，因而可以共享其对象体读取规则；这不等于证明其他变换类也共享。

共同 Load 顺序为 2 字节版本、4 字节长度、指定长度的原始文本字节；其处理细节与 Save 的 `pe_geogtran_to_string` 路径见 [前轮记录](mxd-terra-chain-map.md)。原生按有符号 short 比较版本是否大于 2，并有短读相关行为；本读取器只接受无符号 0..=2，限制长度不超过调用者限额且小于 3072，截断报错并回退游标，均属于有意更严格的策略。它不验证或转换变换文本，不执行坐标计算；不宣称原生对长度作相同校验。

## 独立接口和交接边界

`src/objects/geo_transformation.rs` 提供两个 CLSID 及 `read_geo_transformation_body_10_1`：仅接收已确认的两个类，调用者负责解析对象头、维护引用表和决定何时调用。读取失败不推进游标，成功只消费对象体；未知类、版本、超限和截断均报错。`src/objects/mod.rs` 暴露模块；现有 Map 的 NADCON 实现和调用均未改，主代理可按需要替换或接线，注意引用表及对象头事务性由宿主负责。

可独立落账：**GeoTransformation 类工厂、构造、CLSID 和持久化虚表对位；该类与 NADCON 共用同一 10.1 Load/Save 的静态证据；两类严格对象体读取器及合成边界测试。**不能勾选全部 GeoTransformation 类、宿主接线、真实 GeoTransformation 样本、文本语义、坐标运算或完整 Map 尾部。ArcMap 10.1 环境暂缺，未运行目标软件或在 ArcMap/QGIS 中打开并目视验收；静态证据及合成测试不能替代运行验证。

验证：`cargo test --offline --test terra_host_geo` 的 3 项通过，覆盖两类、版本、原始字节、尾随字节、空值、长度上界、未知类及各截断位置的游标回退；`cargo test --offline` 通过（默认忽略项未运行），本轮新增 Rust 文件的 `rustfmt --check` 和 `git diff --check` 通过。全仓 `cargo fmt --check` 未通过，差异在并行修改的 `tests/terra_integrate_map.rs`，本轮未改该文件。测试均使用仓库内专用 TMPDIR。

主审接线：Map 坐标变换循环改用本模块，接受两类新对象及已见键回引。主体按原始字节保留，另以构造宿主测试验证第二类不早停；样例实档只观察到 NADCON。
