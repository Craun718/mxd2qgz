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
