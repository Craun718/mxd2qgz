# ArcMap 10.1 无样本分支证据升级与取证路径（2026-11-17）

本文对应 `TODO.md` 第 262-277 行的 16 个「Hyperlink、NumberFormat、StdPicture、
OLE 字体、BOLP/UID/HLEP/DEP、Geometry、后继段、TimeData、PropertySet、
Envelope、FeatureDatasetName、TopologyName」开放项。本轮只做只读研究与本文
落档；未修改 Rust 源码、测试、`README.md` 或 `TODO.md`，未运行 ArcMap 或
QGIS，未新增生产实现或接线。

## 证据分级

本文严格区分四类证据：

| 级别 | 含义 | 本文使用方式 |
| --- | --- | --- |
| A 样本证据 | 已按对象边界走查的真实 MXD 字节 | 可证明字段序和该样本形态，不证明全部写侧行为 |
| B 反编译证据 | 10.1.0.3035 原生 DLL 的静态伪代码/汇编/证据包 | 可闭合读侧或写侧分支，不证明运行时状态 |
| C 合成测试 | 构造输入下的读取器行为 | 只证明实现契约，不能升格为 MXD 格式验证 |
| D 运行取证 | ArcMap 10.1 执行、保存或调试器观察 | 本轮均未执行 |

本轮新增的机械复核只有三类：原始 MXD 与脱敏 `/Maps` 导出的 CLSID 字节计数、
仓库内外既有二进制/反编译产物身份复核、AfCore 全量伪代码索引检索。对象边
界仍引用既有走查结论，不把粗扫计数当对象实例边界。

## 基线与样本复核

本轮复核到的关键文件身份如下：

| 文件 | SHA-256 | 说明 |
| --- | --- | --- |
| AfCore.dll 10.1 | `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17` | `temp/arcgis101-bin/AfCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF` |
| SystemCore.dll 10.1 | `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a` | `temp/arcgis101-bin/SystemCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF` |
| FeatureLayer.dll 10.1 | `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60` | `temp/arcgis101-bin/featurelayerdll/FeatureLayer.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` |
| Map.dll 10.1 | `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8` | `temp/arcgis101-bin/mapdll/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` |
| Geometry.dll 10.1 | `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47` | `temp/arcgis101-bin/geometrydll/Geometry.dll.F84366F9_C0B7_4267_A16E_D48D9E6DD261` |
| GdbCore.dll 10.1 | `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4` | `temp/agent-dataset-name/GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA` |
| LayerLib.dll 10.1 | `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02` | `temp/agent-fl-tail/LayerLib.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` |
| LabelPlacement.dll 10.1 | `8415f63c1889e269ba6ae585538f601130c20cef02e4556e875a52217d35ea3d` | `/tmp/mxd101-labelplace/LabelPlacement.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`（本轮未复制入库） |
| DisplayCore.dll 10.1 | `9b3e61f9b581c027e0868b73ff576d966852b6b917972042d3a82bc4b292642e` | `temp/agent-fl-tail/DisplayCore.dll.33A02325_2485_4623_B2CE_71B5736B51DF` |
| oleaut32 介质 2.40.4275 | `baeb2f7c1b8be56738d34e1d1ddf8e0eebd3a633215dc1575e14656be38b939d` | `/tmp/g-cablist/Global_System_OLEAUT32_f3.8C0C59A0_7DC8_11D2_B95D_006097C4DE24`，本轮 pefile 复核 x86、ImageBase `0x65340000`、版本资源 2.40.4275 |

`temp/arcgis101-bin/AfCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF`、
`temp/arcgis101-bin/mapdll/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`、
`temp/arcgis101-bin/SystemCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF` 三份
哈希与既有 10.1 基线一致。`temp/arcgis/native/` 下的同名 DLL 哈希不同，本轮
不作为 10.1 原生证据。

样本复核结果：

| 输入 | SHA-256 | 长度 |
| --- | --- | --- |
| `samples/测试底图.mxd` 原始 MXD | `14772f8e3fedca8e1760bef77ab418011df4d8105f5159a05bc866bf603e3738` | 1089536 字节 |
| 脱敏 `/Maps` 导出 `temp/rea-agent-sample-20261117/maps.bin` | `fc0149f1cdbd4abb893610c0b442382ea0cc4c191c434173113db26b01aeacfe` | 152941 字节 |

原始 MXD 与脱敏导出的 CLSID 16 字节原样计数如下。这些是字符串级命中，只做
证据规模复核；首个原始文件偏移比 `/Maps` 导出偏移固定大 `0x30000`，与既有
对象级偏移自洽。

| CLSID | 类 | 原始 MXD 命中 | `/Maps` 导出命中 | 首个导出偏移 |
| --- | --- | ---: | ---: | --- |
| `{EE535289-41C9-11D1-880A-080009EC732A}` | BOLP | 24 | 12 | `0xda3` |
| `{78FF7FA1-FB2F-11D1-94A2-080009EEBECB}` | UID | 145 | 13 | `0x14a9` |
| `{1B848B0F-5E87-4948-841A-86201FACD925}` | HLEP | 2 | 1 | `0x15d8` |
| `{D75C6301-C05A-484C-A7BD-AE82E5E1FC75}` | DEP | 8 | 4 | `0x151a` |
| `{3036D35E-EDE5-11D0-87FE-080009EC732A}` | Hyperlink coclass | 0 | 0 | 无 |
| `{0BE35204-8F91-11CE-9DE3-00AA004BB851}` | StdPicture | 0 | 0 | 无 |

现有对象级走查已确认的样本边界继续有效：BOLP 键 69 体 `0xdb7..0xec5`；UID
键 82 头 `0x14a9` 等 13 个 `/Maps` 实例；HLEP 键 85 头 `0x15d8`、体
`0x15ec..0x1605`；DEP 键 83 头 `0x151a`、体 `0x152e..0x1542`。Hyperlink 的
`IHyperlink` 循环计数为 0；StdPicture 在原始 MXD 和 `/Maps` 导出中均为零命中。

## 逐项状态

### Hyperlink 版本 0 短体

**状态：读侧短体已由反编译闭合并有合成覆盖；缺 Hyperlink 专属样本字节。**

可闭合证据：

- FeatureLayer.dll `FeatureHyperlink::Load 0x6f6d9170` 与 `Save 0x6f6d9210`
  对称：版本门 `1 < (short)ver` 有符号拒绝；体恒为 `short 版本 + int32 +
  AfCore String + int32`。Save 恒写版本 1，版本 0 是门内兼容短体，不是 10.1
  写侧产物。
- `src/objects/hyperlink.rs` 与 `tests/hyperlink.rs` 已覆盖版本 0、版本 1、
  版本 2 拒绝、负值位模式拒绝、String 限额和截断事务性。

升级路径：

1. 真实样本：取一个含热链或超链接条目的 ArcMap 10.1 MXD；在 `FeatureLayer::Load`
   后继段 `IHyperlink` 循环内按对象边界捕获 `{3036D35E}` 头及其体。10.1 写侧
   预期版本 1，版本 0 只能来自旧文档。
2. 旧版样本：取得 ArcMap 8.x/9.x 保存且可能写 Hyperlink 版本 0 的 MXD，在
   10.1 中打开后导出流。
3. 运行取证：受控构造含 Hyperlink 的 ArcObjects 工程，保存前后读取底层流；
   或在旧版 ArcMap 写出路径上断点确认版本 short。

未解决：版本 0 是否在历史上确有写侧来源；`int32 + String + int32` 的业务语
义仍为候选。

### Hyperlink 循环空标记/回引形态

**状态：循环机制与 ObjectStream 通用空标记/回引编码闭合；Hyperlink 宿主循
环内无专属样本条目。**

可闭合证据：

- SystemCore ObjectStream 读分支 `0x7ffcff70` 的汇编与伪代码在
  `temp/r2-sc-7ffcff70.json` 与 `temp/r2-sc-7ffcff70-asm.json` 中可复查：先读
  16 字节类 GUID；16 字节全零时 `MOV EAX,1` 返回 S_FALSE，不读键；非零再读
  4 字节键，查表命中则对既有对象 `QueryInterface`，未命中才创建、登记并调
  对象 `Load`。
- AfCore `SaveObject` 空对象写 16 字节零；已有键回引写「16 字节类标识 + 4 字
  节既有键」，无体。样本 `0x2d2c` 与键 2139 回引实例可复查
  （`docs/arcmap-10.1-object-body.md` 第五轮）。
- 参考样本的 `FeatureLayer::Load` 后继段 `IHyperlink` 计数恒为 0，因此没有
  Hyperlink 自身的空标记或回引条目。

升级路径：

1. 真实样本：取含空 Hyperlink 槽（16 字节零）和至少两个 Hyperlink 对象的
   MXD，其中一项共享同一对象（例如同一链接被多个字段复用），验证回引键。
2. 运行取证：ArcObjects 建两个 `IHyperlink` 引用同一实例并保存，读取底层流
   确认第二项只出现 20 字节头。

未解决：在 Hyperlink 宿主循环的实际出现频率、条目键范围和请求 IID 上下文中
的表现；通用机制不能替代该宿主的样本字节。

### NumberFormat 短体形态（九类全族）

**状态：九类读侧字段序与合成覆盖闭合；参考样本只有 NumericFormat，其余八类零命中。**

九类 10.1 体均不含 20 字节 ObjectStream 对象头。各读取器以有符号版本门接受
0 及门内版本，负值位模式拒绝（较原生更严）；下表只列流内字段，10.1 Save 版
本来自反编译常量或版本表。

| 类 | 版本门 | 短体字段 | 版本升级字段 | 10.1 Save 版本 |
| --- | --- | --- | --- | --- |
| NumericFormat `{7E4F4719}` | `>1` 拒绝 | v0/v1 同路径：版本 + 4×int32 + 3×short | 无条件分支字段 | 1 |
| PercentageFormat `{7E4F471B}` | `>2` 拒绝 | v0/v1：版本 + short | v2：同槽 NumericFormat 核心 | 2 |
| FractionFormat `{7E4F471C}` | `>1` 拒绝 | v0/v1 同路径：版本 + 2×int32 | 无条件分支字段 | 1 |
| RateFormat `{7E4F4721}` | `>2` 拒绝 | v0/v1：版本 + double + AfCore String | v2：同槽 NumericFormat 核心 | 2 |
| CurrencyFormat `{7E4F471A}` | `>1` 拒绝 | v0/v1 同路径：版本 short | 无条件分支字段 | 1 |
| LatLonFormat `{7E4F471D}` | `>4` 拒绝 | v0/v1：版本 + 2×short；v2 再 + 2×short | v3：独立槽 NumericFormat 核心；v4 再 + short | 表 `(3,3,4)` |
| AngleFormat `{7E4F471E}` | `>3` 拒绝 | v0/v1：版本 + short | v2：独立槽 NumericFormat 核心；v3 再 + short | 表 `(2,2,2,2,3)` |
| ScientificFormat `{7E4F471F}` | `>2` 拒绝 | v0/v1：版本 short | v2：int32 | 2 |
| CustomNumberFormat `{7E4F4722}` | `>1` 拒绝 | v0/v1 同路径：版本 + AfCore String | 无条件分支字段 | 1 |

可闭合证据：

- SystemCore 九类 RTTI→COL→IPersistStream 虚表→GetClassID→Load/Save 定位链
  已记录在 `docs/arcmap-10.1-number-format.md`；Save 常量和版本表支持读侧门
  限。
- `src/objects/number_format.rs`、`src/layers/feature_layer_tail.rs` 的九类
  分派和 `tests/number_format.rs`、`tests/feature_layer_tail.rs` 覆盖九类正
  常体、短体、越界版本、负值位模式、截断和槽级停止。
- 全流 CLSID 扫描与既有样本走查一致：NumericFormat 有样本实例；Percentage、
  Fraction、Rate、Currency、LatLon、Angle、Scientific、CustomNumber 为 0
  命中。

升级路径：

1. 真实样本：取 FieldInfo 属性格式分别设置为 Numeric、Percentage、Fraction、
   Rate、Currency、LatLon、Angle、Scientific 和 CustomNumber 的 ArcMap 10.1
   MXD，按 FieldInfo 槽对象边界逐类捕获头键、版本和体。
2. 受控 COM：创建九类 `INumberFormat` 实现并经 IPersistStream 保存；若无法控
   制 docVersion，则只能验证 10.1 满形态，不能构造历史短体。
3. 旧版样本：Percentage v0/v1、Rate v0/v1、LatLon v0/v1/v2、Angle v0/v1 和
   Scientific v0/v1 主要只能来自旧写侧；需旧产品 MXD 或旧版本运行取证。
4. 归因核对：新样本必须按对象头 CLSID 区分九类，不能把嵌套核心误判为独立对象。

未解决：九类标量与字符串的业务语义；LatLon/Angle Save 版本表的索引与封顶语
义；版本 0 是否有历史写侧来源。

### NumberFormat 同槽升级与 LatLon/Angle 独立槽核心

**状态：静态读写对照闭合；无对应样本字节。**

可闭合证据：

- Percentage：v2 时外壳 short 与内嵌 NumericFormat 核心升级到同一对象槽
  `this-0xc`；流内只是顺序字节，没有独立对象头或键。
- Rate：v2 时 double/String 外壳槽 `this-0x14` 升级为核心槽；流内同样顺序
  写核心，无对象头。
- LatLon：核心在独立槽 `this-0x38`，v3 起；返回值受检，失败即整体失败。
- Angle：核心在独立槽 `this-0x10`，v2 起；原生不检查核心返回值。
- Save 版本表保证 10.1 的 LatLon 至少 v3、Angle 至少 v2，恒写含核心形态。

升级路径：

1. 真实样本：同一字段分别设置 LatLonFormat 和 AngleFormat 后保存；对象边界
   内按上述版本门检查核心位置。
2. 运行取证：受控创建 Percentage/Rate/LatLon/Angle 对象，保存并断点核对
   `NumericFormat::Load 0x7ffce4b0` 调用点与 this 槽。
3. 兼容性样本：旧版本 MXD 若含短体，可验证 v 低于核心门时不读核心、后续字
   段边界是否严格按读侧分支。

未解决：`this-0xc/-0x14/-0x38/-0x10` 的成员名和业务语义；真实对象状态下的
槽复用效果。

### Angle 原生容错路径

**状态：反编译事实闭合，运行态和截断流未取证；当前读取器较严。**

可闭合证据：

- AngleFormat Load `0x7ffb33a0` v2 调用 `NumericFormat::Load 0x7ffce4b0` 后
  无 TEST/JL 检查，继续读 v3 尾 short。`src/objects/number_format.rs` 明确
  记录该差异；当前实现把核心失败上抛，不模拟「继续读」。

升级路径：

1. 运行取证：受控 AngleFormat 对象在 v2/v3 流中让 NumericFormat 核心读取失
   败，用调试器记录核心返回 HRESULT、外层 Load 返回值、游标位置和对象内存。
2. 恶意/损坏样本：取得可让核心截断但后续仍有 Angle 尾 short 的旧流，观察
   ArcMap 是否部分装载、忽略对象或整体报错。
3. 静态补充：反编译 `NumericFormat::Load` 失败路径对 AngleFormat 对象槽的写
   副作用，确认「忽略返回值」后对象是否处于一致状态。

未解决：核心失败后的游标副作用和对象状态；真实损坏流行为。

### StdPicture 第二字语义与解码器分派

**状态：介质 oleaut32 2.40.4275 的头形态与魔数分派观察闭合；第二字语义、各
解码器入口和真实图像样本未闭合。**

可闭合证据：

- StdPicture 体 Save 头为 `0x746c + word2 + int32 data_size`，第二字 Save 写
  0；Load 不校验 word2。图像数据按首 WORD 分派：`0x4D42` BITMAP、`0xFFD8`
  JPEG、`0x4749/0x6967` GIF、`0xCDD7`、`0x0001`、`0x0000`。
- 本轮 pefile 复核 `/tmp/g-cablist/Global_System_OLEAUT32_f3.8C0C59A0_7DC8_11D2_B95D_006097C4DE24`
  为 x86、ImageBase `0x65340000`、2.40.4275；这是介质分发件，不是已确认的
  ArcMap 10.1 运行时 OS DLL。
- 参考样本和原始 MXD 的 StdPicture CLSID 均 0 命中；
  `src/objects/ole_font.rs` 原样保留 word2 和图像 blob。

升级路径：

1. 真实样本：取含位图、JPEG、GIF、EMF/WMF 图标符号的 ArcMap 10.1 MXD，按对
   象边界记录 `{0BE35204}` 头、word2、长度和图像首字节。
2. 受控构造：用 IPicture 或 OleCreatePictureIndirect 分别构造六类图像对象，
   经 IPersistStream 保存到定制流，交叉验证 word2 是否恒 0。
3. 运行库对照：取 Windows XP SP3 `C:\Windows\System32\oleaut32.dll`（5.1.x）
   与介质 2.40.4275 的 `OlePictureImpl::Load/SaveHeader` 汇编逐指令对照。
4. 解码器入口：对 2.40.4275 的六个魔数分支逐一反编译入口，记录非流内的对
   象状态变化；这能闭合「分派」，但仍不能替代第二字语义的运行观察。

未解决：word2 的业务语义；各解码器入口完整字段序；OS 运行库版本一致性。

### OLE 字体持久化跨 oleaut32 版本稳定性

**状态：2.40.4275 反编译与 37/37 StdFont 样本对位闭合；目标 OS 运行库未取
得，跨版本稳定性未证明。**

可闭合证据：

- 介质 2.40.4275 的 `OleFontImpl::Load 0x65399520` / `Save 0x65399626` 闭合
  字体体；样本 37 处 StdFont 逐字节吻合，其中 TextSymbol 字体槽 21 处、
  CharacterMarker 系字体槽 16 处。
- 本轮复核的 2.40.4275 本体在 `/tmp`，仓库 `temp/agent-olestd/` 目前不存在；
  `/tmp/mxd101-typelib-build/oleaut32.dll` 是 Wine DLL，不能作为证据。
- 目标运行时 XP oleaut32 5.1.x 未获取。

升级路径：

1. 运行库对照：取得 XP SP3 oleaut32 5.1.2600 或相应 Server 版本，对字体
   Load/Save、StdPicture 头循环做函数级汇编 diff。
2. 受控写侧：在 XP + ArcMap 10.1 中保存同一工程，导出流比对 37 处 StdFont
   体；同时在受控 COM 程序中调用 `OleCreateFontIndirect` 保存，记录版本、
   charset、标志位包、weight、CY.lo、名字字节。
3. 边界用例：构造 0x21 字节名字、非规范 flags 位、CY 高位非零和 0 字节名字，
   观察两版本 Load 的接受域与截断行为。

未解决：OS 版本差异是否影响字体/图片持久化；非规范位模式的宽容度。

### BOLP/UID/HLEP/DEP 前缀样本结构级证据升级

**状态：需要拆开评级，不能作为整体闭合。**

| 类 | 样本结构证据 | Load 反编译唯一定位 | 当前评级 | 升级后仍缺 |
| --- | --- | --- | --- | --- |
| BOLP `{EE535289}` | 键 69 体 `0xdb7..0xec5` 完整对位；本轮粗扫 `/Maps` 12 处、原始 MXD 24 处 | 未唯一定位；`0x7853d3a0` 只是局部佐证且尾段不吻合 | 样本结构 + 局部反编译 | BOLP::Load 唯一归属和各槽业务语义 |
| UID `{78FF7FA1}` | `/Maps` 13 个对象级实例；本轮粗扫原始 MXD 145 处 | 已唯一定位：SystemCore IPersistStream 虚表 `0x80074574`，Load `0x7fff9320`、Save `0x7fff9380` | 可升级为 UID 体静态闭合 + 样本对位 | UID 标志/GUID/int32 的业务语义 |
| HLEP `{1B848B0F}` | 键 85 头/体完整对位；本轮粗扫 `/Maps` 1 处、原始 MXD 2 处 | 未定位 | 仅样本结构 + 构造器成员形状 | Load/Save 唯一归属和 16 字节块语义 |
| DEP `{D75C6301}` | 键 83 对象级对位；本轮粗扫 `/Maps` 4 处、原始 MXD 8 处 | 已唯一定位：FeatureLayer.dll 虚表 `0x6f7ab73c`、Load `0x6f6cbdd0`、Save `0x6f6cbe70`；A 轮裁决体止于引擎槽 | DEP 体静态闭合 + 样本对位 | 表达式和 int16 的业务语义 |

可闭合证据：

- 本轮把原始 MXD 与脱敏 `/Maps` 的 CLSID 计数交叉复核，证明前缀样本不是单
  一清洗流假象；但计数不新增对象边界。
- UID 的一手证据见 `docs/mxd-boundary-luna.md`：虚表三槽和 23 字节体（short
  版本、u8 标志、16 字节 GUID、int32）已反编译，样本字段序吻合。
- DEP 的一手证据见 `docs/arcmap-10.1-dep-tail.md`：Load/Save 定位、字段序和
  「无 DEP 尾段」裁决。

升级路径：

1. UID：可直接按上述唯一 Load/Save 完成「体静态闭合」的表述升级；若要闭合
   语义，需要类型库成员命名或运行断点观察 GUID 指向和 int32 值来源。
2. DEP：布局已闭合；语义升级只需类型库成员或运行值域取证。
3. BOLP：在 LabelPlacement.dll 全镜像搜索 RTTI COL 簇对应的全部虚表，并以
   「GetClassID 写 `{EE535289}` + IPersistStream 槽 + AfStreamRead 调用」三
   条件过滤；重点复核 `0x7853d3a0` 的调用者是否是 BOLP 子对象接口，而非外
   层 `IPersistStream::Load`。
4. HLEP：LabelPlacement.dll 中以 `{1B848B0F}` CLSID 数据引用、GetClassID 实
   体和 RTTI `.?AVHotLinkExpressionProperties@@` 交叉定位；若该 DLL 确无流框
   架，扩展到 FeatureLayer.dll 和 esri 全模块扫描。
5. 运行取证：标注层属性面板保存前后断点，记录 BOLP/HLEP 对象 `Load` 的返
   回地址与 v2 样本字段状态。

未解决：BOLP/HLEP 的一手 Load 归属；四类业务语义。

### Geometry 版本 0/1

**状态：版本门和 v0/v1 布局反编译闭合；参考样本四类均无 0/1 实例。**

可闭合证据：

- Polyline/Polygon 共享 `0x7192db20`；Point `0x7191c1a0`；Multipoint
  `0x718ff9c0`。版本门为有符号 `(short)ver > 2` 拒绝，因此 v0/v1 在门内。
- v2 才读空间参考段；v0/v1 不读该段。Point 另有 80 字节栈缓冲长度门。当前
  读取器实现 v0/v1 并在合成测试中覆盖；负值位模式拒绝较原生更严。
- 样本只有 Polyline 16 实例且全部 v2；Polygon/Point/Multipoint 为 0 实例。

升级路径：

1. 旧版真实样本：取 ArcMap 8.x/9.x 或早期导出流中含 Polygon/Point/
   Multipoint 的 MXD；v0/v1 主要只能来自旧写侧。
2. 运行取证：受控旧版本 ArcObjects 或旧 ArcMap 保存简单几何；若无法安装旧
   版本，则用 10.1 读取旧档并记录是否按 v0/v1 路径解析。
3. 边界核对：v1 记录后直接接下一对象头，验证无空间参考段；v0 记录重点验证
   是否与 10.1 写侧恒 v2 的历史流真实共存。

未解决：v0 是否有历史写侧来源；记录类型字与 esriGeometryType 的完整映射。

### Geometry S_FALSE 捷径分支

**状态：静态调用存在，未实现；运行态无证据。**

可闭合证据：

- 四类 Geometry Load 在版本 short 的 `AfStreamRead` 返回 1（S_FALSE）时调用
  内嵌接口并原样返回 1。当前 `src/objects/geometry.rs` 不实现该分支；这是
  严格子集，不是对原生的等价模拟。

升级路径：

1. 运行取证：用受控 IStream 包装返回 S_FALSE 和 0 实读，直接调用 Geometry
   对象 `IPersistStream::Load`，记录内嵌接口调用、对象状态和返回值。
2. 目标软件观察：构造导致版本 short 读返回 S_FALSE 的流（例如底层流边界恰
   在对象头后），在 ArcMap 打开路径断点观察是否进入捷径。
3. 静态补充：反编译 `this-0x1c` 接口 `+0x20` 槽的实际实现，确认它是否清空
   空间参考/几何内核状态；这可辅助解释，但不能替代运行态。

未解决：S_FALSE 时对象是否被上层视为空几何、错误或未初始化；是否与零字节
记录路径叠加。

### 后继段版本 0x1c..0x1f 分支

**状态：字段序静态闭合且门控已实现；样本 FeatureLayer 版本 34 不可达。**

可闭合证据：

- FeatureLayer 后继段在 `0x1b < ver && ver < 0x20` 读 `short + 4×String +
  short + double + int32`，随后 v0x1d 加 `ITimeReference`、v0x1e 加
  `ITimeExtent`、v0x1f 加 `IHotLinkExpressionProperties`。
- FL::Save 版本表 `[17,22,23,25,34,34]` 跳过 0x1a..0x1f，10.1 写侧不产生；
  参考样本 FL 版本 34 走 `>=0x20`。
- `src/layers/feature_layer_tail.rs` 已按门接入，前两时间槽接 `dep_tail`
  分派；第三槽未知类停止。样本不可达，缺该分支的字节对位。

升级路径：

1. 旧版真实样本：取 ArcGIS 9.0/9.1/9.2/9.3 保存的 MXD，使 FL 架构版本落在
   0x1c..0x1f，按对象边界走查标量块和三个时间槽。
2. 版本映射取证：先反编译 `AfStreamGetSaveVersion` 的表索引规则并对照
   `esriArcGISVersion`，再决定能否用受控 docVersion 生成；10.1 本身不产。
3. 运行取证：在旧版本 ArcMap 中保存含时间启用或热链属性的图层，导出流记录
   该分支。

未解决：分支内标量和四个字符串的业务语义；`{E720618B}` 槽在该分支的实现类
分布。

### TimeData 族四段

**状态：四段字段序、Time 族实现类和空槽样本形态闭合；非空实现类对象无样
本。**

可闭合证据：

- 后继段 `>=0x20` 四段为 TimeData、TimeTableDefinition、TimeDimension-
  Definition、TimeDataDisplay；样本 `0x156e..0x15d7` 全槽空标记并逐字节吻合。
- TimeReference/TimeExtent/Time/TimeZoneInfo 四类在 SystemCore.dll 有唯一
  RTTI→COL→IPersistStream 定位链；TimeExtent v2 需 Time 槽和 TimeReference
  槽，TimeReference v1 需 TimeZoneInfo 槽。`src/layers/dep_tail.rs` 已实现
  并覆盖嵌套 TimeReference/TimeExtent、空槽、未知类停止和截断。

升级路径：

1. 真实样本：取启用时间、时间表、时间维度或时区属性的图层保存为 10.1
   MXD；至少覆盖 TimeData v2 + TimeReference、TimeTable + TimeExtent 两链。
2. 受控 ArcObjects：创建 `ITimeReference/ITimeExtent/ITime/ITimeZoneInfo`
   并挂接到图层时间属性后保存。
3. 运行取证：保存前断点记录四个槽成员，保存后对头键、版本、字符串和规则
   块逐字节走查。

未解决：时间字段业务语义、Time 判别值 OA date/WKSDateTime 映射、规则块字段
语义。

### PropertySet 版本 0 分支

**状态：v0/v1 同路径反编译与合成覆盖闭合；参考样本两实例均 v1。**

可闭合证据：

- SystemCore PropertySet Load `0x7ffd3080` 有符号 `>1` 拒绝；v0/v1 均读
  `int32 计数 + 计数×[String 键 + Variant::Read 值]`。Save 恒写 1。当前实现
  接受 v0 空集合并合成覆盖。

升级路径：

1. 旧版真实样本：取早期 ArcMap/ArcInfo 保存的连接属性，寻找 WorkspaceName
   体 `IPropertySet` 槽内 v0 对象。
2. 运行取证：受控创建 PropertySet，在旧版本保存或以版本 0 的持久化路径调
   用，记录计数 0、非空字符串、各类 VARTYPE 的字节。
3. 版本来源排查：静态反编译 PropertySet::Save 版本常量已证 10.1 恒写 1，
   因此 v0 只能来自旧写侧或异常写者。

未解决：v0 是否历史合法；各 VARTYPE 和键名业务语义；对象分支 9/13 的边界。

### Envelope 版本 0 分支

**状态：现有证据不支持 v0 接受；读取器显式拒绝正确。**

可闭合证据：

- Envelope::Load `0x71869b60` 只记录 v1 只读 32 字节、v2/v3 读坐标+flags、
  v3 读空间参考；版本 `>3` 拒绝。Save `0x71869da0` 恒写 3。样本 25 个
  Envelope 全 v3。没有 v0 分支或 v0 字节。

升级路径：

1. 静态重查：若未来获得不同 build 的 Geometry.dll，反编译 Envelope Load 的
   版本判断；不能从 10.1.0.3035 现有证据推出 v0。
2. 旧版字节：只有在旧版本或异常写者流中实际出现 v0 后，才允许新增解析分支；
   该分支必须由新反编译证据而不是兼容性猜测支撑。
3. 运行取证：受控旧版本保存 Envelope，记录版本 short。

未解决：符号性比较（当前无符号拒绝负值位模式）在原生二进制中的准确指令；
v0 是否存在。

### FeatureDatasetName 版本 0 分支

**状态：v0/v1 同字段序反编译与合成覆盖闭合；样本 CLSID 零命中。**

可闭合证据：

- GdbCore FeatureDatasetName Load `0x709b7c90` 有符号 `>1` 拒绝；v0/v1 均
  读四字符串 + 1 字节标志 + IWorkspaceName 槽。Save 恒写 1。
  `tests/dataset_name.rs` 覆盖 v0 通过、v2/负值拒绝。
- 本轮复核 `/Maps` 与原始 MXD 的 `{198846CF}` 均 0 命中；既有 11 个 FCN 选
  择子全为 0，也未出现 FDN 对象。

升级路径：

1. 真实样本：取含要素数据集（Feature Dataset）内要素类或拓扑图层的 MXD，
   使 FCN 的 IDatasetName 槽出现 FDN 新对象。
2. 运行取证：ArcObjects 构造 FeatureDatasetName 并挂 WorkspaceName 后保存；
   10.1 预期 v1。v0 需旧档。
3. 边界核对：v0 时四字符串和槽边界与 v1 相同；确认旧写侧是否真的使用 v0。

未解决：FDN 四字符串和标志语义；v0 历史写侧来源。

### TopologyName 版本 0 分支

**状态：v0/v1 同字段序反编译与合成覆盖闭合；样本 CLSID 零命中。**

可闭合证据：

- GdbCore TopologyName Load `0x7095e5c0` 有符号 `>1` 拒绝；v0/v1 均读两字
  符串 + IDatasetName 槽。Save 恒写 1。B 轮起 TN 槽统一分派已接入；
  `tests/dataset_name.rs` 覆盖 v0 通过和越界拒绝。
- 本轮复核 `/Maps` 与原始 MXD 的 `{9FCA50EC}` 均 0 命中。

升级路径：

1. 真实样本：取含拓扑图层的 File GDB/Enterprise GDB MXD，使 FCN 第二选择子
   或 IDatasetName 槽出现 TN 对象。
2. 运行取证：ArcObjects 构造 TopologyName 并保存；10.1 预期 v1，v0 需旧档。
3. 边界核对：v0 时两字符串后仍接 IDatasetName 槽；确认历史拓扑工程版本。

未解决：TN 两字符串语义、槽后虚表通知接口语义、v0 历史写侧来源。

## 共同闭合路径

按优先级：

1. **真实 10.1 样本优先**。任何新样本都要保留原始 MXD，并记录 ArcMap 版本
   、保存路径和生成操作；对象边界走查优先于全文 CLSID 粗扫。
2. **旧版本样本是短体/旧分支的唯一自然来源**。0x1c..0x1f、NumberFormat 短
   体、Geometry v0/v1、TimeData 非空链等在 10.1 写侧大多不产或样本不可达。
3. **受控 COM 运行取证适合单对象契约**。能用 IPersistStream 直接驱动时，记
   录流字节、HRESULT、游标位置和对象状态；这比完整 ArcMap 保存更易定位分支。
4. **OS 运行库对照只对 OLE 两项必要**。目标 XP oleaut32 5.1.x 与介质 2.40
  .4275 必须函数级对比；不能用 Wine DLL 代替。
5. **反编译唯一性升级**。BOLP/HLEP 需要新的 CLSID/RTTI/虚表三条件检索；
   UID/DEP 已可直接按唯一 Load/Save 表述升级。

## 本轮新增检查与辅助文件

本轮未新建 `temp/` 一次性脚本，CLSID 计数用会话内只读 Python 完成；未保存输
出文件。实际引用或复核的辅助产物如下：

- 创建的唯一输出：`docs/mxd-parse-agent-sample-evidence-edge-branches-20261117.md`。
- 样本与身份：`samples/测试底图.mxd`、`temp/rea-agent-sample-20261117/maps.bin`、
  `temp/rea-agent-sample-20261117/scan-result.json`。
- ObjectStream 一手证据：`temp/r2-sc-7ffcff70.json`、
  `temp/r2-sc-7ffcff70-asm.json`、
  `temp/arcgis/arcmap101-object-body-evidence.json`。
- Envelope/几何写向证据：`temp/arcgis/arcmap101-envelope-save-evidence.json`、
  `temp/arcgis/arcmap101-spatialref-pseudocode/0x719d3230.c`。
- AfCore 全量伪代码：`temp/rea-decompile-20261117/AfCore/decompile-index.json`、
  `temp/rea-decompile-20261117/AfCore/decompile-journal.jsonl`、
  `temp/rea-decompile-20261117/AfCore/functions/`。
- 10.1 二进制基线：`temp/arcgis101-bin/`；GdbCore 副本
  `temp/agent-dataset-name/GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA`；
  LayerLib/DisplayCore 副本 `temp/agent-fl-tail/`。
- OLE 介质本体本轮只读复核：`/tmp/g-cablist/Global_System_OLEAUT32_f3.8C0C59A0_7DC8_11D2_B95D_006097C4DE24`；
  `/tmp/mxd101-typelib-build/oleaut32.dll` 为 Wine DLL，已排除。

## 最终未闭合清单

1. Hyperlink 版本 0 与宿主循环专属字节：需要真实/旧档样本或运行取证。
2. NumberFormat 八个非样本类、短体、同槽/独立槽和 Angle 容错：需要真实格式
   样本或受控运行取证；Angle 还需核心失败后的状态观察。
3. StdPicture word2、解码器入口和 OS oleaut32 对照：需要图像样本、XP 运行库
   或受控 IPicture 保存。
4. OLE StdFont 跨 oleaut32 稳定性：需要 XP/实际 OS 运行库本体和受控写侧。
5. BOLP::Load、HLEP::Load/Save 的唯一归属；UID/DEP 语义仍开放。
6. Geometry S_FALSE 运行态；Geometry v0/v1 与后继段 0x1c..0x1f 需旧档或旧
   版本运行。
7. TimeData 非空链、PropertySet v0、FDN/TN v0 和 FDN/TN 专属样本。
8. Envelope v0 现有证据不支持，除非出现新的旧档或反编译证据，不应实现。

未运行 ArcMap 10.1 或 QGIS，未做人工目视验收；上述所有样本对位只是辅助观察，
不能声称目标软件验证。
