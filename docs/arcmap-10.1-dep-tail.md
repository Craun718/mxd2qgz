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
  ④ 版本 <2：int32（缺省 −1）→ ⑤ int32（this+0xc）→ ⑥ 规则块 20 字节
  （`0x7fff5e90`：int32+short+short+int32+short+short+int32，写侧
  `0x7fff5f50` 对称）→ ⑦ int16 计数（≤0 不进循环）→ ⑧ 计数 × [int32 +
  规则块]（每条 24 字节）。
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
