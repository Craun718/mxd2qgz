# 第八轮：安装介质旧档与宿主路径

2026-11-17。范围为当前可访问的 10.1 完整安装目录及已提取的 10.8 Desktop Data1.cab，不把介质版本等同文件保存版本。本轮只新增独立测试与本记录，不更改产品解析器或 TODO。读取器结果是按既有 10.1 静态规则解释真实字节，并非 ArcMap 执行记录。既有版本门证据见 [组图层](mxd-layer-luna.md)、[字段](mxd-round4-layers-20261117.md)、[空间对象](mxd-round2-spatial-20261117.md)及 [独立表](mxd-terra-nextpass-table.md)。本轮不重复其反编译结论；未使用共享 Ghidra 桥接工具 或启动新的原生反编译。

## 介质及身份

- 本机 10.1 安装目录 `Desktop/Setup.ini` 的既有记录为 10.1.3035；本轮重算 Desktop `setup.msi` SHA-256 `55813a637950acbe397b896485fc293dc71d65acf9d27a58ec393deee5b1f775`、`Setup1.cab` `6833035af901b4b46fe508485cdd5d26534bc27630c512f8bf87c5db417ddfe3`、`Setup2.cab` `d25199463d6aca591836fc78615fe8224b7a96bcd831b055acafd6003d8177a4`；10.8 Data1.cab `dc9d8739f0b6fc86595cd707ef9c0937e3ff71b8db29f3716e50a8e906a3e5f3`。
- 10.1 所有 10 个 CAB 的目录按 `.mxd`/`.mxt` 和可选组件后缀枚举。Desktop Setup1 实有 76 个直接条目：前轮提取的 66 个加 10 个带后缀的 MXD；另有 SDK .NET 的 20 个和 SDK Java 的 17 个直接 MXD；其他 CAB 无符合名称的直接条目。SDK .NET CAB SHA-256 `c674e97cde351abe6f37bc0ed7573bca975428a44c0429ec0b5f7313c9901e9d`，Java CAB `1ac3e0983a0ac059d01dd8aee66affcbffdcd7a96b430ea6a10f813c5c558ef7`。例如新发现的 Desktop calibrate 与 10.8 的 calibrate 同 SHA-256 `f883510f90d05a01c4ae152aa008a74b14cab0e8469177b98ef7e3b40fc533b4`，不能重复计为独立文件。
- 还提取和列举了 CAB 中匹配 `*.zip.*` 的 101 个 ZIP，第一层未见 MXD/MXT 文件名；介质的四个许可管理器普通 ZIP 同样未命中，另两个 Resource1.zip 的 Python ZIP 目录读取报损坏。CAB 内还有其他命名形式的压缩成员，未递归穷举；不能称整个介质所有嵌套载荷均已排除。10.8 只覆盖现存 Desktop CAB，不宣称覆盖完整 10.8 介质。
- 本轮重新用 pefile 核对 FeatureLayer、Layer、Geometry、Map 均为 x86、固定文件版本 10.1.0.3035；SHA-256 依次为 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`、`fe52bd9988b66f9d6f906b014e5684f28fc20e9b97b53c90909fa3d44e9efed8`、`c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`、`1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`。这些是资源声明和字节身份，不是签名或运行证明。

## 新的宿主链证据

独立测试从 `/Maps` 根计数开始，以同一引用表顺序登记 Map 和子对象，调用现有后缀读取器。ExtraSafe B=0 的 A 边界仍是已有观察推定，部分未知体跳过可能影响后续引用表；以下不是原生运行结果。

1. SDK .NET `Continents.mxd`，文件 SHA-256 `d20e7f7fea729c1ae9afc7d742745a2cdf16a0472ccedea2a2f46058758685e3`，`/Version` 原始文本 9.2、short 14。首 Map → 第一图层条目 `0x154` → 新 FeatureLayer 键 33 → FieldInfo 循环计数 5；完整体键 43、44、45、46、48 均为 **v2**。原始类头偏移 `0x4f7`、`0x53d`、`0x58f`、`0x5df`、`0x641`，其后是键及 `02 00`。这是 v2 的宿主归属证据；v0/v1 未覆盖，不能勾选整个 v0–2 条目。
2. SDK Java `brazil.mxd` SHA-256 `26ae235ddcbbd8a501e6029d53a9ddec073382d87f71fb25067625ed8f52ba30`，`/Version` 9.2/14；首 Map 图层列表三项 GroupLayer 的条目起点 `0x15a`、`0x1f6`、`0x8d70`，均 v11，子项数 0、7、13，读取器分别分类 0、7、13 项；在 ExtensionHelper 处停止。第二、三项依赖此前有界跳过。SDK .NET `SchematicsForServerSample.mxd` SHA-256 `31ae0b799e3783f5bbc174e89a4a0840795fcf1f68d0d7600254c89e86898f56`，`/Version` 10.0/18，首 Map 有 v12 组图层条目 `0x162`、`0xb234`，子项数 7、1，亦在 ExtensionHelper 停止。v11/v12 只覆盖 ≥11 的 ExtraSafe 子层分支，未覆盖 <11 普通分支，不能整体勾选 GroupLayer 版本分支。
3. SDK Java `LinearRef.mxd` SHA-256 `5eb1835d15610b6b56c853153360476400f6d55cf0693dfb72305ded3ce2715d`，`/Version` 9.2/14。首 Map 顺序到书签后前导，独立表计数 2；首条 `/Maps:0x6a50` 为 StandaloneTable 新对象键 331，版本 9，读到 DatasetName 新对象键 332 时在 `0x6a66` 停止。第二条未被宿主链到达，虽 `0x74ef` 有候选，不能把它算作已解析体。此证据推进非空独立表，但其剩余体及 Map 其他要求仍开放。

## 全流候选及范围

独立测试逐文件读取全部 OLE 流，逐字节检查 16 字节类标识及偏移 +20 的两字节，包含整个 `/Maps`（不只首 Map）、`/PageLayout` 及其他流。它不建立对象表，不知道是否回引或碰巧数据，+20 值不是经过证明的对象版本。共扫描 181 个文件条目：10.1 Desktop 76、SDK 37、10.8 Desktop 68；有重复内容。按根计数字段观察，多 Map 文件分别为 8、1、12，整流扫描覆盖其字节，但**并未完整解析第二及后续根 Map**；首 Map 尾部尚未闭合，不能以搜索命中冒充后续 Map 宿主。须在共享引用表下先完整消费前一 Map 再进入下一根，或取得经独立证实的根边界及对象表。

SDK .NET `/Maps` 发现 115 个 FieldInfo +20=2 候选、Group +20=11/12 共 10 个、StandaloneTable +20=13 共 4 个；Java 分别 822、11、6（表候选为 9 或 13）。这些是命中次数而非实例数；上节仅升级经宿主遍历的具体实例。`/Maps` 及 `/PageLayout` 未找到 Geometry 类 +20 为 0/1 或 Envelope +20 为 0 的候选；SDK 的 `/ESRI ArcToolbox` 中有 Polyline +20=0 和 Envelope +20=0 命中，例如 `USA_Data_ClipTools.mxd` 的 `0x26d8f`、`Continents.mxd` 的 `0x306a6`。该流宿主格式、对象引用表和边界未知，不能判断这些是序列化对象或版本，更不能作为 TODO 的真实旧对象证据。

完整命中、每个 SDK 文件的 `/Version` 原字节及各流数量保存在忽略目录 `temp/terra-round8-old/inventory.txt`，可能含本地路径；共享前须检查脱敏。当前结论只适用于列举到且成功读取的文件和精确类标识，不排除其他封装、类或未知编码。

## 闭合条件和复现

本轮没有可整体勾选的上述开放项：FieldInfo v2 可作为单独拆分项的字节佐证，v0/v1 尚缺；GroupLayer ≥11 已有真实宿主，<11 仍缺；Geometry v0/v1、Envelope v0 以及独立表完整体均未闭合。需要分别从已确认的 Map、图层、元素或工具箱宿主起点顺序读取、沿途登记引用、验证对象新建/回引和完整体终点，并继续检查全部根 Map；截断、未知子体须保守停止。宿主链本身也不能代替 10.1 对旧档的运行读取验证。

提取方式：Desktop `cabextract -F '*.mxd.*'`、SDK 两个 CAB 分别 `cabextract -F '*.mxd.*'` 至 `temp/terra-round8-old/desktop-extra`、`sdk-dotnet`、`sdk-java`；SDK 额外提取到的 `.mxd.xml.*` 是元数据，测试排除。既有两套 Desktop 文件路径见 [前轮](mxd-terra-follow-media.md)。执行：

```sh
mkdir -p temp/terra-round8-old/tmp
TMPDIR="$PWD/temp/terra-round8-old/tmp" cargo test --offline --test terra_round8_old -- --ignored --nocapture
TMPDIR="$PWD/temp/terra-round8-old/tmp" cargo test --offline
cargo fmt --check
```

无 ArcMap 运行环境，未在 ArcMap 10.1 或 QGIS 打开样本、进行人工目视检查或记录目标软件运行结果。不能声称转换或跨版本兼容已验证。

## 旧版 GroupLayer 的原生对象体

核对 x86 FeatureLayer.dll 10.1.0.3035，SHA-256 为
`2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`。
在 QEMU i386 将 `GroupLayer::Save 0x6f7119b0`、`Load 0x6f711270`、
`GetClassID 0x6f711e20`、版本表 `0x6f767c9c` 和相关指令映射到原始地址。
AfCore 10.1.0.3035 的 `AfStreamGetSaveVersion`、标量读写转发及 String
序列化和反序列化也执行原生指令。AfCore.dll 的 SHA-256 为
`031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`。

测试驱动提供内存 IStream、指定文档兼容索引、同一子组的两次引用以及普通
对象流服务。子组类标识来自原生 GetClassID，新对象键 2、同键回引、空
IEnvelope 标记由驱动按已确认的 ObjectStream 编码写读。驱动也提供空扩展
集合、字符串分配及应用层回调。原生 Save/Load 写读根组和子组的标量字段，
并分别选择并调用普通 SaveObject/LoadObject。ExtraSafe 回调被设置为一旦
调用即失败。这个实验覆盖 GroupLayer 对象体和它对普通对象服务的派发，
没有执行原生对象工厂、键表或图层集合的实现。

| 文档兼容索引 | 对象体版本 | 首子组头 | 首子组体 | 同键回引 | 根组体结束 |
| ---: | ---: | ---: | --- | --- | ---: |
| 0 | 4 | 42 | `[62, 130)` | `[130, 150)` | 176 |
| 1 | 9 | 42 | `[62, 168)` | `[168, 188)` | 252 |

偏移以根组体起点为 0。每次原生 Save/Load 都返回 `S_OK`，两次子层回调
得到同一子组，组自身的可见性和 IEnvelope 后 double 在回引之后按预期消费。
把这两份字节接到普通图层列表入口后，Rust 分别识别一个新子组、一个同键回引，
完整读取根组和子组、匹配对象体边界并继续到活动图层索引。v9 中还读取到空扩展
集合与尾字符串。命中同键时，第二个子组头后没有再消费对象体。

两份对象体由原生函数在受控驱动中生成，目前没有旧版 <11 实档宿主。
10.1 可写的两种普通子层体版本均已覆盖。v0–v3、v5–v8 与 v10 的独立版本
字段仍以已有原生读取规则为依据。ArcMap 与 QGIS 人工验收尚未进行。

### 旧 FieldInfo 的数字格式回引

Layer.dll 10.1.0.3035 的 FieldInfo Load `0x7865b520` 在 v0/v1
同样请求 INumberFormat。SystemCore.dll 同版的 `0x7ffd0063` 按键
查询已登记实例的接口。读取器恢复登记类，错引非数字格式时保留
INumberFormat 槽起点。合成 v0/v1 均验证了登记类优先、后继字节及错引
时的引用表状态。这两个版本的真实旧档和 ArcMap 原生打开仍待验证。
