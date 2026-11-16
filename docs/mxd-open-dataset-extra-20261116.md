# MXD 数据源组其余十项核查

记录日期：2026-11-16。目标版本 ArcMap 10.1.0.3035。范围为 TODO「MXD
解析」原行 168、171、172、211、212、234、241、242、271、272。遵守本轮范围，
只改 `src/objects/dataset_name_more.rs` 与本报告；未改 TODO、README、模块接线
或其他源码，未提交。

## 证据边界

- GdbCore.dll 复用已提取文件
  `temp/agent-dataset-name/GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA`，
  SHA-256 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`，
  PE32 x86、10.1.0.3035；身份与已有
  [`arcmap-10.1-dataset-name-slots.md`](arcmap-10.1-dataset-name-slots.md)
  一致。Map.dll 的书签结论复用 10.1 记录
  [`arcmap-10.1-mapsurround-bookmark.md`](arcmap-10.1-mapsurround-bookmark.md)
  与 [`arcmap-10.1-legend-family.md`](arcmap-10.1-legend-family.md)。
- 为优先查 FeatureQueryName/TableQueryName 变体流体，本轮曾将已有 Ghidra
  GdbCore 工程副本置于 `temp/agent-dataset-extra/`，以仓库内 TMPDIR 和独立
  配置启动 headless 检查；日志只到数据类型归档，未得到函数反编译输出。此后
  以已导出的 10.1 GdbCore 原始字节继续核查，没有重复初始化。
- 五个接口名可回溯到 ArcMap 10.1 安装介质原件及对应转储行。原件是
  `/home/gxrs/下载/arcmap10.1/ar10.1/Desktop/Setup1.cab` 成员；转储文本在
  `/tmp/mxd-arcgis101-guiddocs/wine/`：
  - `esriSystem.olb.392FE3A8_65C9_4463_A15B_89F00F8F3FE8`（CAB 长度 363008）；
    `dump2_esriSystem.olb.392FE3A8_65C9_4463_A15B_89F00F8F3FE8.big.tlb.txt`
    第 72 行：`71 IName 4BA33331-D55F-11D1-88820000F877762D typekind=3`，即
    `{4BA33331-D55F-11D1-8882-0000F877762D}`。
  - `esriGeoDatabase.olb.392FE3A8_65C9_4463_A15B_89F00F8F3FE8`（CAB 长度
    1320960）；`dump_esriGeoDatabase.olb.392FE3A8_65C9_4463_A15B_89F00F8F3FE8.big.tlb.txt`
    第 57、60、359 行：`56 IQueryFilter FDFEBD93-ED75-11D0-9A95080009EC734B`、
    `59 ISelectionSet F7AD47D1-D55E-11D1-88820000F877762D`、
    `358 IQueryDescription 2E060684-E14C-44D3-858A442AB6A0A863`（均
    `typekind=3`），对应标准 GUID 分别为
    `{FDFEBD93-ED75-11D0-9A95-080009EC734B}`、
    `{F7AD47D1-D55E-11D1-8882-0000F877762D}`、
    `{2E060684-E14C-44D3-858A-442AB6A0A863}`。
  - `esriGeometry.olb.392FE3A8_65C9_4463_A15B_89F00F8F3FE8`（CAB 长度
    1291776）；`dump2_esriGeometry.olb.392FE3A8_65C9_4463_A15B_89F00F8F3FE8.big.tlb.txt`
    第 4 行：`3 ISpatialReference 4434D680-F4DA-11D1-BCA60000F875BCCE typekind=3`，
    即 `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`。
  类型库证明请求 IID 的接口身份，不证明槽内类标识、字段业务含义或 MXD 样本形态。
- 以 GdbCore 原件 SHA-256 过滤 REA 导出，只使用
  `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4` 记录；
  未使用 FileGDB.dll 的 `0x6fa*` VA。对原始 PE `.rdata` 使用
  `struct.iter_unpack('<I', raw_bytes)` 解码，TQN 次级表 `0x70aba6dc` 的
  `+0x0c/+0x14/+0x18/+0x1c` 分别为 `0x708f46c0/0x708f5070/0x708f5280/
  0x708f5490`。`objdump -D -Mintel` 从 `0x708f5070`、`0x708f5280` 开始
  解码，两入口均落在 `.text` 并从 `push ebp; mov ebp,esp` 指令边界起始。
  Load 汇编显示 short 版本读取、版本 >2 失败、三次字符串读取、
  IWorkspaceName LoadObject 与其他槽操作；Save 显示版本写出、三次字符串写出、
  SaveObject 与其他对象/short 字段操作。入口及局部操作已见原始指令，但未取得
  完整反编译，字段边界与接口语义仍未闭合；不据此实现 VariantStream 体。
  导出中 `0x708f4ab0` 的 32 字节记录不足以将该函数归属于 QueryName Load/Save。
- 仓库参考样本 FeatureBookmark 计数为 0；FeatureDatasetName/TopologyName
  CLSID 均无命中，FCN 可达选择子全为 0。两个数据源类版本 0 只有 10.1 静态门
  和合成解析器覆盖，没有真实版本 0 字节。
  本报告不宣称有真实样本分支验证、ArcMap 运行验证或 QGIS 人工验收。

## TODO 逐项状态

| TODO 原行 | 状态 | 本轮结论与证据 | 尚缺证据 |
| --- | --- | --- | --- |
| 168 | 未闭合 | FeatureBookmark `{EC65B35B-4342-11D2-AE22-080009EC732A}` 的 Load 请求 IID `{4BA33331-…}`，10.1 类型库定名 IName；Load 成功后对接口槽 `+0x14` 调用，并 QI IFeatureClass，失败软置空。详见 mapsurround/bookmark 第五节、legend-family 第十节。 | 流内 IName 对象的 coclass 分布未知，书签样本计数为 0；`+0x14` 调用也未定性。因此没有可据以实现分布分派的证据。 |
| 171 | 未闭合 | GdbCore SHA 过滤后的 REA 记录和按小端程序解码的 TQN 次级表确认 GetIDType/Load/Save/GetSizeMax 槽入口；直接指令显示版本门、三次 String 读写、IWorkspaceName LoadObject/SaveObject 和后续槽操作。 | 未取得完整 Load/Save 反编译；所有字段边界、剩余接口槽语义及 FQN 对应体仍开放。本轮未实现 VariantStream 体，维持现有停止点。 |
| 172 | 未闭合 | 10.1 `esri.clsid.ecfg` 将 ObjectClassName CLSID `{905CEC64-0B2C-11D4-A060-00C04F6BDD84}` 归入 GdbCore；旧报告在 GdbCore 中找到 `IObjectClassNameImpl` 接口 TD，但未找到具体实现类 RTTI，也未找到可确证的实现类虚表链。 | 具体类 TD/COL/虚表、GetClassID 实现及其所属模块仍未知。不能把接口 TD 当作 coclass RTTI。 |
| 211 | 部分 | FeatureDatasetName `Load 0x709b7c90` 的 10.1 静态字段序为版本、String×4、1 字节标志、IWorkspaceName 槽；四 String 位于对象 `+0x0c/+0x10/+0x14/+0x18`，标志在 `+0x1c`。读写对称见 dataset-name 第 FeatureDatasetName 节。 | 反编译只证明读取顺序和写回槽位，没有成员名、语义化 getter/setter 或运行取值。四 String 与标志业务语义仍未定，继续按序号和原值保存。 |
| 212 | 部分 | TopologyName `Load 0x7095e5c0` 的字段序为版本、String×2、IDatasetName 槽；两个 String 位于对象 `+0x0c/+0x10`，槽在 `+0x14`。读写对称见 dataset-name 的 TopologyName 节。 | 两 String 的业务语义和 `+0x20` 通知调用含义未由成员名或运行证据定性；不将其提升为名称/别名等业务字段。 |
| 234 | 部分 | 五 IID 类型库名称由安装介质原件与转储行对应，证据路径与 GUID 见上文；`dataset_name_more.rs` 仅在中文注释标明请求接口，公开字段名与读取行为不变。 | 六类标量/String 语义及其他剩余未名项未定，五个请求接口的槽内对象体也未闭合；这是组合任务，不能整项勾选。 |
| 241 | 静态解析已存在；真实字节未闭合 | `dataset_name.rs` 已按 FCN Load 反编译实现第一选择子：0 请求 IWorkspaceName，非 0 请求 IDatasetName；现有合成测试覆盖两分支。样本中可达值全为 0。 | 没有真实非零选择子字节/对应对象，不能称非零分支获样本验证。源码禁止范围包含 `feature_layer.rs`，本轮不改其读取接线。 |
| 242 | 静态解析已存在；真实字节未闭合 | FCN 版本 >1 时第二选择子非 0 请求 ITopologyName 的静态规则已记录；`dataset_name.rs` 具有 TopologyName 体读取器及选择子分支处理，合成输入覆盖代码路径。 | 样本中该选择子可达值均为 0；没有真实非零值和 TopologyName 对象体字节，不能升级为真实 MXD 验证。 |
| 271 | 静态接受规则已存在；真实字节未闭合 | FeatureDatasetName Load 的 10.1 汇编版本比较为有符号 `ver > 1` 拒绝，因此版本 0 可通过；读取器允许 0，并有合成版本测试。 | Save 恒写版本 1；目前没有真实保存产生的版本 0 流字节或旧版 MXD 实例。 |
| 272 | 静态接受规则已存在；真实字节未闭合 | TopologyName Load 的 10.1 汇编版本比较为有符号 `ver > 1` 拒绝，因此版本 0 可通过；读取器允许 0，并有合成版本测试。 | Save 恒写版本 1；目前没有真实保存产生的版本 0 流字节或旧版 MXD 实例。 |

## 源码改动

仅 `src/objects/dataset_name_more.rs`：按 10.1 类型库名称在五个请求槽的中文
注释中标出接口身份；公开字段名和解析行为不变。六类其他字段保留原始序号名。
FeatureQueryName/TableQueryName 没有新增解析器或字段结构；直接指令只提供部分
TQN 体动作证据，TODO 171 仍开放。未运行 Cargo 测试，避免重复主代理已执行的 661 项
基线；ArcMap/QGIS 人工验收未做。
