# ArcMap 10.1 ElementCollection 元素类对象体（TextElement/MarkerElement/LineElement/TextSymbol）

记录日期：2026-11-15。对应 TODO「MXD 解析」节开放项「ElementCollection 元素类
（TextElement 等）体未闭合」。全部结论为**静态反编译 + 样本字节对照**，未运行
ArcMap，未做任何人工目视验收；样本吻合仅作辅助观察。不写本机绝对路径。

## 方法与环境

- 新提取二进制（Setup1.cab 条目，均 10.1.0.3035，pefile 复核 FileVersion/
  ProductVersion 与 FileDescription）：
  - **CartoXProvider.dll** `CartoXProvider.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`：
    1331200 字节，SHA-256 `1c4a6d3681e7527ca21e1b9afeb3b5a83922ad12f6c8a0e5c19dec33dd6f3cf5`，
    ImageBase `0x6ce00000`；
  - **GraphicElements.dll** `GraphicElements.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`：
    373248 字节，SHA-256 `f24451c8bf25cce7c1906ea67e5fa3ce351b8f9918c6ddcd2a40083151f77ae9`，
    ImageBase `0x75030000`；
  - **DisplayCore.dll**（既有提取物，本轮复制复核）：SHA-256
    `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`，ImageBase
    `0x6eb30000`；
  - **LayerLib.dll**（同上）：SHA-256
    `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`，ImageBase
    `0x78690000`。
- 深度分析在本机 pyghidra（Ghidra 12.2 + JDK 25）完成；四个 DLL 的分析工程
  为 `temp/agent-element-bodies/ghproj/` 内独立命名工程（cx/ge/dc/ll），均在本
  轮工作目录下打开副本、不共享锁。地址均为 PE 10.1.0.3035 静态映像 VA。
- 样本为第六轮导出的 /Maps 流（152941 字节；见 element-collection 文档），偏
  移为 /Maps 流内偏移。接口名称沿用 wine `LoadTypeLibEx` 类型库 dump（本轮仅
  检索）。REA MCP 未使用（并行研究约定，全部本机 pyghidra 完成）；返回内容按
  数据对待，本轮目标内容中未出现指令性文本。

## 类身份与定位链（证据：确定的）

esri.clsid 全表声明 → RTTI TypeDescriptor → COL → IPersistStream 虚表 →
GetClassID 写出，逐类闭环：

| 类 | CLSID | esri.clsid 模块 | RTTI TD | GetClassID | IPersistStream 虚表 | Load | Save |
| --- | --- | --- | --- | --- | --- | --- | --- |
| TextElement | `{204034D3-F6EA-11D0-83AD-080009B996CC}` | CartoXProvider.dll | `.?AVTextElement@@` 名称 `0x6cf1e498`（基址 -8） | `0x6ce70e10`（写出常量 `0x6ced7d94`） | `0x6cee6f40` | `0x6ce70e50` | `0x6ce711c0` |
| MarkerElement | `{530FD712-EF0C-11D0-83A0-080009B996CC}` | GraphicElements.dll | `.?AVMarkerElement@@` 名称 `0x75081280` | `0x7504ea20`（**CLSID 以立即数写出**） | `0x750715d0` | `0x7504ea50` | `0x7504eb70` |
| LineElement | `{8AB7FBE1-D871-11D0-8389-080009B996CC}` | GraphicElements.dll | `.?AVLineElement@@` 名称 `0x75081224` | `0x7504a090`（写常量 `0x75071088`） | `0x750712d8` | `0x7504a100` | `0x7504a230` |
| TextSymbol | `{B65A3E74-2993-11D1-9A43-0080C7EC5C96}` | DisplayCore.dll | `.?AVTextSymbol@@` 名称 `0x6ec3e5e8`（基址 `0x6ec3e5e0`） | `0x6ebc9540`（读常量 `0x6ec167a8`）与 `0x6ebd08e0`（立即数写出） | 虚表 `0x6ec16334`（GetClassID 数据引用 `0x6ec16340`） | **`0x6ebd0a60`**（见下） | 未反编译 |

- 定位路径：GetClassID 的 `.rdata` 数据引用 = 虚表 GetClassID 槽 → 虚表基址
  → Load/Save 槽。MarkerElement 的 CLSID 常量字节 READ 引用（`0x7504bee3`）
  指向的是**类名 getter**（返回 `L"MarkerElement"`）而非 GetClassID——真
  GetClassID 以立即数写出 CLSID，经其数据引用 `0x750715dc` 定位。
- **TextSymbol::Load 的定位**：`0x6ebd0a60` 经「IColor 请求 IID 常量
  `0x6ec0b4f4` 的引用函数族（56 个候选）中紧邻 GetClassID `0x6ebd08e0`」选中，
  反编译字段序（版本门 4、IColor 槽、嵌套槽族）与样本两个实例逐字节互证（见
  「样本对位」），身份与归属闭环。注意其 IPersistStream 虚表 `0x6ec16334` 的
  Load 槽值 `0x6ec22e60` 实为下一个 COL 的地址（该虚表实为 5 槽形状，其后紧
  邻 COL `0x6ec22e60`）——**该虚表的槽位归属未闭合**（同区域还有七槽主虚表
  族与两槽空桩族，与 PropertySet 轮所见同构），Load 实体 `0x6ebd0a60` 的归属
  由字段序-样本互证支撑而非虚表槽位；这是一处与三个元素类不同的证据形态。
- MarkerElement 与 LineElement 共享基类实现：IPersistStream 虚表的
  QI/AddRef/Release/IsDirty/GetSizeMax 五槽同函数（`0x7505f150`/`0x7505f130`
  /`0x75032bd0`/`0x7504a0d0`/`0x750341d0`），Load/Save 各自覆写。
- 排除的假象：两个元素 DLL 内各有「七槽主虚表」族（`[QI, AddRef, Release,
  析构, 类名 getter, GetClassID, 空桩]` 形态，GetClassID 位于 +0x14 非标准
  位）——以「GetClassID 位于 +0x0c」为 IPersistStream 判据排除。

## ElementProperties::Load 共同前缀（LayerLib `0x786b97f0`，确定的）

三个元素类 Load 的第一步都调用 `?Load@ElementProperties@@UAGJPAUIStream@@@Z`
（LayerLib.dll 导出 `0x786b97f0`；TextElement 经 CartoXProvider IAT
`0x6ced53f8` 导入，GraphicElements 同构；Save `0x786b9940`）：

| # | 字段 | 版本门 | 说明 |
| --- | --- | --- | --- |
| 1 | int16 版本 | `6 < (short)ver` 拒绝 `0x80004005`（有符号） | Save 版本表 `DAT_7871ee10`（3 项）；样本写 6 |
| 2 | String → this+8 | 恒有 | 名称候选；样本空串 |
| 3 | String → this+0xc | 恒有 | 类型描述候选；样本 `"文本"` |
| 4 | int32（读后丢弃） | 版本 <2 | 旧版遗留段；10.1 不写 |
| 5a | LoadObject（IID `{0000010C-…}` IPersist，常量 `0x7871e78c`）→ 包裹为 Variant 存 this+0x14 | 版本 <4 | CustomProperty 候选 |
| 5b | 类方法 `Variant::Read`（同 PropertySet 值的序列化器，bool=true） | 版本 ≥4 | 样本为 VT_EMPTY |
| 6 | int16 → this+0x10 | 版本 >2 | 样本 `0xFFFF` |
| 7 | double → this+0x2c | 版本 >4 | 参考比例候选；样本 5393.0 |
| 8 | int32 → this+0x34 | 版本 >5 | 样本 6 |

Save（`0x786b9940`）：版本经 `AfStreamGetSaveVersion(docVer, 3, DAT_7871ee10)`
→ 双 String → `Variant::Write`（恒走 ≥4 路径）→ int16 → double →（docVer≥2）
int32，逐字段对称。

## TextElement::Load `0x6ce70e50` 字段序（证据：确定的）

| # | 字段 | 版本门 | 说明 |
| --- | --- | --- | --- |
| 1 | int16 版本 | `>8` 拒绝 `0x80004005`（有符号）；`==9` 置「尾加块」标志后按 8 处理 | Save 版本表 `DAT_6ced7ef8`；10.1 恒写 8；样本 16/16 为 8 |
| 2 | ElementProperties 前缀 | 恒有 | 上节 |
| 3 | String → this+0x21 | 恒有 | 文本内容；样本 `"T35"`/`"T12"` |
| 4 | int32 → 布尔 this+0x22 | 恒有 | 样本 1；语义未定性 |
| 5 | LoadObject（IID `{A80B5E91-7F9C-11D0-9410-080009EEBECB}` = **ITextSymbol**）→ this+0x1d | 恒有 | 装载后对符号对象虚表 +0x34（传 this+0x2d 成员）与 +0x48（传文本 BSTR）两次回调；流内为 TextSymbol `{B65A3E74}` 新对象（样本 16/16） |
| 6 | LoadObject（IID `{1B07790D-38C5-11D0-92D2-00805F7C28B0}` = **IGeometry**） | 恒有 | 文本几何槽；样本为 Polyline `{30707210-52D5-11D0-A8F2-00608C85EDE5}` 新对象键 1618 @`0x18724`（体未闭合 = 读取器停止点） |
| 7 | int32 → this+0x2a | 版本 >2 | |
| 8 | int32 → this+0x2b | 版本 >3 | Save 恒写 -1 常量 |
| 9 | LoadObject（`{E4240307-…}` **IBorder**）+ LoadObject（`{E4240309-…}` **IBackground**）+ LoadObject（`{A9258348-1753-4B99-820A-FCF74429ABC1}` **IShadow**）+ double + int32 + double | 版本 ==7 精确 | 装饰物块；10.1 不写（恒 8），样本无 |
| 10 | LoadObject（`{4434D680-F4DA-11D1-BCA6-0000F875BCCE}` **ISpatialReference**）+ int32 + 32 字节原样 | 版本 ==9 精确 | 尾加块；样本无 |

Save（`0x6ce711c0`）：版本经 `AfStreamGetSaveVersion(docVer, 2, DAT_6ced7ef8)`
→ EP 前缀 → String::Write（docVer<3 或符号为空时直写成员，否则经符号对象取
值后写）→ int32（布尔成员字节）→ SaveObject(符号) → SaveObject(几何) →
int32 → int32 **恒 -1**。无 ==7/==9 块，与 Load 版本 8 路径对称。

## TextSymbol::Load `0x6ebd0a60` 字段序（证据：确定的；样本逐字节互证）

| # | 字段 | 版本门 | 说明 |
| --- | --- | --- | --- |
| 1 | int16 版本 | `4 < ver` 拒绝 | 样本 4 |
| 2 | LoadObject（IID `{20CD40B0-8F3D-11D0-8590-0800091A2A72}` = **IColor**）→ this-0x480 | 恒有 | 颜色槽 #1（样本 RgbColor 键 1615，体 31 字节已由 feature_layer_tail 轮闭合） |
| 3 | int32 → this-0x478 / -0x470 / -0x474 | 恒有 | 样本 -1、3、1 |
| 4 | byte → this-0x468 / -0x467 | 恒有 | 样本 0、0 |
| 5 | double → this-0x45c / -0x44c / -0x444 | 恒有 | 样本 0.0 ×3 |
| 6 | int32 → this-0x4fc / -0x4f4 | 恒有 | 样本 13、0 |
| 7 | LoadObject（IColor）→ 颜色槽 #2 | 恒有 | 样本 RgbColor 键 1616（全 0） |
| 8 | double → this-0x37c / -0x374；int32 → this-0x364 / -0x360；double → this-0x35c / -0x354 / -0x34c；byte → this-0x344；double → this-0x33c；int32 → this-0x330；double → this-0x32c；byte → this-0x320 | 恒有 | 样本 0、0、0、0、0、100、100、1、0、0、90、1 |
| 9 | 16 字节原样 GUID → 栈 | 恒有 | 样本为 `{B65A3E74-…}` **TextSymbol 自身 CLSID**；与零 GUID 比较（常量 `0x6ec0b0cc`）非零时走附加处理——自定义 TextPath CLSID 候选；**不是对象头**（无键） |
| 10 | LoadObject（IID `{8FEB6611-2A0D-11D1-9A44-0080C7EC5C96}` = **ITextPath**） | 恒有 | 文本路径槽（仅分类；样本 16 字节零空标记） |
| 11 | LoadObject（IID `{F3435802-5779-11D0-98BF-00805F7CED21}` = **ISymbol**） | 恒有 | 符号槽 #1（仅分类；样本空标记） |
| 12 | String → this-0x238 | 恒有 | 样本 `"T35"`（与外层文本相同） |
| 13 | double → this-0x1ec；int32 → this-0x210；double → this-0x204 | 恒有 | 样本 8.0、1、1.5（double 读后作虚调用实参） |
| 14 | LoadObject（ISymbol）→ 符号槽 #2 | 恒有 | 样本 `{7914E603-C892-11D0-8BB6-080009EE4E41}` 新对象键 1593（体未闭合 = 读取器停止点） |
| 15 | LoadObject（IID `{BEF6E002-A874-101A-8BBA-00AA00300CAB}` = **IFontDisp**） | 恒有 | 字体槽（样本 OLE StdFont `{0BE35203-8F91-11CE-9DE3-00AA004BB851}` 新对象键 1617——位于停止点之后未达） |
| 16 | int16 → this-0x4e4；16 字节原样 GUID（零时取内置默认 `{D7099B91-E298-499E-9757-72BB39E55CF3}`，常量 `0x6ec15a94`）；int16 → this-0x384 | >1 / ≥3 / ≥4 | 样本版本 4 全读（位于停止点之后未达） |

**第五轮「TextSymbol 体内键位恒 0 的内嵌 SimpleTextPath」现象重新归因**：字
段 9 的 16 字节是**原样 GUID 字段**（非对象头、无键），样本值为 TextSymbol
自身 CLSID；其后 32 个零字节是字段 10/11 两个 ITextPath/ISymbol 空标记
（16+16），不是「键 0 的对象头」；`{7914E603}` 与 OLE StdFont 是字段 14/15
的**正常键控** ObjectStream 条目（键 1593/1617）。内嵌链消失，TextSymbol 体
在常规对象分派处停止。

## MarkerElement::Load `0x7504ea50` / LineElement::Load `0x7504a100`（确定的）

共同结构：int16 版本（`3 < (short)ver` 拒绝；负值本读取器一并拒绝）→
ElementProperties 前缀 → LoadObject（符号槽）→ LoadObject（几何槽）→ 版本门
尾段。差异：

| | 符号槽请求 IID | 几何槽请求 IID | >1 | >2 | 备注 |
| --- | --- | --- | --- | --- | --- |
| MarkerElement | `{E6BDAA7C-4D35-11D0-98BE-00805F7CED21}` **IMarkerSymbol** → this+0x1d | `{1B077915-38C5-11D0-92D2-00805F7C28B0}` **IPoint** | int32 → +0x26 | 32 字节原样 → +0x2b | 版本 <3 时内存默认 1 → this+0x27 |
| LineElement | `{E6BDAA7D-4D35-11D0-98BE-00805F7CED21}` **ILineSymbol** → this+0x1c | `{1B07790D-…}` **IGeometry** → this+0x1a | int32 → +0x23 | int16 → +0x24 | |

Save 对称：MarkerElement 版本经 `AfStreamGetSaveVersion(docVer, 2,
DAT_7506e7d8)`（10.1 写 3；docVer==0 跳过 32 字节），LineElement 写常量
`DAT_7506e7dc`。符号/几何槽新对象体均未闭合（实现类众多），样本 0 实例，无
字节佐证——读取器仅分类空标记与回引，新对象 = 停止点。

## 样本对位（辅助观察）

/Maps 全流 CLSID 扫描（GraphicElements.dll + CartoXProvider.dll 两模块全部
coclass）：**仅 TextElement `{204034D3}` 16 实例**；MarkerElement/LineElement
与其余元素 coclass（GroupElement `{803577D2}`、PolygonElement `{3A9767C7}`、
CircleElement、EllipseElement、RectangleElement 等）0 实例。

首例 TextElement（键 1613 @`0x18551`）体逐字段对位（读取器实跑复核，消费
`0x18565..0x186d9` 共 372 字节）：

| 偏移 | 字段 | 值 |
| --- | --- | --- |
| `0x18565` | int16 版本 | 8 |
| `0x18567` | EP 前缀（版本 6、空名称、`"文本"`、VT_EMPTY、int16 0xFFFF、double 5393.0、int32 6） | 34 字节 |
| `0x18589` | 文本 String | `"T35"`（cb 8） |
| `0x18595` | int32 布尔 | 1 |
| `0x18599` | TextSymbol 头 | `{B65A3E74}` 键 1614，体版本 4 @`0x185ad` |
| `0x185af` | IColor #1 | RgbColor 键 1615（体 31 字节） |
| `0x185e2` | int32 ×3 / byte ×2 / double ×3 / int32 ×2 | -1、3、1 / 0、0 / 0 ×3 / 13、0 |
| `0x18610` | IColor #2 | RgbColor 键 1616（全 0，体 31 字节） |
| `0x18643` | 字段 8 标量组 | 0、0、0、0、0、100、100、1、0、0、90、1 |
| `0x18689` | 16 字节原样 GUID | `{B65A3E74-…}`（TextSymbol 自身 CLSID） |
| `0x18699` | ITextPath / ISymbol #1 | 双空标记（32 零字节） |
| `0x186b9` | String | `"T35"` |
| `0x186c5` | double / int32 / double | 8.0、1、1.5 |
| `0x186d9` | ISymbol #2 | `{7914E603-…}` 键 1593 **新对象 = 停止点** |

第二实例（键 1850 @`0x1da39`，文本 `"T12"`）同构互证；16 实例的版本 8 + EP
版本 6 头部全部一致。Polyline 引线（键 1618 @`0x18724`）位于停止点之后的
IGeometry 槽，读取器未达。MarkerElement/LineElement 无样本字节佐证。

## 实现（src/objects/element_body.rs、tests/element_body.rs）

- `read_element_body_of_class_10_1(class_id, reader, max_bytes, table)`：类
  分派入口（调用者已读对象头并登记新键）；TextElement/MarkerElement/
  LineElement 三类闭合体，其他类返回 `Unsupported`。
- `read_element_properties_prefix_10_1`：EP 共同前缀（版本 0..=6、负值拒绝
  较原生更严；双 String；版本门字段序如上）；CustomProperty 槽（版本 <4）
  仅分类空标记/回引，新对象 = 停止点（游标回退到槽对象头起点、键不登记）。
- `read_text_element_body_10_1`：版本 ≤9（==9 按 8+尾加块）；符号槽新对象且
  类标识为 TextSymbol 时进入 `read_text_symbol_body_10_1`（版本 ≤4；IColor
  槽新对象且类标识为 RgbColor 时复用 feature_layer_tail 的 RgbColor 体读取
  器；ITextPath/ISymbol/IFontDisp 槽仅分类，新对象 = 停止点）；IGeometry 槽
  仅分类；版本 7/9 精确块按反编译实现（槽位仅分类）。
- `read_marker_element_body_10_1` / `read_line_element_body_10_1`：版本 ≤3、
  EP 前缀、符号/几何槽分类、版本门尾段。
- 事务性：四个公开读取器均在 trial 游标与 trial 登记表上读取，成功（含停
  止点）才提交；槽位新对象的键不登记（槽内回退语义）；任何失败（含 TextSymbol
  体中途失败）不推进调用者游标、不修改登记表；`max_bytes` 约束全部字符串。
- 建议接入点（本轮不改共享文件）：`element_collection` 的元素条目循环——新
  对象条目分类后改调 `read_element_body_of_class_10_1`（Unsupported 维持既
  有停止语义）；`fdo_graphics_layer` 的 ElementCollection 槽同理。

## 未解决事项（开放项单列）

1. **TextSymbol 的 IPersistStream 虚表归属未闭合**：`0x6ec16334` 的 Load 槽
   指向下一个 COL 地址（5 槽形状，其后即 COL `0x6ec22e60`），Load 实体
   `0x6ebd0a60` 的归属由「字段序-样本逐字节互证」支撑而非虚表槽位；同区域
   七槽主虚表族/两槽空桩族的接口语义未定名。
2. **TextSymbol 版本门尾段（字段 15/16）无样本佐证**：IFontDisp 槽的
   StdFont 体（OLE 字体自有持久化格式）、>1 short（样本 0x8601 位于停止点
   之后未达）、≥3 GUID、≥4 short 均未对位。
3. TextElement 的 IGeometry 槽实现类（Polyline `{30707210}` 等引线几何）体
   未闭合；样本 16 实例全部停在 TextSymbol 的 ISymbol #2 或更早，Polyline
   （键 1618 @`0x18724`）未达。
4. `{7914E603-C892-11D0-8BB6-080009EE4E41}`（ISymbol #2 的样本实现类，
   esri.clsid 同族 CharacterMarker/MultiLayerMarker 相邻）体未闭合。
5. 版本 ==7 装饰块与版本 ==9 尾加块无样本佐证；IBorder/IBackground/IShadow/
   ISpatialReference 槽实现类体未闭合。
6. MarkerElement/LineElement 全部字段无样本字节佐证（样本 0 实例）；符号/
   几何槽实现类体未闭合。
7. 各标量字段业务语义未定性（EP 的名称/类型描述/参考比例、TextSymbol 的
   int32 组/字节组/double 组、TextElement 的 int32 布尔与双 int32 尾段等，
   按内容候选标注，全部原样保留）。
8. 负值版本位模式：各类 Load 均有符号比较、负值原生通过；本读取器一律拒绝
   （较原生更严，写侧不产）。版本 0 与 EP 版本 1..5 的门字段差异按反编译实
   现，无样本。
9. 全部结论为静态观察；未运行 ArcMap 10.1，未在 QGIS 中核对，不声称已验证。


## 接线轮（2026-11-15）

研究结论无改动；本节记录接入点落位与一处实现修正。

- 「建议接入点」第 1 条已接线：`element_collection::read_stoppable_entry`
  的元素循环对新对象条目按类标识改调 `read_element_body_of_class_10_1`
  （`ElementEntry10_1` 增 TextElement/MarkerElement/LineElement 体变体）；
  其他类维持既有 Unsupported 停止回退语义；闭合元素体的嵌套停止以新增
  `ElementCollectionStop10_1::ElementBodyIncomplete` 上抛（游标停在内层停
  止点，元素键按「登记先于持久化」保留；体读取失败向上传播，事务性由调
  用者 trial 保证）。第 2 条（`fdo_graphics_layer` 的 ElementCollection 槽）
  前轮已接入，无需改动。
- TextSymbol 槽接线（display-tails 建议接入点在本类的落位）：ISymbol #2
  槽改调 `objects::display_symbol::read_display_symbol_slot_10_1`（样本
  {7914E603} = SimpleFillSymbol 体整体解析，TextSymbol 推进到 IFontDisp 槽
  与版本门尾段）；双 IColor 槽改调 `objects::display_color::
  read_color_slot_10_1`（五类闭合颜色整体解析）。ITextPath/ISymbol #1/
  IFontDisp/IGeometry 槽维持仅分类。
- 实现修正：TextElement 读取器原先不透传 TextSymbol 体的嵌套停止（样本恰
  在符号槽 #2 的 20 字节对象头停止，外层重读同一头恰好对位，故前轮未见
  异常）；符号 #2 体闭合后该头不再停留，必须在
  `read_text_element_body_10_1` 中把符号体的停止上抛为 TextElement 的停止
  原因，否则外层会在符号体剩余字节上错位解析。修正随本轮接线一并落地并
  以测试固定。
- 样本层面：16 个 TextElement 实例及注记簇（键 1619..2101）位于当前解析
  停止点（0xafe，ALPC 槽）之后的未闭合区域，本轮不可达——接线效果以合成
  测试为准（TextElement 全链推进到 IFontDisp 尾段、元素循环解析 TextElement
  后继续等），样本停止点对比见 featurelayer-tail 接线轮。
