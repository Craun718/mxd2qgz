# ArcMap 10.1 Legend/EditTemplate 尾段链：LegendClassFormat、HorizontalLegendItem、EditTemplate

记录日期：2026-11-15。对应 TODO「MXD 解析」节开放项中的「ILegendItem/
ILegendClassFormat coclass、…、EditTemplate 等约 25 类尾段链未闭合」的三
类（LegendClassFormat、HorizontalLegendItem、EditTemplate）。全部为静态反
编译观察，未运行 ArcMap，未做人工目视验收。实现见
`src/layers/legend_format.rs`、`src/layers/legend_item.rs`、
`src/layers/edit_template.rs`（模块文档含同源字段表）。

## 样本边界（先声明）

- 仓库参考样本 `samples/测试底图.mxd` 的 /Maps 流中：LegendClassFormat 与
  HorizontalLegendItem 的 CLSID **0 命中**（样本 LegendClass 的
  ILegendClassFormat 槽为空标记、Legend 条目循环计数为 0，第六/七轮已
  闭合该行为）；两类全部布局结论**无样本字节佐证**，仅有反编译一手证据。
- EditTemplate 的 CLSID 在 /Maps 流 **10 命中**（新对象头，键 76、146、
  1605、1726、1784、1842、1925、2003、2075、2151），体逐字段对位吻合
  （见「样本对位」）。其宿主槽位（FeatureLayer 后继字段链）不在本轮范围。
- 本轮提取的 Renderers.dll 未在样本流中出现，身份核对见「方法与文件身份」。

## 方法与文件身份

- 本地 pyghidra（Ghidra + JDK 25，解释器与工具链同前轮；定位脚本
  RTTI 类型描述符 → CLSID 字节 → xrefs → GetClassID → 虚表 → Load/Save
  槽，产物在 `temp/agent-legend-tails/`：`*_locate_report.json`、
  反编译 `.c` 片段）。
- 本轮从安装介质 `Setup1.cab` 提取 **Renderers.dll**（条目
  `Renderers.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`，257024 字节，
  SHA-256 `399c80c99a6c61de120c21b518d9b29069f2c78b20a705c07178aa3e2b656274`）；
  pefile 复核 PE32 x86（Machine 0x14c）、ImageBase `0x7cac0000`、
  VS_FIXEDFILEINFO/StringFileInfo `10.1.0.3035`（InternalName `Renderers`、
  OriginalFilename `Renderers.dll`、CompanyName Esri、ProductName ArcGIS、
  OLESelfRegister）。方法同 `docs/arcmap-10.1-native-baseline.md`。
- Layer.dll（LegendClassFormat）、FeatureLayer.dll（EditTemplate）、
  LayerLib.dll（BaseLegendItem）、AfCore.dll（Variant::Read）复用既有基线
  副本，SHA-256 与前轮记录一致，地址均为 PE 静态映像 VA。
- 类名/接口名：`esri.clsid.ecfg` 全表（EditTemplate/LegendClassFormat/
  HorizontalLegendItem 三条目所在模块分别为 FeatureLayer.dll/Layer.dll/
  Renderers.dll，与本轮二进制互证）；接口 GUID 经 10.1 类型库 wine
  `LoadTypeLibEx` dump（esriCarto/esriDisplay/esriSystem）核对。

## 定位链速记（证据：确定的）

MSVC 类对这些 coclass 的持久化入口不在 QI 首表，定位时须区分
「identity 虚表」与「IPersistStream 虚表」：

- **LegendClassFormat**（Layer.dll）：标准布局。GetClassID `0x78664070`
  写出 `{7A3F91E6-B9E3-11D1-8756-0000F8751720}`（数据地址 `0x7866f210`）；
  IPersistStream 虚表 `0x7866f8ac`（COL `0x78673440`，TD `0x7867a180` 名
  `.?AVLegendClassFormat@@`）：+0x0c=GetClassID、+0x10=IsDirty、
  +0x14=Load `0x78664b20`、+0x18=Save `0x78664c40`、+0x1c=GetSizeMax。
- **HorizontalLegendItem**（Renderers.dll）：identity 虚表 `0x7caeaf18`
  （COL `0x7caee7e4`）槽序为 QI/AddRef/Release/GetRuntimeClassName
  （`0x7cad4c20` 返回 `L"HorizontalLegendItem"`）/析构/GetClassID
  （`0x7cad4c30` 写出 `{A9401A47-4649-11D1-880B-080009EC732A}`，数据地址
  `0x7caeb274`）/空实现——GetClassID 在 **+0x14 而非 +0x0c**。真正的
  IPersistStream 虚表 `0x7caeb244`（COL `0x7caee71c`，TD 同上）由构造
  `0x7cad4c70` 经静态配置结构 `DAT_7caec4d4` 动态安装于对象 this+0x174
  （QI `0x7cad4f10` 的 `{00000109}` 分支按配置偏移计算该接口指针）：
  +0x14=Load thunk `0x7cad5d30`、+0x18=Save thunk `0x7cad5d70`。Load 本体
  `0x7cad50e0` 尾调 **LayerLib.dll 导出** `?Load@BaseLegendItem@@UAGJPAUIStream@@@Z`
  （IAT `0x7cae9398`，LayerLib `0x786a2b00`）；Save 对称尾调
  `?Save@BaseLegendItem@@UAGJPAUIStream@@H@Z`（IAT `0x7cae9394`，
  LayerLib `0x786a2df0`）。
- **EditTemplate**（FeatureLayer.dll，实现类 `CEditTemplate`，RTTI TD
  `0x6f7d24b8` 名 `.?AVCEditTemplate@@`；`0x6f6cc050` 返回
  `L"EditTemplate"`）：identity 虚表 `0x6f7ab848`（COL
  `0x6f7b5d1c`）同 HorizontalLegendItem 形制，GetClassID `0x6f6cc060`
  写出 `{377035B9-34C7-4A40-866D-E596519A1B06}`（数据地址 `0x6f7ab918`）。
  QI `0x6f6cc350` 把 IPersistStream `{00000109}` 映射到 this+0x14；构造
  `0x6f6cc0a0` 在该偏移安装 IPersistStream 虚表 `0x6f7ab8c0`（COL
  `0x6f7b5ce0`）：+0x0c=GetClassID `0x6f6ce1a0`（同 CLSID，双方互证）、
  +0x14=Load `0x6f6ce1e0`、+0x18=Save `0x6f6ce3c0`。

## 一、LegendClassFormat（Layer.dll，体前缀闭合）

字段序（Load `0x78664b20` / Save `0x78664c40`，逐字段对称）：

| # | 字段 | 门/备注 | 载荷 |
| --- | --- | --- | --- |
| 1 | int16 版本 | 有符号 `>2` 拒绝 `0x80004005`；负值位模式原生通过（实现一并拒绝，较原生更严） | Save 恒写静态常量 2（`DAT_7866d968`=`02 00`） |
| 2 | LoadObject → obj+0x08 | 请求 IID `{A80B5E91-7F9C-11D0-9410-080009EEBECB}` = ITextSymbol（esriDisplay） | 标签符号候选 |
| 3 | LoadObject → obj+0x0c | 同上 IID | 描述符号候选 |
| 4 | LoadObject → obj+0x10 | `{BEB4EDB7-E3B4-11D2-B868-00600802E603}` = ILinePatch（esriCarto） | |
| 5 | LoadObject → obj+0x14 | `{BEB4EDB6-E3B4-11D2-B868-00600802E603}` = IAreaPatch | |
| 6 | double ×2 → obj+0x1c/0x24 | **版本 >1**；Save 恒写（无版本门），10.1 一致（Save 恒写版本 2）；语义未定，原样保留 | |

读取器：四个槽空标记/回引消费后继续，新对象（ITextSymbol/ILinePatch/
IAreaPatch 实现类体未闭合）为停止点（游标回退槽位起点，其后不可达）。

## 二、HorizontalLegendItem（Renderers.dll + LayerLib.dll BaseLegendItem，体前缀闭合）

流序 = Renderers 包装（Load thunk `0x7cad5d30` → `0x7cad50e0`）+ LayerLib
BaseLegendItem::Load `0x786a2b00`：

| # | 字段 | 门/备注 | 载荷 |
| --- | --- | --- | --- |
| 1 | int16 条目版本 | 有符号 `>1` 拒绝 `0x80004005`；负值原生通过（实现一并拒绝） | Save 恒写 1（`DAT_7cae9760`） |
| 2 | int32 → 对象 −8 | 写侧从内存回写；语义未定，原样保留 | |
| 3 | int16 基类版本 | **无拒绝门**（分支用；负值原生落入 <2 默认分支；实现仅接受 0..=3，>3 无写侧来源） | Save：`AfStreamGetSaveVersion(docVer, 6, DAT_7871ee18)`，版本表 `[1,1,1,1,1,3]` → 10.1 恒写 3 |
| 4 | LoadObject → −0x130 | `{34C20002-4D3C-11D0-92D8-00805F7C28B0}` = ILayer（esriCarto）；条目关联图层，真实流预期回引/空标记 | |
| 5 | int16 ×4 → −0x12c/−0x12a/−0x128/−0x126 | 原样保留 | |
| 6 | LoadObject → −0x124 | `{A80B5E91-…}` = ITextSymbol；标签符号候选 | |
| 7 | int32 → −0x120 | 原样保留 | |
| 8 | int16 → −0x11c | 原样保留 | |
| 9 | LoadObject → −0x118 | ITextSymbol；第二标签符号候选 | |
| 10 | LoadObject → −0x114 | `{52AA5BC2-CB21-11D0-87F9-080009EC732A}` = ILegendClassFormat；**新对象且类标识为 LegendClassFormat 时由第一类读取器整体消费**（本轮闭合；其余类停止） | |
| 11 | int16 ×2 → −0x110/−0x10e | 原样保留 | |
| 12 | v2 <2：**不读流**，内存写默认（0、0、" ("、")"、0xFFFF）；v2 ≥2：int16 → −0xce、int16 → −0xcc、String → −0xc8、String → −0xc4、int16 → −0xc0 | 默认值提示名称前后缀候选 | Save 侧「保存版本 >1」写出同组 |
| 13 | v2 >2：LoadObject → −0xbc | `{7E4F470D-8E54-11D2-AAD8-000000000000}` = INumberFormat（esriSystem） | |
| 14 | （this−0x10c 8 字节内存清零） | 无流内字节 | |

Save（LayerLib `0x786a2df0`）逐段对称：版本 short → SaveObject(−0x130) →
short ×4 → SaveObject(−0x124) → int32 → short → SaveObject(−0x118) →
SaveObject(−0x114) → short ×2 → [保存版本 >1] short/short/String/String/
short → [保存版本 >2] SaveObject(−0xbc)。

注：Load 汇编中对象槽 2..5 的 LoadObject 输出参数经共享辅助函数
`FUN_786ab450` 依次取 this−0x124/−0x118/−0x114/−0xbc，与 Save 侧字段序
互证一致。

## 三、EditTemplate（FeatureLayer.dll CEditTemplate，体完全闭合）

字段序（Load `0x6f6ce1e0` / Save `0x6f6ce3c0`，对称）：

| # | 字段 | 门/备注 | 载荷 |
| --- | --- | --- | --- |
| 1 | int16 版本 | 汇编 `CMP word,2; JLE` 有符号（>2 拒绝 `0x80004005`；负值原生通过，实现一并拒绝） | Save：`AfStreamGetSaveVersion(docVer, 5, DAT_6f77c02c)`，版本表 `[0xFFFF,0xFFFF,0xFFFF,0xFFFF,2]`：docVer 83..92 时 **Save 直接返回不写任何字节**；10.0/10.1 恒写 2 |
| 2 | String ×2 → obj+0x08/0x0c | AfCore String::Read 布局；模板名称候选 / 第二字符串候选（样本：名称「JZD1」等，第二串恒空） | |
| 3 | 16 字节 GUID → obj+0x18 | 原样保留；关联要素类/表标识候选（同一数据源多模板样本值相同） | |
| 4 | int32 计数 + 循环 [String 键][Variant::Read 值] | 原生负数/零计数跳过循环（实现拒绝负数，较原生更严）；键=字段名、值=模板默认值（样本 12 项） | 循环内的 Variant 见下节 |
| 5 | 版本 ≥2：String → obj+0x10 | 第三字符串候选（样本「点」）；Save 侧在迭代器耗尽后**恒写**（无版本门），与读侧 ≥2 门在 10.1 一致（Save 恒写版本 2） | |

## 四、AfCore `Variant::Read`（`0x68a7c6a0`，thiscall (this, stream, bool)）

EditTemplate 第 4 步循环使用的独立序列化器（与 `variant_stream` 的
VariantStream 全局 Read 不同族；前轮文档仅记录其存在，本轮按 EditTemplate
调用点 bool=true 闭合分支）。读取流程：

1. `IStream::Read` 2 字节 VARTYPE；bool=false 时另有 `Seek` 分支，
   EditTemplate 恒传 true（读取器不实现 false 路径）；
2. 按 VARTYPE（0x2000 = VT_ARRAY 标志）：

| VT（去标志后） | 载荷 | 证据 |
| --- | --- | --- |
| 0 EMPTY / 1 NULL | 无 | switch case 0/1 |
| 2 I2 / 11 BOOL | 2 字节 | case 2/0xb |
| 3 I4 / 10 ERROR | 4 字节 | case 3/10 |
| 4 R4 | 4 字节 | case 4 |
| 5 R8 / 7 DATE | 8 字节 | case 5/7 |
| 6 CY | 8 字节 | case 6（本族序列化器支持，与 VariantStream 不同） |
| 8 BSTR（非数组） | AfCore `String::Read` 布局（u32 字节长度含终止零 + 载荷），**不是** `ReadString` 的 `n*2+2` | case 8 `String::Read` |
| 16 I1 / 17 UI1 | 1 字节 | case 0x10/0x11 |
| 18 UI2 / 19 UI4 | 2 / 4 字节 | case 0x12/0x13 |
| 9 DISPATCH / 13 UNKNOWN | 全局 `LoadObject`（读 16 字节类 GUID、零 GUID 语义——非对象流 20 字节对象头；COM 上下文未闭合） | case 9/0xd，`DAT_68c6bc2c` = IID_IUnknown |
| 其他 | 返回 `0x80004005` | default |

3. 带 0x2000 标志的固定宽度类型：追加 int32 计数（负计数使
   `SafeArrayCreate` 失败 → `0x80004005`）+ count × 元素字节数的**单次
   整块原始读取**；带标志的 BSTR：追加 int32 计数 + 每元素 `ReadString`
   （`n` 为码元数、`n==0` 无载荷、否则 `n*2+2` 含终止零——与非数组 BSTR
   布局不同）；EMPTY/NULL 带标志时原生改道 BSTR 数组路径（语义未定，
   实现拒绝）。

实现（`edit_template::read_variant_class_value_10_1`）只覆盖 bool=true
路径；DISPATCH/UNKNOWN 与 EMPTY/NULL|ARRAY 显式拒绝（`Unsupported`）；
未知 VARTYPE 报 `InvalidData`（对应原生 0x80004005）。

## 样本对位（辅助观察）

/Maps 流键 1605 实例（`0x17f62` 对象头）体逐字段吻合：版本 2、
String「JZD1」、空 String、GUID `{2A8B3331-5238-4025-972E-452A69535B06}`、
计数 12 的 [String 键（JZDH/YSDM/ZDZHDM/JBLX/OBJECTID/ZZBZ/YZBZ/BSM/
XZBZ/JZDLX/BDCDYBSM/Ring）][Variant 值：BSTR 空 ×8、I4 0、R8 0.0 ×3]、
版本 ≥2 String「点」，体恰好终结于其后 SimpleMarkerSymbol 新对象头
（`0x150` 处 20 字节对象头）。其余 9 实例同构（键 76/146 的名称与 GUID
不同）。测试 `tests/legend_tails.rs::sample_edit_template_body_matches_field_order`
以合成等价字节固定该对位。**对位为辅助观察，不替代目标软件运行验证**；
宿主槽位（实例前的 `02 00 01 00 00 00` 尾段）未定名。

## 接入点建议（本轮未接线，共享文件未动）

- `feature_layer_tail`（LegendClass::Load 第 ⑤ 步 ILegendClassFormat 槽，
  样本空标记处）：新对象且类标识为 LegendClassFormat 时接
  `read_legend_class_format_body_10_1`。
- `map_surround`（Legend::Load 第 8 步 ILegendItem 条目循环）与未来
  LegendItem 族读取器：LegendItemBody 停止点按 CLSID 分派
  HorizontalLegendItem（`read_horizontal_legend_item_body_10_1`）；该体
  第 10 步 ILegendClassFormat 槽同样接第一类读取器。
- FeatureLayer 后继字段链的 EditTemplate 宿主槽（样本对位存在，槽位归属
  未闭合）：待后继字段链闭合后接 `read_edit_template_body_10_1`。

## 实现与测试（自身契约，不代表格式已验证）

- `src/layers/legend_format.rs` / `src/layers/legend_item.rs` /
  `src/layers/edit_template.rs`：显式 10.1 严格读取器，trial 游标 + 暂存
  登记表（成功含停止点才提交；错误不推进调用者游标、不改登记表）；
  未知字节原样保留，截断/非法长度/版本超限明确报错；不接分派链。
- `tests/legend_tails.rs` 25 项：版本分支（含负值位模式拒绝）、逐字节
  截断扫描（游标事务性与重读一致）、停止点回退与登记表事务、资源限额、
  Variant 全 VARTYPE 分支（含数组、对象分支拒绝）、样本键 1605 对位。
- `tests/robustness.rs` 集中清单 28 → **31** 个入口（新增三类各一）。
- `cargo fmt --check` 与 `cargo test` 全过；未运行目标软件，人工目视
  验收未做。

## 开放项

1. LegendClassFormat/HorizontalLegendItem（含 BaseLegendItem 体）无样本
   字节佐证；真实流中 ILayer 槽的条目形态（回引/空标记分布）未观察。
2. 各 int16/int32 业务语义未定性（HorizontalLegendItem 条目 int32、四个
   int16、−0x120/−0x11c/−0x110/−0x10e、≥2 组 5 字段；LegendClassFormat
   两个 double）；String/GUID 候选（名称、数据源标识、几何类型提示等）
   未最终闭合。
3. EditTemplate 宿主槽位（FeatureLayer 后继字段链的哪个成员）未闭合；
   其前 `02 00 01 00 00 00` 尾段未定名。
4. Variant::Read 的对象分支（VT_DISPATCH/UNKNOWN，全局 LoadObject 的
   16 字节类 GUID/零 GUID 语义）与 bool=false 的 `Seek` 分支未闭合；
   EMPTY/NULL|ARRAY 的原生改道路径语义未定。Variant::Write（`0x68a7cb50`）
   未逐段反编译（按对称性推断，未留证）。
5. ITextSymbol 实现类（TextSymbol 等）、ILinePatch/IAreaPatch 实现类、
   INumberFormat 实现类对象体未闭合，读取器在其新对象处停止。
6. EditTemplate 版本表前四档（docVer 83..92）Save「不写任何字节」与
   ObjectStream 无外部长度体的交互（读侧如何跳过）未观察；10.1 恒写
   版本 2，该分支在 10.1 产物中不可达。
7. BaseLegendItem 版本 0/1 分支无字节佐证（10.1 写侧恒写 3）；实现
   接受域（0..=3 / 0..=2 / 0..=1）较原生有符号比较更严，为读取器约束。
8. 全部结论为静态反编译观察（EditTemplate 另有样本辅助对位），未经
   ArcMap 10.1 运行验证，不得声称已验证。
