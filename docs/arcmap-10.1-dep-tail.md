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

经 objdump 复核 SystemCore.dll 10.1.0.3035（SHA-256
`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`），
Time、TimeReference、TimeExtent、TimeZoneInfo 分别在 `0x7ffe17aa`、
`0x7ffebc26`、`0x7ffe8226`、`0x7fff77c8` 使用有符号上限跳转。
TimeExtent 在 `0x7ffe82ba` 对版本 ≤1 跳过尾字节；时区体在
`0x7fff603b`、`0x7fff6094` 对版本 ≥2 跳过旧偏移字段。
构造时区用例覆盖 v1、-1 和最小有符号值的旧字段与体末尾。
负版本尚无实档或人工验证。

- **TimeReference::Load**：① int16（`1 < (short)ver` 拒绝）→ ② LoadObject
  (ITimeZoneInfo `{0F8FD6AB-CE6D-4BB2-A5EF-2F6035E304C5}`，`0x80073924`) →
  ③ short（this+0xc）→ ④ short（this+0xe）。
  同版 ObjectStream 在 `0x7ffd0063` 查键后对原实例请求 ITimeZoneInfo。
  Rust 按登记类校验回引。构造测试验证合法回引读至体末尾，误引 Time 时
  停在时区槽首并保留原有键。该回引尚无实档或人工验证。
- **TimeExtent::Load**：① int16（`2 < ver` 拒绝）→ ② LoadObject(ITime
  `{66810D21-8DE0-44EE-B26E-465AC09F161F}`，`0x800730fc`；起点）→ ③ 同 IID
  （终点）→ ④ LoadObject(ITimeReference `{F6CBA4A3}`，`0x8007361c`) →
  ⑤ 版本 >1：1 字节（this+0x14；Save 恒写）。第 ⑤ 字节与 FL 版本 0x1c..0x1f
  分支的 `{BD724B95}` 槽（map-object IID 表）非同一层——该分支请求的
  ITimeExtent 实现类即本类。
  SystemCore 10.1.0.3035 的 ObjectStream 在 `0x7ffd0063` 按键定位实例后
  请求相应接口。TimeExtent 读取器对三个槽按登记类校验回引。构造用例
  分别将起点和终点误引为 TimeReference、将时间参考误引为 Time，确认停在
  对应槽首且保留原有键。尚无该组合的实档或人工验证。
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

三维容器的 IEnvelope 回引沿用 SystemCore.dll 10.1.0.3035
`0x7ffd0063` 的查键和接口查询路径。Scene `0x7d2a4b16` 与
Globe `0x72f60153` 请求 IEnvelope，因此在共享表已记录类时
采用登记类校验和呈现回引。构造测试覆盖两个容器的非空回引、
后继边界及登记类错误时的事务回滚。真实三维宿主及 ArcMap/QGIS
人工验收仍待验证。

GlobeLayers.dll 10.1.0.3035 的 `0x72f60729` 和 `0x72f6075a`
对 v6、v7 增补的两槽均调用 LoadObject，请求同一个 IGeometry IID。
负 HRESULT 分别从 `0x72f60738` 和 `0x72f60769` 退出。按
SystemCore.dll 的登记键接口查询路径，读取器以已登记类校验该回引，
支持与几何新体相同的七个已确认类。构造测试覆盖 Point 的登记类
覆盖回引头、非几何登记类的事务回滚，以及容器后继字节位置。
真实三维宿主仍待人工验收。

Scene.dll 的 `0x7d2a4c0d` 和 GlobeLayers.dll 的 `0x72f60819`
经各自 IAT 调用 LayerLib.dll 10.1.0.3035 的
`ExtensionHelper::LoadExtensions`（`0x786c22e0`）。
[同一 helper 的扩展边界](mxd-terra-round9-table.md#独立表的未知扩展)
表明无法创建实例时，原生按声明体长跨过该项。两类容器复用扩展
列表的长度和预算校验，并保留未知类的原始体；已识别类仍需完整
读取。构造用例验证非空未知扩展、后继字段及截断回滚。原始体未
产生引用键，第三方扩展可实例化时的内部对象及人工验收仍待验证。

### LELP 容器和关联图层接线

Scene.dll 的 `0x7d2a4af4` 和 GlobeLayers.dll 的 `0x72f60131` 都将同一个 `ILayer {34C20002-4D3C-11D0-92D8-00805F7C28B0}` 传给普通 `LoadObject`。两处 Save 分别以 `this+0x20` 和 `this+0x34` 的对象调用 SaveObject。普通对象槽没有可跳过的外层长度；关联槽仅在已有读取器证实读完 FeatureLayer、CompositeGraphicsLayer、FDOGraphicsLayer、GraphicsLayer3D 或 GlobeGraphicsLayer 的对象体后提交。其余图层的普通对象体或子槽仍待接入。两类 3D 图层也在 Map 图层列表按标志 1 或 ExtraSafe 分派，ExtraSafe 路径在条目范围内试读并检查完整消费。

两处关联槽的回引走 SystemCore.dll 10.1.0.3035 的
`0x7ffd0063` 按键查询 ILayer。解析器以登记类识别回引，复用
逐层遮罩取证已确认的 ILayer 类集合，其中包括缺少遮罩接口的类。
构造测试覆盖两个宿主、头部类与登记类不一致、非图层登记类的回滚
以及后继字节位置。第三方类和真实三维宿主仍待验证。

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

## Callout 和文字背景的版本边界

同摘要 DisplayCore.dll 10.1.0.3035 中，BalloonCallout、LineCallout、
SimpleLineCallout 和 MarkerTextBackground 的 Load 分别在
`0x6eb4198a`、`0x6eb5900a`、`0x6eb825b1` 和 `0x6eb643b3`
用有符号 `jg` 或 `jle` 检查版本上限 1。负版本与版本 1 复用
相同字段序。Rust 保留负版本位模式，合成输入验证三类 Callout 的
完整体及 MarkerTextBackground 的嵌套停止点。

该取证针对这些类的 IPersistStream::Load。负版本实档与 ArcMap/QGIS
人工验收待补。

## Callout 和文字背景的 ISymbol 接口核对

复核同摘要 DisplayCore.dll 10.1.0.3035 的完整对象虚表、Ghidra
导出和 objdump 指令。BalloonCallout 的构造指令 `0x6eb40b1f`
安装主虚表 `0x6ec0cc8c`，SimpleLineCallout 的 `0x6eb80dec`
安装 `0x6ec10adc`。两表首项分别为 `0x6eb40e80` 和
`0x6eb80fd0`。LineCallout 和 MarkerTextBackground 的主 QI
分别为 `0x6eb577f0` 和 `0x6eb63b70`。

逐一比较四个入口及公共回退 `0x6ebf6200 → 0x6ebf6b50` 的
IID 常量原始 16 字节。没有一项匹配 ISymbol
`{F3435802-5779-11D0-98BF-00805F7CED21}`（`0x6ec0ca2c`
及 `0x6ec0cb6c`），未匹配时在 `0x6ebf6c42` 返回
E_NOINTERFACE。FeatureLayer 后继段 ㉖ 的候选据此排除这四类。
对象体内请求 ISymbol 的嵌套槽仍按各自持久化字段解析。其他候选
仍待核对，本次未进行 ArcMap/QGIS 人工验收。

## FeatureLayer 后继段的符号回引

FeatureLayer.dll 10.1.0.3035 在后继段 ㉖ 请求 ISymbol。
SystemCore.dll 同版的 ObjectStream 在 `0x7ffd0063` 按键定位登记
实例并查询请求接口。读取器用登记类校验此处的回引并保留该类。
登记类超出已核对的范围时停在槽起点。仅有键的表项沿用回引头的类标识，
因为该表项没有记录原实例类。

构造测试使回引头与已登记的 SimpleMarkerSymbol 或 RgbColor
不一致。前者读完此段，后者在 ㉖ 停止并保留既有键。更多 ISymbol
实现、对应实档及 ArcMap/QGIS 人工验收仍待核对。

### 新对象的接口限制

同摘要 DisplayCore.dll 中，Template 和 LineDecoration 的构造指令
`0x6eb86b7a`、`0x6eb5c130` 分别安装主表 `0x6ec110b4`、
`0x6ec0e468`。首项 QI `0x6eb86e30`、`0x6eb5c230`
分别匹配 ITemplate（`0x6ec0e0e8`）、ILineDecoration
（`0x6ec0e0f8`），然后进入公共回退 `0x6ebf6200 → 0x6ebf6b50`。
该路径对 ISymbol 返回 E_NOINTERFACE。FeatureLayer ㉖ 在消费新对象头
之前也检查类是否属于已核对的 ISymbol 集合。合成测试验证两类都停在
对象头起点且不登记新键。

将同版反汇编中向 ECX 装入两个 ISymbol IID 地址的指令与 Ghidra 函数
入口、完整对象虚表和 RTTI 交叉核对，得到 22 个应答入口，均能对应
已接入的 DisplayCore 符号类。该检索覆盖直接装载这两个地址的指令，
其他寻址形式及其他模块仍待追查。ArcMap/QGIS 人工验收未进行。

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

同摘要 RasterLayer.dll 的 objdump 指令显示，ColorSymbol 在
`0x7c7134ac..0x7c7134b1`、ColorRampSymbol 在
`0x7c7120bc..0x7c7120c1` 用有符号 `jg` 检查版本上限。
ColorRampSymbol 在 `0x7c7121a0..0x7c712254` 也按有符号版本
分派尾段，负版本保留旧 short，并跳过 v3–v5 附加字段。Rust 在模型中
保留原始版本位模式，按有符号值选择布局。合成用例覆盖负版本、同键回引、
截断及后继边界。负版本实档及 ArcMap/QGIS 目视尚未验证。

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

同摘要 WMSLayer.dll 的 `0x80d65145..0x80d6514a` 用有符号
`jg` 检查上限，负版本继续读取相同的两个 String 和对象槽。
`0x80d65171..0x80d6517c` 明确请求 IPictureMarkerSymbol。
SystemCore 10.1.0.3035 在 `0x7ffd0063` 按键取登记实例并查询接口。
读取器保留原始版本位模式，以登记类检查嵌套回引。合成用例覆盖
负版本、截断、有效回引和指向 SimpleMarkerSymbol 的错误回引。
真实工程及 ArcMap/QGIS 人工验收仍待补充。

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

### LinePattern 与 GradientPattern

10.1 Desktop 类型库声明三种 IFillPattern。除 SolidColorPattern 外，
复核同版 DisplayRepresentation.dll 的以下两个持久化虚表：

| 类 | CLSID | GetClassID | IPersistStream 虚表 |
| --- | --- | --- | --- |
| LinePattern | `444F0F52-DABF-4639-B239-9A88B3C3F830` | `0x6ee60d10` | `0x6ee99ffc` |
| GradientPattern | `00F11C2D-FCBB-4B7F-8135-E7AF481CE592` | `0x6ee4d730` | `0x6ee98b24` |

两表的 Load `0x6ee4d770` 和 Save `0x6ee60d70` 直接委派给
DisplayLib 的 GraphicAttributeList。两类已接入 BasicFillSymbol 的
IFillPattern 槽，沿用图形属性和共享键读取。构造测试覆盖三种图案的
非空属性、回引及截断回滚。实档上的属性对象组合和 ArcMap 10.1、
QGIS 人工验证仍待完成。

### Variant 中的图形属性对象

DisplayLib.dll 10.1.0.3035 的 GraphicAttributeList::Load
`0x6ed896e0` 对每项调用 AfCore Variant::Read(stream, true)。
该调用的对象值分支在读取保留区后复用宿主 ObjectStream，详见
[Variant 对象值布局](arcmap-10.1-variant-propertyset-slots.md#带保留区的-objectstream-对象值)。
上述图案、线式、标记布置和几何效果类的 IPersistStream 已分别核对，
其对象头之后直接进入 GraphicAttributeList。

共用 Variant 分派现在可读取这些对象，保留类标识、键、对象体及其原始
字节。递归读取继续消耗同一个 Variant 深度预算。构造测试覆盖嵌套线式、
后续键回引、逐字节截断和深度越界时的回滚。实档中哪些属性 ID 会保存
各类对象仍待核查，尚无 ArcMap 10.1 或 QGIS 人工验证。

### GradientPattern 的颜色属性

同版 DisplayRepresentation.dll 的 `0x6ee4d7f8` 和 `0x6ee4d869`
分别以属性 ID 0、1 调用 GraphicAttributeList::GetValueAsObject。
两处结果均交给 `0x6ee4e420`；该助手在 `0x6ee4e459` 推入
`0x6ee9874c` 的 IColor IID `20CD40B0-8F3D-11D0-8590-0800091A2A72`。
因此这两个属性可以承载颜色对象。构造用例通过 SymbolMimic 和
RepresentationRule 的正常分派读取带两个颜色属性的 GradientPattern，
验证 RgbColor 新对象、同键回引、后续符号层对位与截断回滚。

附带检查了本地参考、安装介质和 SDK 语料中可打开的 186 个 MXD/MXT
容器的可读流。SymbolMimic、RepresentationRule 和三种 BasicSymbol
的 CLSID 字节均未命中。该检查仅用于寻找可验证对象。
尚无实档或 ArcMap 10.1、QGIS 人工验证。

### 效果槽中的嵌套规则与符号层

DisplayRepresentation.dll 10.1.0.3035 的效果列表在
`0x6ee4c3d5–0x6ee4c3e2` 将 `0x6ee9478c` 的 IGeometricEffect IID
交给 ExtraSafeReadObject。除专用效果类外，RepresentationRule、
BasicFillSymbol、BasicLineSymbol 和 BasicMarkerSymbol 的 QueryInterface
也比较并接受该 IID，对应指令分别在 `0x6ee73b15`、`0x6ee34f9f`、
`0x6ee3654f`、`0x6ee3766f`。其持久化体沿用本页已确认的布局。

效果槽现可解析上述嵌套对象和回引。首次定义消耗同一符号递归预算，
ExtraSafe 声明体内必须恰好读完，错误时回滚整个符号及其新键。
构造测试覆盖规则、标记符号层、后续规则回引和深度不足的回滚。
其他属性对象组合、实际 MXD 和目标软件人工验证仍待完成。

### 表示规则槽的跨模块候选

复查 ArcGIS Desktop 10.1 安装介质中的 72 个可解析类型库，按接口
继承追踪 `IRepresentationRule`，声明该接口的 coclass 为
DisplayRepresentation.dll 的 RepresentationRule。Desktop 的
`esri.clsid.xml` 还注册了以下两个名称相近的类。按 CLSID、原生
GetClassID 和 QueryInterface 复核了安装介质中的同版 PE：

| 类 | 模块，文件版本 10.1.0.3035 | CLSID | GetClassID | QueryInterface |
| --- | --- | --- | --- | --- |
| BGLRepresentationRule | BGLSymbols.dll | `F506F6BF-05B2-4D27-9AB1-6CBAC3961F60` | `0x6bf1b663` | `0x6bf1b880` |
| MERepresentationRule | MappingCore.dll | `44D3082B-E2EB-44DD-B0BE-A664DB75FCC9` | `0x799575b3` | `0x79956fb0` |

objdump 指令与 PE 数据常量显示，两者均接受 IPersistStream。两个类
入口及其通用回退（分别为 `0x6bf4d2e0` 和 `0x79a0d0b0`）均
未将 SymbolMimic 请求的 IID `2B7BE3AB-BB72-49D0-A6EE-63ACD1F9068C`
列为可返回的接口，对未识别 IID 返回 E_NOINTERFACE。因此该槽的
解析保持现有分派。证据范围限于这两个候选和上述类型库，未覆盖
第三方注册组件，也未进行 ArcMap 10.1 或 QGIS 人工验证。

### 三维图形容器中的矩形元素

10.1 esriCarto 类型库的 RectangleElement
`3A9767C2-F253-11D0-83A4-080009B996CC` 声明两类容器所请求的
IElement。图形容器的普通元素槽现复用 GraphicElements.dll
`0x7505a400` 对应的矩形读取器，共用宿主对象键。
[原生矩形体](mxd-page-layout-surround-20261118.md)记录了矩形几何、
元素属性及填充符号的顺序。构造 Scene 容器验证首次定义、同键回引、
后继字节和截断回滚，尚未在 ArcMap 或 QGIS 人工验证。

### 三维图形容器中的组元素

10.1 esriCarto 类型库声明 GroupElement
`803577D2-F8A3-11D0-83AF-080009B996CC` 实现两类容器请求的
IElement。两类容器复用 GraphicElements.dll `0x75035ef0` 的
[组元素布局](mxd-page-layout-tail-20261118.md)，沿用容器对象键表
和组递归上限。组内 MapFrame 复用已有 IMap 槽读取与回引校验，
从进入容器时共享表已标记的 Map 键确认目标。构造 Scene 用例覆盖
已确认 Map 键、只有普通登记的同键、失败回滚，以及组内和容器列表内
的元素回引。容器中新出现的 Map 对象与 ArcMap/QGIS 人工验证待补。

### 三维图形容器中的独立地图框

10.1 esriCarto 类型库声明 MapFrame
`DD94D770-836D-11D0-87EC-080009EC732A` 实现 IElement。三维容器的
普通元素槽复用 MapElements.dll 的地图框布局，见
[MapFrame 原生取证](mxd-page-layout-tail-20261118.md)。IMap 槽使用
进入容器时已确认的 Map 键。构造 Scene 用例让组内地图框与容器
列表中的独立地图框引用同一个 Map，验证各自的新键和容器终点。
首次定义的内嵌 Map 与目标软件人工验证仍待补。

### 三维图形容器中的独立装饰框

10.1 esriCarto 类型库声明 MapSurroundFrame
`83FFCAE1-EDCA-11D0-8683-0000F8751720` 实现 IElement。
两个三维容器的元素槽复用 MapElements.dll 10.1.0.3035
`0x7957b870` 对应的[装饰框读取器](mxd-page-layout-surround-20261118.md)。
构造 Scene 用例覆盖 v0–v3 的装饰框、同键回引及容器终点。
ArcMap/QGIS 实档目视尚未进行。

### 三维图形容器中的圆和椭圆元素

GraphicElements.dll 10.1.0.3035 的 SHA-256 为
`f24451c8bf25cce7c1906ea67e5fa3ce351b8f9918c6ddcd2a40083151f77ae9`。
PE RTTI 和 objdump 对位 CircleElement 的持久化虚表
`0x7506e9ac`，以及 EllipseElement 的 `0x7506eebc`。
GetClassID `0x75031010 / 0x75032e20` 分别返回
`974111DB-C5D2-11D2-9F28-00C04F6BC8DD` 和
`4EDA1081-12EA-11D3-9F8F-00C04F6BC8DD`。
10.1 的 Carto 互操作程序集同样登记这两个 CLSID，并为两类声明 IElement。
两者共享 Load `0x75033760`，在 `0x75033795` 按有符号
short 拒绝大于 1 的版本，接着依次读 RectShapeElement、
ElementProperties 和 IFillSymbol（`0x750337a3 / 0x750337ae /
0x750337e1`）。两个独立 Save `0x75031990 / 0x75033820`
采用同一字段顺序。矩形的独立 Load 在 `0x7505a435` 也用有符号
上限比较，三类复用相同字段读取器。

Scene 和 Globe 容器保留圆与椭圆体及原对象类标识。构造用例
覆盖两种容器、两种元素、回引、负版本、越界版本、后继字节
与截断回滚。其他宿主的覆盖和 ArcMap/QGIS 人工验收待验证。

### 三维和组元素中的 FrameElement

ArcMap 10.1 的 Carto 互操作程序集把 FrameElement
`E01BA2C5-24B2-11D3-B8AA-00600802E603` 列为 IElement。
同版 Layer.dll 持久化虚表 `0x7866df48` 的
Load/Save `0x7865b9c0 / 0x7865b9d0` 直接委派给
LayerLib 的 BaseFrameElement，见[原生边界](mxd-page-layout-surround-20261118.md)。
现在通用 IElement 分派沿用这个读取器。三维容器和布局内组
可读取独立 FrameElement，布局顶层也使用同一分派并登记实际 CLSID。
构造 Scene 用例验证 v2 阴影槽、登记类、回引、后继字节与
逐字节截断回滚。ArcMap/QGIS 实档目视待验证。

### 三维图形容器中的 MultiPatchElement

同版 GraphicElements.dll 的 PE RTTI `0x750812b4` 对应持久化虚表
`0x75071a48`。GetClassID `0x75052ee0` 写出
`E91AE5C9-2C16-11D4-80E2-00C04FA0ADF8`，与 10.1 Carto 互操作
程序集的 IElement 类标识一致。objdump 复核 Load `0x75052f50`，
按有符号 short 拒绝大于 1 的版本，随后读 ElementProperties、
IFillSymbol 和 IGeometry（`0x75052f93 / 0x75052fc0 / 0x75052ff0`）。
最后于 `0x75052fff / 0x75053021` 读 int32 和 short。
Save `0x75053050` 以相同顺序写出，并在 `0x7506e7c8` 取版本 1。

通用元素槽复用现有属性、填充符号和几何对象读取器。构造 Scene
验证非空的 MultiPatch 几何对象、共享键登记、负版本、原始尾段、
后继字节和逐字节截断回滚。尾段业务含义及 ArcMap/QGIS 实档目视待验证。

### 三维图形容器中的 Text3DElement

同版 GraphicElements.dll 的 PE RTTI `0x75081610` 及持久化虚表
`0x75072f70` 对应 GetClassID `0x750606a0`、Load
`0x75061e80`、Save `0x750620b0`。从 `0x75071128` 返回的
CLSID `B9865D0B-EB37-11D4-A116-00508BD60CB9` 与 10.1 Carto
互操作程序集中的 IElement 类一致。Load 的有符号 short 版本上限
为 1，随后读 ElementProperties、IFillSymbol、IGeometry，前缀与
MultiPatchElement 相同。objdump 核对后续指令如下。

| 调用地址 | 字段 |
| --- | --- |
| `0x75061f30 / 0x75061f50` | int32、short |
| `0x75061f61 / 0x75061f73` | 两个 AfCore 成员 String |
| `0x75061f8d / 0x75061fa6 / 0x75061fc7` | 两个 short、IPoint |
| `0x75061fea / 0x75062003` | 两个 double |
| `0x75062022 / 0x75062037 / 0x7506204c / 0x75062061` | 四个 int32 |
| `0x7506207c / 0x75062091` | 两个 double |

Save 的版本表 `0x7506e7b0` 为 `[-1,1]`，写出时顺序一致。
构造 Scene 验证字符串、标量位模式、几何与 IPoint 共用对象键、
错误登记类和逐字节截断回滚。标量业务意义及 ArcMap/QGIS
实档目视待验证。

### 三维图形容器中的 PMFTitleTextElement

同版 GraphicElements.dll 的 RTTI `0x75081348` 对应持久化虚表
`0x750720b0`。GetClassID 经 `0x75056e00 → 0x750568b0` 从
`0x75071138` 复制类标识 `1F44BE49-3113-4D58-B1CE-C326328569ED`，
与 10.1 Carto 互操作程序集的 IElement 类一致。构造器在
`0x75056683` 创建 `0x75071f90` 所指的 TextElement，并于
`0x7505669c` 查询其 IPersistStream。Load 经
`0x75056dd0 → 0x75056910` 调内嵌持久化接口 +0x14。
Save 经 `0x75056d70 → 0x75056930` 调同一接口 +0x18。
委派前后无额外流字段或第二个对象头。通用 IElement 读取器据此保留
PMF 包装类，并复用 TextElement 的体读取器。

构造 Scene 验证文本、外层对象键、同类回引、后继字节及逐字节
截断回滚。ArcMap/QGIS 实档目视待验证。

### SimpleLine3DSymbol

从 10.1 安装介质提取的 _3DSymbols.dll 为 PE32，版本资源
10.1.0.3035，SHA-256 为
`8d966587dbcc58c11d51f03eea25c44a91e8d23b46313a4b71f2a39cc775601a`。
SimpleLine3DSymbol 的 RTTI 主表 `0x68061ef4` 指向 QI
`0x68045300`，分别在 `0x6804534f` 和 `0x68045395` 比较
ILineSymbol 与 ISymbol。回退 `0x68057870` 接受 IPersistStream，
其持久化虚表 `0x680620b4` 由构造器 `0x68044e44` 安装。
GetClassID 经 `0x68049f30` 进入 `0x68045cf0`，返回
`{470B7275-3552-11D6-A12D-00508BD60CB9}`。

Load 经 `0x68049f00` 进入 `0x680459f0`，先读有符号版本
（上限 1），再请求 IColor（IID 常量 `0x6805fdcc`）。
依据 AfCore IAT 名称和调用实参，随后按 double、int32、double、
int32、LONG 读取，位置依次为 `0x68045a8f`、`0x68045ac2`、
`0x68045ae1`、`0x68045b6a`、`0x68045b85`。标量暂按原序保留。
合成输入验证非空颜色、结束偏移和逐字节截断回滚。其他 3D 符号
实现、实档和 ArcMap/QGIS 人工验收仍待核对。

### TextureLineSymbol 与嵌套填充

同摘要 _3DSymbols.dll 的主 QI `0x68053c00` 比较 ILineSymbol
（`0x68053c4f`）与 ISymbol（`0x68053c95`），并通过
`0x68053dca` 进入 IPersistStream 公共回退。持久化虚表
`0x68062a74` 的 GetClassID 经 `0x680575e0 → 0x68053610`
返回 `{B5710C9C-A9BC-4A16-B578-54BE176ED57B}`。Load 经
`0x680575b0 → 0x680563d0` 读取有符号版本（上限 2）、
一个 double 和 IID `{3F6D8301-5A5A-4B84-969F-433AF07EE1A3}`
的对象槽。版本大于 1 时在 `0x68056469` 多读一字节。

TextureFillSymbol 的主 QI `0x6804f5c0` 接受该 IID。
其持久化表 `0x680627d4` 的 GetClassID 经
`0x68053090 → 0x6804eed0` 返回
`{8D738780-C069-42E0-9DFA-2B7B61707BA9}`。
Load `0x680506f0` 首先读有符号版本，再请求 IID
`{E9F95F61-2BEC-11D6-A4CD-444553547777}` 的对象，之后还有
两个 IColor、ISymbol 和版本相关标量。该首槽由 GeometryMaterial 主 QI `0x68027530` 在 `0x68027537`
匹配。IPersistStream 虚表 `0x680603ac` 的 GetClassID
`0x68027d30` 返回 `{0E6F4B27-2BD0-11D6-A4CC-444553547777}`。
Load `0x68027d80` 在 `0x68027dc4..0x68027dc9` 将版本减一
再比较无符号上限，接受 1 和 2。随后消费三个 int32、四个 double
与两个 RGB 颜色接口槽（`0x6806043c`）。三个尺寸均为正时，v1
在 `0x68027ef0` 相乘得到像元字节数，v2 在 `0x68027edc`
读取声明长度。Rust 检查乘法与长度上限后保留原始字节。

纹理线可读含新建 GeometryMaterial 的 TextureFillSymbol。合成输入
覆盖两种像元长度、跨槽颜色回引、后继字节及逐字节截断回滚。
未知材质或嵌套对象仍保留停止位置。其他材质实现、实档和 ArcMap/QGIS
人工验收仍待核对。

#### 10.1 介质中的材质接口调用方

逐文件检索所提取 Desktop DLL 中材质 IID
`{E9F95F61-2BEC-11D6-A4CD-444553547777}` 的完整字节，
命中 _3DSymbols、VRML3DFile 和 _3DSymbolsUI。核对命中
地址的调用指令及 PE 导入表，三者版本均为 10.1.0.3035。

- _3DSymbols.dll 在 `0x68027537` 直接比较 IID，归属
  GeometryMaterial 的 QI。相同常量处的其他直接引用是接口请求。
- VRML3DFile.dll（SHA-256
  `e30601f639da31a4203a85b70e668611a7da9f34974b8427a3e14a05d8ef79a9`）
  在 `0x80c88b0b` 通过 AfCreateInstance 创建 GeometryMaterial
  CLSID，调用 OleRun 后在 `0x80c88b38` 对该实例请求材质接口。
- _3DSymbolsUI.dll（SHA-256
  `fd587aca4ce9981a2fd7da197ef70886b4b2a8f0d98f13310e06fcd0c14ad0fb`）
  在 `0x1001d9e9` 以调用方传入的 CLSID 创建对象，
  `0x1001da22` 请求材质接口。另一分支 `0x1001da4c`
  把同一 IID 直接交给 AfCreateInstance。界面还会在
  `0x1001de23` 查询外部传入的实例。

因此，上述介质的显式 IID 命中中，GeometryMaterial 是已确认实现。
经外部 CLSID 或其他间接方式提供的实现仍需样本或运行取证。
此次为静态核对，未进行 ArcMap/QGIS 人工验收。

补充核对 Desktop 提取目录的 72 份 MSFT 类型库。逐个解析接口、
继承关系与 coclass 引用表，从 IGeometryMaterial IID 出发查找直接
及经派生接口声明的 coclass。唯一命中是 esri3DAnalyst 的
GeometryMaterial，CLSID `0E6F4B27-2BD0-11D6-A4CC-444553547777`。
对同目录 1203 个 MZ 容器开展 IID 搜索，类型库以外的命中仍是
上列三个 DLL。其他类型库的相同字节属于引用，类型关系核对未发现
额外 coclass。现有读取器覆盖介质中有明示证据的实现。经外部 CLSID
加载的实现或真实 MXD 中的未知材质仍需宿主证据。

### 3D 标记符号的持久化入口

同摘要 _3DSymbols.dll 中，三个已定位的类工厂产品均从各自 QI
委派到 Marker3Dbase 的 `0x6802c940`。委派指令分别位于
`0x680214f6`、`0x68032282`、`0x6804a406`。
基类在 `0x6802c9e9` 与 ISymbol IID 比较，未匹配的查询
经 `0x6802caea` 进入 IPersistStream 公共回退 `0x68057870`。

从构造器的虚基接口表写入点取得持久化表，再按表中 GetClassID
委派和 CLSID 常量对位。下表地址均为 PE32 首选虚拟地址。

| 类 | 持久化虚表 | CLSID | Load 委派与目标 | 版本上限 |
| --- | --- | --- | --- | --- |
| CharacterMarker3DSymbol | `0x6805fce8` | `6E8EC8F7-E90A-11D5-A129-00508BD60CB9` | `0x68024b00 → 0x68022e70` | 4 |
| Marker3DSymbol | `0x6806142c` | `773F7274-AEFB-11D5-8112-00C04FA0ADF8` | `0x680363a0 → 0x68033970` | 10 |
| SimpleMarker3DSymbol | `0x68062444` | `773F7270-AEFB-11D5-8112-00C04FA0ADF8` | `0x6804e900 → 0x6804d840` | 6 |

三种 Load 各自读取版本及后续字段。Marker3Dbase 的
`0x6802ce60` 虽也有读取版本和对象槽的实现，其版本上限为 3。
应从上表三个实际派发入口确定布局。派生类的非空子槽和后继版本分支
仍待逐项对位，未进行 ArcMap/QGIS 人工验收。

#### SimpleMarker3DSymbol 的对象体

同版 Load `0x6804d840` 的版本门 `0x6804d886` 为有符号上限 6。
先读 IColor、double、int32。小于 4 时在 `0x6804d934`
读取 IGeometry（IID `0x6805fdac`）。随后依次读 double、
int32×2、double×3。大于 1 时增读一项 double，之后所有版本读
short。大于 2 时在 `0x6804da55` 读 24 字节向量，大于 4 时
在 `0x6804daa9..0x6804db08` 附加 double×4、short，
大于 5 时在 `0x6804db3e` 再读 short。字段暂按流序保存。

Rust 接入共享 ISymbol 分派并复用已有颜色和几何读取器，未知嵌套体
使当前新符号回滚。合成输入覆盖负版本及 0–6 的版本门、颜色、
对象终点与逐字节截断。旧版非空几何实档及 ArcMap/QGIS 人工验收待补。

#### CharacterMarker3DSymbol 的对象体

同版 Load `0x68022e70` 以有符号上限 4 检查版本。读 IColor、
int32×2、三组 double×3（中间组是 `0x68022faa` 的
24 字节块）。版本低于 3 时还读两个 double。随后
`0x68023084` 请求 IFontDisp，读 int32×2、short，
在 `0x68023117` 请求 IID `{7914E5E7-C892-11D0-8BB6-080009EE4E41}`，
DisplayCore 同版 CharacterMarkerSymbol 的 QI 在 `0x6eb9a447`
接受该 IID。版本大于 1 时附加一字节，大于 2 时附加 double×3
和 short，大于 3 时附加一个 short。

Rust 复用颜色、StdFont 和 CharacterMarkerSymbol 的对象体，携带
相同的对象键表，未知子槽回滚当前新符号。合成输入覆盖负版本和
v0–v4 的边界与逐字节截断，非空字体及字符标记实档与 ArcMap/QGIS
人工验收待补。

#### Marker3DSymbol 的对象体

同版 Load `0x68033970` 按有符号上限 10 判断版本。依次读取
IColor、double、IGeometry、int32、24 字节向量、short、
int32×2、double×3；版本大于 1 时加一个 double，随后共有
一个 short。自 v3 在 `0x68033c08` 请求 IPicture（IID
`0x6805f84c`），自 v4 读取 double×4 与 short。仅 v5/v6
在 `0x68033ce5..0x68033d1e` 读取两个 double 和一个 short。
v6、v8 分别新增一个 short，v9 新增三个 short。v10 在
`0x68033fd5` 调用 AfCore String::Read。

v9 还从特定几何实例恢复保存于点 ID 的字符串，见下节。
构造测试覆盖负版本及 v0–v10 的空几何布局、截断和未知几何回滚。
实档及 ArcMap/QGIS 人工验收待补。

#### Marker3DSymbol v9 兼容探测的进入条件

用同版 esriGeometry 类型库核对虚表成员。查询助手
`0x68035fa0` 请求 IPointIDAware，Load 在 `0x68033e3e`
取其 PointIDAware 属性。该值非零时，`0x68030d90 → 0x68031040`
请求 IPointCollection。Load 在 `0x68033e6f` 取 PointCount，
只在点数等于 10 时继续。随后 `0x68033ea0` 取索引 0 的点，
`0x68033eb1` 读 IPoint.ID，并与 `0x00b8342d` 比较。
该标记来自几何的点 ID。命中后 `0x68033ef0..0x68033f6a`
循环读取索引 1–8 的 IPoint.ID，从最高字节开始将每个字节展开为
UTF-16 码元，`0x68033f7a` 以零结尾构造 AfCore String。
`0x68031390` 只更新 COM 对象引用。v9 分支直到返回均未对
IStream 读字节，真正的 String::Read 由 v10 版本门控制。

Rust 在带键的 MultiPatch 成功解码后，将命中标记的字符串与对象键
一同登记。v9 根据对象键读取字符串，新对象和回引共用同一条路径。
构造测试覆盖空点集、带标记的十点 MultiPatch 和类标识故意不一致的
回引，确认后继字节在原处。真实迁移对象及 ArcMap/QGIS 人工验收待补。

### Texture.dll 与 Garcia.dll 的非持久化 ISymbol 候选

核对 Desktop 介质的 PE32 文件，版本均为 10.1.0.3035。以 RTTI
完整对象表定位主 QI，再逐项比较 QI 和回退所用的 IID 字节。

- Texture.dll（SHA-256
  `871812d15d133b0c3922169f9a6358c0abe6732a630fa798507ccdb97d17147a`）
  的 TextureSymbol、TextureSymbol2D 主表分别为 `0x802a53d4`、
  `0x802a5300`。共用 QI `0x802a1e10` 在
  `0x802a1e5b` 对 ISymbol 返回对象 +0x14。该子接口 QI 经
  `0x802a21a0 → 0x802a1420` 委派回主 QI。主入口和回退
  `0x802a23c0` 无 IPersistStream 或 IPersistVariant 匹配，
  未识别接口在 `0x802a24b2` 返回 E_NOINTERFACE。
- Garcia.dll（SHA-256
  `4515f337021841351bed37f3f5c8a6109492a05cbe1a0cb944cd20ff4358c2f1`）
  的 AgentSymbol、ZSnapAgentSymbol 主表分别为 `0x100435e4`、
  `0x10043698`。两者共用 QI `0x100011b0`，在
  `0x100011b7` 匹配 ISymbol。主入口及回退 `0x100397b0`
  所比较的 IID 中没有上述持久化接口，未匹配时在
  `0x100398a2` 返回 E_NOINTERFACE。

四类均从普通 ObjectStream 新对象候选中排除。Texture.dll 的这两类
与本章已接入的 _3DSymbols.dll 纹理线和填充使用不同的虚表和接口。
本次为静态取证，尚未进行 ArcMap/QGIS 人工验收。

### 栅格符号的 ROP2

RasterLayer.dll 10.1.0.3035 的 ColorSymbol 持久化表位于
对象 +0xbc，Load 在 `0x7c713502` 将颜色槽后的首个 int32
读到该接口 −0x64，即对象 +0x58。ColorRampSymbol 的持久化表
位于 +0xd4，`0x7c71210f` 读入相对 −0x7c 的同一成员。
两类的 ISymbol 表（`0x7c767d94`、`0x7c7679fc`）位于对象
+0x18，+0x1c 槽经 `0x7c711110` 委派 DisplayLib.dll
10.1.0.3035 的 Symbol::get_ROP2（`0x6ed95300`）。
`0x6ed95316` 读取 ISymbol 相对 +0x40，也对应对象 +0x58。
RasterRGBSymbol 委派同一个 ColorSymbol Load。Rust 保留三个原始
整数并提供首项的 ROP2 访问器，后两项的历史用途仍待核对。

对提取的介质 DLL 扩展主 QI 地址检索，允许 IID 地址出现在
候选函数体内的任意字节位置。相对仅识别向 ECX/EDX 载入 IID 的检索，
新增的命中为已接入的 RasterRGBSymbol。该遍历仍受 RTTI 主表、
扫描窗口和间接派发限制。ArcMap/QGIS 人工验收未进行。

### 栅格渐变符号的颜色效果

同摘要 RasterLayer.dll 10.1.0.3035 的 ColorRampSymbol Load
在 `0x7c7121ee`、`0x7c712205`、`0x7c71221c` 将 v3
增加的三个 short 分别写到持久化接口 −0x10、−0xe、−0xc，
对应完整对象 +0xc4、+0xc6、+0xc8。绘制路径
`0x7c711be2..0x7c711bfe` 零扩展这些值，按 x86 调用约定逆序压栈，
调用 DisplayLib 的 IColor 版 ApplyEffects（IAT `0x7c76720c`）。
同版 DisplayLib 在 `0x6ed775ad` 用首项调用 ApplyTransparency，
在 `0x6ed775bb` 用后两项调用 ApplyBrightnessContrast。因此
三个字段按流序为透明度、亮度和对比度。访问器按 ApplyEffects 的
short 参数类型返回有符号值，原模型保留位模式。
合成用例覆盖 v3 起的字段和更早版本的缺省状态。实档显示效果及
ArcMap/QGIS 人工验收待补。

### 栅格渐变符号的图例属性

同摘要 RasterLayer.dll 的 ColorRampSymbol QI 在
`0x7c711447..0x7c711469` 匹配的两个接口返回对象 +0x98，
使用虚表 `0x7c767abc`。对照 esriCarto 10.1 类型库，
IColorRampSymbol::Invert、LegendClassIndex 和
IColorRampSymbol2::Horizontal、ShowTics 的 getter 分别在
+0x28、+0x34、+0x44、+0x4c。对应入口
`0x7c711780`、`0x7c711970`、`0x7c7117d0`、`0x7c711820`
读取完整对象 +0xb8、+0xb4、+0xba、+0xbc。

持久化表位于 +0xd4，Load 的 `0x7c71218b`、
`0x7c7121c8`、`0x7c712287`、`0x7c71225c` 读入这些成员。
所以 v5 两项按流序为 ShowTics、Horizontal。Rust 保留原始值，
访问器按非零判断布尔属性，并保留旧版本 v5 字段缺省的状态。

颜色槽后的第二个 int32 读入完整对象 +0x60（ColorRampSymbol
`0x7c71212a`、ColorSymbol `0x7c713516`）。写侧从同一成员保存，
其业务语义仍待核对。第三个 int32 读入栈临时值，写侧分别在
`0x7c7123be`、`0x7c7135df` 填零，读取模型保留流内值。
实际图例效果尚未在 ArcMap/QGIS 中人工验收。

### 栅格渐变的边界刻度索引

同摘要 RasterLayer.dll 的 ColorRampSymbol 构造入口在
`0x7c71126b..0x7c711279` 初始化 Horizontal 为 0、ShowTics 为
−1、完整对象 +0xcc 的整数为 3。Load 自 v4 在 `0x7c71223a`
读入该整数。Draw `0x7c712028..0x7c71204a` 按 Horizontal
选择水平或垂直路径。水平路径 `0x7c711c64..0x7c711c8b`
在 Invert 非零时采用该整数减 2，否则采用 0；垂直路径
`0x7c711eb3..0x7c711eda` 使用相反条件。ShowTics 非零且
LegendClassIndex 等于所选索引时，调用 GDI 绘制边界线。

解析模型保留原始 v4 字段，并可按构造默认值计算该索引。
合成测试覆盖早期默认值、方向切换、反转和禁用刻度线。
字段的赋值入口见下文，实际图例视觉效果仍待 ArcMap/QGIS 验证。

#### 边界刻度字段的 SDK 实档取值

复核上文 SHA-256 对应的 10.1 SDK 工程 BloomfieldTownship.mxd。
从已通过 RasterStretchRenderer 图例组读取确认的三个对象终点
`0x5761`、`0x5bec`、`0x5fd1`，按 v5 尾段布局取末 8 字节。
三个对象头均为 ColorRampSymbol v5，键依次为 218、233、248。
三条记录的 v4 整数、ShowTics、Horizontal 均依次为
`3`、`0`、`0xffff`。此处确实保存了整数的构造默认值，
同时保存了关闭刻度和水平布局，因此这些符号绘制时不会采用
该整数画边界刻度。该实档无法区分整数的其他赋值来源。

同版 Save 在 `0x7c71248b..0x7c712490` 直接从持久化接口
−0x8 取值写流。该路径中没有查询图例组类别数。完整对象
+0xcc 的已确认来源为构造默认值、Load 的持久化值及下文的图例组赋值入口。本次为实档字节复核，
未在 ArcMap 或 QGIS 中目视核对图例。

#### 边界刻度整数的赋值入口

同版 esriCarto 类型库声明 IColorRampSymbol::put_LegendGroup
在虚表 +0x40，对应虚表 `0x7c767abc` 的 `0x7c7119f0`。
该入口将图例组对象保存到接口相对 +0x28。图例组非空时，
`0x7c711a28..0x7c711a30` 将接口相对 +0x34 的成员地址
传给 ILegendGroup 的 +0x24，即 ClassCount getter。符号接口
相对完整对象 +0x98，故该成员为 +0xcc，与 Load v4 和 Save
读写的成员吻合。图例组为空的分支直接返回，不重置该整数。
因此该字段保存的是图例类别数，运行时重新设置图例组可以更新它。

Rust 访问器从解析结果取得整数，旧版缺省为 3。测试覆盖保存值
及缺省值，ArcMap/QGIS 人工验收待补。

### DummyLayer 系列的关联槽边界

复核 [Layer.dll 10.1.0.3035](mxd-luna-deep-layer-masking-20261118.md#其余内置-ilayer-实现)
同摘要提取物。注册例程 `0x7866b7d0` 分别用 `0x7866f260`
和 `0x7866f270` 的 CLSID 建立 DummyLayer 和 DummyGraduatedMarkerLayer
工厂。两类对象的共用 QI `0x7865cf90` 在 `0x7865cfa9`
比较 ILayer IID，另有 `0x7865cfd5` 的接口分支。未命中时
委派 `0x786672b0`，比较 IUnknown 及另外两个接口 IID 后在
`0x786673a2` 返回 E_NOINTERFACE。检查两段比较所引用的 PE
数据，均无标准 IPersistStream 或 IPersistVariant IID。

普通关联 ILayer 槽的已登记实例回引与首次加载的要求不同。
在该 10.1 实现中，两类均可提供 ILayer，但其查询路径不给出
ObjectStream 首次加载对象体所用的两个持久化接口。因此无界关联槽
继续接受已登记的 ILayer 回引，首次出现的这两类仍返回不支持。
这项取证仅覆盖上述模块和接口路径，尚未在 ArcMap 中运行验证。

Scene 与 Globe 的构造测试覆盖两个类的已登记引用和首次出现。
回引按登记类识别并读取完整容器；新对象返回 Unsupported，容器
游标和引用表回滚。

### GdbRasterCatalogLayer 的持久化入口

复核同版 RasterLayer.dll 10.1.0.3035（SHA-256
`9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6`）。
主 QI 在 `0x7c716e8a` 比较 IPersistStream，命中后由
`0x7c71728a` 返回完整对象 `+0x50`。构造器 `0x7c7166d5`
在此安装虚表 `0x7c768d30`。其 GetClassID `0x7c71a200`
复制 `0x7c7691e0` 的 CLSID，Load/Save 为
`0x7c71a230` / `0x7c71a890`。现有 Ghidra 导出显示这是栅格目录
专用对象体，入口在四字节版本后探测 `0xff00ff` 的 RasterIO 记录块。

Load 在 tag 30 读取动态对象数，并从 tag 31 起按序调用 ReadObj。
这些 RasterIO 对象的外层为类标识、无普通 ObjectStream 键。
RasterIO End 位于 `0x7c71a7ac`。复核 short 读取后的
`0x7c71a7db..0x7c71a7dd`，非负 HRESULT 跳到 `0x7c71a801`，
只有失败才从 `0x7c71a7e8` 返回。v3 起读取 short，v4 接着读取
long，所有成功分支都进入 `0x7c71d350` 的 ExtraSafe 扩展循环。
此前把失败返回块当作 v3 及以上的无条件终点，现已更正。

Save 取 `0x7c7678ec` 的六项版本表 `[-1, 1, 1, 3, 3, 4]`，
并写为有符号四字节值。保存端在 `0x7c71adf2` 结束 RasterIO，
`0x7c71adfb` 和 `0x7c71ae32` 使用相同的 short/long 版本门，
`0x7c71ae59` 调用 `0x7c71d290`。扩展段两端均先处理四字节计数，
随后分别用 ExtraSafeReadObject 和 ExtraSafeWriteObject，
Load 请求 IUnknown（IID 数据 `0x7c76775c`）。

两个入口在扩展段后都查询完整对象 `+0x88` 的聚合对象。构造
`0x7c715db0` 用 `0x7c769180` 的 FeatureLayer CLSID
`{E663A651-8AAD-11D0-BEC7-00805F7C4268}` 创建它。Load 的
`0x7c71a843` 与 Save 的 `0x7c71ae77` 经 `0x7c721080`
获得持久化接口后，分别从 `0x7c71a853` / `0x7c71ae8b`
把同一流传入其 Load/Save 槽。该尾段由委派体确定终点，
尚需核对聚合 FeatureLayer 的实际读取分支和 RasterIO 动态记录类，
然后才能接入普通 ILayer。真实宿主及 ArcMap/QGIS 人工验收待验证。

#### 栅格目录的 RasterIO 槽分派

复核同摘要 RasterLayer.dll 的 `0x7c71a230` 和 `0x7c71a890`，
结合导入表区分 RasterIO 的 long、short、double 和宽字符串重载。
tag 1、3–5、7–9 为 long，tag 6 为 double，tag 10 为 short，
tag 11 为宽字符串。tag 2 经 `0x7c71a2ee` 调 ReadObj，随后
`0x7c71a301 → 0x7c720340` 按 `0x7c769270` 查询
ISpatialReference；Save 直接传持久化视图 `+0x84` 成员。

tag 30 是渲染器集合计数。读取循环由 `0x7c71a453` 对
`31 + i` 调 ReadObj，`0x7c71a468 → 0x7c7203b0` 查询
`0x7c769450` 的 IRasterRenderer（与普通 RasterLayer 同 IID）。
保存端从持久化视图 `+0x94` 的集合取第 i 项，查询同样的接口后
才输出该项。因此声明计数与实际写出的对象标签数可能不同，不能以
固定数量的连续对象体确定结束点。对象记录没有外层 ObjectStream 键，
未知新类的体也没有可据以跳过的正长度，需要按类分派。已确认的
空间参考与栅格渲染器持久化读取器可复用。

设集合计数为 N，Load 从 v2 起查询 `31 + N` 的 long 选择值，
值为 1 时查询 `32 + N .. 35 + N` 的 long；v3 起该分支另查
`36 + N .. 39 + N` 的 long 和 `40 + N` 的宽字符串。
这些字段的读取与后续接口设置见 `0x7c71a4ac..0x7c71a6c9`。
布局对齐仍需验证缺失标签的遍历、嵌套渲染器完成状态及选择值为 1
时的间接调用。对本机去重后的 106 份 MXD/MXT 筛查，成功提取的
104 个 `/Maps` 和 104 个 `/PageLayout` 均未命中目录图层 CLSID。
筛查限于这两条流内的类标识候选，解析器接线及人工验收仍开放。

读取器已接入普通 ILayer 分派和 Map 有界条目。按递增标签读取
RasterIO，允许动态渲染器标签省略，复用已确认的无键对象体。
RasterIO 结束后按版本门读取 short、long，复用 ExtraSafe 扩展读取，
最后以内联 FeatureLayer 的完成状态确认对象终点。聚合体没有独立
对象头，不登记额外图层键。未知 RasterIO 类、未读完的委派体和
截断都会使普通槽回滚；有界条目另校验声明终点。构造 Scene 宿主
覆盖 v0–4、稀疏渲染器记录、版本化尾段、后继条目与截断回滚。
真实宿主、第三方 RasterIO 对象和 ArcMap/QGIS 人工验收仍待验证。

Scene 构造回归进一步使用 v3/v4 的非空 tag 32，将
SimpleRasterRenderer 的 CLSID 直接接旧共用渲染器体。测试确认
已省略的 tag 31 不影响分派、四个渲染器标量完整保留、外层不会
登记渲染器键，且仍能读取后继选择值、聚合图层和宿主尾段。
在渲染器体内截断会回滚整个关联槽，真实工程验收仍待进行。

### RasterBasemapLayer 读写入口与版本门

复核 BasemapLayer.dll 10.1.0.3035（SHA-256
`c8f4692970c5311e5d591f317c5e6c0379e9e6b9ad62a2383c71ecb6c8f64aad`）。
主 QI `0x6b688f40` 在 `0x6b689021` 匹配 IPersistStream，
由 `0x6b68924f` 返回完整对象 `+0x220`。构造路径在
`0x6b68842e` 安装子表 `0x6b6aec78`。GetClassID
`0x6b68b370` 从 `0x6b6ae368` 写出
`{57520261-2608-430B-904E-7B0D48C578D5}`，Load/Save 槽分别指向
`0x6b68af00` 和 `0x6b68b1f0`，与普通 BasemapLayer 不共用对象体。

Load 读有符号 short 版本，`0x6b68af5a` 检查上限 5。
后续依次读 short、16 字节 GUID、ISpatialReference 普通槽
（`0x6b68afea`）、请求 `{00A527CF-28DA-4043-9CF7-E9911172F05F}`
的普通槽（`0x6b68b01e`）。版本恰为 2 时，
`0x6b68b038` 跳过 `0x6b68b045/0x6b68b068` 两次 short
读取。之后 `0x6b68b0b4` 调 ExtraSafeReadObject，请求
`{D02371C7-35F7-11D2-B1F2-00C04F8EDEFF}`，结束时 v4 起
读取字符串，v5 再在 `0x6b68b1c8` 调 ExtraSafeReadObject，
请求 `{EF80DA8F-686B-4D8E-87ED-C2213273632F}`。

Save 在 `0x6b68b218` 使用单项版本表 `[5]`。写端的 GUID、
两个普通对象、两个 short、ExtraSafe 对象、字符串和最终
ExtraSafe 对象调用与 v5 读序相符。接入前还需确认两个非空间参考
接口的实现类、首个 ExtraSafe 槽对栅格子类的分派，以及尾槽边界。
当前仅完成静态取证，尚无真实宿主或 ArcMap/QGIS 人工验收。

同一 10.1 Desktop 介质的 esriCarto 类型库将三个槽的 IID 分别
定名为 IArrayOfIShader、IRasterLayer、IMosaicLayer。
IArrayOfIShader 的公开实现是 ArrayOfIShader，复用普通底图已核实
的版本和计数。IRasterLayer 的已知实现 RasterLayer、WCSLayer 和
ImageServerLayer 可在首个 ExtraSafe 槽的声明范围内试读，仅在
完整消费时提交。IMosaicLayer 的 MosaicLayer 新体按声明长度保留，
内部不做分派。两个 ExtraSafe 槽均检查边界，普通着色器数组仅接入
已确认的空数组体。构造 Scene 宿主覆盖 v0–5、RasterLayer 回引、
非空有界镶嵌原始体、后继边界和截断回滚。非空着色器、镶嵌体内部
对象、真实工程和 ArcMap/QGIS 人工验收仍待验证。

#### 普通和栅格底图的非空着色器数组

BasemapLayer.dll 同摘要的 ArrayOfIShader::Load 在
`0x6b66f86c` 调用完整对象虚表 `0x6b6adedc` 的 `+0x2c` 槽，
到 `0x6b66f320`。助手在 `0x6b66f37b` 以普通 LoadObject
请求 IUnknown，随后在 `0x6b66f39b` 查询 IShader
（`{CB0F9195-3E7C-4A2B-8C95-A8F2FAC10C4C}`），没有外层体长。
同介质 esriCarto 类型库列出 RasterShader 和 RasterDRAShader 两个
实现。虚表 `0x6b6af540` 和 `0x6b6af364` 的 GetClassID
分别确认 RasterShader 与 RasterDRAShader。前者的 Load
`0x6b690180` 读 short 版本、两个 long 和 double；后者的
Load `0x6b68f3f0` 读 short 版本、一个 long 和三个 float。
两者的版本上限均为 1。float 类型经 `0x6b6aa1c4` 的导入签名确认。

普通和栅格底图复用元素体读取器，按登记类识别回引。普通底图遇到
未知元素仍停在数组头之前。构造 Scene 宿主验证两类非空元素、回引、
后继槽和截断回滚。真实非空数组及 ArcMap/QGIS 人工验收仍待验证。

### MosaicLayer 的独立持久化前缀

从 ImageServerLayer.dll 10.1.0.3035（SHA-256
`0256f97057275fedb550c947b0f8333184427755a394ef09099d22349fe5bb14`）
补充 pyghidra 导出并以 objdump 核对。构造路径 `0x78017cfb`
安装完整对象 `+0x7c` 的持久化表 `0x78032a4c`。
GetClassID `0x78019750` 返回 MosaicLayer CLSID
`{3452FCB8-D10C-44E4-B971-8C53DB39D517}`，Load/Save 为
`0x78019780` / `0x78019950`。

Load 读取 short 版本，`0x78019823..0x78019828` 仅接受 1、2。
接着 `0x7801e060` 以普通 LoadObject 请求 IName
（IID 数据 `0x78030630`），再读 AfCore String 和 short。
`0x78019886..0x7801989a` 从完整对象 `+0xa8` 取得委派对象的
IPersistStream，调用其 Load 并传入同一流。v2 随后依次调用
TimeData、TimeDataDisplay 和 TimeTableDefinition 的读取方法。
Save 的版本表 `0x7802ca70` 为 `[-1,-1,-1,-1,2]`；写出
相同前缀、委派体与三个时间段。

构造路径 `0x78017e0c` 调 `0x7801eb20`，为独立的 `+0xe8` 成员创建
`{FFCAEE7F-B07F-45A0-8586-C9CE98F9B215}`。CartoXProvider.dll
10.1.0.3035（SHA-256
`1c4a6d3681e7527ca21e1b9afeb3b5a83922ad12f6c8a0e5c19dec33dd6f3cf5`）
的注册指令 `0x6ced16ab..0x6ced16b5` 将该 CLSID 与
`0x6cee3620` 的名称 MosaicLicenseHelper 绑定。同一模块
`0x6ce4da99` 接受构造器所请求的接口
`{DF72A438-E456-4F70-B776-B85238584AD4}`。该成员与上文的委派对象分开，见下节复核。RasterBasemapLayer
仍保留 MosaicLayer 原始体。真实宿主与 ArcMap/QGIS
人工验收尚未进行。

#### 许可助手的本地接口边界

CartoXProvider 同摘要 DLL 的 MosaicLicenseHelper 构造在
`0x6ce4d955..0x6ce4d960` 保存控制 IUnknown；无聚合者时用完整对象
自身。`0x6ce4d993` 安装主表 `0x6cee369c`，首槽
`0x6ce4da80` 在 `0x6ce4dab0` 为自定义 IID 返回 `+0x30` 视图。
该视图的 QI `0x6ce4d890` 经完整对象 `+0x08` 转发给控制
IUnknown。ImageServerLayer 的创建调用 `0x7801eb4b` 传空聚合者，
在 `0x7801eb78` 取得自定义视图。

若对许可助手视图查询 IPersistStream，该 IID 沿上述转发到
本地助手的主 QI。
主 QI 匹配自定义 IID 以外走 `0x6cea5640`，只比较 IUnknown、
ISupportErrorInfo 及 `{9F9650F1-5F49-4041-BA0F-D10BAFF1D7BC}`，
未命中在 `0x6cea5732` 返回 E_NOINTERFACE。MosaicLayer 的持久化委派
读取的是另一个成员，见下节。


RasterBasemapLayer 的有界 MosaicLayer 槽现在尝试读取版本 1/2、
普通 MosaicDatasetName 新体或回引（也接受空名称）、原始字符串和
short。前缀完整时提交其登记键，并记录委派段相对对象体的起始偏移。
整段原始字节仍保留。前缀不能完整读取时保留原有有界结果，撤销
本次前缀试读的引用键。构造 Scene 宿主覆盖新名称体、委派起点、
原始字节及后继容器字段。当时尚未对位委派体，后续复核见下。

本机介质与历史提取目录按 SHA-256 去重后的 119 个 LYR 均为复合
文档。用项目的 inspect/extract 读取其中全部 127 条流，包括
119 条 `/Layer` 及各 4 条大小预览流，未命中 MosaicLayer、
RasterBasemapLayer 或 MosaicLicenseHelper 的 CLSID 字节。在 106 份
去重 MXD/MXT 中成功提取的各 104 条 `/Maps` 和 `/PageLayout`
也未命中 MosaicLayer 或 MosaicLicenseHelper。该筛查不能发现无独立
对象头的委派体，真实 MosaicLayer 的委派边界仍待取证。

构造回归用完整名称体后接超限字符串长度，核对有界槽继续保留完整
原始字节和外层 MosaicLayer 键，撤销前缀试读登记的名称键，并保留
下一字段位置。ArcMap/QGIS 人工验收尚未进行。

#### 许可助手的类工厂

同版 `esri.clsid.ecfg` 解包后的类表将
`{FFCAEE7F-B07F-45A0-8586-C9CE98F9B215}` 登记为
MosaicLicenseHelper。CartoXProvider.dll 的注册代码
`0x6ced16d3` 使用工厂构造器 `0x6ce9e770`，安装
`0x6ceee95c`。CreateInstance 槽 `+0x0c` 到 `0x6ce9e910`。
沿该入口的非聚合分支核对构造调用，`0x6ce9ec91` 向
`0x6ce4d910` 传空控制对象，`0x6ce9ecb5..0x6ce9ecc0`
从新对象 `+0x30` 的虚表查询调用者要求的 IID。
聚合分支则在 `0x6ce9ec11` 传入调用者提供的外层对象。

这把前节的本地 QI 结果绑定到已登记工厂的非聚合构造路径，
介质中的类表只提供 CLSID 与程序标识。此工厂对应 `+0xe8` 成员，
不决定持久化委派的边界。


#### MosaicLayer 的聚合 GroupLayer

复核同摘要 ImageServerLayer.dll 10.1.0.3035 的构造指令，修正
此前把许可助手当作持久化委派对象的判断。`0x78017e00` 将
`+0xe8` 地址交给许可助手构造调用，而 `0x78017e11` 切换到
`+0xa8`。后者在 `0x78017e3e` 接收 AfCreateInstance 的输出：
CLSID 数据 `0x78032c48` 是 GroupLayer
`{EDAD6647-1810-11D1-86AE-0000F8751720}`，请求 IUnknown，
聚合者为完整对象 `+0x6c`。Load 在 `0x78019886` 使用
持久化视图 `+0x2c`，换算为完整对象 `+0xa8`，正是聚合 GroupLayer。
由 `0x7801f331` 查询 IPersistStream 后，`0x7801989a` 直接
把同一流交给 GroupLayer::Load。Save 对称调用其 Save。

读取器在 MosaicLayer 前缀后复用已确认的 GroupLayer 完整体分派，
没有独立对象头或新增 GroupLayer 键。v1 要求委派体到达对象边界；
v2 保存 GroupLayer 终点供后续时间段读取。构造 Scene 宿主核对
委派名称、终点和外层边界。真实宿主及 ArcMap/QGIS 人工验收仍待验证。

#### MosaicLayer v2 的时间段

同摘要 ImageServerLayer.dll 的 `0x780198ad` 以 v2 为门，
`0x780198b8`、`0x780198cd`、`0x780198e2` 依次通过
LayerLib.dll 导入表调用 TimeData、TimeDataDisplay、
TimeTableDefinition 的 Load。三个调用都从聚合 GroupLayer 返回
后的同一流读取。Rust 复用相应的既有段读取器，按 MosaicLayer
的顺序调用，并要求最后到达 ExtraSafe 对象体边界才提交时间段及
其键。时间段不完整时仍保留已确认的前缀与 GroupLayer，剩余原始
字节可复查。构造测试覆盖三个段中的不同标量及后继容器边界。
真实非空宿主及 ArcMap/QGIS 人工验收仍待验证。

#### MosaicLayer 的普通 ILayer 宿主

同一 10.1 介质的 esriCarto 类型库将 MosaicLayer 列为 ILayer
实现。普通对象分派复用上述读序。v1 在聚合 GroupLayer 结束时提交，
v2 到三个时间段全部读完时提交；失败回滚对象游标和引用表。
Map 有界条目另核对声明终点，RasterBasemapLayer 的有界槽仍可在
未知分支保留原始字节与已证前缀。构造 Scene 宿主覆盖 v1、v2 的
后继字段和体内截断。真实宿主、逐层遮罩接口和 ArcMap/QGIS 人工
验收仍待验证。

### DimensionLayer 的持久化前缀

对 [Dimension.dll 10.1.0.3035](mxd-luna-deep-layer-masking-20261118.md#dimensionlayertopologylayer-和-networklayer)
同摘要 PE32 提取物执行 `objdump`。主 QI `0x6e9e97d0`
于 `0x6e9e9817` 比较 IPersistStream IID（常量 `0x6e9f427c`），
命中后在 `0x6e9e9b17` 返回完整对象 `+0x10`。构造指令
`0x6e9e9506` 将持久化虚表 `0x6e9f4c68` 装在该偏移，
Load、Save 分别为 `0x6e9eac80`、`0x6e9ead80`。

Load 先读有符号 short，拒绝高于 4 的版本。在 `0x6e9eacd7`
直接对构造时创建的聚合 FeatureLayer 持久化接口调用 Load，
传入同一 IStream，因此聚合体前无独立对象头或引用键。外层仅 v2
在 `0x6e9eaced` 调用 AfCore String::Read，再于
`0x6e9ead17` 读两个字节。外层 v4 在 `0x6e9ead53`
读取末尾四字节。需继续核对 Save、聚合体完成边界及末字段语义，
然后接入普通 ILayer 分派。真实非空 3D 宿主和 ArcMap/QGIS
人工验收未执行。

同一镜像的 Save `0x6e9ead80` 在 `0x6e9ead97` 获取目标文档
版本，`0x6e9eada7` 用 AfStreamGetSaveVersion 查
`0x6e9f4aa0` 的四项 short 表 `[1, 3, 3, 4]`，
并在 `0x6e9eadbc` 写出两字节对象版本。`0x6e9eaddb`
直接调用同一聚合成员的 Save，传原 IStream 和清除脏标记参数。
聚合 Save 失败时结束写出。随后比较的是目标文档版本索引；
索引至少为 3 时，在 `0x6e9eae04` 写出成员 LONG，与读侧 v4
末字段对应。该写出表没有对象版本 2，v2 附加字段仍需按读侧兼容。
字段业务语义和真实 3D 宿主验收待核对。

普通 ILayer 分派已接入 DimensionLayer。内联 FeatureLayer 必须读到
`TailCompleted` 才能继续读取外层字段，失败时整个对象回滚。
Scene 与 Globe 的构造输入覆盖 v1–v4、旧字段、末尾 LONG、
外层引用键、后继容器边界和逐字节截断。真实宿主及 ArcMap/QGIS
人工验收未执行。

### TopologyLayer 的持久化入口

复核 [FeatureLayer.dll 10.1.0.3035](mxd-luna-deep-layer-masking-20261118.md#dimensionlayertopologylayer-和-networklayer)
同摘要 PE32 提取物。QI `0x6f72894b` 匹配 IPersistStream
IID `0x6f74eb6c`，在 `0x6f728b18` 返回完整对象 `+0x60`。
该偏移的虚表 `0x6f7b1784` 由 `0x6f728472` 安装。
GetClassID `0x6f72b600`、Load `0x6f72aee0`、
Save `0x6f72b3c0` 分别位于 `+0x0c/+0x14/+0x18`。

Load 在 `0x6f72aefb` 读两字节版本，`0x6f72af08` 的
有符号版本上限为 5。随后进入名称、定宽字段及对象槽读取。
本次仅确定独立持久化入口和版本门。对象槽类型、旧版分支和完整体
结束边界仍需对位后才能接入普通 ILayer 槽。未执行原生运行或
ArcMap/QGIS 人工验收。

#### TopologyLayer 的对象槽与尾段

同版 FeatureLayer.dll 的 Load `0x6f72aee0` 在版本后调用
AfCore String::Read（`0x6f72af1a`），再读三个 short 和两个
八字节字段，随后于 `0x6f72afad` 请求 IName（IID 常量
`0x6f7aaf28`）。v1 在 `0x6f72b043` 跳至收尾路径。
继续时，`0x6f72b068` 请求 IFeatureRenderer（IID 常量
`0x6f7a98cc`），循环索引与 6 比较。v2 在循环后结束。

继续读取的路径于 `0x6f72b0bd` 请求 IUnknown（IID 常量
`0x6f74eb7c`）。v3 跳过 `0x6f72b124` 的 short 并结束，
v4 读该 short 后在 `0x6f72b165` 结束。v5 常规路径另读
两个 short 和有符号 LONG 数量，正数循环每项读一个四字节值
和一个 short（`0x6f72b1c9`、`0x6f72b1e9`）。
计数为零或负数时跳过循环。仍需与 Save 交叉核对渲染器数量、
早于 v1 的控制流及 IUnknown 槽的实际类。未进行 ArcMap/QGIS
人工验收。

#### TopologyLayer v5 的写出记录

同版 FeatureLayer.dll 的 Save `0x6f72b3c0` 写出常量
`0x6f74f188` 的 short 5。名称和定宽前缀后，
`0x6f72b481` 调用名称写出帮助函数 `0x6f72b250`。
渲染器循环从索引 0 开始，每次在 `0x6f72b4a8` 调用
SaveObject，递增后按 `<= 6` 回跳，共写七个槽。随后
`0x6f72b4d6` 无条件以空对象调用 SaveObject。这与 Load
读取七个 IFeatureRenderer 槽后请求 IUnknown 的顺序吻合，
也明确了同版原生写出的 IUnknown 槽为空。

尾段依次写三个 short、一个 LONG 计数，再写计数项的四字节值
与 short。地址分别是 `0x6f72b4f7/0x6f72b510/0x6f72b529`、
`0x6f72b543`、`0x6f72b582/0x6f72b59e`。旧体中可能
存在的非空 IUnknown 仍需独立证据。解析接入及 ArcMap/QGIS
人工验收未执行。

普通 ILayer 分派已支持 v1–v5。名称槽复用已核对的 IName 类体，
七个渲染器槽复用已有矢量渲染器读取器，空或已登记的 IUnknown
条目可继续，未知新对象体返回不支持。Scene 与 Globe 构造测试覆盖
五种版本、尾记录、共享登记表、后继字段及逐字节截断回滚。
未核对的非空名称与渲染器类、旧版 IUnknown 新对象
以及真实宿主的 ArcMap/QGIS 人工验收仍待取证。

#### TopologyLayer 渲染器数组的初始化

同版 FeatureLayer.dll 构造函数在 `0x6f728531–0x6f72854a`
以步长 4 初始化完整对象 `+0xe8` 起的七项数组。元素构造函数
`0x6f6b6580` 只清零指针。Save 的循环从持久化接口基址
`+0x88` 开始（完整对象 `+0xe8`），对应同一数组。
非空对象由后续设置路径产生。当前定位到 `0x6f72958d`、
`0x6f729959`、`0x6f729d25` 和 `0x6f729280` 的写入。

其中写入首槽的路径在 `0x6f72943c` 调用 `0x6f728b50`，
该帮助函数使用常量 `0x6f7adb34` 中的 SimpleFillSymbol CLSID
`{7914E603-C892-11D0-8BB6-080009EE4E41}`。该标识属于内嵌符号，
应继续沿外层对象的构造和持久化调用确定渲染器 CLSID。
没有使用符号类标识推定渲染器类。实际非空拓扑图层和 ArcMap/QGIS
人工验收仍待验证。

首槽的外层对象来自 `0x6f72940e` 调用的构造帮助函数
`0x6f7027d0`。该函数于 `0x6f7027fb` 将
`0x6f7ae064` 的 CLSID `{F3435801-5779-11D0-98BF-00805F7CED21}`
传给 AfCreateInstance，即 SimpleRenderer。
`0x6f729455` 将 SimpleFillSymbol 设入该渲染器，
`0x6f729572` 经 `0x6f704c80` 对同一对象查询
IFeatureRenderer（IID 常量 `0x6f7ae694`），再写入首槽。
Scene 与 Globe 构造测试覆盖此渲染器及其符号的新键、后续槽和截断回滚。
其他渲染器槽的原生赋值类及真实宿主仍待核对。

续查同版 FeatureLayer.dll 的两条赋值链。第二槽（完整对象
`+0xec`）在 `0x6f7297ee` 调用同一 SimpleRenderer 构造帮助函数
`0x6f7027d0`，并在 `0x6f729808` 调用 `0x6f728cd0` 构造
SimpleLineSymbol（`0x6f7aaff8`，CLSID
`{7914E5F9-C892-11D0-8BB6-080009EE4E41}`）。在
`0x6f72993e` 查询 IFeatureRenderer 后，`0x6f729959` 写入第二槽。
第三槽（`+0xf0`）对应构造点 `0x6f729bba`，符号构造点
`0x6f729bd4` 调用 `0x6f728e10`，使用 `0x6f7ab018`
的 SimpleMarkerSymbol CLSID `{7914E5FE-C892-11D0-8BB6-080009EE4E41}`。
查询点为 `0x6f729d0a`，写入点为 `0x6f729d25`。
两条路径的外层渲染器和内嵌符号均已有读取器。此处确认的是
原生初始化路径，其他写入路径及真实工程验收仍待核对。

第七槽（完整对象 `+0x100`）也使用 SimpleRenderer。
`0x6f72917f` 调用构造帮助函数 `0x6f7027d0`，
`0x6f7291a9` 构造 SimpleFillSymbol 并在 `0x6f7291c2`
设入渲染器。经 `0x6f729207` 查询 IFeatureRenderer 后，
`0x6f729280` 将对象写入第七槽。构造测试同时填入首槽与
第七槽，核对各自的对象键、后继 IUnknown 槽及截断回滚。
真实工程中的第七槽仍待验证。

Scene 与 Globe 构造回归将上述线、点符号分别放入第二、第三槽，
与首槽和第七槽的面符号共存。检查四个渲染器及符号的独立键、
外层后继字段和逐字节截断回滚。真实工程验收未执行。

#### TopologyLayer 的非正版本分支

同版 FeatureLayer.dll 的 Load `0x6f72af08–0x6f72af0d` 用有符号
比较拒绝大于 5 的版本。其后在 `0x6f72b03e`、`0x6f72b09a`、
`0x6f72b105` 和 `0x6f72b160` 分别仅对 1、2、3、4 提前收尾。
因此 v0 与负版本走完整路径，包括七个渲染器、IUnknown、
三个尾 short 及计数数组。读取器已对齐，Scene 和 Globe 构造回归
覆盖 v0、-1 和后继字段边界。尚未在实档中验证非正版本。

TopologyLayer 的七个渲染器槽经 `0x6f72b068` 请求
IFeatureRenderer，普通对象流沿用按键取实例再查询接口的规则。
共用读取器现在从引用表确认已核对的渲染器类。Scene 与 Globe
构造输入验证回引头为 Map、登记实例为 SimpleRenderer 时可继续，
以及回引头为 SimpleRenderer、登记实例为 TopologyLayer 时
整个容器回滚。未核对的第三方渲染器及真实宿主仍待取证。

#### TopologyLayer 的 SDK 实档

对仓库样本和本地提取件的 MXD、MXT、SXD、3DD 文件按 SHA-256
去重，得到 142 份 CFB。用 CLI 提取 140 条 /Maps、140 条
/PageLayout 和 16 条 /3D Analyst 流，扫描 TopologyLayer 的
CLSID 内存序。唯一候选位于 .NET SDK 的 Montgomery.mxd，
/Maps `0x19fe`。该位置经实际分派确认是条目 2 的新对象，键 95、
对象版本 5。名称槽在 `0x1a52` 使用 TopologyName，键 96。

SDK 回归确认名称对象体可完整读取，保留七个渲染器槽和空 IUnknown，
且未解析剩余为零，下一图层条目始于 `0x2380`。扫描未发现
/3D Analyst 中的同类字节候选。实档验证覆盖 SDK MXD 的 Map
宿主，ArcMap/QGIS 人工验收及真实 3D 宿主仍待完成。

#### DimensionLayer 的 SDK 实档

10.1 .NET SDK 的 Montgomery.mxd 在 /Maps `0x291` 有
DimensionLayer 条目，登记键为 31。SDK 回归确认外层版本为 4，
内联 FeatureLayer 读到 TailCompleted，随后读取外层 LONG，
未解析剩余为零。下一条 TopologyLayer 从 `0x19f4` 开始。
实档证据覆盖 Map 宿主及 v4 布局。真实 3D 宿主、旧版实档和
ArcMap/QGIS 人工验收仍待验证。

### NetworkLayer 的 SDK 实档

10.1 .NET SDK 的 SanFrancisco.mxd 与 Java SDK 的 portland.mxd
分别在 /Maps `0xd0596` 和 `0x75f98` 包含有界 NetworkLayer。
实档回归确认两者都含非空渲染器记录，读取条数等于声明计数，
两个条目的未解析剩余均为零。真实 3D 宿主与 ArcMap/QGIS
人工验收仍待验证。

### SchematicLayer 的持久化入口

Desktop 介质的 SchematicLayer.dll 为 PE32，版本资源 10.1.0.3035，
SHA-256 为 `b2dcc605e8953f5e28a3ca25249c4bf5763557173dc52a994d744b6ca69e4d02`。
GetClassID `0x7d8e9b90` 从 `0x7d90b854` 复制
`{2EF082D6-0B5F-4B70-9416-35D75EFA7FED}`，与介质类登记一致。
QI 在 `0x7d8e46c6` 比较 IPersistStream IID 并在
`0x7d8e4958` 返回完整对象 `+0x40`。构造指令
`0x7d8e3be1` 在该偏移安装虚表 `0x7d90cb74`，其中
Load 为 `0x7d8e8b40`、Save 为 `0x7d8e9500`。

Load 在 `0x7d8e8b55` 读取两字节版本，`0x7d8e8b66`
比较有符号上限 7。v1、v2 附加分支分别从
`0x7d8e8c2c`、`0x7d8e8c4d` 判定。后继对象槽和完整消费
边界仍待对位，尚未接入读取器或进行 ArcMap/QGIS 人工验收。

#### SchematicLayer 的名称与关联图层槽

同版 SchematicLayer.dll 的 Load 在通用前缀读取字符串、两个
short、三个八字节字段和另一字符串。v1 起在 `0x7d8e8c3c`
读一个 short。v2 起在 `0x7d8e8c67/0x7d8e8c7f` 读两个
八字节字段，随后于 `0x7d8e8c9c` 请求 IName，IID
`{4BA33331-D55F-11D1-8882-0000F877762D}` 位于 `0x7d90cd88`。

v4 起在 `0x7d8e8dc6/0x7d8e8dee` 读取两个同接口槽，
IID `{1B077914-38C5-11D0-92D2-00805F7C28B0}` 位于 `0x7d90cde8`。
v5 起在 `0x7d8e8e1d` 读取有符号 LONG 项数，正数分支
在 `0x7d8e8e6c` 请求 ILayer（IID 位于 `0x7d90ce48`），
随即读条目字符串。计数循环后在 `0x7d8e8f7c`、
`0x7d8e8f98`、`0x7d8e8fb8`、`0x7d8e8fd4`
依次读 short、double、short、short。旧记录分支、每项完整布局、
末段的流消费及写侧版本差异仍需核对。未进行原生运行验收。

#### SchematicLayer 关联项的写出边界

同版 Save `0x7d8e9500` 在 `0x7d8e9520` 用
AfStreamGetSaveVersion 查 `0x7d90c334`。该地址前六项 short
为 `[-1, 4, 4, 4, 6, 7]`。写侧另在 `0x7d8e9640`
检查对象版本是否至少为 6。表长度和各分支的完整适用范围仍需
结合版本参数和控制流复核。

读侧 v5 起的每个关联项先在 `0x7d8e8e6c` 调 LoadObject，
再在 `0x7d8e8e8c` 调 String::Read。到计数回跳
`0x7d8e8f61` 之前，其余调用处理内存中的对象与字符串。
写侧计数循环对应地在 `0x7d8e9abf` 调 SaveObject，在
`0x7d8e9ae1` 调 String::Write，并于 `0x7d8e9afe`
比较循环计数。关联项的持久化边界因此为 ILayer 和一个字符串。
早期兼容记录以及循环前后的整段版本条件仍待核对。

#### SchematicLayer 末段的流边界

v4 起的成对对象槽使用的 IID `{1B077914-38C5-11D0-92D2-00805F7C28B0}`
是 IEnvelope，可复用[已核实的范围对象读取器](mxd-10.1-raster-layer-tail.md)。
在 v5–v7 的 Load 路径中，最后一个流读取是 `0x7d8e8fd4`
的 short。其后 `0x7d8e8fe6` 起使用已载入对象和成员恢复
运行状态。包括 `0x7d8e906c` 的 ConvertToFullPath，入参为
字符串和 IName，`0x7d8e909f` 的间接调用也使用已取得的对象，
没有传入当前 IStream。Save 在关联项循环后依次于
`0x7d8e9b18/0x7d8e9b2d/0x7d8e9b46/0x7d8e9b58`
写 short、double、short、short，与读侧末段对齐。
末字段的业务语义和真实工程验收仍待核对。

#### SchematicLayer v3–v5 的兼容记录

同版 Load 在 `0x7d8e8cac–0x7d8e8cb6` 先将版本减 3，
再按无符号 short 与 2 比较，只有 v3–v5 进入此段。v0–v2
跳过此段，v6–v7 也跳过。进入后依次在 `0x7d8e8cc9`
读字符串，在 `0x7d8e8cfc` 读有符号 LONG 数量，并在
`0x7d8e8d2c` 按正数计数逐项调用 LoadObject。请求 IID
`{3036D35D-EDE5-11D0-87FE-080009EC732A}` 位于 `0x7d90cf88`，
是[已核实的 IHyperlink](arcmap-10.1-hyperlink.md)。
循环后在 `0x7d8e8d51/0x7d8e8d8b` 分别读字符串和 LONG。
最后一个 LONG 直接进入清理路径，无第二个条目循环。

Save 在 `0x7d8e9640–0x7d8e9645` 对对象版本至少为 6
直接跳过兼容段；低版本写出两个空字符串和两个零 LONG。
读侧允许非空 IHyperlink 集合，仍需以构造记录验证复用读取器
以及后继 IEnvelope 槽边界。真实工程验收未执行。

普通 ILayer 分派已接入上述版本记录。v1 的额外 short 按
`0x7d8e8c31` 的等值分支读取。关联图层递归使用已有完整体分派和
深度限制。Scene、Globe 构造输入覆盖 -1 和 v0–v7、非空
FeatureHyperlink、嵌套 SchematicLayer、末字段与截断回滚。
未核对的非空名称或子图层类仍按各共用槽的支持范围处理。
真实工程及 ArcMap/QGIS 人工验收未执行。

#### SchematicLayer 的 SDK 实档

按 SHA-256 去重的 142 份本地 CFB 工程共提取 296 条
/Maps、/PageLayout、/3D Analyst 流。SchematicLayer CLSID
候选仅见于 .NET SDK 的 SchematicsForServerSample.mxd 的 /Maps。
SDK 回归沿用同一引用表读取四个根条目，在两个直接 Map 对象的
`0x121c2`、`0x20312` 验证 v6 示意图层。两层均含非空关联
子图层，实际条目数与声明一致，未解析剩余均为零，根列表末尾
保留四字节流尾。ArcMap/QGIS 人工验收未执行。

SDK 回归现也遍历 Map 持久化集合中的内嵌 Map。该对象在根列表
由键 533 回引，其中 `0xcd41` 的第三个 SchematicLayer v6
与另外两个条目通过同样的关联层计数和完整消费检查。

### TemporalFeatureLayer 的持久化入口

复核[同摘要的 TrackingLayer.dll 10.1.0.3035](mxd-luna-deep-layer-masking-20261118.md#temporalfeaturelayer-的聚合查询)。
PE 数据中两套虚表 `0x808a12a0`、`0x808a1f58` 共用
GetClassID `0x8086eb90`、Load `0x8086ebd0` 和 Save
`0x8086f840`，分别位于槽 `+0x0c/+0x14/+0x18`。

Load 在 `0x8086ec27` 调内部 Variant 读取帮助函数
`0x8087ca70`。该函数在 `0x8087cae0` 从原流读取两字节
VARTYPE。外层在 `0x8086ec36` 比较的是此 Variant 的类型 5
（VT_R8），在 `0x8086ec48` 将其 double 值与常量
`0x808b07c8` 的 1.51 比较。达到该阈值的路径另读一个 LONG，
并连续四次调用 `0x8087c940`。因此解析应先核对 Variant 线格式
和 double 版本分支。其余阈值、内联 FeatureLayer 的流位置和
末段尚待对位。未执行 ArcMap/QGIS 人工验收。

#### TemporalFeatureLayer 的内联持久化调用

复核同版 TrackingLayer.dll，`0x8086ef26` 从当前持久化接口
基址 `+0x2450` 取成员，`0x8086ef45` 向它查询
IPersistStream（IID 常量 `0x8089f340`）。`0x8086ef55`
直接用原 IStream 调用该接口的 Load 槽，前面没有另读该成员
的对象头。成功后 `0x8086efa6` 另调用 `0x8087c940`，
随后 `0x8086efc6–0x8086f001` 还可能对另一成员调用 Load。
须继续追踪成员初始化，确认这些接口分别对应哪个具体类。

首个 VT_R8 的另外三个比较阈值为 10.0（`0x8086ecbf`）、
1.5（`0x8086eea4`）和 1.2（`0x8086eed2`），均由同镜像
常量解码得到。外围分支和各委派体的结束边界仍待对位。

#### TemporalFeatureLayer 的两个内联接收类

两套持久化虚表的 MSVC Complete Object Locator
（`0x808a6354/0x808a545c`）都给出接口偏移 `+0x10`。
因此 Load 的成员偏移 `+0x2450/+0x2458` 分别对应完整对象
`+0x2460/+0x2468`。在同版 TrackingLayer.dll 的初始化路径，
`0x8086a59e` 取 `+0x2460` 地址作为 CoCreateInstance 出参，
`0x8086a5b3` 使用 `0x8089f280` 的 CLSID
`{B2034200-7C86-11D6-B84D-00010265ADC5}`。介质登记将该类命名为
TemporalOperator。其内联 Load 位于 `0x8086ef55`。

第二个成员对应[聚合 FeatureLayer 的初始化](mxd-luna-deep-layer-masking-20261118.md#temporalfeaturelayer-的聚合查询)。
Load 在 `0x8086efc6` 检查它是否存在，随后用 `0x8089d54c`
的 IPersistStream IID 查询接口，成功时在 `0x8086f001`
直接传原 IStream 加载。两处均为内联体。中间的
`0x8087c940` 字段、TemporalOperator 自身的 Load 布局
及第二次委派后的末段仍待核对。

#### TemporalOperator 的内联体边界

Desktop 介质的 TrackingCore.dll 是 PE32，版本资源为 10.1.0.3035，
SHA-256 `3a023ddedd06d9859843383bf5c2daa5e21bc3119428765d24cfeb71d816a166`。
CoTemporalOperator 的接口表 `0x8072e6d0` 将 IPersistStream
映射到 `+0x10`。CComObject 实例的该偏移虚表为 `0x8072f760`，
其中 GetClassID `0x80706a60`、Load `0x80706aa0`、
Save `0x80706c70`。另一个返回同 CLSID 的入口 `0x807122f0`
位于 ClassID 辅助对象的虚表。

Load 在 `0x80706ae7/0x80706b20/0x80706b39` 依次读取
4、4、8 字节，随后于 `0x80706b5c` 调用 `0x8071ed70`。
以首个四字节值中的 `0xff00` 掩码作比较，小于 `0x100` 时额外在
`0x80706b79/0x80706b96/0x80706bb3` 读 LONG、int、double。
ArcMap/QGIS 人工验收未执行。

`0x80706af4–0x80706b08` 将首个 LONG 拆为低字节和
`0xff00` 掩码。条件比较用的是掩码结果，因此仅当该字节为零
才读额外的 16 字节，首字段的高 16 位不参与判断。
`0x80706b5c` 的帮助函数 `0x8071ed70` 接收的是已读
八字节值的两个半字及成员指针。它对成员调用两个虚表槽，
没有传入 IStream，因而不占额外流字节。

Save `0x80706c70` 在 `0x80706ca8` 获取文档版本索引。
索引至少为 1 时给首个 LONG 置 `0x100` 位，随后写两个
四字节值和一个八字节值（`0x80706ce2/0x80706d01/0x80706d55`）。
索引小于 1 才进入额外三个定宽字段的写出分支。
这将内联 TemporalOperator 的流长度闭合为 16 或 32 字节。
外层时态图层的中间字符串和尾段仍待接入。

#### 时态图层的内联体之间的字符串

同版 TrackingLayer.dll 的 `0x8087c940` 在 `0x8087c99a`
读四字节长度。零值仅消费长度字段。非零长度包含两个末尾字节，
上限为 `0x100000`，最小为 2。在 `0x8087c9e4` 按
长度减 2 分配 BSTR，并分别在 `0x8087ca06/0x8087ca26`
读取载荷和两个尾字节。该路径检查实际读取字节数，未比较
尾字节内容。写侧 `0x8087cc70` 以 BSTR 字节长度加 2 写出
非空指针的长度，空指针写零。此布局对应项目的 ATL 字节 BSTR
读取器，可保留载荷中的嵌入零。

外层 Load 在 TemporalOperator 之后的 `0x8086efa6` 使用
该帮助函数，Save 在写聚合 FeatureLayer 之前的 `0x8086fc30`
使用对应写函数。相同读助手也用于外层版本大于等于 1.51 的
四个早期字段。聚合 FeatureLayer 后的其余尾段仍待取证。

#### 时态图层的聚合体后继读取

同版 TrackingLayer.dll 的 Load 在聚合 FeatureLayer 返回后，
先释放成员 `+0x24a4/+0x24a8` 的原有引用，再依据流接口选择
读取方式。判定依据和写侧对应关系见下节。

在上述成员之后，`0x8086f168/0x8086f198/0x8086f1b3`
依次读取四字节整数、字节 BSTR、short。Save 的对应调用是
`0x8086fd92/0x8086fdae/0x8086fdcc`。`0x8086f238`
比较版本 10.0，且由运行状态参与判定是否继续。对于继续路径，
`0x8086f264` 先记流位置，`0x8086f2a4` 尝试内联 Load；
失败时 `0x8086f2c1` 恢复先前位置。此后至少还存在
`0x8086f3e8/0x8086f40a/0x8086f42c` 三个定宽读取，
因此前述 short 不能作为对象终点。后段完整布局仍待核对。

#### 时态图层的末段版本门

同版 TrackingLayer.dll 的 Load 在前述试读和运行时设置之后，
依次读取 short、double、short、字节 BSTR、两个 Variant
（`0x8086f3e8/0x8086f40a/0x8086f42c/0x8086f449`、
`0x8086f46f/0x8086f4e5`）。Save 的对应调用位于
`0x8086fe37–0x8086ff44`。在 `0x8086f557` 比较的
`0x808b07b0` 常量为 double 9.3，Save 在 `0x8086ff74`
使用同一阈值。较早版本在这里收尾。

达到阈值的正常读序为 double、两个 short、四字节 int、
Variant、四字节 int、两个 short、两个 Variant、三个 double、
最后一个 short。最后四项读取点为
`0x8086f79f/0x8086f7ba/0x8086f7d5/0x8086f7f0`，
与 Save `0x80870119/0x80870134/0x8087014f/0x8087016a`
的定宽写出一致。非空 Variant 对象类和前段依赖运行状态的读取
分支仍需取证，尚未在 ArcMap 或 QGIS 中人工验收。

#### 时态图层后继对象的流接口分派

Load 在 `0x8086ec11` 向输入流查询 IObjectStream，IID
`{18A45BA7-1266-11D1-86AD-0000F8751720}` 位于
`0x8089efa0`，结果保存在调用帧 `[esp+0x10]`。
聚合体之后，`0x8086f03b` 和 `0x8086f0d0` 依据该结果
选择对象读取方式。取得接口时，通过虚表 `+0x44` 读取有键对象，
两个请求 IID 分别为 `{B75D7088-1333-11D5-B7C9-00010265ADC5}`
和 `{78C74237-17CF-11D5-B7CF-00010265ADC5}`。
缺少接口时，才在 `0x8086f074/0x8086f0ff` 按 Variant 读取，
并在对象非空时查询相同接口。

Save 对称地在 `0x8086fcd6/0x8086fd3c` 调对象流虚表
`+0x40`，或者在 `0x8086fcfd/0x8086fd69` 写
VT_UNKNOWN。两路分派取决于流是否提供 IObjectStream。
实际持久化类以及其他末段 Variant 槽仍待取证。

#### 时态图层后继接口的类型库对位

从 10.1 Desktop 的 esriTrackingAnalyst.olb（SHA-256
`cd21f8b0c5b767092929b4e100cf3a64fea06f8c4f951e0a8b5b23828fa9632a`）
提取 MSFT 类型库。按 GUID 表定位 TypeInfo，再用名称表和
coclass 接口引用链交叉核对。聚合 FeatureLayer 后第一个槽的 IID
`B75D7088-1333-11D5-B7C9-00010265ADC5` 对应 ITemporalLegendGroup，
第二个槽的 IID `78C74237-17CF-11D5-B7CF-00010265ADC5`
对应 ITemporalReference。类型库分别列出 TemporalLegendGroup
和 TemporalReference coclass。介质注册表把它们绑定到
TrackingRenderer.dll 的 CLSID `5F947261-1406-11D5-B7CA-00010265ADC5`
与 `78C7423A-17CF-11D5-B7CF-00010265ADC5`。
后续须核对这两个类的 Load 以及实际写出对象是否还包括其他实现。

#### 时态图例组与时间参考的持久化入口

核对 Desktop 的 TrackingRenderer.dll PE32 版本 10.1.0.3035，
SHA-256 `4126f9a4fc669d4e2c287eacb20227c446db3d699c11e6ba42f426055ce0cda1`。
通过两个 CComObject 的 MSVC RTTI 和持久化 GetClassID 核对虚表：

| 类 | 接口偏移及虚表 | GetClassID | Load | Save |
| --- | --- | --- | --- | --- |
| TemporalLegendGroup | `+0x08` / `0x8093af0c` | `0x808ffe30` | `0x808ff850` | `0x808ffaa0` |
| TemporalReference | `+0x18` / `0x8093b03c` | `0x80901a40` | `0x80901a80` | `0x80901b60` |

TemporalLegendGroup 的 Load 有八字节读、两个字符串助手调用和
Variant 对象循环，后续布局需与 Save 对位。TemporalReference
在 `0x80901a9c` 读八字节，在 `0x80901ac3` 委派，
并于 `0x80901af4/0x80901b09/0x80901b24/0x80901b36`
另读 8、8、4、4 字节。委派对象身份和分支条件尚待核对，
ArcMap/QGIS 人工验收未执行。

#### TemporalReference 的 32 字节体

同版 TrackingRenderer.dll 的 Load `0x80901a80` 先在
`0x80901a9c` 读取 64 位整数。`0x80901ac3/0x80901ad9`
把它的低、高 32 位分别交给内存成员处理，调用均未传入 IStream。
成功路径随后读取两个 double 和两个四字节 int，最后的
`0x80901b52` 仅调用接收一个零值的成员方法。因而成功读取的
对象体固定为 32 字节，无独立版本或内联对象头。

Save 在 `0x80901ba7` 写出由两个成员值合成的 64 位整数，
再于 `0x80901bc6/0x80901bdb/0x80901bf6/0x80901c0b`
写出对应 double、double、int、int。字段业务语义和真实时态图层
宿主仍待核对，ArcMap/QGIS 人工验收未执行。

#### TemporalLegendGroup 的版本化记录

TrackingRenderer.dll 10.1.0.3035 的 Load `0x808ff850`
依次读 double 版本、两个字节 BSTR 和有符号四字节计数。
字符串助手 `0x808f3440` 以四字节长度开头，非零长度含两个
尾字节，与时态图层的字节 BSTR 约定一致。正计数时每项在
`0x808ff964` 读取 Variant，非空 VT_UNKNOWN 在
`0x808ff9a2` 查询 IID `167C5E9E-AF20-11D1-8817-080009EC732A`，
即[图例类别接口](arcmap-10.1-featurelayer-tail.md)。

循环后固定读取两个 short，版本至少 2.0 在 `0x808ffa37`
再读一个 short，至少 10.1 在 `0x808ffa70` 再读一个字节 BSTR。
阈值常量分别在 `0x8093d358/0x8093d368`。Save 使用相同
末段判断（`0x808ffc61/0x808ffc8e`），文档版本索引 0–4
写出对象版本 1.0，其余索引写 10.1。
Variant 内嵌对象的线格式及非空实例仍需验证后接入读取器。

#### 时态图例组的 Variant 对象头

TrackingRenderer.dll 10.1.0.3035 的 Variant 读取助手
`0x808f3590` 在对象分支 `0x808f3681` 调用 `0x808f32d0`。
它在 `0x808f3318` 直接调 OLE ReadClassStm，从输入流读
CLSID，随后尝试创建对象，查询 IPersistStream（IID 常量
`0x809389c4`），并于 `0x808f33d2` 将原 IStream 传给
对象 Load。这一层无 ObjectStream 键，也无外层对象体长度。
图例类别对象体内部若再次读对象，其格式仍由该类的 Load
和原输入流接口决定，须单独核对。当前只确认 Variant 中对象头
的边界，非空图例类别和 ArcMap/QGIS 人工验收仍待验证。

#### TemporalLegendClass 的对象体

在 TrackingRenderer.dll 10.1.0.3035 中，TemporalLegendGroup Save
于 `0x808ffb9f`–`0x808ffbfb` 将类别数组中的对象作为 VT_UNKNOWN
写出。组内的兼容转换路径于 `0x808fff1d` 创建 TemporalLegendClass。
该类的 CLSID 为 `CB53C720-14A6-11D5-B7CE-00010265ADC5`，
GetClassID `0x808fc7e0` 从 `0x80938eb4` 复制该值。持久化虚表
`0x8093acf4` 的 Load 为 `0x808fc100`，Save 为 `0x808fc420`。

| 依序 | 流字段 | Load / Save |
|---|---|---|
| 1 | f64 版本、4 字节整数 | `0x808fc180`、`0x808fc1a0` / `0x808fc46e`、`0x808fc48a` |
| 2 | 2 字节存在标记，等于 `0xffff` 时附 Variant；加载 VT_UNKNOWN 时查询 ISymbol | `0x808fc1be`、`0x808fc205` / `0x808fc4bd`、`0x808fc509` |
| 3 | 两个字节 BSTR | `0x808fc274`、`0x808fc296` / `0x808fc526`、`0x808fc538` |
| 4 | 2 字节存在标记，等于 `0xffff` 时附 Variant；加载 VT_UNKNOWN 时查询 ILegendClassFormat | `0x808fc2b5`、`0x808fc2fc` / `0x808fc567`、`0x808fc5b3` |
| 5 | f64、4 字节整数、f64 | `0x808fc367`、`0x808fc388`、`0x808fc3aa` / `0x808fc5d2`、`0x808fc5ea`、`0x808fc602` |
| 6 | 版本至少 2.0 时追加 4 字节整数，阈值常量在 `0x8093d358` | `0x808fc3d6`–`0x808fc3eb` / `0x808fc619`–`0x808fc62f` |

两个 Variant 都调用同组共用的助手，因此对象分支从无键 CLSID
开始。子对象的 Load 仍接收原 IStream，接入前须核对符号和
LegendClassFormat 的对象体内部是否继续调用有键 IObjectStream。
上述读写顺序来自静态反汇编，尚无真实非空类别和软件人工验收。

#### 时态图例格式的内层对象流

Desktop 介质的 Layer.dll 和 AfCore.dll 均为 PE32、10.1.0.3035。
其 SHA-256 分别为
`fe52bd9988b66f9d6f906b014e5684f28fc20e9b97b53c90909fa3d44e9efed8`、
`031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`。
LegendClassFormat Load 在 `0x78664b8c/0x78664bae/0x78664bd5/0x78664bf3`
用原流调用 AfCore LoadObject（导出 `0x68a7b6d0`）。
该函数通过 `0x68a7c4b0` 查询 IObjectStream（IID 常量
`0x68c6dca8`），成功时在 `0x68a7b72c` 调虚表 `+0x44`。
查询失败时转到 `0x68a7b490` 读取 16 字节类标识，在非空路径
`0x68a7b530` 委派对象加载。

因此，在提供 IObjectStream 的 MXD 路径中，Variant 内的
LegendClassFormat 本体只有无键类标识，体内四槽仍按普通有键对象槽读取。
接入时可复用现有体读取器及原流登记表，外层 Variant 不登记新键。
普通 IStream 的回退路径、实际非空类别及软件人工验收仍待验证。

#### 时态图层 10.0 内联试读的接收者

TrackingLayer.dll 10.1.0.3035 的 Load、Save 在入口将
IPersistStream 指针存入 EBX（`0x8086ebe2`、`0x8086f973`）。
该接口在完整对象 `+0x10`，所以试读采用的 EBX 相对成员
`+0x2494` 实为完整对象 `+0x24a4`。此前追踪的 IFind
是 `0x808721c5` 设置的完整对象 `+0x2494`，其委派 QI
虽会回到外层对象，但不属于本次内联试读。

构造时 `0x80869f92` 把完整对象 `+0x24a4` 地址交给
`0x8087a8b0`。该助手创建 CLSID
`DD2FE1F0-224B-11D5-B7D5-00010265ADC5`（类型库中的
TemporalRecordSet），调用 OleRun，再将查询得到的 ITxFeatureClass
存入该成员。Load 于 `0x8086f266`、Save 于 `0x8086fdef`
读取该成员，经 `0x8087b6a0` 查询 IPersistStream。
Load 仅在外层版本至少 10.0 且 `0x8086f228` 的
ITemporalFeatureClass 查询结果为空时试读，失败时恢复原流位置。
Save 依据版本和 IPersistStream 可用性决定是否写体，未写独立存在标记。

Desktop 的 TrackingWorkspace.dll 为 PE32、10.1.0.3035，
SHA-256 `892c7b52d27d0f0c3d2f4ee8954273cc603e744d1ee79292183a4517eea21e8e`。
TemporalRecordSet 接口表 `0x80ad2f7c` 记录 ITxFeatureClass
于 `+0x0c`、IPersistStream 于 `+0x10`（表项 `0x80ad3078`）。
共用 GetClassID、Load、Save 为 `0x80a8ba40/0x80a8ba80/0x80a8bb00`。
Load 先在 `0x80a8baa1` 读两字节版本，仅版本 1 继续读取
四字节整数，随后调用 `0x80aac410`。Save 在文档版本索引
至少 4 时写出该前缀，并于 `0x80a8bb7a` 调用 `0x80aac360`。
内联子记录仍待闭合，尚无原生样本和人工验收。

#### 时态图层 10.0 的前置时间对象

Save `0x8086f86d` 取得文档版本索引，跳表 `0x808701b8`
对索引 0 返回失败、1 和 2 写外层版本 1.51、3 写 9.3，
大于 3 写 10.0。因此 ArcMap 10.1 当前文档版本索引 5
走外层版本 10.0 分支。

版本 10.0 的早期附加段依次为字节 BSTR、三个 short、
ITimeReference 对象、字节 BSTR、double、四字节 int、
两个 ITimeExtent 对象、short。读侧调用位于
`0x8086ece1`–`0x8086ee93`，写侧位于
`0x8086fa29`–`0x8086fb54`。三个对象槽只在查询到
IObjectStream 时存在，未取得接口时直接读写后续字段。
请求 IID `0x8089f020/0x8089f030` 通过 esriSystem.olb
对位到 ITimeReference 和 ITimeExtent，可复用现有两类读取器。
实际工程中的非空实现及 ArcMap/QGIS 人工验收仍待核实。

#### TemporalRecordSet 的内联子记录

TrackingWorkspace.dll 10.1.0.3035 的 Load `0x80a8ba80`
在 v1 的四字节值后调用 `0x80aac410`。子记录另起一个 short
版本，同样仅接受 1，随后读取 LONG、两个字节 BSTR、一个 short、
最后一个字节 BSTR。调用地址依次是
`0x80aac440/0x80aac460/0x80aac481/0x80aac49e/0x80aac4b0/0x80aac4d1`。
三个字符串共用 `0x80a61ee0`，读取四字节长度，非零长度包含
两个尾字节，范围为 2 至 `0x100000`。同版 Save
`0x80aac360`–`0x80aac402` 按同序写出，三个字符串共用
`0x80a61b00`。子记录没有对象头，最短完整体连同外层前缀为
26 字节。

原生外层在 `0x80a8bae4` 调用子记录 Load 后，于
`0x80a8bae9` 返回此前四字节读取的 HRESULT。因此原生
试读可能在子记录出错后仍保留部分流消费。Rust 读取器只有
在两个版本及全部字段验证通过时推进游标，避免将未证实的
偏移作为后续图层字段起点。缺省记录和不完整记录的外层
试读分派仍需随时态图层接入核对。用构造数据验证内联体、
后继字段、版本异常与截断，尚未在 ArcMap 或 QGIS 中人工验收。

#### 时态图例组和类别读取器

MXD 的 IObjectStream 路径读取类别、类别符号和类别格式的
VT_UNKNOWN 无键头。符号调用共用符号体分派，格式调用
LegendClassFormat 体读取器。两处调用以诊断占位键 0 构造体，
不向表登记该键，体内经原流读取的子对象照常登记。符号共用
体的原生依据见 [DisplayCore 符号槽](arcmap-10.1-display-slots-round3.md)；
格式的前述 AfCore 分派证据适用于相同原流。

构造记录验证非空类别、无键 SimpleLineSymbol、其有键 RGB
颜色、无键 LegendClassFormat 及后继边界。截断造成整个组
回退，包括已登记的嵌套键。未知 Variant 类型、未知无界对象类
或内层停止均返回错误。非 IObjectStream 路径与真实工程的
ArcMap/QGIS 人工验收仍待验证。

#### 时态图层前段读取器

TrackingLayer.dll 10.1.0.3035 的 Load 在
`0x8086eec1` 为版本至少 1.5 读取四字节值，在
`0x8086eef9/0x8086ef17` 为版本至少 1.2 再读两个 short，
随后在 `0x8086ef55` 加载 TemporalOperator。本轮实现
1.51、9.3 和 10.0 三个确认的写出版本，包含 10.0 的有键
时间对象槽及 TemporalOperator 两种长度。聚合 FeatureLayer
复用现有读取器，只有达到 TailCompleted 才提交游标和引用表。

构造测试逐个核对版本路径及操作器长度，验证抵达聚合体后仍能
读取其版本，聚合体截断时回退。此入口尚需连上外层后继段和
普通 ILayer 分派。ArcMap/QGIS 人工验收尚未执行。

#### 时态图层接入普通图层读取

TrackingLayer.dll 10.1.0.3035 的尾部 Variant 助手
`0x8087ca70` 以 `0x8087cc54` 的类型索引表分派定宽值、
ATL BSTR 和对象。对象分支 `0x8087c7d0` 在 `0x8087c818`
调用 ReadClassStm。当前接入定宽值、BSTR 和空对象标记。
非空尾部 Variant 实现仍需逐槽对位。

聚合体之后读取有键时态图例组与时间参考，并在 10.0 路径
试读 TemporalRecordSet 的两个 v1 版本和完整字段。无运行时
接口查询结果可供静态解析，因此记录试读以其可完整解析为条件。
普通无长度 ILayer 仅在后继字段全部读出后提交，有外层长度的
条目还要求恰好消费其体。图层名称、可见性和数据源沿用聚合
FeatureLayer。构造测试覆盖三个版本的尾部边界和截断。
真实时态图层样本、少见旧版本、非空尾部 Variant 以及
ArcMap/QGIS 人工验收仍待核实。

#### 时态图层尾部的文本符号和颜色

TrackingLayer.dll 10.1.0.3035 在 `0x8086f4c2` 和
`0x8086f515` 分别查询 ITextSymbol 和 IActionCollection。
后者的常规创建点 `0x808728e2` 使用类型库中 ActionCollection
的 CLSID `4C48C64E-9199-11D5-B7FE-00010265ADC5`。
9.3 以后最后三个 Variant 共用 `0x8087ab50` 转换，
其首个目标接口是 IColor。前两个来源成员在
`0x8086a0b6` 和 `0x8086a142` 附近创建 RgbColor
（CLSID 常量 `0x8089eb24`），第三个成员可为空。

读取器复用 TextSymbol 和现有颜色体读取器。无键 Variant
保留内部对象槽对原引用表的访问。写侧 `0x8086ffeb` 对
空的第三成员使用 VT_EMPTY，通用写侧 `0x8087ced1`
把此类 Variant 转成并写出字节 BSTR。构造测试核对无键
RGB 颜色、空 Variant 的边界及颜色截断。ActionCollection
对象体与其他实现类仍待研究，未进行 ArcMap/QGIS 人工验收。

#### ActionCollection 的集合体

Desktop 的 TrackingActions.dll 为 PE32、10.1.0.3035，
SHA-256 `b12b5e3ac5bf92a24e8bd6d9f7847c33ac2b659351f2533abcff97837c7f0e74`。
CoActionCollection 的 IPersistStream 在完整对象 `+0x18`，
CComObject 虚表 `0x80592334` 的 GetClassID、Load、Save 分别为
`0x804da890/0x804da8d0/0x804daa50`。`0x8052b640` 也返回该
CLSID，但属于 ClassID 辅助对象。

Load 调 `0x804b7650`，按顺序读取 double 版本、四个字节
BSTR、double、int 和两个字节。Save 共用对称助手
`0x804b7730`。BSTR 读助手 `0x804b8330` 验证非零长度
含两尾字节且不超过 `0x100000`。版本至少 1.0
（阈值在 `0x80596a48`）时，Load 在
`0x804da937/0x804da949/0x804da958/0x804da968` 再读四个
四字节值，末值为有符号计数。正计数逐项调用 Variant 助手
`0x804c41f0`，其中对象分支经 `0x804c4050` 在
`0x804c409e` 调 ReadClassStm。非空 VT_UNKNOWN 对象在
`0x804da9e4` 查询 IAction。

时态图层的无键 ActionCollection 现在可消费集合字段和空条目，
未知动作体返回明确停止错误。构造测试覆盖非空集合对象的
后继字段边界、未知动作和截断。实际动作类体及真实工程在
ArcMap/QGIS 中的人工验收仍待完成。

#### ActionCollection 的嵌套集合

TrackingActions.dll 10.1.0.3035 的 CoActionCollection 接口表在
`0x80590de0` 将 IAction（IID `CC018A6E-24FB-11D4-B34C-00104BA2ABCC`，
常量 `0x8058d9a4`）映射到完整对象 `+0x10`。
集合循环 `0x804da9e4` 查询的正是该接口。因此无键动作项可为
另一个 ActionCollection，其对象体仍按相同的版本和计数递归读取。
读取器限制最多 16 层，构造测试验证限额内的后继边界及超过限额
的错误。其他 IAction 类体与软件人工验收仍待验证。

#### 邮件告警动作的持久化入口

TrackingActions.dll 10.1.0.3035 的
`CComObject<CoEmailAlertAction>` 通过 RTTI 绑定到完整对象
`+0x22c` 的 IPersistStream 虚表 `0x80593d34`。
GetClassID `0x804fa210` 读常量 `0x8058fb08`，
与登记的 EmailAlertAction CLSID
`2719A8FA-AD90-4E69-8DB2-20224D4A1A6A` 一致。
Load 槽 thunk `0x8052cd00` 调整 this 后进入 `0x804ec2a0`，
Save 槽 thunk `0x8052def0` 进入 `0x804fa250`。

两侧分别在 `0x804ec2e7/0x804fa29b` 对成员查询得到的
IPersistStream 传递原 IStream，随后通过另一个成员虚表
于 `0x804ec2f4/0x804fa2b2` 再次委派。调用之间无
外层 CLSID、键或长度。该类内层边界需逐个绑定委派接收者
后才能接入集合。初始化另外存在 GeometryBag 创建路径
`0x804faaa4`，该步骤本身只确认运行时成员身份，尚未
证明 GeometryBag 对应上述任一流段。

对照的 ActionBase 持久化 Load thunk `0x8052c5f0`
经 `0x804da270` 调 `0x804c0240`，Save 包装器
`0x804da2d0` 调 `0x804c0770`。双方先消费与动作集合
共用的设置前缀，再分别调用 `0x804d7900/0x804d7a10`
及 `0x805129a0/0x80512c00`。基类 Load 此后比较
9.3 和 9.2（常量 `0x80596b28/0x80596b20`）。
这些是进一步核对共享体的入口，尚未确认
EmailAlertAction 中的内联记录完整边界，未做软件人工验收。

#### 邮件告警动作的两个内联接收者

TrackingActions.dll 10.1.0.3035 的 Load thunk `0x8052cd00`
从完整对象 `+0x22c` 减去 `0x220`。因此
`0x804ec2a8` 的接口相对 `+0x248` 是完整对象
`+0x254`。邮件告警初始化在 `0x804f9f6d`–`0x804f9f83`
以该地址创建 ActionBase（CLSID 常量 `0x8058ee50`，
`EB7DAC60-5052-49EF-9F70-E1484557E419`），请求 IUnknown。
第一段即 ActionBase 的内联持久化体。

第二次调用在 `0x804ec2e9`–`0x804ec2f4` 将接口
指针减去 `0x0c`，使用邮件告警对象自身的主虚表。
构造入口 `0x80525990` 在 `0x8052599c` 安装主虚表
`0x80593c6c`，其 `+0x14/+0x18` 指向邮件字段
Load `0x804f9ac0` 和 Save `0x804f9cd0`。

这段的读写顺序已对位到八字节值、IMessageTemplate Variant、
两个字节 BSTR、四字节项目数及正计数循环。前四项读侧
`0x804f9af9/0x804f9b51/0x804f9b80/0x804f9b8c`，
写侧 `0x804f9d00/0x804f9d76/0x804f9d8b/0x804f9da0`。
计数写于 `0x804f9de5`，读侧的循环在 `0x804f9c42`
和 `0x804f9c4e` 各调用一个助手，后者为字节 BSTR。
循环首助手的流宽、IMessageTemplate 的实现类以及 ActionBase
完整流边界尚待核实。尚无真实记录与软件人工验收。

#### 邮件地址与消息模板的记录边界

TrackingActions.dll 10.1.0.3035 的邮件地址循环每项只读
一个字节 BSTR。Load 的 `0x804f9c42` 先调用 `0x804c83c0`
清理内存字符串，该调用没有 IStream 参数。实际读流在
`0x804f9c4e`，写侧对应 `0x804f9f0f`。因此计数之后
按条目连续读取 BSTR 即可。

Save 在 `0x804f9d43` 为缺省模板创建 MessageTemplate，
使用 CLSID `A7B7EF92-36AE-4318-A122-431B8892E0F3`，
并在 `0x804f9d76` 以 VT_UNKNOWN 写入。该类 GetClassID
`0x80535b00` 与 IPersistStream 虚表 `0x80593f68` 的
Load `0x805353b0`、Save `0x80535620` 相连。
模板体依次为 8 字节值、一个字节 BSTR、4 字节值、四个
字节 BSTR、4 字节计数、计数项循环。读侧定宽字段分别在
`0x80535404/0x80535435/0x805354b7`，写侧对应
`0x80535668/0x80535691/0x80535702`。每项由
`0x8053557f/0x80535593` 读取 BSTR 与 4 字节值，
对应 `0x80535800/0x80535816`。消息模板 Variant
的无键类标识沿用集合中已核对的 Variant 助手。
ActionBase 的完整边界仍需闭合后才能连上邮件告警动作，
尚无真实 MXD 或软件人工验收。

#### ActionBase 的图层条件与兼容前缀

TrackingActions.dll 10.1.0.3035 的 ActionBase Load
`0x804c0240` 在共用设置前缀后调用 `0x804d7900`。
该助手在 `0x804d794c` 查询 IObjectStream，有接口时于
`0x804d7971` 走虚表 `+0x44`，请求 ILayer
（IID 常量 `0x8058e3d0`）。无接口时改走 `0x804d799a`
的 Variant 助手。两个分支随后都在 `0x804d79cd` 读
一个四字节整数。Save `0x804d7a10` 在
`0x804d7a8b` 写有键对象，或在 `0x804d7a9c` 写
Variant，再于 `0x804d7aae` 写四字节值。

下一助手 `0x805129a0` 通过 IStream::Clone 或记录位置后
Seek 回退试探首个 LONG。若值为 `0x000bf894`，原流
在 `0x80512a60/0x80512aa7/0x80512aed` 读取该标记和
后续两个四字节值。之后固定读取两个字节 BSTR、两个四字节值。
Save `0x80512c00` 在文档版本索引大于 4 时才写
标记与两值（`0x80512c65/0x80512c9c/0x80512cca`），
再写固定尾段（`0x80512cee`–`0x80512d54`）。

返回 ActionBase 后，版本至少 9.3 时先读 short 标记，
随后读无键 IPropertySet Variant（`0x804c0362/0x804c0396`）。
随后存在随版本变化的字符串与计数段。成功的 IObjectStream
路径在 `0x804c0542` 还有一个 ILayer 槽，部分分支在
`0x804c0582` 再读 IName。这里的 ILayer 与前缀中的
ILayer 属于不同字段。字符串分支和计数段仍待与 Save 对位，
尚未接入解析或完成软件人工验收。

#### ActionBase 的末段字符串和计数记录

TrackingActions.dll 10.1.0.3035 的 ActionBase Load 在
对象版本至少 1.0 时才进入末段（`0x804c03d6`）。
首先读一个字节 BSTR（`0x804c03ed`），版本至少 9.2 再读
一个（`0x804c0421`）。较早版本调用的 `0x804b1f60`
直接复制前一个内存字符串，不读取流。文档版本索引为 0
时再读一个兼容 BSTR（`0x804c0452`），随后固定读取三个
BSTR（`0x804c0465/0x804c047b/0x804c048d`）。
Save 对应调用为 `0x804c0921/0x804c0947/0x804c099d`
和 `0x804c09d0/0x804c09e8/0x804c0a00`。

随后内联助手 `0x804c2960` 在 `0x804c29b4` 读取
四字节有符号计数，正计数时在 `0x804c29f3` 逐项读取
一个字节 BSTR。对称写助手 `0x804c2840` 在
`0x804c2899` 写计数，并于 `0x804c2907/0x804c2923`
逐项写长度和字节。末段计数之后是 1、4、4 字节三个
标量（读 `0x804c04b4/0x804c04cf/0x804c04ea`，
写 `0x804c0a32/0x804c0a58/0x804c0a74`）。
对于提供 IObjectStream 且所有调用成功的路径，接着读写
ILayer 槽。文档索引非零时再读写 IName 槽
（Load `0x804c0542/0x804c0582`，Save
`0x804c0aaf/0x804c0acf`）。此后 Load 的分支用于恢复
运行时状态。

共用设置前缀的保存跳表 `0x804b780c` 以
`0x8053fd40` 调整后的版本索引为输入，0/1 对应对象
版本 1.0，2 对应 9.2，3 对应 9.3，4 及以上对应 10.0。
因此文档索引与对象自身版本需要分别处理。真实工程中的
非空 IName 实现与 ArcMap/QGIS 人工验收仍待验证。


#### 邮件告警动作接入集合

TrackingActions.dll 10.1.0.3035 在 `0x804c036b`–`0x804c037b`
把 short 转为条件状态标志，`0x804c0396` 的 IPropertySet
Variant 读取只受前一步读取结果控制。写侧 `0x804c0833`
和 `0x804c08ce` 也在成功路径依次写标志和 Variant。
因此标志为零时依然消费 Variant。

读取器接入 ActionBase 9.3 与 10.0 的 MXD IObjectStream
写出路径，两者均走非零文档版本索引布局。图层和数据集名称
复用现有已闭合分派。IPropertySet 与 MessageTemplate 的
无键对象头直接读取，体内对象引用继续使用原表。构造集合
含非空邮件告警、非空消息模板和收件地址，校验后继偏移及
截断。较早对象版本仍需文档索引或真实记录佐证，尚未在
ArcMap 或 QGIS 中人工验收。

#### 数据修改动作的内联记录

TrackingActions.dll 10.1.0.3035 的
`CComObject<CoDataModificationAction>` 持久化虚表
`0x80593a1c` 位于完整对象 `+0x22c`。
GetClassID `0x804ec260` 从 `0x8058f97c` 返回
`D7E27D9F-7810-4410-A9CA-878F42C7A1B2`。
Load 和 Save 复用邮件告警的两段委派包装器。构造侧
`0x804ebfc0`–`0x804ebfd8` 将 ActionBase 创建到
同一个完整对象偏移 `+0x254`。第二段经主虚表
`0x80593964` 进入本类 Load `0x804ebee0` 和 Save
`0x804ebf50`。

第二段依次为八字节值与一个字节 BSTR。Load 调用点
`0x804ebf07/0x804ebf2c`，Save 对应
`0x804ebf67/0x804ebf79`；写侧八字节常量
`0x805bd0e8` 为 double 1.0。读取器复用已有的
9.3 与 10.0 ActionBase 路径，构造测试核对非空 BSTR、
下一动作 CLSID、集合后继字段和截断。真实工程的
ArcMap/QGIS 人工验收尚待验证。

#### 数据分区动作的内联记录

TrackingActions.dll 10.1.0.3035 的 DataPartitionAction
GetClassID `0x804f2bf0` 返回常量 `0x8058f9ec`，
即 `65AB5EA8-DC59-4C7F-87A1-AD9FCD70EA40`。
Load 包装器 `0x804f2c30` 依次委派给构造点
`0x804f2908` 创建的 ActionBase（完整对象 `+0x25c`）
及主虚表上的本类 Load `0x804f25d0`。
本类 Save 为 `0x804f2800`，写出 double 10.1
（常量 `0x805bd0e0`）。

本类字段依次为 double 版本、两个字节 BSTR、一个字节、
有符号计数及每项两个字节 BSTR、另一个有符号计数及每项
一个四字节整数。数组读写助手为 `0x804f3150/0x804f2f50`。
版本至少 10.1 时，读侧 `0x804f267a` 放行追加字段，
`0x804f269e` 读无键 Variant，非空时查询 ISpatialReference，
随后在 `0x804f27ba/0x804f27c0` 读两个字节 BSTR。
相应写调用在 `0x804f2886/0x804f2891/0x804f289c`。
读取器复用已知坐标系体和 9.3、10.0 ActionBase 路径。
构造测试覆盖两种尾部分支、非空数组及截断，真实工程的
ArcMap/QGIS 人工验收仍待完成。

#### 数据汇总动作的内联记录

TrackingActions.dll 10.1.0.3035 的 DataSummaryAction
IPersistStream 虚表 `0x805941f8` 中，GetClassID
`0x804f4860` 返回 `A5AF3B32-3506-4115-9C6E-E325E5C3DECD`。
Load/Save 包装器 `0x804f48a0/0x804f4920` 先调用
完整对象 `+0x258` 的 ActionBase（构造点 `0x804f45dc`），
再通过主虚表 `0x805940d4` 的 `+0x24/+0x28`
进入本类 `0x804f4190/0x804f43f0`。

本类先消费 double 版本和四个四字节值。读侧计数助手
`0x804f5290` 在 `0x804f5329` 读四字节计数，正计数
每项在 `0x804f533d`–`0x804f538b` 读 BSTR、四字节值
及四个单字节。写侧助手为 `0x804f4f80`。其后固定
七个 BSTR 和三个单字节，版本至少 1.0 再读一个 BSTR
（读门 `0x804f439d`、写门 `0x804f4534`）。
Save 写 double 9.3（常量 `0x805bd0d8`）。

动作集合接入上述边界，构造测试覆盖非空计数项、两种末尾
版本路径、后继字段及截断。真实工程中的字段语义及
ArcMap/QGIS 人工验收仍待验证。
