# ArcMap 10.1 Hyperlink 族体：coclass Hyperlink（C++ FeatureHyperlink）

记录日期：2026-11-15。本轮闭合 FeatureLayer::Load 后继字段段 IHyperlink 计数
循环（请求 IID `DAT_6f7ac2a4={3036D35D}`，第五轮类型库定名 IHyperlink）的元
素类体。全部为静态取证（本机 pyghidra，Ghidra 12.2 + JDK 25，打开既有 FL 工
程副本），未运行 ArcMap 10.1，未人工验证。样本 `samples/测试底图.mxd` 的
/Maps 流中 FL 后继段 IHyperlink 计数恒 0（首 FeatureLayer 体 `0x5de`=0，其
余层未逐一走查但第五轮记录未见非空实例），本体**无样本字节佐证**，字段序全
部来自反编译，经 Save 侧对称与版本常量字节互证；合成输入测试覆盖
（`tests/hyperlink.rs`）。本文无本机绝对路径；地址均为 FeatureLayer.dll
10.1.0.3035 静态映像 VA。

## 实现模块归属更正（证据：确定的）

- `esri.clsid.ecfg` 声明 `{3036D35E-EDE5-11D0-87FE-080009EC732A}` progID
  `Hyperlink`，模块 **FeatureLayer.dll**（esriCarto 前缀段）。
- 五轮记录「`{3036D35F}` 等同族常量仅存在于 Map.dll」按字节复核只对
  `{3036D35F}`（IFeatureBookmark）成立：`{3036D35E}` 与 `{3036D35D}` 在
  FeatureLayer.dll 各 1 处（`DAT_6f7ac2b4`/`DAT_6f7ac2a4` 相邻，前者即本
  coclass CLSID、后者为 FL::Load 计数循环的请求 IID），Map.dll/Layer.dll
  数据段两者皆无；Map.dll 中 `{3036D35F}` 2 处与 `.?AVFeatureBookmark@@`
  RTTI（第五轮已载）互证书签类在 Map.dll。三个 GUID 分属两个实现模块，
  不构成「同族同模块」。
- esriCarto 类型库：接口 IHyperlink `{3036D35D}`（#389，typekind=3）、枚举
  esriHyperlinkType `{1558D2D3}`（#390）、coclass Hyperlink `{3036D35E}`
  （#506，typekind=5）。名称↔GUID 绑定为类型库文件内静态数据；接口成员
  语义该 dump 格式未含，本文字段语义全部标候选。

## 类身份与定位链（证据：确定的）

- C++ 实现类名 **FeatureHyperlink**（RTTI `.?AVFeatureHyperlink@@`，TD
  `0x6f7d2780`，名字字符串在 TD+8；类名与 progID `Hyperlink` 不同——与
  FdbFeatureClassName 同款的命名差异，不影响流内类身份）。另有
  `ClassFactory<FeatureHyperlink>` RTTI（工厂在 FeatureLayer.dll，与 ecfg
  模块声明一致，无「工厂宿主/声明模块」分离）。
- RTTI TD `0x6f7d2780` → 4 个子接口 COL（`0x6f7b6678`/`0x6f7b668c`/
  `0x6f7b66a0`/`0x6f7b66b4`，判据 [C]==0 且 [C+0xc]==TD）→ 虚表
  `0x6f7ac284`（首 COL 前置）。虚表槽字节直读（无调整器，全类 this=0）：
  +0x0c = GetClassID `0x6f6d9310`、+0x10 = IsDirty `0x6f6a7d00`、+0x14 =
  Load `0x6f6d9170`、+0x18 = Save `0x6f6d9210`、+0x1c = GetSizeMax
  `0x6f6d92e0`。
- 三方互证：ecfg 声明 ↔ GetClassID `0x6f6d9310` 以立即数对
  `0x11D0EDE53036D35E`/`0x2A73EC090008FE87` 写出 `{3036D35E}` ↔ 虚表槽指
  向。Load/Save/GetClassID 的引用除本虚表外无其他（引用扫描核对）。

## Load `0x6f6d9170` 字段序（证据：确定的——反编译 + Save 对称互证）

版本门 `1 < (short)ver` 有符号拒绝 `0x80004005`；Save `0x6f6d9210` 写静态
版本常量 `DAT_6f77ba84`（字节 `01 00`）=1，对称。

| # | 字段 | 版本门 | 对象偏移 | 语义 |
| --- | --- | --- | --- | --- |
| ① | int16 版本 | 恒有（>1 拒绝） | — | 负值位模式原生过门；读取器拒绝（更严） |
| ② | int32 | 恒有 | this+0x4 | esriHyperlinkType 枚举候选（值序未证） |
| ③ | AfCore String::Read | 恒有 | this+0x8 | 链接目标候选（URL/文档路径） |
| ④ | int32 | 恒有 | this+0xc | 语义未定 |

体 = short + int32 + String + int32；无嵌套对象条目、无版本分支字段（版本
门只做拒绝）。Save 侧有**写向专用**规范化：docVersion <3 时经 `FUN_6f6d9350`
若链接非 URL 且含「?」则截断至「?」之前（`AfPathIsURL` + `String::Find` +
`String::Left`）；读侧无对应分支，10.1（docVersion=3）写满形态。

## 实现（src/）

新模块 `src/objects/hyperlink.rs`：`HYPERLINK_CLASS_ID` 常量 +
`read_hyperlink_body_10_1(reader, max_bytes, table, key)`（严格事务子集：
版本门有符号 + 负值位模式拒绝、String 受 `max_bytes` 限额、失败不推进游标
不修改登记表；无嵌套对象，`table` 仅在签名上保持一致）。

**接线建议（写给整合轮）**：`src/layers/feature_layer_tail.rs` 后继字段段
读取器 `read_feature_layer_successor_10_1` 第 ⑤ 步 IHyperlink 计数循环当前
对新对象一律 `HyperlinkNewObject` 停止回退；接入方式与 FieldInfo 条目分派
同构——循环条目为 `NewObject { class_id, key }` 且
`class_id == HYPERLINK_CLASS_ID` 时调 `read_hyperlink_body_10_1` 整体解析
（解析成功即继续循环；体内无嵌套停止点，失败路径只有整体报错），其余类维
持现有停止。`HyperlinkSlot10_1` 需新增携带体的变体（如
`Hyperlink(Box<HyperlinkBody10_1>)`；该枚举当前为 `Copy`，加体变体需去
`Copy` 或用 `Box`+显式匹配）。样本计数恒 0，接线后样本行为不变（无字节佐
证），仅合成测试覆盖。

## 测试

`tests/hyperlink.rs` 6 项：类标识字节互证、版本 1 满体（int32+链接+int32，
尾部字节不消费）、版本 0 按门接受、版本 2/负值位模式拒绝（游标与登记表不
动）、String 限额拒绝、逐前缀截断事务性。全部为合成输入（样本计数恒 0 无
实例），通过不代表 MXD 格式已验证。

## 未解决事项

1. ②④ int32 与 ③ String 的业务语义未定性：esriHyperlinkType 枚举值序、
   ② 是否即 Type、③ 与 IHotlinkContainer 热链字段的区分，均无样本佐证；
   ④ 语义完全开放。
2. 版本 0 短体（版本门之下唯一变体）无样本；`FUN_6f6d9350` 的「?」截断仅
   按反编译记录，未运行验证。
3. IHyperlink 循环条目的空标记（16 字节零）与回引形态按 ObjectStream 通用
   语义处理，无 Hyperlink 特有字节佐证。
4. 全部结论为静态取证；未运行 ArcMap 10.1，未人工目视验收，不声称已验证。
