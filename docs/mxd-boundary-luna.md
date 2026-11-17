# FeatureLayer PreJoin 与后继六字节边界复核

日期：2026-11-16。对应 `TODO.md` 第 240 行。本记录更新本轮边界重建和修复，未修改 TODO、README、公共模块入口或 FDO 代码。结论依据 ArcMap 10.1.0.3035 静态原生读写实现，样本用于辅助逐字节对位；未在 ArcMap/QGIS 中人工验收。

## 原生文件身份与读写证据

从 README 所列 Desktop `Setup1.cab` 提取的 FeatureLayer.dll，经 REA/Ghidra 12.1.4 以 PE x86 打开：SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`，版本资源为 `10.1.0.3035`，与 [原生文件基线](arcmap-10.1-native-baseline.md) 一致。用于确认委派实现的 LayerLib.dll SHA-256 为 `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`；由同一 CAB 提取，未独立复核其版本资源或签名。

以下均为本地目标文件的 `shipped-artifact` 静态证据，地址为 PE 静态映像地址：

- FeatureLayer.dll `FeatureLayer::Save` 汇编 `ev_2ad683c373db5c04b0cd0b4feb23d671197fb1d803777a90d44e6487c07f81ee`：在调用委派跳板 `0x6f73fec2` 成功后，紧接着对对象 `+0x2c0` 执行 4 字节写入。跳板指令 `JMP [0x6f74e510]` 见 `ev_0f44f694aae276f87582332c390b5ab184d809e56a74184b792a6ef2aba49ddf`。
- LayerLib.dll `CInternalLayerBase::WritePreJoinDefExpressions`，`0x786c5cb0`：完整汇编 `ev_dbc83c8a823582d6837b516851dd2b4b049bfd0f0c861de0605ac8bcb2e9075e` 显示先写 4 字节计数，随后每项写两个字符串；计数写入与字符串循环之间没有六字节写操作。名字检索证据为 `ev_c5e24748e7e5cad3821d67dc12645f18c29f04698071f25628aa49992383ab41`。
- FeatureLayer.dll `FeatureLayer::Load` 汇编 `ev_b63e7941e8cf7d15d2ebfcd7f6637beaab840c10d9d4dd439eb1db11a10d3f56`：调用委派跳板 `0x6f73febc` 后，对对象 `+0x2c0` 请求 4 字节读取，中间没有 FeatureLayer 流读取。
- LayerLib.dll `CInternalLayerBase::ReadPreJoinDefExpressions`，`0x786c5d30`：完整反编译 `ev_a0ce5ce1100e9bca56984188da5557c1e184d1fc85af32b16d931e918f4d94f8` 显示读取 4 字节计数，正计数时每项恰读两个字符串，无额外六字节读取。

据此，原生 PreJoin 体布局为 `[int32 count][count × (String, String)]`；`+0x2c0` 标量紧随 PreJoin 体，不属于 PreJoin 保留头。

## 更正：UID 尾部及参考样本边界

撤销本记录前一版对 `0x14ee`、`0x14f2` 和四个字符串的字段归属：该表与同页 hex 明显矛盾，不能作为证据。此前把 UID 对象体认作 19 字节的判断也不成立。

SystemCore.dll 10.1.0.3035（SHA-256 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`）的 UID 持久化虚表 `0x80074574` 中，GetClassID、Load 和 Save 分别为 `0x7fff8ca0`、`0x7fff9320`、`0x7fff9380`，虚表字节证据 `ev_215f0c1229760058d78f0aeb15be3cb2821d2d5ab13341818320541f4ddd0952`。GetClassID 写出 `{78FF7FA1-FB2F-11D1-94A2-080009EEBECB}`；Load 证据 `ev_8f99630f5e7840723958e44baea8b9155c86b3ff049f99f4d5ddcc58b8a9c72a` 和 Save 证据 `ev_d7180bb580808b59a42283c24a6791fb68f5af0d053833f5fdd93c9ae1f09799` 均显示字段依次是 2 字节版本、1 字节标志、16 字节 GUID、4 字节 int32，共 23 字节。最后 int32 的业务含义未定。Load 对版本不设上限；Save 写版本 1。两者均为静态观察。

FeatureLayer::Load 的 UID 对象之后字段序由本轮 REA 证据 `ev_a6152635db9c14167799eac9961b9254fb3890a45b2ba9f2a20b23c384b75052` 复核：String +0x28c、short +0x1d8、double ×2、short +0x1da、int32 +0x244、short +0x1be、PreJoin、int32 +0x2c0、short ×3、String ×4。下表从样本 `/Maps` UID 对象头 `0x14a9` 开始逐字段对位，区间均为半开区间；String 长度前缀不包含在长度内。

| 起点 | 结束 | 字段 | 原始值或解释 |
| --- | --- | --- | --- |
| `0x14a9` | `0x14b9` | UID 类 GUID | `{78FF7FA1-FB2F-11D1-94A2-080009EEBECB}` |
| `0x14b9` | `0x14bd` | 对象键 | 82 |
| `0x14bd` | `0x14bf` | UID 版本 | 1 |
| `0x14bf` | `0x14c0` | UID 标志 | 1 |
| `0x14c0` | `0x14d0` | UID GUID | LELP 类 GUID |
| `0x14d0` | `0x14d4` | UID 尾部 int32 | 0 |
| `0x14d4` | `0x14da` | +0x28c String | 长度 2，空串 |
| `0x14da` | `0x14dc` | +0x1d8 short | 0 |
| `0x14dc` | `0x14e4` | +0x290 double | 0.0 |
| `0x14e4` | `0x14ec` | +0x298 double | 0.0 |
| `0x14ec` | `0x14ee` | +0x1da short | -1 |
| `0x14ee` | `0x14f2` | +0x244 int32 | 0 |
| `0x14f2` | `0x14f4` | +0x1be short | 0 |
| `0x14f4` | `0x14f8` | PreJoin count | 0 |
| `0x14f8` | `0x14fc` | +0x2c0 int32 | 0 |
| `0x14fc` | `0x14fe` | +0x2c4 short | -1 |
| `0x14fe` | `0x1500` | +0x2c6 short | 0 |
| `0x1500` | `0x1502` | +0x2c8 short | -1 |
| `0x1502` | `0x1508` | +0x2cc String | 长度 2，空串 |
| `0x1508` | `0x150e` | +0x2d0 String | 长度 2，空串 |
| `0x150e` | `0x1514` | +0x2d4 String | 长度 2，空串 |
| `0x1514` | `0x151a` | +0x2d8 String | 长度 2，空串 |
| `0x151a` | `0x152e` | DEP 对象头 | 类 GUID 和键 83 |

旧解析器把 UID 尾部的四个零字节当作 `+0x28c` 的零长度字符串，少读 UID 4 字节，又比实际的 6 字节空串少读 2 字节，共错位 6 字节。随后额外跳过六字节恰好抵消了这个错误，但字段归属仍然错误。修复后的解析器完整读取 UID，不在 PreJoin 内额外跳过六字节。

## 代码与测试

`src/layers/feature_layer_tail.rs` 的 `read_feature_layer_successor_10_1` 现按 10.1 顺序消费 PreJoin 计数及字符串对，随后直接读取 `+0x2c0`、三个 short 和四个 String；删除了错误的六字节消费和 `PreJoinBlock10_1.reserved`。引用表仍在暂存副本中处理，只有整个后继解析成功才提交；解析失败时外层游标与引用表维持原状。

`tests/feature_layer_tail.rs` 校验参考样本 UID 的 23 字节体、尾部 int32 和紧随其后的空串，并覆盖 UID 的所有截断位置。完整后继链合成用例包含一对非空 PreJoin 字符串，紧随其后的 `+0x2c0` 和完整尾段成功解析至 `TailCompleted`。该用例还逐字节截断 PreJoin 计数之后至尾段末尾，检查每种截断均报错、游标回到输入起点且预置引用表未改变。另一完整链用例的零计数 PreJoin 也已移除旧的六字节填充。

静态分析未观察 ArcMap 运行行为；本轮未进行 ArcMap/QGIS 人工目视验收。样本结果与原生实现对位一致，但不能替代目标软件中的转换验收。

验证：独立构建目录下 `cargo test` 通过，其中 CRS 参考样本回归通过；QGIS API 人工环境用例按项目配置忽略。`cargo fmt --check` 通过。全量测试首次尝试因 `/tmp` 空间不足中断，迁移本轮构建目录后通过。
