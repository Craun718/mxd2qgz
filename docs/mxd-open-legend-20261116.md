# MXD Legend / MapSurround 本轮开放项核查

记录日期：2026-11-16。目标为 ArcMap 10.1.0.3035。本轮按 TODO 原行号核查
Legend / MapSurround / 书签相关开放项，修改仅涉及 `src/maps/map_surround.rs`
与本报告；不改 TODO、README、模块接线文件或共享稳健性清单。未提交。

## 证据边界

- Map.dll 核验文件为
  `temp/agent-map-surround/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`：
  SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`，
  PE32 x86、ImageBase `0x793a0000`、Esri `Map DLL`，文件与产品版本均为
  `10.1.0.3035`。本轮复核使用该介质提取文件，不把 10.8 观察移作证据。
- 静态反编译证据包括已有 Map.dll 研究底稿
  [`arcmap-10.1-mapsurround-bookmark.md`](arcmap-10.1-mapsurround-bookmark.md)、
  [`arcmap-10.1-legend-family.md`](arcmap-10.1-legend-family.md)、
  [`arcmap-10.1-legend-tails.md`](arcmap-10.1-legend-tails.md)，以及
  `/tmp/gh-c-legend/out/map4_cp_caller_793c8990.c`、
  `map4_cp_caller_7943b2f0.c`、`map4_cp_caller_79448500.c` 三份同源
  Ghidra 反编译输出。Ghidra 桥接工具 本轮的 Ghidra 入口对这些 PE VA 未定位到函数
  （其错误要求先解析当前导入映像中的函数坐标）；因此没有把失败调用当作
  新证据，复核依赖可直接检查的既有反编译产物。它们是静态证据，不代表运行
  行为。
- 仓库参考样本的 MapSurround 与空间书签循环计数均为 0；LegendClassFormat
  和 HorizontalLegendItem 无样本实例，Legend 条目循环计数也为 0。主代理复核的
  本轮基线样本解析停止偏移 `0x1622`，此后相关对象体不可由该次解析结果验证。
  所以本报告不声称有真实非空 MXD 形态或 ArcMap/QGIS 人工验收。
- 下文“推断”仅作为字段角色候选。没有成员名、setter/getter 对位或受控
  运行证据的标量继续按原始位模式保留。

## TODO 逐项结论

| TODO 行 | 状态 | 本轮结论与证据 | 剩余缺口 |
| --- | --- | --- | --- |
| 206 | 部分 | LegendClassFormat Load/Save 字段序已闭合：两个 `ITextSymbol` 槽、`ILinePatch`、`IAreaPatch`、版本门后的 double×2。请求 IID 和实现类由 10.1 类型库、CLSID 与读写侧反编译互证；解析器位于 `src/layers/legend_format.rs`，补丁体由 `src/layers/patch.rs` 分派。可支持的角色只有两处文本符号和线/面图例补丁。 | 两个 double 及符号的确切显示语义仍无成员级命名/运行证据；保持原值，不给它们起业务字段名。 |
| 207 | 部分 | BaseLegendItem 已确认 ILayer、两个 ITextSymbol、ILegendClassFormat、INumberFormat 对象槽；版本≥2 的两个字符串由默认值 `" ("` / `")"` 支持“名称前后缀”候选。证据见 `docs/arcmap-10.1-legend-tails.md` 第二节及 `src/layers/legend_item.rs`。 | 条目 int32、多个 short 以及前后缀之外字符串的业务含义未定；当前读取器继续保留原始数值。 |
| 208 | 部分 | Legend 共同前缀的序列、门控和写侧默认值已核对：名称字符串、ILegendFormat、标题字符串、版本 1 的两个遗留 double、三 short、集合选择 short（写侧恒 `0xffff`）、LegendItem 计数/条目、版本 3/4/5 尾字段；实现位于 `src/maps/map_surround.rs`。 | 除名称/标题及集合选择槽的结构角色外，short/int32 和尾组字段无类型库成员名或运行证据，仍按原样保存。 |
| 213 | 部分 | 共同前缀 Load `0x794413d0` / Save `0x79441510` 已闭合字段序：short 版本、String、请求 IMap 的对象槽、版本≥2 short、版本≥3 32 字节。请求 IID 支持“关联地图”角色；后 32 字节读入后丢弃、写侧取对象内存，WKSEnvelope 仅是候选。源码 `src/maps/map_surround.rs` 保留 raw 字段。 | short、32 字节槽业务语义及其是否为包络仍未定；没有运行/类型库成员证据，不能下定论。 |
| 214 | 部分 | 核对后发现 AOIBookmark 与 FeatureBookmark 没有共同的流内前缀读取器：AOI 为版本 + IEnvelope + 名称；Feature 为版本 + int32 + IName + 名称（`docs/arcmap-10.1-mapsurround-bookmark.md` 第四、五节，读取器在 `src/maps/map_surround.rs`）。此结构差异本身有静态反编译证据。 | FeatureBookmark 的 int32 含义仍未知；IName 实现类分布和 `+0x14` 调用语义未闭合。不能把两类包装成一个“书签共同前缀”。 |
| 235 | 部分 | 已实际复核三个 ScaleLine 族共同前缀调用者的反编译输出，解决“未反编译”这一子缺口。`0x793c8990` 与 `0x7943b2f0` 均为：版本 short → MapSurround 前缀 → int32、double×2、int32 → 两个 IEnvelope；版本>1 再读 double 和第三个 IEnvelope。`0x79448500` 有版本 0..2 门、共同前缀，之后依次装载一个未定名接口、执行虚槽 `+0x38` 调用、再装载三个接口；版本<2 走默认初始化，≥2 多一个对象槽。 | 三函数对应的 coclass 身份仍未从 RTTI / identity 虚表闭合；前两函数的 Envelope 与标量具体业务名、第三函数 IID/虚槽语义未知。因此不能把三个调用者强行命名为 ScaleLine、SteppedScaleLine 或实现类，也不能定性字段。 |
| 245 | 部分 | Map::Load `0x793ff4d0` 与 Save `0x79401850` 的循环已反编译：int32 计数 + 普通 LoadObject/SaveObject，无 ExtraSafe 8 字节前缀；`src/maps/map_surround.rs` 有非空条目分类及合成测试。 | 样本计数为 0，真实 MXD 的非空条目与类分布没有验证。 |
| 246 | 部分 | 书签循环读写结构已闭合；AOI/FeatureBookmark 体读取器可解析对应条目，另有空标记、回引、新对象处理。证据见 Map::Load/Save 与 `src/maps/map_surround.rs`。 | 样本计数为 0；真实非空书签条目以及 IName 实现分布未验证。 |
| 248 | 部分 | LegendClassFormat 的版本门和五个对象槽/尾段字段已有静态解析实现与合成测试。 | 当前样本没有对象实例，不能报告真实 LegendClassFormat 样本字节形态。 |
| 249 | 部分 | HorizontalLegendItem / BaseLegendItem 字段序已由 Renderers/LayerLib 10.1 Load/Save 反编译，解析器含对象槽停止语义与合成测试。 | 当前样本 Legend 条目数为 0；ILayer 等槽的真实流内空标记/回引分布和完整条目字节尚未由 MXD 验证。 |
| 250 | 部分 | BaseLegendItem 的版本与字段门已实现；v2<2 时名称前后缀组不读，v2≥2 读五字段组，v2>2 再读 INumberFormat。 | 样本无条目，旧版 BaseLegendItem 的真实流形态尚无字节佐证。 |
| 252 | 部分 | Legend v1 旧体已在 `read_legend_prefix_10_1` 中显式读取两个遗留 double；Legend v1 的无 ≥3/4/5 尾字段分支有合成回归测试（`tests/map_surround.rs::legend_version_one_reads_legacy_doubles_without_gates`）。LegendFormat v0/1 与 BaseLegendItem v0/1 的门也由对应读取器保留。 | 没有真实 v0/v1 MXD 实例；版本 0 全路径和嵌套旧对象形态仍需样本或 ArcMap 运行取证确认。 |

## 代码与接线

- 本轮改动：`src/maps/map_surround.rs` 的模块文档和 LegendFormat 类标识
  注释，移除“LegendFormat 无读取器”“装饰物尚未逐类反编译”等过期说法；
  实际字段和未知项按上表保持保守。
- 解析器本体、循环接线和合成异常测试在本轮开始前已存在，本轮没有改动
  解析逻辑或新增测试，也没有改 `mod.rs`。主代理需按现有模块组织确认其
  接线；本轮未要求其改 `TODO.md`。
- 未运行全局 Cargo 检查，遵照本轮安排交由主代理统一验证。ArcMap 10.1
  未运行，QGIS 未打开转换结果，人工目视验收未做。
