# Map、组图层、书签与 Envelope 续查

2026-11-17。范围对应交接时 TODO 119–121、132、180、259、284；未编辑 TODO 或公共入口。伪代码是静态反编译而非原始源码，合成测试不代表运行验证。

## 身份及独立复核

本机 `temp/arcgis101-bin/` 下 Map.dll 和 FeatureLayer.dll 经 PE 版本资源核对均为 x86、10.1.0.3035，InternalName 分别为 Map、FeatureLayer，OriginalFilename 分别为 Map.DLL、FeatureLayer.DLL；SHA-256 分别为 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`、`2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`。这核实文件身份，非签名或运行状态。Geometry.dll 身份及摘要见 [第四轮空间记录](mxd-round4-map-spatial-20261117.md)。

复查 `temp/pseudocode/AfCore/functions/0x68ab4680.c` 和 `0x68ab46f0.c`：10.1 `String::Write` 写 4 字节长度，随后写 `(字符数 × 2) + 2` 字节；`String::Read` 先读长度再按该值分配及读入，判断 HRESULT 和读入字节数是否为零，而不是严格证明全量读取。这证实该已知调用点的静态行为，不证明其他版本或全部宿主属性。导出索引和局限见 `temp/pseudocode/AfCore/decompile-index.json`；10.1 AfCore 另一份导出身份见其 README 记录。

## 逐项状态

| 行 | 状态 | 证据与剩余条件 |
| --- | --- | --- |
| 119 | 推进，未关闭 | Map 书签后的计数是 IStandaloneTable，不是后续 IGeoTransformation；照明后 12 字节已保存原始值，参见 [第四轮 Map](mxd-round4-map-spatial-20261117.md)。本次重跑介质根解析：10.1 66 份中 58 个单 Map 根、40 份到达前导且独立表计数均为零；10.8 68 份中分别为 56、11、零。其余路径未到达，不能证明不存在。三个成员的业务用途、非空表与后续坐标变换仍缺证据。 |
| 120 | 未关闭 | 10.1 和单独 10.8 的 String 静态规则已有；不能推广到 8.x、9.x 所有写侧。参见 [版本记录](mxd-open-version-20261116.md)。 |
| 121 | 未关闭 | 对象属性依赖各版本、各类 Load/Save；没有其他版本的完整类族覆盖，不能从字符串规则推导。 |
| 132 | 未关闭 | GroupLayer 10.1 Load/Save 版本分支与读取器已有，仍缺可从真实宿主链定位的相应版本对象字节；见 [图层记录](mxd-layer-luna.md)。 |
| 180 | 部分实现，未关闭 | `dataset_name_more::read_bookmark_name_slot_10_1` 已支持 FeatureClassName、空槽和回引，书签体只在名称对象完整时继续；其他 IName 实现与分布未观察。见 [第四轮复核](mxd-round4-review-20261117.md)。未知类不猜体长度。 |
| 259 | 合成接线推进，未关闭 | 新增 `tests/terra_next_map.rs`，从 Map 后缀连续消费 AOI、Feature 书签新对象及回引，并核对名称、键表及尾部边界。它是构造数据；现有介质有界扫描未发现这两类候选，不能证明实际非空循环不存在，见 [介质记录](mxd-round4-page-20261117.md)。 |
| 284 | 未关闭 | 10.1 Load 下 Envelope v0 与 v2 同布局，现有读取器及合成边界测试覆盖；缺可确认宿主、对象边界的真实 v0 旧档，见 [空间记录](mxd-round2-spatial-20261117.md)。 |

## 验证和交接

`cargo test --test terra_next_map`、`cargo test --test round4_map_spatial -- --include-ignored --nocapture`、`cargo test --test round4_names_format --test envelope_v0_round2`、完整 `cargo test` 均通过；`cargo fmt --check` 通过。测试进程的 TMPDIR 指向仓库 `temp/terra-map-feature-private/tmp`。重新使用已有 Ghidra 桥接工具 导出和证据包；受 `/tmp` 空间紧张及并行目标会话隔离约束，没有启动新的 Ghidra 导入。未在 ArcMap 10.1 或 QGIS 打开并人工检查，未进行目标进程运行实验。主代理可整合新测试及本表，不宜勾选上述未关闭条目。
