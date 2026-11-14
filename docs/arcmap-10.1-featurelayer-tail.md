# ArcMap 10.1 FeatureLayer 尾段链第一环：渲染器及直接后继类

记录日期：2026-11-14。对应 TODO「MXD 解析」节开放项中的「越过条目 1 需渲染器
/LegendGroup/FieldInfo/EditTemplate 等约 30 类尾段链闭合」——本轮闭合链条第一
环（SimpleRenderer 及其嵌套的符号、颜色、图例组、图例类共 5 类）。全部为静态
反编译观察，未运行 ArcMap，未做任何人工目视验收；实现见 `src/layers/feature_layer_tail.rs`
（模块文档含同源字段表）。

## 样本边界（先声明）

仓库参考样本 `samples/测试底图.mxd` 的图层列表条目 1（FDOGraphicsLayer）体内，
关联 FeatureLayer 的渲染器槽新对象（SimpleRenderer，键 35）位于 `/Maps:0x3ed`。
本轮闭合的 5 类全部有样本字节佐证：`0x3ed..0x602` 逐字段吻合（辅助观察，见
下文「样本对位」）。链条在此之后仍未闭合。

## 方法与文件身份

- REA MCP（Ghidra 12.2 提供者，JDK 25）反编译 + 样本字节对位（`temp/r6-maps.bin`，
  `cargo run -- hex` 与自写脚本）。与并行代理共享 REA 服务器，活动目标被切换
  后以「open + 操作同批」重试恢复。
- 本轮新提取 **DisplayCore.dll**（Setup1.cab，符号/颜色实现）：SHA-256
  `9b3e61f9…a926e4d2`，pefile 复核 `10.1.0.3035`。
- 复用既有基线：FeatureLayer.dll、Layer.dll（SHA-256 与第六轮记录一致）、
  LayerLib.dll。类名/接口名：`esri.clsid.ecfg` 与 10.1 类型库。
- 辅助脚本与提取件在 `temp/agent-fl-tail/`。

## 一、SimpleRenderer（链条入口，样本键 35 @0x3ed）

- 类身份三方互证：`esri.clsid` 声明（模块 FeatureLayer.dll）、GetClassID
  `0x6f722e00` 写出 `{F3435801-5779-11D0-98BF-00805F7CED21}`、样本 `0x3ed`
  对象头逐字节相同。
- IPersistStream 虚表 `0x6f7b0dfc`：槽 +0x14 = 调整器 thunk `0x6f724cf0` →
  实体 Load `0x6f722b70`；Save `0x6f722cf0`（对称）。
- 字段序：
  1. int16 版本：`3 < (short)ver` 有符号拒绝（样本 3）；
  2. LoadObject（ISymbol `{F3435802-5779-11D0-98BF-00805F7CED21}`）——样本为
     SimpleMarkerSymbol 新对象（键 36）；
  3. LoadObject（ILegendGroup `{167C5E9F-…}`）——样本 LegendGroup 新对象
     （键 39）；
  4. LoadObject（ILegendItem `{52AA5BC0-…}`）——样本 16 零空标记；
  5. int16 → obj+0x128（样本 0）；
  6. LoadLegacyRotationFieldAndType（LayerLib.dll `0x786f93d0`，本类调用不带
     版本 short）：String 旋转字段 + int32 旋转类型（样本空串 + 0）；
  7. String → obj+0x12c 标签（样本空串）；
  8. 版本 >2：RotationRenderer::Load（LayerLib.dll `0x786f7e40`：short 版本
     ≤1 + int32 ×2 + String ×3 + double ×6）+ SizeRenderer::Load
     （LayerLib.dll `0x7870bbc0`：short 版本 ≤1 + int32 + String + double ×2）。
- 写侧：Save 对称，版本表 docVersion→3；docVersion<4 时省略第 8 段（与读侧
  版本 >2 门在 10.1 一致）。

## 二、SimpleMarkerSymbol（样本键 36 @0x403，DisplayCore.dll）

- GetClassID `0x6ebbf310` 写出 `{7914E5FE-C892-11D0-8BB6-080009EE4E41}`；
  IPersistStream 虚表 `0x6ec15724`：+0x14 thunk `0x6ebc0460` → Load
  `0x6ebbefa0`；Save `0x6ebbf190`。
- 字段序：① short 版本（>2 拒绝；样本 2）→ ② LoadObject（IColor
  `{20CD40B0-…}`）→ ③ double（样本 6.0，尺寸候选）→ ④ int32 ×3（样本
  0/13/0，13 为 marker 样式候选）→ ⑤ double ×3（样本 0）→ ⑥ 1 字节（样本 1）
  → ⑦ double（样本 1.0）→ ⑧ LoadObject（IColor，轮廓色）→ ⑨ 版本 >1：short
  （样本 0xFFFF）。
- 构造中的 `FUN_6eb406f0` 两处调用不读流（虚调用返回值），无流内字节。

## 三、RgbColor（样本键 37/38/154，DisplayCore.dll）

- GetClassID `0x6eb3d930` 写出 `{7EE9C496-D123-11D0-8383-080009B996CC}`；
  IPersistStream 虚表 `0x6ec0c70c`：+0x14 thunk `0x6eb3dbf0` → Load
  `0x6eb3d8b0` → Color 核心 `0x6eb35310`；Save `0x6eb3d8f0`/`0x6eb35470`。
- 字段序：① short 版本 ≤1 → ② 第二个 short 版本 ≤1（Color 核心）→ ③ 1 字节
  分支标志：==0 → 3 个 double（经 Color 虚表 +0x34 装载，样本三实例 flag 均
  0）；!=0 → 4 字节 CMYK（虚表 +0x1c）→ ④ 1 字节 → ⑤ 1 字节（内存值按
  `0xff - 流内字节` 取补）。
- flag==0 分支体长 31 字节，与样本三实例（键 37 @`0x419`、38 @`0x481`、
  154 @`0x2c0b`）逐字节吻合。

## 四、LegendGroup（样本键 39 @0x4b6，Layer.dll）

- GetClassID `0x78665c80` 写出 `{167C5EA2-AF20-11D1-8817-080009EC732A}`；
  IPersistStream 虚表 `0x7866fa3c`：+0x14 Load `0x78665cc0`，+0x18 Save
  `0x78665e20`。
- 字段序：① short 版本（>3 拒绝；样本 3）→ ② short → obj+0xc（样本 -1）→
  ③ 版本 ≥2：short → obj+0xe（<2 时内存默认 -1；样本 -1）→ ④ String →
  obj+0x14（样本空串）→ ⑤ int32 计数 + 循环 LoadObject（ILegendClass
  `{167C5E9E-…}`）（样本 1 → LegendClass 键 40）→ ⑥ 版本 >2：short →
  obj+0x12（样本 0）。
- 写侧：版本表 docVersion→3；docVersion>4 时才写第 ⑥ 段（读侧按版本 >2 门，
  10.1 两者一致）。

## 五、LegendClass（样本键 40，Layer.dll）

- GetClassID `0x78662fe0` 写出 `{167C5EA3-AF20-11D1-8817-080009EC732A}`；
  IPersistStream 虚表 `0x7866f728`：+0x14 Load `0x78663560`，+0x18 Save
  `0x78663640`。
- 字段序：① short 版本（反编译未见上限拒绝，样本 2；实现按 ver>1 门分支、
  不设额外上限）→ ② LoadObject（ISymbol）——样本为对 SimpleMarkerSymbol
  键 36 的回引头，无体 → ③ String 标签（样本空串）→ ④ String 描述（样本
  空串）→ ⑤ LoadObject（ILegendClassFormat `{52AA5BC2-…}`）——样本 16 零
  空标记；新对象体未闭合 = 停止点 → ⑥ 版本 >1：int32 → obj+0x1c（样本 -1）。

## 样本对位（辅助观察）

`temp/r6-maps.bin` `0x3ed..0x602` 五类嵌套体逐字段吻合：SimpleRenderer 体
终结于 `0x5b0`；其后回到关联 FeatureLayer 的后继字段——`+0x144` GUID（16 零）、
String「BSM」、double ×2、short、IHyperlink 计数 0、IColor 空标记、String ×2、
int32，到 `0x602` 的 `{A2BAAE2C-…}` 计数循环（FieldInfo 族）之前。该循环及其
后的类不在本轮范围（`feature_layer` 读取器当前在 `+0x144` GUID 后即停止，
后继字段的字段序闭合属下一步集成范围）。

## IID 表更正

第三轮研究文档（docs/arcmap-10.1-map-object.md）IID 表中 `{F3435802-…}`
（ISymbol）的 Data2 误记为 5799；经本轮二进制字节与类型库双重复核应为
**5779**（与 SimpleRenderer CLSID 同 Data2）。

## 未解决事项

- ILegendItem 槽（`{52AA5BC0}`）coclass 体未闭合（样本空标记，无字节佐证），
  新对象即停止；
- ILegendClassFormat 槽（`{52AA5BC2}`）coclass 体未闭合（样本空标记），
  LegendClass 在此停止；
- ISymbol 槽只闭合 SimpleMarkerSymbol；SimpleLineSymbol / SimpleFillSymbol /
  MultiLayerMarkerSymbol 等同族未闭合（新对象即停止）；
- IColor 槽只闭合 RgbColor；CmykColor/HlsColor/HsvColor/GrayColor 同族未闭合；
- RotationRenderer/SizeRenderer 的 double ×6（样本前 3 个 0、后 3 个 360.0）
  语义为候选（「3D 随机旋转上下限」），对称写侧 getter 未逐个定名；各
  int32/String 业务命名按内容推断（style 候选等）；
- FeatureLayer 渲染器槽之后的后继字段段（`+0x144` GUID 至 FieldInfo 循环）
  的字段序闭合与接入未做；
- 全部结论为静态观察，未经目标软件运行验证；样本对位仅为辅助观察。
