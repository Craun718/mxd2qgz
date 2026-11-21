# ArcMap 10.1 DEP 体终点、停止点后方延续段与 TimeData 族槽实现类

记录日期：2026-11-16（A 轮并行研究）。对应第十轮遗留开放项「DEP 尾段
（`0x155a..0x15d8`，126 字节）结构未闭合」与任务书另三项（`0x15d8` 对象头
归属、LELP ㉔ IGraphicsContainer 槽实现类、LELP/后继段 ㉖ ISymbol 槽未闭合
类、TimeData 族槽实现类）。**本轮裁决：DEP 体没有尾段**——DisplayExpression
Properties::Load 的体在表达式引擎槽处完结，「DEP 尾段 126 字节」实为
FeatureLayer 后继段 ㊵/㊶ 两段，样本 `0x15d8` 处对象头即 ㊷ 槽
HotLinkExpressionProperties（键序与读/写侧一致，第十轮「归属存疑」撤销）。
实现见 `src/layers/dep_tail.rs`（新模块，未接分派链）。全部为静态反编译与
样本辅助对位，未运行 ArcMap 10.1，未人工目视验收。

## 方法与文件身份

- 本机 pyghidra（Ghidra + JDK 25）工程副本（/tmp/gh-a-deptail/，由 /tmp/r3
  与 /tmp/f-lelp-gh 既有工程复制，未动共享工程）：
  - FeatureLayer.dll：SHA-256 `2baf8706…98bb60`，**ImageBase `0x6f6a0000`**
    （pefile/Ghidra 双重复核；既有文档「ImageBase 0x6f280000」为笔误，函数
    地址不受影响——旧地址均按 0x6f6a0000 基换算）；
  - LayerLib.dll：SHA-256 `55514f05…330e02`，ImageBase `0x78690000`；
  - SystemCore.dll：SHA-256 与第九轮记录一致（f63cf949…），ImageBase
    `0x7ffb0000`（temp/arcgis101-bin/ 既有提取物）；
  - LabelPlacement.dll：SHA-256 `8415f63c…35ea3d`，ImageBase `0x78530000`。
  全部 pefile 复核 10.1.0.3035。
- 样本字节：temp/r6-maps.bin（152941 字节，/Maps 流内偏移）；逐字节走查脚本
  temp/agent-a-deptail/walk_dep_tail.py（`0x155a..0x161c` 全断言通过）。
- 类名/IID：esri.clsid 全表（temp/agent-fl-tail/ecfg/）、esriSystem 与
  esriCarto 类型库 dump（/tmp/mxd-arcgis101-guiddocs/wine/）。
- 定位链（DEP 与 TimeData 族四类同法）：CLSID 数据常量引用 → GetClassID
  实体 → IPersistStream 子虚表（COL 簇 + RTTI `.?AV<类名>@@`）→ +0x14/+0x18
  = Load/Save。

## 一、DisplayExpressionProperties 体终点（证据：确定的）

第十轮「FeatureLayer.dll 侧 GetClassID/Load 未定位」撤销：DEP 实现就在
FeatureLayer.dll，定位链为 RTTI `.?AVDisplayExpressionProperties@@`
（TypeDescriptor `0x6f7d2460`，名字串 `0x6f7d2468`）→ 7 个 COL
（`0x6f7b5a08..0x6f7b5a6c`，其中 IPersistStream 子对象 COL `0x6f7b5a08`
off=48/cd=4）→ 子虚表 `0x6f7ab73c`（8 槽：+0x0c GetClassID `0x6f6cc010` =
thunk `0x6f6cba10` 实体、+0x14 Load `0x6f6cc000` = thunk → **实体
`0x6f6cbdd0`**、+0x18 Save `0x6f6cbfa0` = thunk → **实体 `0x6f6cbe70`**）。
GetClassID 实体 `0x6f6cba10` 写出常量 `0x6f7ab7b8` =
`{D75C6301-C05A-484C-A7BD-AE82E5E1FC75}`，与 esri.clsid 声明及样本 `0x151a`
头逐字节相同（三方互证）。主虚表 `0x6f7ab6b0` +0x0c = 名字 getter
`0x6f6cba00` 返回 `L"DisplayExpressionProperties"`（旁证）。

Load `0x6f6cbdd0` 字段序（Save `0x6f6cbe70` 对称，保存版本经
`AfStreamGetSaveVersion(docVer,5,表)` 取值）：

| # | 字段 | 版本门 | 对象偏移 | 样本（键 83 @0x151a） |
| --- | --- | --- | --- | --- |
| ① | int16 版本 | `1 < (short)ver` 拒绝 `0x7fffbffb` | — | 1 |
| ② | String（表达式） | 恒有 | −0x10 | cb=12「[BSM]」（含终止零 2 字节） |
| ③ | int16 | 恒有 | −0xc | −1 |
| ④ | LoadObject(IAnnotationExpressionEngine `{AA157206-…}`，`DAT_6f7ab788`) | 恒有 | −0x8 | AnnotationVBScriptEngine 新对象键 84（体 4 字节） |

**体在 ④ 完结，无任何后继字段**（Load 读流序列仅此四项；成功后仅清脏标志
位）。`annotate_props::read_display_expression_props_prefix_10_1` 的「前缀」
即全部体——第十轮「尾段 0x155a..0x15d8 属 DEP」的工作假设撤销。业务命名
（表达式/int16 语义）维持候选。

## 二、停止点后方 126 字节的分解（样本对位，辅助观察）

`0x155a..0x15d8`（126 字节）= FeatureLayer 后继段 ㊵ + ㊶，随后 ㊷/㊸ 至
`0x1606` 收尾。FL 体版本 34（0x22）下四段门全部命中（>0x1a / ≥0x20 / >0x20 /
>0x21），与读侧、写侧（FL::Save `0x6f6eb1b0` 对称序列）均一致：

| 段 | 样本区间 | 内容 |
| --- | --- | --- |
| ㊵ short +0x300 + String ×3 | `0x155a..0x156d`（20B） | short 0；3 × 空 BSTR（cb=2，各 6 字节） |
| ㊶ TimeData（`0x7870d8b0`） | `0x156e..0x1583`（22B） | ver 2、short 0、ITimeReference 槽空标记、tail short 0 |
| ㊶ TimeTableDefinition（`0x7870df80`） | `0x1584..0x15ad`（42B） | ver 1、String ×4 空、ITimeExtent 槽空标记 |
| ㊶ TimeDimensionDefinition（`0x7870e250`） | `0x15ae..0x15bb`（14B） | ver 1、String ×2 空 |
| ㊶ TimeDataDisplay（`0x7870f6c0`） | `0x15bc..0x15d7`（28B） | ver 1、short 0、0.0、int 4、0.0、int 8 |
| ㊷ HotLinkExpressionProperties 槽 | 头 `0x15d8..0x15eb`，体 `0x15ec..0x1605` | CLSID `{1B848B0F}`、键 85、体 ver 1/空串/−1/16B 零 |
| ㊸ short +0x2ca | `0x1606..0x1607` | 0 |

四段函数均按第十轮已定位的 LayerLib 地址独立反编译复核（字段序与第九轮记录
一致，无交插短字段）。`0x15d8` 对象头归属裁决：**即 ㊷ HLEP（键 85）**，键
序与读/写侧一致；第十轮「按 10.1 写侧顺序 ㊷ 应在 ㊶ TimeData 之后，归属存
疑」的疑点源于把 126 字节整体误归 DEP，本轮撤销。

**FL::Load 之后的流**：`0x1608` 起 ElementCollection
（`{CE8F3972-E9BE-11D1-A232-080009B6F22B}`）对象头，键 86，体首 `0x161c`
（ver=4 形态）。归属已明：`fdo_graphics_layer.rs` 第 5 步
LoadObject（IElementCollection2 `{B2BFC403-…}`）槽（this+0x11c）的样本新
对象——现有 `parse` 诊断「ElementCollection2 槽起的字段不可达」即指此；
ElementCollection 体读取器已在 `src/objects/element_collection.rs` 闭合
（元素体轮），FL 延续段接线后即可达。

### 顺带复核（不改既有结论）

- ㊱ PreJoinDefExpressions 计数后的 6 零字节（`0x14f2..0x14f7`）：本轮以
  LayerLib `ReadPreJoinDefExpressions 0x786c5d30`（仅
  [count][count×(String+String)]，前置 `FUN_786c6f40` 为链表清理不读流）与
  FL 读/写两侧调用点汇编（`0x6f6eab0f` / `0x6f6ec1eb`，调用后直连 ㊲
  +0x2c0 读）三方复核——**10.1.0.3035 读/写侧均不产生也不消费这 6 字节**，
  归属仍未决（维持「计数 + 6 字节保留头」消费策略）。
- 样本 ㉙ UID（键 82 @`0x14a9`）体内 GUID 值 =
  `{01004145-0D1C-11D2-A26F-080009B6F22B}`（= LabelEngineLayerProperties
  coclass）。该 UID 槽语义候选升级：标注属性类标识候选（此前「UID 值语义
  未定」）；另注：`0x14c0` 处 LELP 形 20 字节即此 GUID 值与 ㉚ cb=0 空串的
  重叠，非对象头。

## 三、LELP ㉔ IGraphicsContainer 槽（部分闭合：IID 更正 + 样本空槽）

- **请求 IID 更正（对第九轮记录）**：LELP::Load（LabelPlacement.dll
  `0x785468c0`）㉔ 槽 LoadObject 常量 `DAT_7857cb7c` =
  `{D09411D1-4BB4-11D1-B6CC-080009B996CC}`，与 esriCarto 类型库接口 #193
  IGraphicsContainer 逐字节相同。第九轮表格所记
  `{D09411D1-D0B4-11D1-880A-080009EC732A}` 系笔误（该值不在类型库；
  `880A-080009EC732A` 族尾与 BOLP 同族，疑为转录串扰）。
- **样本槽位 = 空标记**：自 FIDSet 头 `0xf63` 反向逐字节回推（Envelope 键
  74 体终于 `0xf2b`——ver 3、四坐标、flags 0、PCS 键 30 回引；⑳㉑ double
  ×2 = 0.0、㉒㉓ int32 ×2 = 0；㉔ 槽 `0xf43..0xf52` 16 零；㉕ short −1、
  ㉖ int32 2、ALPC 条目键 0、ALPC 尾 int32 1、⑫ short 0），与 0xb18 起
  LELP 体前向链一致，无剩余字节。样本 11 层其余实例未逐一走查（层 32 为
  唯一深度走查对象，同前几轮口径）。
- **实现类未闭合（负证据）**：样本无实例、无可定位的创建点。LELP 构造
  （`0x78545900` 一带，含默认标注属性初始化 `0x78546eb0`）对该成员仅置
  空/经注册表默认字体初始化 TextSymbol（`0x785472a0` 经
  `AfCreateInstance({B65A3E74})`=TextSymbol），未见容器实现类创建；
  IGraphicsContainer IID 常量在四份已开 DLL 中仅三处引用（LayerLib
  `0x78722838`、LabelPlacement `0x7857cb7c`、FeatureLayer `0x6f7aa5ac`，
  均为 LoadObject 请求 IID 形，非实现类 QI）。实现者候选（Composite
  GraphicsLayer、FDOGraphicsLayer 等容器 coclass）在 afcore 等未开模块，
  本轮未展开。读取器维持「新对象 = 停止点」语义。

## 四、TimeData 族槽实现类（证据：确定的，无样本字节）

时间四槽的实现链全在 SystemCore.dll；槽请求 IID 只定接口形状，对象类身份
由流内类标识决定。esri.clsid 声明 + GetClassID 常量 + 类型库 coclass 三方
互证；**样本中全部槽位为空标记，无任何实例字节**——以下字段序仅有反编译
证据，读取器按合成输入验证。

定位链（每类）：RTTI `.?AV<类名>@@` → COL 簇 → IPersistStream 子虚表
（+0x0c GetClassID 引用 CLSID 数据常量）→ +0x14/+0x18。

| 类 | CLSID | GetClassID | Load | Save（恒写版本） | IPersistStream 虚表 |
| --- | --- | --- | --- | --- | --- |
| TimeReference | `{EFB2E7DB-78F4-4E24-B01F-4F9C7AB800C5}` | `0x7ffeac00` | `0x7ffebbf0` | `0x7ffebc90`（1，`0x8006d050`） | `0x800738ec` |
| TimeExtent | `{5DC783DE-283A-4963-AB53-25A05C5D76CC}` | `0x7ffe4650` | `0x7ffe81f0` | `0x7ffe82f0`（2，`0x8006d070`） | `0x800735c4` |
| Time | `{E1721810-8210-45B1-8590-FC4C911FBA20}` | `0x7ffe14a0` | `0x7ffe1780` | `0x7ffe1870`（1，`0x8006d0c0`） | `0x80073084` |
| TimeZoneInfo | `{78FAD5F1-60FA-458A-8D93-630DA920448D}` | `0x7fff6330` | `0x7fff7790` | `0x7fff77f0`（2，`0x8006d040`） | `0x80074158` |

- **TimeReference::Load**：① int16（`1 < (short)ver` 拒绝）→ ② LoadObject
  (ITimeZoneInfo `{0F8FD6AB-CE6D-4BB2-A5EF-2F6035E304C5}`，`0x80073924`) →
  ③ short（this+0xc）→ ④ short（this+0xe）。
- **TimeExtent::Load**：① int16（`2 < ver` 拒绝）→ ② LoadObject(ITime
  `{66810D21-8DE0-44EE-B26E-465AC09F161F}`，`0x800730fc`；起点）→ ③ 同 IID
  （终点）→ ④ LoadObject(ITimeReference `{F6CBA4A3}`，`0x8007361c`) →
  ⑤ 版本 >1：1 字节（this+0x14；Save 恒写）。第 ⑤ 字节与 FL 版本 0x1c..0x1f
  分支的 `{BD724B95}` 槽（map-object IID 表）非同一层——该分支请求的
  ITimeExtent 实现类即本类。
- **Time::Load**：① int16（`ver < 2` 接受，即 0/1）→ ② int32 判别值：
  ==1 → 8 字节整块（OA 日期 double 候选，`PANK`）；==2 → 16 字节整块
  （`WKSDateTime`，`PAU_WKSDateTime__K`）；其他 → 原生 `0x80004005`。
- **TimeZoneInfo::Load**（体 helper `0x7fff6010`）：① int16（`2 < ver`
  拒绝）→ ② 版本 <2：int32（bias 候选，内存缺省 −1）→ ③ String ×3 →
  ④ 版本 <2：int32（缺省 −1）→ ⑤ int32（this+0xc）→ ⑥ 172 字节规则块
  （`0x7fff5e90` 读，`0x7fff5f50` 写）→ ⑦ int16 计数（≤0 不进循环）
  → ⑧ 计数 × [int32 + 规则块]（每条 176 字节）。见[规则块复核](#timezoneinfo-规则块与非空-timeawarehurricanes-宿主)。
- 槽请求 IID 与 FL 版本 0x1c..0x1f 遗留分支的三个 LoadObject（`DAT_6f7adab4`
  ={F6CBA4A3}、`DAT_6f7adad4`={BD724B95}、`DAT_6f7adde4`={E720618B}）接口
  相同——该分支的 ITimeReference/ITimeExtent 槽实现类同为本节两类
  （map-object 文档「ITimeReference/ITimeExtent」定名复核无误）。

## 五、㉖ ISymbol 槽（FL 版本 >0xd）未闭合面（负证据清单，未闭合）

`read_symbol_slot_full_10_1` 现分派 19 个类标识（SimpleMarker/SimpleLine/
SimpleFill/MarkerLine/CartographicLine/Hash/Template/LineDecoration/
MultiLayerLine/MultiLayerMarker/MultiLayerFill/CharacterMarker/PictureMarker/
PictureLine/ArrowMarker/LineFill/MarkerFill/GradientFill/DotDensityFill）+
嵌套 StdFont/StdPicture；加 TextSymbol（`element_body`，经 LELP ⑯ 槽接线）
即任务书口径的「已闭合类」集。本轮负证据梳理：

- DisplayCore.dll 内 ISymbol IID `{F3435802}` 常量两处（`0x6ec0ca2c`/
  `0x6ec0cb6c`，.rdata），全 .text 69 处 push 立即数引用（QI 应答与
  LoadObject 请求混布）——按引用逐点归类的成本/收益不佳，本轮未展开。
- 任务书「LELP ㉖」按槽位语义应为 **FeatureLayer 后继段 ㉖**（LELP 自身
  ㉖ 为 int32 标量）；样本该槽 = SimpleMarkerSymbol 键 77（已闭合形态）。
- **分派缺口（建议整合轮处理）**：`read_symbol_slot_full_10_1` 未含
  TextSymbol——㉖ 槽出现 TextSymbol 新对象时会按未知类停止回退（样本无此
  形态；LELP ⑯ 槽的分派已含 TextSymbol，两处清单不一致）。
- 剩余 ISymbol 实现者（标注/Callout、文本背景族等 esriDisplay coclass）无
  样本实例，维持停止点。

## 实现（src/layers/dep_tail.rs，未接分派链）

- 常量：`TIME_REFERENCE_CLASS_ID`/`TIME_EXTENT_CLASS_ID`/`TIME_CLASS_ID`/
  `TIME_ZONE_INFO_CLASS_ID`（四类 CLSID 内存序，证据注于常量文档）。
- 体读取器：`read_time_body_10_1`、`read_time_zone_info_body_10_1`、
  `read_time_reference_body_10_1`、`read_time_extent_body_10_1`（槽位未知类
  = `Unsupported` 停止点：游标回退槽位起点、键不保留、登记表回滚；体结构
  字段文档含上述地址证据）。
- 槽位分派：`read_time_data_family_10_1`（㊶ 四段；TimeData 槽新对象为
  TimeReference 时整体解析、TimeTable 槽新对象为 TimeExtent 时整体解析，
  未知类记入 `TimeFamilyStop10_1`）——样本 `0x156e..0x15d7` 空槽形态有
  合成对照测试（逐字节同构）。
- 测试：模块内 `#[cfg(test)]` 12 项（各体正常/版本拒绝/判别值拒绝/嵌套
  TZI/回引/未知类停止回滚/截断扫描/样本形态四段）。
- `src/layers/mod.rs` 仅加 `pub mod dep_tail;` 一行；未改任何既有分派点。

## 建议整合接线点（归整合轮）

1. `feature_layer_tail::read_feature_layer_successor_10_1` ㊴ 槽 DEP 臂：
   DEP 前缀成功后**撤销 `SuccessorStop10_1::ExpressionPropsBody` 停止**
   （DEP 体已完整），让既有 ㊵/㊶/㊷/㊸ 代码继续——样本停止点预期由
   `0x155a` 推进到 `0x1606`（TailCompleted），其后 `0x1608` 为 FDO
   IElementCollection2 槽的 ElementCollection 头（键 86，体读取器已闭合，
   由 fdo_graphics_layer 第 5 步接线消费）。
2. 同函数 ㊶ TimeData/TimeTable 槽的 `TimeSlotUnsupported` 停止臂改调
   `dep_tail::read_time_reference_slot_10_1`/`read_time_extent_slot_10_1`
   （即 `read_time_data_family_10_1` 的槽位语义）；版本 0x1c..0x1f 分支的
   ITimeReference/ITimeExtent 槽同理。
3. `read_symbol_slot_full_10_1` 补 TextSymbol 分派（或明示两处分派清单差异
   的理由）。
4. `mxd_maps` 诊断：`ExpressionPropsBody` 透传内容改为 DEP 完整体 + 后续段
   摘要（归整合轮）。

## 未解决事项

- ㊱ 计数后 6 零字节归属（读/写侧 10.1.0.3035 均不产生、样本恒在）。
- LELP ㉔ IGraphicsContainer 槽实现类（无样本实例、afcore 未开）。
- ㉖ ISymbol 槽剩余实现者（Callout/文本背景族等）与 DisplayCore 69 处
  ISymbol 引用逐点归类。
- TimeData 族四类全部标量/字符串业务语义（bias、时区名、规则块字段、
  Time 判别值的 OA date/WKSDateTime 语义、TimeExtent 尾字节）均为候选；
  四类无样本实例，仅合成验证。
- TimeReference/TimeExtent 在真实数据流（启用时间的图层）中的嵌套形态未见
  样本；TimeData 段 ver <2 分支、TimeExtent ver ≤1 分支、TimeZoneInfo
  ver <2 bias 分支仅反编译证据。
- 全部结论为静态取证，未运行 ArcMap 10.1，未人工目视验收；样本对位为辅助
  观察（temp/agent-a-deptail/walk_dep_tail.py 断言通过）。

## IGraphicsContainer 实现续查

10.1 介质提取的 72 份 OLB 中，esriCarto、esri3DAnalyst、esriGlobeCore 共声明七个实现 `{D09411D1-4BB4-11D1-B6CC-080009B996CC}` 的 coclass。LELP 的 `0x785468c0` 在版本 ≥3 时于 `0x78546b2c` 传入该 IID 加载槽。类型库结果说明接口归属，持久化由二进制中的 QueryInterface 和虚表确认。

| 类 | CLSID | 本次范围 |
| --- | --- | --- |
| CompositeGraphicsLayer | `9646BB83-9512-11D2-A2F6-080009B6F22B` | FeatureLayer.dll 主 QI `0x6f6b0770` 命中该 IID 返回 +0x34。已有 Load `0x6f6b2100` 的读取器。 |
| GraphicsSubLayer | `D3A9D37D-EC3C-40C7-87B6-7057AEB605F2` | FeatureLayer.dll 主 QI `0x6f70e6a0` 命中该 IID 返回 +0x38。没有独立可用的对象流持久化接口。 |
| FDOGraphicsLayer | `34B2EF83-F4AC-11D1-A245-080009B6F22B` | AnnoLayer.dll 主 QI `0x6b18e8c0`，已有对象体读取器。 |
| Map | `E6BDAA76-4D35-11D0-98BE-00805F7CED21` | Map.dll 主 QI `0x793e4620`，已有对象体读取器。 |
| PageLayout | `DD94D76E-836D-11D0-87EC-080009EC732A` | PageLayout.dll 主 QI `0x7b6cedf0`，已有布局读取器。 |
| GraphicsLayer3D | `4C87A8F0-01E3-11D4-AB69-0008C73FD50C` | Scene.dll 主 QI `0x7d2a15b0`，IGraphicsContainer 位于 +0x38，IPersistStream 位于 +0x5c。 |
| GlobeGraphicsLayer | `A415A073-CF06-4CDC-8664-439D0D5E7169` | GlobeLayers.dll 主 QI `0x72f51ad0`，IGraphicsContainer 位于 +0x3c，IPersistStream 位于 +0x5c。 |

- FeatureLayer.dll 版本资源为 `10.1.0.3035`，PE x86，SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`。已有 `FeatureLayer/6f70e290.c` 和 `6f70e540.c` 确认类名与虚表归属。objdump 显示主 QI 的 12 个 IID 均非 IPersist、IPersistStream 或 IPersistVariant，后备链 `0x6f73c440 → 0x6f73c0d0` 仅比较 IConnectionPointContainer、IUnknown、ISupportErrorInfo 和 `9F9650F1-5F49-4041-BA0F-D10BAFF1D7BC`。回到后备链后，对流加载使用的两个持久化 IID 均落入 `E_NOINTERFACE`。同模块 `0x6f70e2b7` 引用登记 CLSID `0x6f7aa65c`。这一裁决针对独立对象体，CompositeGraphicsLayer 内联的子层条目继续按各自版本布局读取。
- Scene.dll 和 GlobeLayers.dll 均为 PE x86，FileVersion `10.1.0.3035`，SHA-256 分别为 `b41f37424a13b0257ce3c5c3258038a3640876aba48b33e05b332f7af3594006` 和 `79346fc1fa0d25c01ac41ae2a7b0ee3014847c38f1661ff7d23cce75fe6fd1b8`。IPersistStream 子表经 RTTI/COL 对位，分别为 `0x7d2c8a2c` 和 `0x72f78410`；三项持久化入口 GetClassID/Load/Save 分别为 `0x7d2a4c60/0x7d2a4a10/0x7d2a4870` 和 `0x72f60a40/0x72f60080/0x72f5fb10`。GetClassID 指令各引用登记 CLSID 常量 `0x7d2c8c4c` 和 `0x72f787b0`。
- 两类 3D 容器的字段序与六类可持久化对象的 LELP 接线见下。CompositeGraphicsLayer 可含 FeatureLayer，FeatureLayer 可再含 LELP；需要共享对象表及递归限制。

### 三维图形容器的对象体

已有反编译目录缺少 Scene.dll 与 GlobeLayers.dll 的这四个 IPersistStream 入口。用 pyghidra 加载上节已验明身份的 PE，补出 `0x7d2a4a10 / 0x7d2a4870` 和 `0x72f60080 / 0x72f5fb10` 的伪代码，并用 objdump 复核指令。四份可复查的输出在忽略目录 `temp/raster-renderer-101/` 内，按入口地址命名。

| 类 | 对象体顺序 |
| --- | --- |
| GraphicsLayer3D | 版本 short（1–5）、AfCore String、short×4、ILayer 槽、IEnvelope 槽、有符号元素计数、IElement 槽循环。版本 ≥2 再读 short，≥3 再读 ISpatialReference，≥4 调 LayerLib ExtensionHelper，≥5 再读 short。 |
| GlobeGraphicsLayer | 版本 short（1–7）、AfCore String、short×2、ILayer 槽、IEnvelope 槽、有符号元素计数、IElement 槽循环。每个槽后按版本读下述附加属性。循环后各版本均调 ExtensionHelper，接 short×2；版本 ≥4 再读两个 IEnvelope 槽。 |

两类均从加载时接受的版本区间分派。Scene 的 `0x7d2a4af4 / 0x7d2a4b16 / 0x7d2a4b63 / 0x7d2a4bcb` 以及 Globe 的 `0x72f60131 / 0x72f60153` 一带分别对位到 ILayer、IEnvelope、IElement 和 ISpatialReference。几何附加槽请求 `0x72f78660` 的 `{1B07790D-38C5-11D0-92D2-00805F7C28B0}`（IGeometry）。Scene Save 选择版本后写出当前尾部；旧版本接受范围由 Load 的单独分支确定。

Globe 元素附加属性的定宽段按顺序如下：

1. 版本 ≥2 时先读 1 字节标志×3、int32、版本 ≥3 的 1 字节标志、1 字节标志×2、double×2。
2. 接 1 字节标志、double×3、4 字节原始值、float、1 字节标志、三组 double×3、1 字节标志×2、一组 double×3、1 字节标志、两组 double×3。Load 的七次 `0x72f609f0` 和 Save 的七次 `0x72f609a0` 均逐条复核，每次恒处理 24 字节。
3. 版本 ≥5 时再读 double×2、int32×4、short×3、int32。
4. 接 1 字节标志和 double。版本 ≥6 再读一个 IGeometry，≥7 再读第二个。

定宽段在 v2、v3–4、v5–7 分别为 215、216、258 字节。解析器保留原始字节与几何槽条目，字段用途待访问器证据。两类扩展列表使用 [LayerLib 的旧式 ExtensionHelper](mxd-10.1-raster-layer-tail.md)，支持零长度条目与 EditTemplateManager。关联图层接线范围见下。未知嵌套体在 LELP 回退至图形容器槽起点，截断或非法版本回滚整个 LELP 读取。

构造测试覆盖所有接受版本、LELP 后继标量、Globe 两个几何版本门、MarkerElement 与 Point 新对象和回引、非空模板扩展与零长度扩展、未知关联图层及逐前缀截断。对本地 66 个 MXD 路径的 `/Maps`、`/PageLayout` 和可提取的三个 `/3D Analyst` 流扫描两个 CLSID，未命中字节候选。没有真实非空宿主，也未进行 ArcMap/QGIS 人工验收。

### LELP 容器和关联图层接线

Scene.dll 的 `0x7d2a4af4` 和 GlobeLayers.dll 的 `0x72f60131` 都将同一个 `ILayer {34C20002-4D3C-11D0-92D8-00805F7C28B0}` 传给普通 `LoadObject`。两处 Save 分别以 `this+0x20` 和 `this+0x34` 的对象调用 SaveObject。普通对象槽没有可跳过的外层长度；关联槽仅在已有读取器证实读完 FeatureLayer、CompositeGraphicsLayer、FDOGraphicsLayer、GraphicsLayer3D 或 GlobeGraphicsLayer 的对象体后提交。其余图层的普通对象体或子槽仍待接入。两类 3D 图层也在 Map 图层列表按标志 1 或 ExtraSafe 分派，ExtraSafe 路径在条目范围内试读并检查完整消费。

LELP 的 IGraphicsContainer 槽接入类型库及持久化链确认的六类内置对象。图形图层复用完整对象判定；Map 复用原有前缀/后缀的 `Complete` 终点；PageLayout 复用对象体读序，保留外层在 `0x7b708d00` 处理的打印设置。普通布局对象里的 IMap 回引要求对象头和共享表都确认 Map 身份。该身份只在读取已识别的 Map 新对象头后登记，随整个槽的试读回退。对象体未完整消费时，游标回到容器头之前，该对象及其子对象的键一并撤销。新对象每进入一次跨图层/图形容器的无界分派占用一级限额，最多 8 级；已见键的回引不占用限额。深度上限为读取器防栈耗尽设置。

测试从仓库参考工程提取已验证边界的 Map、PageLayout 和 FDOGraphicsLayer 体，再构造非空 LELP 或 3D 关联宿主，以验证实际对象体可完整消费、根 Map 回引仍能解析、宿主后继字段仍可读取。补充合成的 CompositeGraphicsLayer、普通/有界 3D 图层、未知嵌套类和深度边界。真实非空 LELP 宿主及 ArcMap/QGIS 人工验收仍未验证。

关联槽续查将 BasemapLayer 和 ImageServerLayer 接入同一完整体判定。BasemapLayer 沿用 [Basemap Load 与子层证据](mxd-round3-basemap-20261117.md) 的 `Complete` 结束点。ImageServerLayer 沿用 [IPersistVariant 对象体](mxd-10.1-variant-image-server-layer.md)，即由 ObjectStream 对当前流构造 VariantStream 包装，其内部对象仍使用同一引用表。Scene v1 嵌套 ImageServerLayer v22 的合成数据验证了图层、名称和属性集三个键的登记，并覆盖尾字段截断的外层回滚。Globe v1 嵌套 BasemapLayer v6 的合成数据验证了容器尾字段和宿主哨兵的边界。

另复核 FeatureLayer.dll 10.1.0.3035（SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`）既有伪代码 `0x6f711270` 及 `0x6f7119b0`。GroupLayer 末尾 IDisplayFilter 的读门是版本 **12**，而版本 11 只改变子图层的普通/ExtraSafe 包装。PE 中 `0x6f7aafb8` 与 `0x6f7aa4bc` 两个常量分别与 IDisplayFilter 和 IEnvelope 的 IID 相符。尾部 `0x6f6c8100` 释放既有接口，`0x6f715540` 查询子层接口；这两个助手均未调用流读取。旧版本遮罩循环遍历的是已装载的子层，而独立的逐层遮罩记录由 Map 的加载流程读取。GroupLayer 尾段及其普通关联对象入口已按以下边界接入。


GroupLayer 完整体读取复用 LayerLib 旧式扩展与 DisplayCore 的 TransparencyDisplayFilter 读取器。版本 5 读扩展列表，版本 6 起读字符串（版本 6 再读 short），版本 7 起读 short，版本 8 起读两个 double，版本 9 起读 short，版本 10 起读三个 short，版本 12 再读 IDisplayFilter。组内子图层与关联槽共享引用表；普通子体须完整读取，ExtraSafe 子体须在声明范围内读完。未支持的子图层、过滤器或扩展使普通关联体整体回滚。Map 图层列表优先尝试完整读取 GroupLayer，并保留原前缀入口处理已界定但尚未完整支持的对象。

构造测试覆盖 GroupLayer v0–12、普通与 ExtraSafe 新子层、非空包络、模板扩展、非空透明度过滤器、后继哨兵和截断回滚。SDK Java brazil.mxd 的首个 v11 组图层从 Map 图层列表到达完整体边界，外层 A 范围无剩余；第二、第三个组及 SDK .NET 的两个 v12 组复核了子项数。版本 <11 的真实 GroupLayer 及 ArcMap/QGIS 人工验收仍开放。


## TimeZoneInfo 规则块与非空 TimeAwareHurricanes 宿主

复核 `SystemCore.dll` 10.1.0.3035（PE32 x86，SHA-256
`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`）。
现有 Ghidra 导出 `0x7fff7790`、`0x7fff6010` 确认版本门、对象体顺序及规则循环。
在读 helper `0x7fff5e90` 与写 helper `0x7fff5f50` 的原始指令中，
七个 AfStreamRead/AfStreamWrite 长度实参依次为
`4, 0x40, 0x10, 4, 0x40, 0x10, 4`，共 172 字节。两个宽串各含
32 个 UTF-16 码元，两个定长日期块各为 16 字节。先前将后两种宽度误记为
short，导致基本规则提前结束并把规则内字节当成计数。当前保存定长块原始字节，
按有符号计数解释年度规则循环，并在分配前校验数组的字节限额和流长度。

SDK .NET 的 `TimeAwareHurricanes` 在 `/Maps:0x113e` 进入 FeatureLayer v34
的 TimeData，内部 TimeReference 键 73、TimeZoneInfo v1 键 74。基本规则
范围是 `0x1202..0x12ae`，其后计数为 2。规则条目分别从 `0x12b0` 和
`0x1360` 起，年度键为 2006、2007，规则集合结束于 `0x1410`。
后继 TimeReference 两个 short 为 0 和 −1，TimeTableDefinition v1 从
`0x1414` 开始。FeatureLayer 对象与 ExtraSafe 边界 `0x17b9` 对齐。
Map v44 在 `0x2507` 结束，之后恰好剩四字节根尾值，PageLayout 同样读至流尾。

构造测试覆盖 v1/v2、负计数、非零定长块、截断和数组资源限额。
未定名的时区标量仍保留存盘值，ArcMap 与 QGIS 人工验收未执行。

## FeatureLayer 后继段的统计图符号

证据来自 ArcGIS Desktop 安装介质的 `DisplayCore.dll` PE32，文件版本
10.1.0.3035，首选基址 `0x6eb30000`，SHA-256
`9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`。
以既有 Ghidra 导出的函数为主，直接读取 PE 的 RTTI、虚表、版本常量和 IID
字节，并与 `esriDisplay.olb` 类型条目 343–345 的 CLSID 对照。以下地址为首选虚拟地址。

| 类 | CLSID | IPersistStream 虚表 | GetClassID / Load / Save | QueryInterface |
| --- | --- | --- | --- | --- |
| BarChartSymbol | `5031736A-BD70-11D3-9F79-00C04F6BC709` | `0x6ec0d2f4` | `0x6eb48710` / `0x6eb48430` / `0x6eb485b0` | `0x6eb46ba0` |
| PieChartSymbol | `50317368-BD70-11D3-9F79-00C04F6BC709` | `0x6ec109fc` | `0x6eb80160` / `0x6eb7fea0` / `0x6eb80010` | `0x6eb7e6a0` |
| StackedChartSymbol | `50317369-BD70-11D3-9F79-00C04F6BC709` | `0x6ec1100c` | `0x6eb86290` / `0x6eb85fa0` / `0x6eb86120` | `0x6eb84980` |

三者的 QI 均接受 FeatureLayer 后继段 ㉖ 所请求的 `ISymbol`
`{F3435802-5779-11D0-98BF-00805F7CED21}`（`0x6ec0ca2c`）。三个 Save
均写 short 版本 1，分别取自 `0x6ec0b304`、`0x6ec0b298`、`0x6ec0b284`。
每个体的首段依次为 short、double×4、byte、double。随后的类特有字段如下。

| 类 | 首段之后的标量 | 公共段前的对象槽 |
| --- | --- | --- |
| BarChartSymbol | byte×2、double×2 | ILineSymbol |
| PieChartSymbol | int32、byte×2 | ILineSymbol |
| StackedChartSymbol | byte×3、double | IFillSymbol |

公共段 Load `0x6eb4a5d0`、Save `0x6eb4a7f0`。依次读取 short 版本（Save
恒写 2，取自 `0x6ec0b300`）、int32×2、double、仅版本 1 读取的 ILineCallout
槽、int32 计数、计数项的 (IFillSymbol, double)，末尾为 IMarkerBackground 槽。
ILineCallout 和 IMarkerBackground 的 IID 分别在 `0x6ec0d6c4` 和 `0x6ec0d6d4`，
均由 LineCallout 的 QI `0x6eb577f0` 接受。后者也供 MultiLayerMarker 使用。
IFillSymbol 和 ILineSymbol 的 IID 常量分别位于 `0x6ec0ce8c` 和 `0x6ec0d334`。

解析器保留标量顺序，字段业务含义待验证。遇到未知嵌套对象时，回退统计图符号
整体及本次登记的键。截断、负计数及超过读取器限制的声明长度作为错误。
原生实现会忽略部分嵌套读失败，无法据此确定后一对象的起点。

参考样本及介质、SDK 现有语料共有 111 个去重工程的 221 个 Maps/PageLayout
流，未发现这三个 CLSID 的字节候选。合成测试覆盖三类对象、版本 1 的
LineCallout、共享回引、未知槽、逐字节截断及 FeatureLayer ㉖ 后续字段对位。
ArcMap 与 QGIS 中的人工验收未做。FeatureLayer ㉖ 的其他 ISymbol 实现仍待追查。

DisplayCore 的 `0x6ec0ca2c` 在现有反编译中由 22 个可映射到 RTTI 主虚表的
QueryInterface 函数直接比较，均已纳入符号分派。该扫描仅覆盖模块内的这组直接比较。安装介质 DLL 的 ISymbol IID 字节引用和
RTTI 主虚表补扫定位了其他模块的候选，已确认并接入的类见下一节。SymbolMimic
等其他候选仍需区分接口查询、消费调用和持久化入口。

## FeatureLayer 符号槽的栅格和 WMS 实现

以下地址为安装介质 ArcGIS Desktop 10.1.0.3035 的 PE32 首选虚拟地址。
`RasterLayer.dll` 基址 `0x7c710000`，SHA-256
`9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6`。
从 Ghidra 现有导出复核 QI、类工厂注册表 `0x7c765bd0` 及 IPersistStream
虚表。ColorSymbol 和 ColorRampSymbol 的 QI 都直接比较 ISymbol 的
`0x7c767c1c`、IFillSymbol 的 `0x7c767c2c`。RasterRGBSymbol 的 QI
沿 `0x7c7140f0 → 0x7c713100` 接受这两个接口。

| 类 | 类工厂注册 CLSID | QueryInterface | IPersistStream 虚表 | GetClassID / Load / Save 实现 |
| --- | --- | --- | --- | --- |
| ColorSymbol | `B81F9AE0-026E-11D3-9C1F-00C04F5AA6ED` | `0x7c713100` | `0x7c767e94` | `0x7c713600` / `0x7c713470` / `0x7c713550` |
| ColorRampSymbol | `40987040-204C-11D3-A3F2-0004AC1B1D86` | `0x7c711440` | `0x7c767b78` | `0x7c712510` / `0x7c712080` / `0x7c7122c0` |
| RasterRGBSymbol | `99DCCB66-2E09-11D3-A626-0008C7BF3347` | `0x7c7140f0` | `0x7c76a2a8` | 经 `0x7c72ca70` / `0x7c72ca30` / `0x7c72cb00` 转调 ColorSymbol 的三项实现 |

RasterRGBSymbol 的 IPersistStream::GetClassID 返回 ColorSymbol 的 CLSID，因而一般
按 ColorSymbol 读到持久化记录。若流内使用其工厂注册 CLSID，读取器按其实际 Load
委派读取同一体并保留流内类。三种注册 CLSID 与 `esriCarto.olb` 的 933–935 号
类别相符。ColorSymbol 体为 short 版本（上限 1，Save 恒写 1）、IColor、三个
int32（Save 末项置零）。ColorRampSymbol 体先读 short 版本（上限 5）、IColor、
三个 int32、IColorRamp，然后按下表读取剩余字段。IColor 与 IColorRamp 的 IID
分别取自 `0x7c767bec`、`0x7c767c0c`，复用已有对象槽和共享键表。

| 顺序 | 字段 | Load 门 |
| --- | --- | --- |
| 1 | short | 所有版本 |
| 2 | short | 版本 <2 |
| 3 | int32 | 所有版本 |
| 4 | short ×3 | 版本 ≥3 |
| 5 | int32 | 版本 ≥4 |
| 6 | short ×2 | 版本 ≥5 |

Save 的文档版本表 `0x7c7678fc` 为 `[2,2,2,3,5]`。v0、v1 和 v4 的尾段
按原生 Load 保留。字段含义未定的标量按原始顺序存储。未知 IColor 或 IColorRamp
对象使读取停在对应嵌套对象边界。截断回滚整个外层符号及其新登记的键。

SDK .NET 的 `BloomfieldTownship.mxd`（SHA-256
`eb2513113c4cf2528561a89371b8d8d5fb8cd44038f18ff8af32193c7db328b1`）
在 /Maps 的 RasterStretchRenderer `0x4b73..0x6d0f` 的 RasterIO 标签 9
内含无键图例组，组体 `0x5626..0x6335`。读取器确认组中的三个
ColorRampSymbol v5 键 218、233、248 及各自对象范围
`0x5656..0x5761`、`0x5ae1..0x5bec`、`0x5f58..0x5fd1`。
SDK Java 的 `portland.mxd` 的 RasterClassifyRenderer
`0x73b30..0x75da0` 内含图例组 `0x73bb1..0x73f5a`，完整读取五个
ColorSymbol v1（键 953、956、959、962、965）及其后续图例字段。
两段均从渲染器入口验证结束偏移。/Maps 根尾值分别位于 `0x777b` 和
`0x77373`。语料未发现 RasterRGBSymbol 注册 CLSID 实例，合成测试覆盖其委派体。

`WMSLayer.dll` 基址 `0x80d60000`，SHA-256
`d204a1efc758e6b721c356976f3be4384a8f23ce1ef085f3cccc4c4f53630da6`。
现有导出目录无该模块，使用 objdump 指令、PE IAT、RTTI 和虚表复核。
URLPictureMarkerSymbol 的 GetClassID `0x80d650d0` 返回 `0x80d8ba6c`
处的 `4213CB01-BEE0-4F3F-BF07-0CD0A4C95EB2`，注册表为 `0x80d89f43`。
QI `0x80d63c10` 在 `0x80d63c3f` 和 `0x80d63c2b` 分别比较 ISymbol
和 IMarkerSymbol。IPersistStream 虚表 `0x80d8c538` 的 Load 和 Save
经 `0x80d65f70`、`0x80d65e40` 转至 `0x80d65110`、`0x80d651d0`。
持久化体依次为 short 版本（Load 上限 1，Save 取 `0x80d8b900` 处常量 1）、
两个 AfCore String、一个 IPictureMarkerSymbol 槽（`0x80d8be00`）。
原生可将该槽写为空并在内存中初始化默认对象，读取器保留流里的空槽或内嵌
PictureMarkerSymbol。两字符串按原始顺序保留，解析不访问 URL。语料未发现此 CLSID，
合成测试覆盖空槽、内嵌体、错误类及逐字节截断。

上述 SDK 实档验证为本地 Rust 解析，尚无 FeatureLayer ㉖ 的对应实档，也未在 ArcMap
或 QGIS 中人工验收。其他 DLL 中的持久化 ISymbol 实现仍待追查。

### SymbolMimic 的持久化入口

`DisplayRepresentation.dll` 首选基址 `0x6ee10000`，SHA-256
`2bd5100c7727212feef568a9210efeb3b95f1172745575361738e85a5a754e10`。
用 objdump 复核 `SymbolMimic` 的 QI `0x6ee790f0`。在 `0x6ee79217`
请求 ISymbol，在 `0x6ee792ca` 请求 IPersistStream 并返回 `this + 0x28`
（虚表 `0x6ee9b420`）。GetClassID `0x6ee78b40` 指向 `0x6ee9b564` 的
`EC284786-A394-44E9-94AA-AA05830747E2`，类工厂条目在 `0x6ee93234` 引用
同一常量。Load `0x6ee7a7d0` 和 Save `0x6ee7a880` 同为 short 版本
（`0x6eeb224c` 初值 1）、int32、对象槽、int32。对象槽请求 IID
`2B7BE3AB-BB72-49D0-A6EE-63ACD1F9068C`（`0x6ee9ac20`），
`esriDisplay.olb` 第 408 项定名 IRepresentationRule。
同模块 RepresentationRule 的 QI `0x6ee73a50` 直接接受该接口。
Load 在 `0x6ee7a815` 以有符号比较拒绝大于 1 的版本。规则槽返回非负
HRESULT 时，`0x6ee7a854` 继续读取尾随 int32，包含空槽的 S_FALSE。

共享 ISymbol 读取器已接入空规则的 SymbolMimic，因此 FeatureLayer 后继
槽可按正常分派读取。合成测试覆盖两项原始值、完整边界、逐字节截断及遇到
非空槽时回退外层对象和键。非空规则的读取范围见下节。尚无实档及
ArcMap 10.1、QGIS 人工验证。

### SymbolMimic 中的 RepresentationRule

同版 DisplayRepresentation.dll 的 GetClassID `0x6ee73550` 返回
`1079E40A-83C1-467B-B828-E5C06EF5F6B7`。主 QI
`0x6ee73bab–0x6ee73d21` 对 IPersist 和 IPersistStream 返回完整对象
`+0x80`。构造指令 `0x6ee736bd` 写入虚表 `0x6ee9ab7c`，
其 Load / Save 分别为 `0x6ee73eb0` / `0x6ee74220`。

Load 先读有符号 short（上限 1），再于 `0x6ee73f10` 调用内联效果
列表读取器 `0x6ee4c300`。该列表读取 short（上限 2）、int32 计数。
效果列表之后按序读取一个 int32、IBasicSymbol 计数及循环、覆盖值
计数及循环、次序索引计数及循环、尾随 int32。各循环以有符号比较跳过
非正计数。IBasicSymbol 的接口身份由 `0x6ee97f44` 的 IID 与同版
esriDisplay 类型库对位。

共享符号读取器支持同类对象键回引，并读取效果和符号层类型已知的
RepresentationRule。覆盖值和次序列表可含成员，具体布局见下节。
遇到未支持的效果或符号层时会回退整个 SymbolMimic。实档及
ArcMap 10.1、QGIS 人工验证仍待完成。

### RepresentationRule 的覆盖值和次序列表

复用同一份 10.1.0.3035 的 DisplayRepresentation.dll。Load 在
`0x6ee73ff4`、`0x6ee74013` 读取两项 int32，随后在
`0x6ee74036` 调用 IAT `0x6ee94068` 对应的 AfCore `String::Read`。
Save 在 `0x6ee7431f`、`0x6ee7433f`、`0x6ee74363` 按相同顺序
写出，末次调用为 `String::Write`。解析器按流序保留覆盖记录中的两项
标识和文本。两项标识的具体业务对应仍待核查。

次序列表的每项由 `0x6ee740de` 读入一个 int32，写侧
`0x6ee7444f` 对位。解析器记录原始序列，包括重复值，并在循环前核验
计数、资源限额和剩余字节。合成测试覆盖非空覆盖值、非空次序列表、
截断回滚和后续规则回引。尚无实际 MXD 或目标软件的验证结果。

### RepresentationRule 的 IBasicSymbol 持久化分派

同版 DisplayRepresentation.dll 的 CLSID、GetClassID 和持久化虚表
按下表对位。三类均由 RepresentationRule 的 IBasicSymbol 槽承载。

| 类 | CLSID | GetClassID | IPersistStream 虚表 | Load / Save |
| --- | --- | --- | --- | --- |
| BasicFillSymbol | `BC38C464-8B02-4B0C-81D5-8237847A46CE` | `0x6ee356d0` | `0x6ee97e7c` | `0x6ee35790 / 0x6ee358a0` |
| BasicLineSymbol | `ED8FB823-10FA-468B-8DCD-4327C0DF0B22` | `0x6ee369d0` | `0x6ee98184` | `0x6ee36a10 / 0x6ee36b20` |
| BasicMarkerSymbol | `1BFF73A0-D134-4E88-8BC5-788BD351B12D` | `0x6ee38e20` | `0x6ee984a0` | `0x6ee38ee0 / 0x6ee39090` |

Fill 和 Line 的 Load 依次读取有符号 short（上限 1）、内联效果列表、
普通对象槽及尾 int32。槽的 IID 分别取自 `0x6ee97f34` 和
`0x6ee981f0`，与 10.1 类型库的 IFillPattern、ILineStroke 一致。
Fill 在构造期间使用 `0x6ee97f84` 的 SolidColorPattern CLSID，
Line 的 ILineStroke 在类型库中由 LineStroke 实现。

Marker 的 Load 读取有符号 short（上限 2）、内联效果列表、
GraphicAttributeList、v2 起的 AfCore String、ExtraSafe 对象槽和
尾 int32。ExtraSafe 槽在 `0x6ee38fa2` 调用
`ExtraSafeReadObject`，请求 `0x6ee96b90` 的 IID。
`0x6ee38f4b` 调用 DisplayLib.dll 的 GraphicAttributeList::Load。
同版 DisplayLib.dll（SHA-256
`e16c4c2610f20896005cf7ae56965dd60a61530b27daa3d566682f6ac11982c6`）
的导出表和现有反编译将入口绑定到 `0x6ed896e0`。其布局为
有符号 short（上限 1）、int32 计数，每项为 int32 属性标识加
`Variant::Read(stream, true)`。解码后的值按属性标识应用到预置属性表。

三类的非空效果及图案、线式和标记子对象仍待接入读取器。
本轮没有进行实档、ArcMap 10.1 或 QGIS 人工验证。


### BasicFillSymbol 与 SolidColorPattern

同版 DisplayRepresentation.dll 的 SolidColorPattern
（`29742A05-5999-43E7-9FA1-5370C684F49E`）通过
`0x6ee78840` 输出 CLSID。持久化虚表 `0x6ee9b124` 的 Load
`0x6ee788a0` 和 Save `0x6ee788c0` 分别委派给 DisplayLib 的
GraphicAttributeList::Load 和 Save。图案体直接使用属性列表布局。
属性值复用带保留区的 AfCore Variant 对象流读取器，属性 ID 及值按流序保留。

已接入无效果成员且使用 SolidColorPattern 的 BasicFillSymbol。
规则、符号层、图案及属性值沿用同一对象键表。构造测试覆盖新填充符号、
非空属性表、图案和符号层回引及逐字节截断回滚。其他图案类、效果成员、
实际 MXD 和 ArcMap 10.1、QGIS 人工验证仍待完成。

### BasicLineSymbol 与 LineStroke

同版 DisplayRepresentation.dll 的 LineStroke
（`BD2A7F5C-1D8B-4221-B3B0-7F40B0305503`）通过
`0x6ee625a0` 输出 CLSID。持久化虚表 `0x6ee9a168` 的 Load
`0x6ee62600` 与 Save `0x6ee62620` 分别经 IAT
`0x6ee94184`、`0x6ee94188` 委派给 DisplayLib 的
GraphicAttributeList::Load 和 Save。属性体与 SolidColorPattern
复用同一读取器。BasicLineSymbol 的 `0x6ee36a89` 读取 ILineStroke，
随后在 `0x6ee36acd` 读取尾 int32。

已接入无效果成员且使用 LineStroke 的 BasicLineSymbol，保留属性 ID
和 Variant 值，并沿用规则的对象键表。构造测试同时验证填充、线符号
及各自子对象的回引、非空属性值和逐字节截断回滚。效果成员、实际 MXD
及 ArcMap 10.1、QGIS 人工验证仍待完成。

### BasicMarkerSymbol 与 MarkerPlacementOnPoint

同版 DisplayRepresentation.dll 的 BasicMarkerSymbol Load
`0x6ee38f62–0x6ee38f6c` 仅在版本 ≥2 时读取布置名称，随后
`0x6ee38fa2` 调用 ExtraSafeReadObject，IID
`43A0A7C4-B01C-491A-9481-BF9EE1584D42` 在同版 esriDisplay
类型库中定名为 IMarkerPlacement。Load 在 `0x6ee39030` 读取尾 int32。

MarkerPlacementOnPoint 的构造指令 `0x6ee2d475` 将
CLSID `160C7BFF-2237-4A67-B163-89C223DB0A1D` 传给内联初始化，
`0x6ee2d4bb` 在完整对象 `+0x30` 安装虚表 `0x6ee97334`。
`0x6ee2d641–0x6ee2d6e4` 的 QI 确认该偏移对应 IPersistStream。
Load `0x6ee2ff40` 通过 IAT `0x6ee94184` 委派至
DisplayLib 的 GraphicAttributeList::Load，复用同一属性表布局。

读取器接入无效果成员的 BasicMarkerSymbol，支持空布置槽、
MarkerPlacementOnPoint 新对象及其同类键回引。ExtraSafe 有界对象必须
读到声明终点才提交键。合成测试覆盖 v1/v2、名称、空槽、
新对象、回引及声明长度偏小或偏大时的回滚。其余布置类、非空效果、
实际 MXD 以及 ArcMap 10.1、QGIS 人工验证仍待完成。

### 共用属性表的 11 种标记布置类

从 10.1 Desktop 类型库枚举声明 IMarkerPlacement 的 11 个 coclass。
在 DisplayRepresentation.dll 10.1.0.3035 中，逐项核对传入
`0x6ee2fa20` 的 CLSID 与构造后完整对象 `+0x30` 的虚表。
下列虚表的 GetClassID、Load、Save 槽均指向
`0x6ee2fee0`、`0x6ee2ff40`、`0x6ee26520`。
Load 和 Save 直接委派给 GraphicAttributeList 的读写入口。

| MarkerPlacement 类后缀 | 构造调用点 | 持久化虚表 |
| --- | --- | --- |
| AlongLine | `0x6ee27ea4` | `0x6ee96b14` |
| Decoration | `0x6ee29843` | `0x6ee96d0c` |
| InsidePolygon | `0x6ee2a360` | `0x6ee96eec` |
| AtExtremities | `0x6ee2c044` | `0x6ee9709c` |
| OnLine | `0x6ee2ca64` | `0x6ee971f4` |
| OnPoint | `0x6ee2d48f` | `0x6ee97334` |
| OnVertices | `0x6ee2dc66` | `0x6ee974f4` |
| PolygonCenter | `0x6ee2ecc4` | `0x6ee9762c` |
| RandomAlongLine | `0x6ee30144` | `0x6ee978b4` |
| RandomInPolygon | `0x6ee31d94` | `0x6ee97a44` |
| VariableAlongLine | `0x6ee32ae0` | `0x6ee97c44` |

解析器在 BasicMarkerSymbol 的 ExtraSafe 槽接入全部 11 个 CLSID，
保留布置对象的属性 ID 和 Variant，并支持同类键回引。构造测试逐类
验证新对象、回引及声明范围不符时的回滚。未公开布置类仍需原生证据。
尚无真实 MXD 样本及 ArcMap 10.1、QGIS 人工验证。


### RepresentationRule 与符号层的非空几何效果

DisplayRepresentation.dll 10.1.0.3035 的效果列表 Load
`0x6ee4c300` 与 Save `0x6ee4c4d0` 按有符号 short 版本及 int32
计数分派。v2 起每项先通过 AfCore String 读写名称，所有版本随后通过
ExtraSafe 读写 IGeometricEffect。读取器保留原始空槽，旧版空槽由原生
在运行时实例化默认效果的行为留待人工核对。

同版 72 份类型库列出 22 种专用几何效果类，以及 RepresentationRule 和
三个符号层类。以下为 22 种专用类的构造 CLSID 引用及持久化虚表。
它们的 Load / Save 一律为 `0x6ee2ff40 / 0x6ee26520`，直接委派给
GraphicAttributeList。

| GeometricEffect 类后缀 | 构造 CLSID 引用 | 持久化虚表 |
| --- | --- | --- |
| Radial | `0x6ee1c8d7` | `0x6ee95bbc` |
| Dash | `0x6ee149c8` | `0x6ee94e14` |
| AddControlPoints | `0x6ee110c7` | `0x6ee94714` |
| Offset | `0x6ee1b0c8` | `0x6ee95924` |
| Buffer | `0x6ee139b7` | `0x6ee94af4` |
| Reverse | `0x6ee1e097` | `0x6ee95e94` |
| Cut | `0x6ee13fb7` | `0x6ee94c84` |
| Arrow | `0x6ee117fb` | `0x6ee948ec` |
| Jog | `0x6ee1995a` | `0x6ee95674` |
| Donut | `0x6ee16e38` | `0x6ee9517c` |
| EnclosingPolygon | `0x6ee18378` | `0x6ee9530c` |
| Simplify | `0x6ee1f7a7` | `0x6ee96274` |
| Smooth | `0x6ee1fd47` | `0x6ee963dc` |
| RegularPolygon | `0x6ee1d26a` | `0x6ee95d2c` |
| Wave | `0x6ee247cb` | `0x6ee9687c` |
| TaperedPolygon | `0x6ee235da` | `0x6ee9672c` |
| Move | `0x6ee1a75a` | `0x6ee957c4` |
| Rotate | `0x6ee1e59a` | `0x6ee95fb4` |
| Scale | `0x6ee1ef0a` | `0x6ee96124` |
| Extension | `0x6ee18ac8` | `0x6ee954cc` |
| OffsetTangent | `0x6ee1becb` | `0x6ee95a74` |
| Suppress | `0x6ee22517` | `0x6ee9658c` |

规则及三种符号层共用该效果列表读取器。逐类构造测试覆盖 v2 名称、
v1 无名称、规则与符号层之间的键回引，以及 ExtraSafe 边界回滚。
效果槽内其他类体和属性组合仍待核查。尚无真实 MXD 或目标软件人工验证。
