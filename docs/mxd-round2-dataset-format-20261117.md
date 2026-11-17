# 第二轮：数据源与格式（2026-11-17）

本轮静态复核 ArcMap 10.1.0.3035，未运行 ArcMap 或 QGIS。指定行号是分派时 TODO 行号；分类 STATIC 表示还可以静态推进，SAMPLE 表示缺具有相应结构的真实文件，ARC 表示问题本身需要目标软件运行观察。合成字节只验证读取器，不能代替真实文档或运行结果。

## Hyperlink 字段的新证据：228、229

FeatureLayer.dll 10.1 原件 `temp/arcgis101-bin/featurelayerdll/FeatureLayer.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`，PE ImageBase `0x6f280000`。esriCarto 10.1 类型库来源和 SHA-256 见 [首轮记录](mxd-dataset-format-evidence-20261117.md#补充246-的真实模板-rust-试读和-225-枚举值)。在首轮 `temp/dataset-format-media-20261117/memberdump.c` 的副本中将 `GetNames` 缓冲容量改为 32 并以 UTF-8 输出名称，用 Wine `LoadTypeLibEx` 读取同一 `carto.tlb`：IHyperlink `{3036D35D-EDE5-11D0-87FE-080009EC732A}` 七个成员按顺序为 FeatureId 的 get/put、Link 的 get/put、LinkType 的 get/put、Jump。64 位 Wine 的 `oVft` 为 24、32、40、48、56、64、72；对应原目标 x86 的槽偏移分别为 0x0c、0x10、0x14、0x18、0x1c、0x20、0x24。

原生 RTTI COL `0x6f7b668c` 的对象偏移为 0x10，紧邻虚表 `0x6f7ac258`；持久化 COL `0x6f7b6678` 偏移为 0x14，持久化虚表 `0x6f7ac284` 的 Load 是 `0x6f6d9170`。这两种接口的 this 相差 4 字节。x86 虚表槽 +0x0c/+0x10 为 `0x6f6d8eb0`/`0x6f6d8ee0`，读取/写入接口 this+8；槽 +0x14/+0x18 为 `0x6f6cc420`/`0x6f6d8f00`，后者对接口 this+0xc 调 String 操作；槽 +0x1c/+0x20 为 `0x6f6d8f20`/`0x6f6d8f50`，读取/写入接口 this+0x10。Load 在 `0x6f6d91c2`、`0x6f6d91d1`、`0x6f6d91e4` 分别访问持久化 this+4/+8/+0xc。因此第二字段为 FeatureId，第三为 Link，第四为 LinkType。第二字段不是枚举；枚举值依然不得据此限制未知值。类型库及接口映射是静态证据，不代表真实 Hyperlink 样本验收。`src/objects/hyperlink.rs` 已修正文档，保留公开字段名和原始值以兼容现有调用者。

复核可用 `objdump -d -Mintel --start-address=0x6f6d8eb0 --stop-address=0x6f6d8f60 <上述 DLL>` 和 Load 对应地址；COL 与虚表可用 PE 内存映射读取。类型库导出器副本在 `/tmp/round2-memberdump.c`，其输出按宿主指针大小给出的槽位须换算至 x86，不可直接作为目标偏移。

## 指定项逐项状态与最小后续输入

| 原行 | 状态及后续路径 |
| --- | --- |
| 178 | 部分，STATIC：TableQueryName 的 Load/Save 局部已定位；需追踪写侧 Write_exref、对象虚调用及 FeatureQueryName 虚表与完整读写链。不得将它等同 QueryTableName。 |
| 179 | 未闭合，STATIC：对 10.1 配置声明的模块及工厂、类标识写入和 RTTI/虚表继续定位 ObjectClassName。 |
| 218 | 部分，STATIC：FDN v1 对象体已有真实局部链，需将四个成员槽与 10.1 getter/setter 绑定。 |
| 219 | 部分，STATIC：TopologyName 读写顺序已有；需绑定对象偏移与类型库属性。 |
| 228 | 闭合：第二字段 FeatureId、第四字段 LinkType，见上。 |
| 229 | 闭合：第三字段是 IHyperlink::Link；跳转操作 Jump 及其他 Hotlink 接口不能据此视为同一字段。 |
| 230 | 部分，STATIC：九类标量需逐类对照访问器和类型库槽位。 |
| 231 | 部分，STATIC：Rate、CustomNumber 等字符串需逐类对照访问器及槽位。 |
| 232 | 部分，SAMPLE：反编译已有同槽/独立槽的静态证据，缺四种对象的宿主结构样本；可用 ArcObjects 在 10.1 设置格式并保存工程或受控对象流。若要求内存槽状态则另需 ARC 调试。 |
| 240 | 部分，STATIC：现已定名的 IID 见首轮报告；需具体枚举剩余未定名引用并逐个对照类型库及引用点，当前不能声称穷尽。 |
| 241 | 部分，STATIC：五个请求 IID 已定名，六类字段及 OleDB 尾段族仍需 getter/setter、虚表和类型库交叉绑定。 |
| 249 | 部分，SAMPLE：选择子及 TopologyName 解析已有实现和合成测试；需含真实非零第二选择子的工程，可在 ArcMap 10.1 用包含拓扑的要素数据集建图并保存，确认实际保存确实出现该槽。 |
| 265 | 部分，SAMPLE：v0 与 v1 读侧同字段及合成测试已知；需真实历史 v0 文档或能产出该版本的受控历史写侧。10.1 Save 恒写 v1，不能假定改扩展名或普通另存即可生成。 |
| 266 | 部分，SAMPLE：通用对象流空/回引已有实现；需宿主循环实际出现空槽或共享对象的真实流，可由受控 ArcObjects 容器实验生成，前提是其允许构建对应状态。 |
| 267 | 部分，SAMPLE：各类版本门已有合成测试；需各历史短体的对象级文件或支持相应版本的受控写侧。10.1 写侧版本表可能不产生某些短体。 |
| 268 | 部分，SAMPLE：与 232 相同，需含四类格式的宿主对象链；ARC 仅用于要求观察内存槽状态时。 |
| 269 | 部分，ARC：需要对 10.1 Angle 核心 Load 人为制造可控失败，记录 HRESULT、游标及对象副作用；静态调查可继续，但仅凭合成输入不能确定运行时外部流行为。读取器继续严格报错。 |
| 279 | 部分，SAMPLE：FDN v0 合成分支已有；需真实历史 v0 文件或可写出该版本的受控历史实现。现有 10.1 写侧恒 v1。 |
| 280 | 部分，SAMPLE：TN v0 合成分支已有；需真实历史 v0 文件或可写出该版本的受控历史实现。现有 10.1 写侧恒 v1。 |

本轮 `cargo test` 通过（包含默认忽略项），`cargo fmt --check` 因并行变更 `src/maps/map_object.rs` 和 `tests/map_object.rs` 的格式差异失败；本轮未改这些文件。未执行 ArcMap/QGIS 人工检查。测试依赖介质的其他并行测试为默认忽略，不影响默认全量。
