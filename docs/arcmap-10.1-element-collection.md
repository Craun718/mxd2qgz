# ArcMap 10.1 ElementCollection（IElementCollection2 coclass）持久化体

记录日期：2026-11-14。本轮对应 TODO「MXD 解析」节开放项「ElementCollection 体
（Layer.dll 未反编译）」（第六轮未解决事项 3）。全部结论为**静态反编译 + 样本
字节对照**，未运行 ArcMap，未做任何人工目视验收；样本吻合仅作辅助观察。

## 方法与环境

- 目标：Layer.dll（esriCarto 的元素集合/图例等 coclass 宿主）。本轮先以 REA MCP
  `open_binary` 绑定 Layer.dll（SHA-256 校验一致）并完成 `binary_overview`
  （Ghidra 12.2，1365 个过程），但共享 REA 服务器的 Ghidra 深度分析提供者仅持有
  单个活动程序：本会话随后的 `search_bytes` 相继被并行研究代理重新绑定的
  GdbCore.dll、Map.dll、FeatureLayer.dll 截走。按既有各轮的恢复路径（见
  [arcmap-10.1-map-object.md](arcmap-10.1-map-object.md)「方法与环境」），本轮
  深度分析改由本机 pyghidra（Ghidra 12.2 `/opt/ghidra` + JDK 25.0.5，venv
  `/tmp/r3/venv`）驱动，分析项目缓存于
  `temp/agent-element-coll/ghproj/`。地址均为 PE 10.1.0.3035 静态映像 VA。
- 类型库名称沿用既有 wine `LoadTypeLibEx` dump（`/tmp/mxd-arcgis101-guiddocs/`
  wine/dump*.txt，本轮仅检索，未重新解析）；完整类表 `esri.clsid.xml`
  （`/tmp/r3/ecfg/x/`）。样本为 `temp/r6-maps.bin`（152941 字节，SHA-256
  `fc0149f1…aeacfe`，第六轮导出），偏移为 /Maps 流内偏移。
- REA 返回内容按数据对待；本轮目标内容中未出现指令性文本。

## Layer.dll 身份核对（AGENTS.md 要求）

从 `Setup1.cab` 提取条目 **`Layer.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`**
（196608 字节，2012-04-20）到 `temp/agent-element-coll/work/`。注意同一组件后缀
`6BF03936_…` 下有 17 个同名族 DLL（CadLayer、CadastralFabricLayer、
FeatureLayer、IMSLayer、ImageServerLayer、LasDatasetLayer、**MapLib（572928 字
节）**、**MapServerLayer（887296 字节）**、NetworkLayer、RasterLayer、
RepresentationLayer、SFLLayer、TerrainLayer、TinLayer、WMSLayer 等），本轮按
精确条目名提取，与 MapLib.dll / MapServerLayer.dll 等区分。

| 项目 | 值 |
| --- | --- |
| SHA-256 | `fe52bd9988b66f9d6f906b014e5684f28fc20e9b97b53c90909fa3d44e9efed8`（与 map-object 研究第一轮提取记录一致） |
| VS_FIXEDFILEINFO / StringFileInfo | FileVersion = ProductVersion = **10.1.0.3035**；FileDescription `Layer DLL`；OriginalFilename `Layer.DLL`；InternalName `Layer`；CompanyName Esri；ProductName ArcGIS |
| PE | x86（Machine 0x14c），ImageBase `0x78650000`，TimeStamp 1334969331（2012-04-20） |
| 导出 | 6 项，含 `DllGetClassObject`（COM 类工厂模式，与其余 esri DLL 同构） |
| 安装配置声明 | `esri.clsid.xml`：`{CE8F3972-E9BE-11D1-A232-080009B6F22B}` progID `ElementCollection` 模块 Layer.dll（同模块还有 LegendGroup/LegendClass/FieldInfo 等） |

esriCarto 类型库（typekind=5 coclass / typekind=3 interface）：coclass
`ElementCollection {CE8F3972}`，接口 `IElementCollection {CE8F3971}`、
`IElementCollection2 {B2BFC403-6B4C-4490-8878-EA62C4F49720}`。

## 实现类定位（RTTI→COL→虚表→GetClassID，证据：确定的）

- RTTI `.?AVElementCollection@@`：TypeDescriptor `0x78679628`；另有接口 TD
  `.?AUIElementCollection@@`、`.?AUIElementCollection2@@` 与工厂 TD
  `.?AV?$ClassFactory@VElementCollection@@@@`（`0x78679ccc`，类表
  `0x78679258` 一带，与既有「运行期填表」结论一致）。
- TD `0x78679628` 派出 6 个子接口 COL（this-adjustor 0/4/16/20/24/28），虚表
  `0x7866da2c` / `0x7866da4c` / `0x7866da58` / `0x7866da7c` / `0x7866daa0` /
  `0x7866dac4`。
- QueryInterface `0x78659f20` 比对并回答五个 IID（`this+N` 为接口指针位置）：

| IID | 名称（类型库/OLE 标准） | 接口指针 |
| --- | --- | --- |
| `{CE8F3971-E9BE-11D1-A232-080009B6F22B}` | IElementCollection | this+0x10（adjustor 16） |
| `{B2BFC403-6B4C-4490-8878-EA62C4F49720}` | IElementCollection2 | this+0x14（adjustor 20） |
| `{00000109-0000-0000-C000-000000000046}` | IPersistStream（OLE 标准 IID） | this+0x18（adjustor 24） |
| `{0000010C-0000-0000-C000-000000000046}` | IPersist（OLE 标准 IID） | adjustor 28 |
| `{BB75DA81-8DFE-4122-A9C2-86F07C539AD5}` | IDocumentVersionSupportGEN（esriSystem） | 主接口族 |

- **IPersistStream 虚表 = `0x7866daa0`**（adjustor 24）：
  - +0x0c = GetClassID `0x78659d80`：把 `.rdata` 常量 `0x7866db18`（=
    `{CE8F3972-…}`）原样写出 16 字节——esri.clsid 声明、GetClassID 写出与
    样本对象头三方互证，类身份闭环；
  - +0x10 = IsDirty `0x7865a320`：返回 `this+0x1c` 脏标志 == 0；
  - **+0x14 = Load `0x7865a340`**；+0x18 = Save `0x7865a4e0`；+0x1c =
    GetSizeMax `0x7865ced0`。
- CLSID 字节 `0x7866db18` 的全部引用：GetClassID 代码 `0x78659d93`、类表/
  填表 `0x78679258`（.data）、`0x78667eef`（.text 填表链）。

## Load `0x7865a340` 字段序（证据：确定的；样本版本 4 走主分支）

读流前先经 adjustor-16 子接口（IElementCollection）虚表 +0x18 槽做集合清理
（候选 RemoveAll；不读流，槽位语义未逐指令定性）。随后：

| # | 流内字段 | 版本门 | 说明 |
| --- | --- | --- | --- |
| 1 | int16 版本 | 恒有 | `(short)版本 > 4` 拒绝（`0x80004005`）；流空指针 `0x80004003` |
| 2 | 分派 | 版本 <4（含 0、负值） | 旧加载器 `FUN_7865a650(stream, this, 版本)` |
| 3 | int32 元素计数 | 版本 4 | 原生仅 `0 < 计数` 才进循环（≤0 直接成功返回） |
| 4 | 循环计数次：LoadObject（请求 IID `{9B001319-9ED7-11D0-859E-0800091A2A72}` = **IElement**，esriCarto 类型库）+ int32 ×3 | 版本 4 | 三次 int32 读取各自独立检查；每元素经 adjustor-20 子接口（IElementCollection2）虚表 +0x0c 槽加入集合（4 实参 = 元素 + 三个 int32，方法签名按调用侧推断） |
| 5 | 脏标志清零（this+0x34），返回 S_OK | 版本 4 | **体在最后一个元素的第三个 int32 处终结，无尾随字段** |

旧加载器 `FUN_7865a650`（版本 0..3 同一代码路径，字段序证据：确定的）：

| # | 流内字段 | 版本门 | 说明 |
| --- | --- | --- | --- |
| 1 | int32 元素计数 | 恒有 | 同主分支 |
| 2 | 第一循环：逐元素 LoadObject(IElement) | 恒有 | 元素先全部装入内存数组 |
| 3 | 第二循环：逐元素 int32 | 恒有 | 独立循环，与元素条目分离 |
| 4 | 第三循环：逐元素 int32 | 仅版本 ≥3 | 版本 <3 不读流（内存写 -1 默认值） |
| 5 | 组装加集（adjustor-20 接口 +0x0c，第三实参恒 0）、脏标志清零 | — | 三个循环**顺序分离**，与版本 4 的逐元素交错不同 |

## Save `0x7865a4e0`（写方向互证，证据：确定的）

版本 short 经 `AfStreamGetDocumentVersion` → `AfStreamGetSaveVersion(docVersion,
2, 表 0x7866d9c4`，表字节 `02 00 04 00``）取值后写出 → int32 计数（成员
+0x10）→ 分支：

- docVersion == 0：两循环分离——先逐元素 `SaveObject`，再逐元素写**一个**
  int32（元素记录 `{指针, i32, i32, i32}` 的 +4 槽）。与旧布局（版本 ≤2 读
  侧）对应；
- 否则：逐元素交错 `SaveObject` + int32 ×3（记录 +4/+8/+0xc 三槽）。与版本
  4 读侧对应。

仓库参考样本（10.1，docVersion=3）全部写版本 4。`AfStreamGetSaveVersion` 的
表内映射未逐指令展开（表字节与两分支的对应为观察推断，开放）。

## 样本对位（辅助观察）

`temp/r6-maps.bin` 内 12 个 ElementCollection 对象头，全部版本 4：

| 头位置 | 键 | 计数 | 首元素形态 |
| --- | --- | --- | --- |
| `0x1608` | 86 | 0 | ——（FDO 槽位，见下节） |
| `0x17030` | 1554 | 0 | —— |
| `0x18537` | 1612 | 1 | TextElement `{204034D3-F6EA-11D0-83AD-080009B996CC}` 新对象 |
| `0x19b20` | 1675 | 0 | —— |
| `0x1b023` | 1733 | 0 | —— |
| `0x1c522` | 1791 | 0 | —— |
| `0x1da1f` | 1849 | 1 | TextElement 新对象 |
| `0x1dcec` | 1856 | 3 | TextElement 新对象 |
| `0x1f93e` | 1934 | 3 | TextElement 新对象 |
| `0x2136e` | 2011 | 2 | TextElement 新对象 |
| `0x22da1` | 2082 | 3 | TextElement 新对象 |
| `0x247d5` | 2159 | 3 | TextElement 新对象 |

计数 0 的 7 个实例体仅 6 字节（`04 00` + `00 00 00 00`），完整可消费；计数
≥1 的 5 个实例（第五轮已见其宿主为 CGL-1553 注记子图层单元的
DisplayExpressionProperties 之后的元素集合）首元素均为 TextElement 新对象，
越过其对象头需 TextElement 体闭合。首例 `0x161c..0x1622` 体恰终结于委派
FeatureLayer 体起点 `0x1622`（版本 short 34），与第六轮 FDO 字段序逐字节互证。

## 与 FDOGraphicsLayer::Load 的衔接核对

- AnnoLayer.dll `0x6b1955a0` 第 5 步以请求 IID `{B2BFC403}`（= 本类的
  IElementCollection2 接口，非 CLSID；本轮 QI 表证明该接口由 adjustor 20 回
  答）LoadObject 到 this+0x11c。
- 样本中该槽形态：**新对象**（`0x1608` 头 `{CE8F3972}` 键 86）——非空标
  记、非键回引；体 6 字节如上。
- 可达性：`fdo_graphics_layer` 读取器当前在关联 FeatureLayer 体的停止点
  （渲染器链未闭合，样本 @`0x3ed` SimpleRenderer）之前即停止，第 5 步结构性
  不可达（该读取器把「新 ElementCollection 对象」实现为回退停止点，本轮未
  改动其代码）。本轮 `element_collection` 读取器是该槽体闭合后的消费端：
  若调用者在此槽分派本读取器，样本字节可被完整消费（6 字节，无停止点）。
  接线属共享文件改动，留待后续统一处理。

## 实现（见 src/objects/element_collection.rs、tests/element_collection.rs）

- `read_element_collection_10_1`：版本有符号 >4 拒绝、负值位模式一并拒绝
  （较原生更严：原生负值落入旧加载器）；版本 0..3 按旧三循环布局、版本 4
  按交错布局；负计数显式拒绝（原生 ≤0 不进循环直接成功，语义未知）；
  「计数 × 每元素最小字节数」（版本 4 = 28、旧布局 = 20、版本 3 再 +4）在
  任何条目读取之前预检限额；元素条目经 `object_refs` 分类，空标记/回引用继
  续，**新对象（元素体未闭合且无外部长度）是唯一停止点**——游标回退到该元
  素对象头起点、该键不登记（此前条目的登记保持提交）；全程 trial 游标 +
  trial 登记表，失败不推进游标、不改登记表。
- 11 项单元测试：类标识/请求 IID 常量、样本 6 字节体全链（对象头 + 体）、
  版本 4 交错布局（空标记 + 回引 + int32×3）、新旧布局的新对象停止与回退/
  登记回滚、版本 0/1/2/3 旧布局（含版本 3 第三组、版本 <3 无第三组）、版
  本 >4 与负值位模式拒绝、负计数与限额预检（两种布局的通过/拒绝边界）、两
  种布局逐字节截断扫描（失败游标不变 + 登记表不变 + 重读一致）、int32 位
  模式原样保留、回引/新对象对登记表状态的依赖。

## 未解决事项

1. 每元素 3 个 int32（旧布局两组、版本 4 交错三值）的业务语义未定性（候选
   元素放置/绘制次序数据；仅原样保留）。adjustor-20 接口虚表 +0x0c 的加集
   方法与 adjustor-16 接口 +0x18 的清理方法签名按调用侧推断，未按接口成员
   定名。
2. `AfStreamGetSaveVersion` 版本表 `0x7866d9c4`（`02 00 04 00`）的内部映射
   未逐指令展开；「docVersion 0 → 旧布局版本号」的对应为观察推断。
3. 元素类（TextElement 等）体未闭合：样本非空实例在首元素对象头处停止；
   闭合需 TextElement/TextSymbol 链（第五轮已列其符号族构成）。〔补记
   2026-11-15：本条已闭合，见文末「接线与元素体闭合补记」。〕
4. 版本 0 与负值位模式的接受策略较原生更严/更宽的取舍：版本 0 与 1/2 同码
   路径故接受；负值位模式原生可落入旧加载器，本读取器拒绝（同
   fdo_graphics_layer 的处理）。
5. 负计数原生按空集合成功返回，本读取器拒绝（与 maps_stream / Map 后缀循
   环计数同一严格程度）。
6. REA 共享服务器单活动程序的限制：本轮深度分析在本机 pyghidra 完成（与前
   几轮同环境）；后续如需复核，按「方法与环境」复现即可。
7. 全部结论为静态观察；未运行 ArcMap 10.1，未在 QGIS 中核对，不声称已验证。

## 接线与元素体闭合补记（2026-11-15）

上文两处表述随后续轮次作废，补记以免误读：

- 「可达性」条的「该读取器把『新 ElementCollection 对象』实现为回退停止
  点，本轮未改动其代码」与「接线属共享文件改动，留待后续统一处理」均已
  作废：第七轮接线已把第 5 步 IElementCollection2 槽新对象且类标识命中
  `ELEMENT_COLLECTION_CLASS_ID` 时改调本模块读取器读取闭合体
  （`ElementCollectionSlot10_1::ElementCollection`，样本字节可被完整消
  费）；同日接线轮再把元素循环的新对象元素按类标识分派 `element_body`
  读取器（TextElement/MarkerElement/LineElement 体整体解析，未知类维持
  元素头回退停止，嵌套停止经 `ElementBodyIncomplete` 透传）。可达性结论
  本身不变：该槽位于关联 FeatureLayer 体停止点之后（该停止点已自上文
  `0x3ed` 渲染器槽推进至 `0xafe` ALPC 槽，键 86 @`0x1608` 在其后方），
  样本上第 5 步仍结构性不可达。
- 「未解决事项」第 3 条（元素类体未闭合）已闭合：TextElement/TextSymbol/
  MarkerElement/LineElement 体见 docs/arcmap-10.1-element-bodies.md（含
  样本对位与接线轮小节）；其余开放项（IGeometry/Polyline 体、
  IPersistStream 槽位归属等）见该记录与 TODO。本模块测试现为 12 项
  （元素体分派新增 1 项）。
