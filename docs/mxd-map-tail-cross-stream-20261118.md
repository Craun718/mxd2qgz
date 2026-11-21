# Map 对象末尾、跨流 MapFrame 与字段语义复核

本轮以 ArcMap 10.1.0.3035 的原生 Load 为主要证据，补充反汇编和定点反编译，再用仓库参考工程及安装介质工程核对。地址均为未重定位映像 VA。没有运行 ArcMap，也没有做 ArcMap/QGIS 人工目视检查。

## Map 版本 34 起的后段

核对研究目录 Map.dll 的 SHA-256 为 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`，与 10.1.0.3035 基线一致。复用 `Map/793ff4d0.c`、`Map/79401850.c`，用 `objdump -d -Mintel` 复核 `0x79400ab0..0x794017d8`，用 PE 导入表确认 IAT `0x79480030/34/64` 分别为 LONG、double、short 的 AfStreamRead，`0x79480048` 为 AfCore LoadObject，`0x79480084` 为 AfCore String::Read。

| 读侧条件 | 原生门地址 | 字节序 |
| --- | --- | --- |
| ≥34 | `0x79400ab0` | LONG、double、IEnvelope、IMap、double、LONG、第三对象、short |
| ≥36 | `0x79400c76` | LONG 计数，逐个 double |
| ≥37 | `0x79400cf3` | LONG |
| ≥38 | `0x79400d44` | double、LONG、double、三个对象 |
| ≥39 | `0x79400ed3` | 对象、LONG、String、short、两个 LONG；仅 39–44 再读一 short（门 `0x7940104c`） |
| ≥41 | `0x794010be` | LONG、对象 |
| =42 | `0x7940113b` | 两个 short |
| ≥43 | `0x794011a7` | short、String |
| ≥44、≥45、≥47、≥48 | `0x794011e4`、`0x79401217`、`0x794012c3`、`0x794012ee` | 分别追加对象、short、short、LONG |
| =49 | `0x7940134e` | LONG 计数，每项对象、short |
| =51 | `0x794013f2` | short、对象 |

版本 36 读取的每个 double 传至子接口虚表 `+0x1c`，版本 37 的 LONG 传至同一接口 `+0x10`。`0x79418c10 → 0x79419320` 用常量 `0x794886d8` 查询 `{A709B584-9FB9-4954-8096-02068C16A677}`；esriDisplay 10.1 类型库 TypeInfo 65 为 IDisplayTransformationScales，对应 AddUserScale 和 put_ScaleSnapping。值仍保留原始位。最后一个读点为 `0x7940143d` 的 LoadObject。随后分支对读出的对象调用接口，公共段 `0x794015af` 起更新 Map 状态；`0x794016c6..0x79401713` 的间接调用实参来自 Map 子对象及其 QI 链，没有传入文件流。此前 v34 调查曾把 `0x23` 前的早期集合逻辑误排在本段；这里按跳转地址定位，`0x79400ab0` 此后 v34/35 没有额外集合读取。

实现接受空槽、已见键回引以及首个 IEnvelope 槽内已支持的 Envelope 类，未支持的新对象停在对象头，截断停在该字段起点。计数拒绝负数、上限超出或最小所需长度超出剩余数据。参考工程按版本 51 顺序消费从 `/Maps:0x253e1` 开始的 392 字节，其中 17 个用户比例尺由 `0x2542b` 的计数引导；Map 结束在 `0x25569`，此处根尾 raw=1，流结束在 `0x2556d`。这与原生读序独立吻合，两个 ExtraSafe 栅格图层内部仍有保留字节。没有将特定样本的长度写进读取规则。

## SDK LinearRef 的跨流 MapFrame

后续已读完整 SDK 地图框，并接入同会话的 /PageLayout 元素读取，见[后续研究](mxd-map-frame-body-20261118.md)。下文保留本轮的前缀取证过程。

已有 [PageLayout/SystemCore 跨流证据](mxd-luna-next-cross-stream-20261118.md)证明同一个 ObjectStream 先遍历 /Maps 再切换到 /PageLayout。本轮从 10.1 Desktop `Setup1.cab` 定向提取此前研究导出目录所缺的 `MapElements.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`；PE x86、FileVersion/ProductVersion `10.1.0.3035`、InternalName MapElements、SHA-256 `67a6cbb2dfd1fe2626d32a8fbb1835b38e26b9543833b52628a85e7330d98768`。Ghidra 12.2/JDK 25 headless 补导 GetClassID `0x795709f0`、Load `0x79570a20`、Save `0x79571140`，证据位于忽略的 `temp/mxd-tail-current/`。

- `esri.clsid.xml` 将 MapFrame CLSID `{DD94D770-836D-11D0-87EC-080009EC732A}` 映射到 MapElements.dll；GetClassID 复制 `0x7959beac` 的同一 GUID。RTTI 的持久化虚表 `0x7959c468` 的 Load 槽 `+0x14` 指向 `0x79570a20`；虚基子对象的 thunk `0x79574ef0` 也跳到该入口。
- Load `0x79570a41..6c` 读 2 字节版本并拒绝有符号值大于 4。随后 `0x79570a9b..b0` 用原流调用 LoadObject，请求常量 `0x7959c990` 中的 IMap IID `{E6BDAA75-4D35-11D0-98BE-00805F7CED21}`。这发生在 MapFrame 的其他字段之前；IID 与文件中的 Map CLSID 相差一位，分别对待。
- 10.1 SDK Java LinearRef 实档：`/Maps:0x4` 为 Map 新键 31，体版本 29，顺序读至根尾 `0x8409`；`/PageLayout` 根为键 416，唯一 ExtraSafe 元素 `0x1e`、包装内对象头 `0x26` 为 MapFrame 新键 417，体版本 2；其 `0x3c..0x50` IMap 槽为同 CLSID、键 31 的 20 字节回引。沿真实宿主顺序保留键表可以识别回引；对照只解析 PageLayout 时，这一 IMap 头无法识别为回引，读取器不越过它。此结果无需 GUID 扫描或解析 Debug 字符串。
- 当前读取器只消费到 IMap 槽结束，不跳过元素内的 559 个剩余字节。后面的 BaseFrameElement、对象集合和支持版本仍须分别闭合；没有把这一个已证回引推广到所有跨流对象体或所有 ArcMap 版本。

## ElementProperties 与 FeatureIDSet

- LayerLib.dll（10.1.0.3035、资源内部名 LayerUI、SHA-256 `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`）的 `0x786b97f0` 字段归属见[访问器对位](mxd-terra-chain-element.md)。本轮直接读取 esriCarto 10.1 类型库（SHA-256 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`）的 TypeInfo 159/160：AnchorPoint 的 getter/put 槽为 `+0x38/+0x3c`，`esriAnchorPointEnum` 的记录在文件偏移 `0xf0b58`，0–8 依序对应左上、上中、右上、左中、中心、右中、左下、下中、右下。读取器保留原始 LONG，对已知值额外提供枚举。Load `0x786b9843..0x786b985c` 把 v0/v1 旧 LONG 读入暂存位置，`0x786b9895` 等后续路径覆写该暂存位置；该值不进入任何属性。早于 10.1 的写入者最初赋予此字段的用途仍未知。
- FeatureLayer.dll（10.1.0.3035、SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`）的 Add `0x6f6d96b0` 直接把传入整数交给 `0x6f6d9940`；该插入器仅按 32 位值哈希、判重和插入，无数据源参数或校验调用。Load `0x6f6d97d0` 也逐项把流内整数交给同一插入器；Save `0x6f6d9880` 序列化集合。因此实际 OID 对应关系**不是格式保证**，无需修改读取结果或拒绝无法解析为 OID 的成员。具体宿主生成非空集合时通常传什么值，仍待源数据与实档对应。此前的[重复成员研究](mxd-luna-followup-feature-id-set-20261118.md)继续适用。

## 书签与装饰物

同一已验身份的 Map.dll 与上述 esriCarto 10.1 类型库复核：

- ISpatialBookmark TypeInfo 659 的 Name getter/put 为 `+0x10/+0x14`；IAOIBookmark TypeInfo 661 的 Location 为 `+0x1c/+0x20`；IFeatureBookmark TypeInfo 662 的 FeatureId 为 `+0x1c/+0x20`。Map.dll `0x793a2a90/0x793b56f0` 对这些接口 IID 返回完整对象 `+0x10`，相应虚表分别为 `0x79481528/0x79482bac`。AOI 的 `0x793a2c60/0x793a2c90` 访问接口视图 `+8`，与 `0x793a2cd0` 从持久化视图 `+0x14` 写入的 IEnvelope 槽一致。FeatureBookmark 的 `0x793b5ba0/0x793b5bd0` 也访问接口视图 `+8`，与 `0x793b5c50` 的流内 LONG 一致，确认为 FeatureId。
- 两个 Name 的 getter 分别是 `0x793a2b90` 和 `0x793c1ff0`，访问完整对象 `+0x1c/+0x20`，各自与 Load 字符串落点相同。FeatureClass 通过持久化 IName 的 Open 结果获得，见[书签名称研究](mxd-terra-round9-bookmark.md)；不保证任意 IName 能返回实际要素类。同轮纠正 AOIBookmark 旧注释中的有符号版本比较：Load 的 `jg` 不拒绝负值，Rust 仍只接受 0 和 1。此次只改文档和诊断，版本接受范围未变。
- MapSurround 的 32 字节段不是仅据长度的推测：`0x794414c1..0x794414d5` 对栈上目的地构造 WKSEnvelope，用专用 AfStreamRead 重载读取后直接清理并返回，不复制到装饰物；Save `0x79441510` 取持久化视图 `−0x84` 写同类型。类型库 TypeInfo 178 把 IMapSurround `+0x24` 定名 FitToBounds，原生虚表 `0x7948a410` 该槽为 `0x79441260`。它从输入显示变换取得拟合包络，比较结果需要更新时在 `0x794412e5..0x794412eb` 赋给接口视图 `+0x40`，即完整对象 `+0x58`，也就是 Save 的包络存储。前一 short 后续已定名为 [FixedAspectRatio](arcmap-10.1-mapsurround-bookmark.md#固定纵横比2026-11-18)。

## 验证与剩余条件

CLI `parse` 前后对照同一批 135 份工程（仓库 1、10.1 介质 66、10.8 介质 68），均返回成功；到达根尾的文件分别由 0、37、12 份增至 1、48、23 份。10.8 组仅为回归对照，未改目标版本。`cargo fmt --check`、`git diff --check` 和 `cargo test --no-fail-fast -- --include-ignored` 通过（878 项）。Cargo 回归覆盖版本门、负数/超大计数、截断不越过对象范围、未知新对象未入表，以及实档跨流回引。基线中的 LinearRef 忽略测试会在首个未知 MapFrame 失败，本轮以受原生写读证据支持的读取器取代该探针。

「MXD 解析」节开放项由 60 降至 57；未完成项仍逐项保留在 [TODO 的 MXD 解析节](../TODO.md#mxd-解析)：读写链尚未完全覆盖、其他 MapFrame 和其他嵌套类、未定字段及现有资料没有的旧实档形态。本机参考/介质样本及静态工具可用；未找到可直接使用的 ArcMap 10.1 实机环境，因此依赖打开、另存、派发或异常状态的验证保持开放。

## Map v51 的段落文本元素

Map.dll 10.1.0.3035（SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`）的 `793ff4d0.c` 与原始指令 `0x794013f2..0x7940143d` 对位。版本至少为 51 时，读取 `+0x452` 的 short 后调用普通 LoadObject。请求地址 `0x79485a38` 的实字节为 `{9B001319-9ED7-11D0-859E-0800091A2A72}`，通过同版本 Carto 互操作程序集的 GuidAttribute 定名为 IElement。

10.8 介质八份工程的 Map 版本均为 51，该槽中实际读到键 81、CLSID `{C84598B1-C4BE-4203-9132-ADA2BE57F30C}`，对象头位于 `/Maps:0x1635` 或 `0x1732`。类体使用已按 [CartoXProvider.dll 10.1.0.3035 的 Load 和 Save](mxd-us-media-renderer-paragraph-20261118.md#对象体证据) 验证的 ParagraphTextElement 读取器。仅在装饰和委派的 TextElement 读完后提交键表和终点。其他新 IElement 类仍保留在其槽起点。

合成测试验证最后一个槽的文本读取、逐字节截断回退和其后根尾未被吞掉。八份对照工程由 v51 槽停止推进至确认的地图根尾，10.8 介质的地图根覆盖达到 68/68，布局终点仍为 41/68。10.1 介质（66/66）和仓库样例的地图及布局覆盖不变。未在 ArcMap 或 QGIS 人工验收。

## Map v38 至 v39 的时间对象槽

复核 ArcMap 10.1.0.3035 的 x86 Map.dll（SHA-256
`1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`）。用现有
Map Load 伪代码定位，再用 objdump 核对四次 LoadObject 的 IID、目的成员和版本门。

| 版本及槽位 | 指令与 IID 常量 | 读取范围 |
| --- | --- | --- |
| ≥38，`+0x420` | `0x79400db4..0x79400dc5`，`0x79487e30` | ITimeExtent |
| ≥38，临时对象 2 | `0x79400de3..0x79400df5`，`0x79487e20` | ITimeValue，目前覆盖 TimeExtent 实现 |
| ≥38，临时对象 3 | `0x79400e3b..0x79400e4d`，`0x79487e10` | ITimeReference |
| ≥39，`+0x424` | `0x79400ee5..0x79400eeb`，同 `0x79487e30` | ITimeExtent |

三个 IID 依次为 `BD724B95-018F-4367-9883-9560D92293A7`、
`4CE86E17-E819-4C72-BD98-E55AE59B0317`、`F6CBA4A3-BC48-47BC-9F15-2A561B9E6C71`。
复用 [SystemCore 对象体](arcmap-10.1-dep-tail.md)，完整读取才提交外层与内嵌键。
TimeInstant 作为 Map 的 ITimeValue 宿主、自定义时间对象和运行时赋值仍待验证。

SDK .NET 的 PrintMultiPage（Map v41）在 `0x7f8/0x873/0x8fe` 读取
TimeExtent 键 41、44、47，airportgolf（Map v44）在 `0x2422/0x249d/0x2528`
读取键 101、104、107。每个 TimeExtent 均为 v2，并含两个新建的 v1 Time 对象。
两张 Map 分别结束于 `0x9a3/0x25e5`，其后仅剩四字节 Maps 流尾值。
第三个 v38 槽的非空 TimeReference 及 TimeExtent 截断、未知子类使用构造输入验证。
ArcMap 与 QGIS 的人工验收未执行。

## Map v33 起的内嵌 Map 和根列表回引

在 10.1.0.3035 的 Map.dll（哈希见上文）复核 Load 指令
`0x79400a31..0x79400a42`。普通 LoadObject 请求 `0x79488460` 的
IMap IID `E6BDAA75-4D35-11D0-98BE-00805F7CED21`，保存于 `+0x1b8`。
磁盘 Map 类标识为 `E6BDAA76-4D35-11D0-98BE-00805F7CED21`。
新 Map 由原读取器递归消费，共用暂存引用表和图形对象嵌套限额；
内层未闭合时在 IMap 对象头停止，内嵌键不提交。

Map v45 的 SDK .NET SchematicsForServerSample 在 `/Maps:0xcbe5`
定义内嵌 Map 键 533。它的有界 SchematicLayer 是遮罩遍历原先的停点。
SchematicLayer.dll 10.1.0.3035（SHA-256
`b2dcc605e8953f5e28a3ca25249c4bf5763557173dc52a994d744b6ca69e4d02`）
的主虚表 `0x7d90c91c` 首项 `0x7d8e4350` 是 QueryInterface。
以 objdump 复核固定 IID 的比较链和 `0x7d8e4948` 的末级调用。该调用
进入 `0x7d904180`，仍仅比较固定 IID，未命中在 `0x7d904272`
返回 E_NOINTERFACE。两级都没有 ILayerMasking 比较或未知 IID 动态委派。
SchematicLayer 体仍按已确认的外层范围保留原始未知内容。

根列表第四槽 `0x2c493` 是键 533 的回引。根读取器复用之前解析的
前缀与后缀生成第四个数据框，共用根级键与后继布局的回引检查。
Maps 根尾写在 `0x2c4a7`，其后没有未读字节；PageLayout 也读至流尾。
实档回归验证四个根槽与布局，构造测试验证内嵌体截断时的游标和引用表。
当前覆盖该 IMap 槽内的标准 Map 实现。未知集合类和其他宿主的新 Map
仍需独立证据，ArcMap 与 QGIS 人工验收未执行。

### Map v44 的 TimeInstant

Map.dll 10.1.0.3035 在 `0x794011e4..0x79401209` 对 v44 起的 `+0x44c` 执行 LoadObject，使用与 v38 临时槽相同的 ITimeValue IID 常量 `0x79487e20`。两处均复用已由 SystemCore.dll 10.1 核实的 TimeInstant 读侧，消费版本、ITime 和 ITimeReference。对象体完整才登记外层及嵌套键。

两份 SanFrancisco SDK 工程在 `/Maps:0xdbc13` 读取 TimeInstant 键 5644 和 Time 键 5645，随后读完 Map 对象，并在 `0xdbc65` 消费根循环尾值 1。流中仍有此前的有界未知图层数据，PageLayout 仍有未知内容。构造测试覆盖 ITimeValue 槽和逐字节截断回滚。ArcMap/QGIS 人工验收尚未进行。
