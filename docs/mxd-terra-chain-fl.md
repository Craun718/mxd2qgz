# FeatureLayer 包络槽下游接口链（2026-11-17）

11 月 18 日已[闭合范围缓存语义](#续查数据集范围缓存与-sql-查询图层2026-11-18)。
前几节保留 11 月 17 日的阶段性结论和取证过程。

承接 [包络槽续查](mxd-terra-follow-feature.md) 的原 TODO 296。本文仅为静态研究，地址为 x86 映像 VA、成员偏移基于 FeatureLayer Load 的第一个显式参数；不代表 ASLR 运行地址。本轮未修改 TODO 或解析器，未运行 ArcMap、QGIS，未进行人工目视验收。

## 身份与方法

本轮重新计算并用 pefile 检查提取文件：FeatureLayer.dll 的 Machine 为 `0x14c`，FileVersion `10.1.0.3035`、InternalName `FeatureLayer`、OriginalFilename `FeatureLayer.DLL`，SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`；ArcMap.exe 为 `0x14c`，版本 `10.1.0.3035`、InternalName `ArcMap`、OriginalFilename `ArcMap.exe`，SHA-256 `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97`。与仓库 10.1 基线一致；版本资源是声明、哈希是字节身份，均不证明签名或运行。

独立 Ghidra 桥接工具 6.9.0 CLI 使用 Ghidra 12.2、JDK 25 和仓库专用 TMPDIR 对 FeatureLayer.dll 发起反编译；导入和分析后桥接 Unix socket 报 `Operation not permitted`，未产生函数 Evidence。未切换共享 Ghidra 桥接工具 目标。另用隔离 pyghidra/Ghidra 12.2、JDK 25 独占项目分析 ArcMap.exe，并以 objdump、PE 原始字节交叉核查。分析物及失败诊断位于忽略目录 `temp/terra-chain-fl/`，可能含本机路径，分享前应检查。伪代码不是源码，以下直接控制流以机器指令为准。

## 修正第二级 IID

`0x6f705d00` 使用 `0x6f7ae424`，原始 16 字节 `7d 85 0a 72 a9 45 bf 4d aa 06 73 71 5c 65 4b 01`，COM GUID 小端字段解码为 `{720A857D-45A9-4DBF-AA06-73715C654B01}`。

`0x6f705db0` 使用紧邻的 `0x6f7ae434`，原始字节 `61 62 e8 89 2d 2b a7 45 be 01 64 19 0d bc 9b db`，解码为 **`{89E86261-2B2D-45A7-BE01-64190DBC9BDB}`**。此前 [包络槽续查](mxd-terra-follow-feature.md) 将第二字段误写为 `2D2B`；该记录中的第二级 IID 应以本文为准。原始字节与 ArcMap.exe `0x552048` 一致。两个 IID 的正式接口名、继承声明和实现类仍未知，不能仅凭 GUID 邻接推断。

## 可确认的调用关系

1. FeatureLayer Load `0x6f6ea8ae..0x6f6ea8ce` 对 `+0x1f4` 预释放/清零，调用 LoadObject 请求 IEnvelope；失败进入错误路径。
2. 成功后 `+0x130` 非空时，先对其查询第一级 IID，检查结果非空，再从该接口查询修正后的第二级 IID；`0x6f6ea93a..0x6f6ea94b` 将 `+0x1f4` 指针传入第二级接口虚表字节偏移 `+0x10`，调用后 `0x6f6ea94d..0x6f6ea953` 对该接口执行 `Release`。两次 QI 的中间助手对失败 HRESULT 返回空指针；此处第二次 QI 后没有空指针检查。首个接口已 QI 成功不构成第二个接口一定成功的静态证明，实际可达性及异常处理须运行验证。
3. 本轮在同版 ArcMap.exe 发现独立使用该修正 IID：`0x507b80` 的 QI 助手由 `0x5030f0` 调用；其调用点 `0x4ffd34..0x4ffd77` 在取得非空接口后，将初始化为零的 16 位局部变量地址传给同接口虚表 `+0x14`，随后按该 short 是否为零分支。这提供了第二个方法槽和调用形态，但尚不足以命名该方法、解释 short 的业务意义或证明与 FeatureLayer 调用的是同一具体对象实例。
4. 同版 Map.dll `0x794671d9` 和 MappingCoreLib.dll `0x79bbef69` 也将第一级 IID 传给 QI；这两处是查询助手，不是接口定义或实现方法。MappingCoreLib.dll 哈希 `c2bf81dcfcb65473ce35b953c4f4b76392a594085460c299326cc20bed59986a`，Map.dll 哈希 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`；两者版本资源均声明 `10.1.0.3035`、x86，InternalName 和 OriginalFilename 与模块相符。

对本机 `temp/arcgis`、`temp/arcgis101-bin`、`temp/agent-fl-tail2`、`temp/mxd101-wine`、`temp/terra-depth-legend` 中大小不超过 60 MB、以 MZ/MSFT 开头或扩展名为 .olb/.tlb 的 3365 个文件做 GUID 字节有界扫描；修正后的第二级 IID 命中 FeatureLayer.dll 和 ArcMap.exe，第一级还命中上述 Map.dll 和 MappingCoreLib.dll 的部分本地副本。扫描含重复文件和非目标文件，并非全部安装介质；无命中不证明实现不存在，接口也可能用拆分比较、重定位或其他模块实现。

## 结论与后续

新证据将第二级 IID 修正并定位 ArcMap 中额外的 `+0x14` 调用，但仍未找到两个接口的正式名称、`+0x10` 方法声明或承接该方法的具体类和实现。现阶段只可称 `+0x1f4` 为传入该方法的持久化 IEnvelope，不能认定为实际绘制范围、缓存范围或数据范围；也不能从额外 short 分支给 Envelope 赋名。这不改变已确认的文件字段布局，不实施解析器语义变更。

可单独落账的已完成研究动作：**10.1 FeatureLayer 第二级 QI 的 IID 原始字节纠错，并交叉定位 ArcMap 同 IID 的另一方法槽调用**。原 TODO 296 的业务语义与具体实现仍未闭合，不可勾选；尚需找到能识别 IID 的 10.1 类型信息或 QI 实现，追踪其虚表 `+0x10` 目标及成员读写，并在可用环境中对可控图层、视图和数据源进行 ArcMap 10.1 运行观察。任何静态推断、合成测试或扫描结果均不能替代运行及 ArcMap/QGIS 目视验收。

## 续查：数据集范围缓存与 SQL 查询图层（2026-11-18）

`FeatureLayer` 的 IPersistStream 视图 `+0x1f4` 是**持久化的数据集范围缓存**。
具有空间数据源时，图层的范围 getter 取得数据集范围并更新缓存；无法取得数据源的
IGeoDataset 接口时，回退到这个包络。SQL 查询类还有一条反向路径：加载图层时，
可用缓存恢复 SQL 数据源内部的范围。该字段不宜作为地图当前显示范围或实际绘制裁剪框，
也不能把落盘缓存视为数据源此刻的权威范围。

### FeatureLayer：字段位置与读取路径

复核研究目录原有 FeatureLayer.dll 的 SHA-256 与上节一致，PE x86，版本
10.1.0.3035。复用 `FeatureLayer/6f6e9940.c`、`6f6e5ab0.c`、
`6f6dd450.c`，结合 `objdump -d -Mintel` 复核 Ghidra 输出删去的分支。

- 构造器 `0x6f6db996..0x6f6db9be`、RTTI 与两份 10.1 类型库互证：
  完整对象 O 的 IGeoDataset、IPersistStream、ILayer 接口分别位于
  `+0xc8/+0xd0/+0xd8`。持久化 `+0x1f4` 和范围 getter 的
  `+0x1fc` 都是 **O+`0x2c4`**；构造器 `0x6f6dbe41` 将该成员初始化为 0。
- esriGeoDatabase TYPELIB TypeInfo 19 将 `IGeoDataset +0x10` 定名为
  `get_Extent`。FeatureLayer 的该槽（虚表 `0x6f7acf78`、实现
  `0x6f6e5ab0`）先查数据源的 IGeoDataset：`0x6f6e5af4` 请求
  `34C20004-4D3C-11D0-92D8-00805F7C28B0`。数据源提供该接口时，
  `0x6f6e5c0c` 调其 `get_Extent`；结果空指针或空包络时还调用
  LayerLib 的 `GetExtentFromSpatialReference`（IAT `0x6f74e4e0`）。
  `0x6f6e5c91..0x6f6e5ca6` 将结果经复制及 QI 存回 IEnvelope 缓存槽。
- 数据源缺失或不提供 IGeoDataset 时，`0x6f6e5b04..0x6f6e5bb4` 取
  同一缓存并复制到范围输出；如有图层空间参考，复制前赋给缓存包络。
  显式 AOI 是另一成员：esriCarto TypeInfo 5 的 `ILayer.AreaOfInterest
  +0x18` 对应 FeatureLayer `0x6f6dd450`。其成员 `this+0x1ac`
  与持久化槽 `+0x1b4` 同为 O+`0x284`。在
  `0x6f6dd50f..0x6f6dd520`，该成员为空时，AOI getter 转调同一图层
  的 `IGeoDataset.get_Extent`。这是对空值的静态结论，未观察目标软件运行。

### SQL 查询图层：缓存的恢复路径

从 10.1 Desktop 介质的两个 CAB 补查 DLL/OLB 中的完整 IID，找到此前未检索的
`SqlFDB.dll`。该 DLL 提取自 10.1 Desktop 介质，PE x86、固定文件版本
10.1.0.3035，SHA-256 `fb440a01e9435947892f0ad9cbb76aaf1cd7ca40c09e5cea0cdba11c23294b8b`。

| IID（见上节） | SqlClass 的 QI 命中指令 | 返回偏移 | RTTI 基类描述 |
| --- | --- | --- | --- |
| `720A857D-45A9-4DBF-AA06-73715C654B01` | `0x7fee1877..0x7fee1888` | O+`0x10` | `0x7ff3538c`：IQueryLayerClass |
| `89E86261-2B2D-45A7-BE01-64190DBC9BDB` | `0x7fee1920..0x7fee1933` | O+`0x38` | `0x7ff35178`：IQueryLayerExtent |

- SqlClass 构造器 `0x7fedac9b` 安装 `IQueryLayerExtent` 虚表
  `0x7ff2f1e4`。FeatureLayer::Load 的最后一步
  `0x6f6ea94b` 调用此接口 `+0x10` 的范围设置方法。对应 SqlClass 实现
  `0x7fee1620` 将 IEnvelope 以其 WKSEnvelope 查询槽 `+0xdc` 复制为
  四坐标（完整 SqlClass O+`0xd8`），将 O+`0x8d` 标为已备好范围。
  空包络指针返回 E_POINTER；FeatureLayer 这个调用点没有传播该方法的返回码。
  `+0x14` 的 `0x7fee1660` 把同一就绪标志返回为 VARIANT_BOOL，
  解释了前轮 ArcMap 中同接口的 16 位输出。这里按行为描述私有槽，
  未断言原始 IDL 方法拼写。
- 同一 SqlClass 的 IGeoDataset 虚表 `0x7ff2f0d4` 中，
  `get_Extent 0x7fee0ed0` 从 `0x7fee0f13` 检查同一 O+`0x8d`。
  置位时直接跳到 `0x7fee10ad`，从 O+`0xd8` 组装范围；否则走查询/
  计算路径，结果写入同一缓存后置位。因此保存的查询图层范围会参与数据源的
  Extent 返回；没有从这些指令推断实际视图裁剪、运行时绘制或用户界面行为。

接口位置分别以完整 FeatureLayer 和完整 SqlClass 为基准，不跨对象相加。
类型信息来自介质 esriGeoDatabase TYPELIB（SHA-256
`92634979daa3ed0008b36d152e1fdf39cba04944f6bc7bbb5ea4c786bb44bb9a`）
与 esriCarto TYPELIB（SHA-256
`8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`）。
已在 Rust 接入范围缓存的命名访问器；槽位格式、回引和中间模型原有 CRS 映射保持
原行为。显式 AOI 空值的受控运行见下节。未在 ArcMap/QGIS 人工检查。

## 显式 AOI 的原生运行验证

2026-11-19。FeatureLayer.dll、AfCore.dll、SystemCore.dll 均为 x86 10.1.0.3035。
FeatureLayer.dll 的 SHA-256 与本文前述一致，AfCore.dll 为
`031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`，
SystemCore.dll 为 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`。

用 pefile 提取原始指令和接口表，按 PE 映像地址映射到 32 位 ELF，以
`qemu-i386` 执行。由原生 ILayer 表 `0x6f7acfec+0x18` 进入 AOI getter
`0x6f6dd450`，同一对象的 IGeoDataset 表 `0x6f7acf78+0x10` 指向完整
Extent getter `0x6f6e5ab0`。探针提供受控包络、数据源接口和空间参考，
记录复制、引用计数、数据源 Extent 和 LayerLib 空间参考回退的调用。

在空对象加载用例，先把旧包络放进 O+`0x284`，再执行原生 Load 的
`0x6f6ea571..0x6f6ea591`。原生 `0x6f6acc10` 释放旧引用并清零，
AfCore `LoadObject 0x68a7b6d0` 请求 ObjectStream 接口，原生
SystemCore `0x7ffcff70..0x7ffd0035` 从内存流消费 16 字节空类标识，
返回 `S_FALSE`，不读取键。该槽完成后 O+`0x284` 仍为空。

| 受控输入 | `AreaOfInterest` 实际输出 |
| --- | --- |
| 显式 AOI、缓存同时存在 | 返回显式 AOI 的副本并赋图层空间参考 |
| 原生加载空 AOI，数据源缺失，缓存有效 | 返回缓存包络的副本 |
| AOI、数据源、缓存均缺失 | `S_OK`，输出空指针 |
| AOI 缺失，数据源拒绝 IGeoDataset，缓存有效 | 返回缓存包络的副本 |
| AOI 缺失，数据源返回有效范围 | 返回数据源范围，并将副本写入缓存 |
| AOI 缺失，数据源返回失败，空间参考回退也返回失败 | 确认调用空间参考回退，AOI 仍返回 `S_OK` 和空指针 |

原生运行覆盖受控接口调用，实际数据源及 ArcMap/QGIS 人工验收仍待验证。
复现探针在忽略目录 `temp/mxd-feature-aoi/`，`result.bin` SHA-256
为 `c66bb34a5ea21b6386ee4f36d86f0fae51e0e8c6e0f01ad9d194e6b93767a55a`。
Rust 后继段现从 v9 起接受显式 AOI 槽中的 Envelope 新对象，复用已有
包络读取器及引用表，结果保存在 `explicit_area_of_interest`。空对象和
回引继续用 `envelope_1` 分类。测试以 v10 尾段确认包络键登记、后续范围
ID 的游标衔接及截断回滚。持久化范围缓存仍走独立槽。
