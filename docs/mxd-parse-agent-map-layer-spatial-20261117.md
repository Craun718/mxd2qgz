# MXD 解析 Map、Layer 与空间范围续研记录

记录日期：2026-11-17。本轮是只读静态 REA/Ghidra 研究；没有运行 ArcMap，
没有修改 Rust 源码、测试、`README.md`、`TODO.md` 或既有研究文档。结论限于
ArcMap 10.1 Desktop 10.1.0.3035 的下列 PE x86 目标，地址均为对应静态映像 VA：

| 模块 | SHA-256 |
| --- | --- |
| Map.dll | `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8` |
| FeatureLayer.dll | `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60` |
| AfCore.dll | `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17` |

对应本地文件位于 `temp/arcgis101-bin/{mapdll,featurelayerdll,AfCore.dll...}`。
本轮使用 REA 6.9.0/Ghidra 12.1.4（bridge 对 12.1.4 验证）。参考样本只用于核
对既有记录与当前读取器现状；本轮没有为任何条目新增字节佐证。

## 结论摘要

| 原 TODO 项 | 本轮状态 |
| --- | --- |
| 124 GroupLayer 版本分支字节佐证 | 未闭合。新增 10.1 Save `0x6f7119b0` 与版本表；合成/真实样本/运行取证方案分级更明确。 |
| 282 FeatureLayer `+0x1f4` 语义 | 未闭合。新增 Load 后数据源接口绑定链，仍不推翻“实际绘制/缓存范围”候选。 |
| 283 FeatureLayer `+0x1b4` 无值行为 | 静态写侧机制新增；空指针保存路径闭合为 16 零字节，但“成员为空时无副作用”需运行取证。 |
| 287 Map+0x298 coclass 身份 | 静态身份闭合为 Geometry Envelope `{30707212-52D5-11D0-A8F2-00608C85EDE5}`；未运行目标软件。 |

## GroupLayer 版本分支

既有 `GroupLayer::Load 0x6f711270` 证据保持不变：版本 short 有符号大于 12 拒
绝；子图层计数循环按版本 `<11` 普通 `LoadObject`、`>=11` `ExtraSafeReadObject`
分派。本轮复核伪代码仍显示版本 `>2` 的 IEnvelope 槽与版本 `>=11` 的
IDisplayFilter 槽，不改既有读取器。

本轮新增 Save 侧证据：

- GroupLayer IPersistStream 虚表 `0x6f7af244` 槽 `+0x0c` 指向 Save
  `0x6f7119b0`。
- Save 调 `AfStreamGetSaveVersion(docVer, 5, 0x6f767c9c)` 写版本 short。
  `0x6f767c9c` 十六个字节为 `04 00 09 00 0b 00 0b 00 0c 00 00 00 ...`，按
  10.1 已证超表尾项规则，docVer 5（10.x）写 GroupLayer 体版本 `12`。
- Save 对子图层循环使用：docVer `<2` 写普通 `SaveObject`，docVer `>=2` 写
  `ExtraSafeWriteObject`。10.1 MXD（docVer 5）因此写 ExtraSafe 条目。
- Save 还按 docVer 分支：`<4` 子图层循环后把 `local_14` 置零；docVer 5 继续后
  段，尾部写 `+0x84` 对象。这与 Load 侧 `>=11` 槽相容，但没有给出 10.1 内每
  一个可能 GroupLayer 架构版本的样本。

所以第 124 行仍是字节证据缺口，不能因静态规则闭合。方案分级如下：

1. **最低有效佐证（合成/手工构造 MXD）**：优先在真实 ArcMap 10.1 中创建并保
   存含一个 GroupLayer 的 MXD，随后至少做一个空组和一个非空组（非空组至少一
   个子图层）变体；若需触碰版本分支，还需按 10.1 可用保存兼容档生成旧文档版
   本，或使用受控 old-version 兼容保存流程。记录 `/Version`、对象头、版本
   short、子图层循环计数、`<11`/`>=11` 前缀、可见性与后续字段边界。
2. **无损运行捕获**：在 10.1 中保存 MXD 前后对照 `/Maps`；或对 Save 调用点/
   `AfStreamWrite` 做只读断点与流缓冲观察。优先记录对象流游标而不是依赖进程
   内存偏移，后者随 ASLR 与加载状态变化。
3. **受控 COM/驱动运行取证**：可用 COM 客户端创建 Map、插入 GroupLayer 并触
   发文档保存；需确认 10.1 的持久化路径与 UI 保存一致后再把它当作佐证。
4. **不建议**：仅按 Save 版本表生成 GroupLayer 架构版本并称为真实 10.1 字节
   佐证。它可以用于交叉验证解析器，但不能闭合本 TODO。

## FeatureLayer +0x1f4 槽字段语义

既有证据确认版本 `>=0xf` 时 `0x6f6e9940` 以 IEnvelope `{1B077914-...}` 装载
成员 `+0x1f4`；参考样本呈现紧窗口与宽数据范围混合，业务成员名未取得。

本轮新增数据源后处理链：

1. `0x6f6ea845` 版本门 `cmp word [ESP+0x20],0xf`；`0x6f6ea8ae`
   `lea ESI,[EBX+0x1f4]`。
2. Load 后先对 `this+0x130` AddRef，再调用 `0x6f705d00`。该函数用
   `DAT_6f7ae424={720A857D-45A9-4DBF-AA06-73715C654B01}` 做数据源 QI。
3. 成功且非空时调用 `0x6f705db0`，其 IID 为
   `DAT_6f7ae434={89E86261-2D2B-45A7-BE01-64190DBC9BDB}`；成功后读取器把
   `+0x1f4` Envelope 作为参数传给该接口虚表 `+0x10`。
4. 本地既有类型库记录未命中上述两个 IID；不能把它们写成“绘制缓存接口”。
5. `+0x1f4` 的 Save 写出在 `0x6f6ec03a..0x6f6ec046`：
   `SaveObject([this+0x1f4])`，前导仍为 short `+0x12e` 与 int32 `+0x1f0`。

因此可确认的是：该 Envelope 会被传给数据源的某接口方法，请求 IID 均已字节闭
合但未命名。无法确认它是实际绘制/缓存范围的正式属性名或生命周期来源。候选保
留为“实际绘制/缓存范围；显示窗口与数据相交；未绘制层可能回退宽范围”。闭合
路径是：（a）在完整 10.1 类型库（含接口库引用）中按两个 IID 查成员签名；
（b）只读运行断点观察 `0x6f705db0` 与虚表 `+0x10` 的 `this`、`+0x1f4` 值、
IID QI 结果和调用栈；（c）同步记录 ArcMap 图层可见性、当前视图、缩放/平移与
未绘制状态，再保存 MXD 对照值；（d）必要时导出并匹配相关接口 vtable 与同类
数据源实现的 GetClassID。

## FeatureLayer +0x1b4 槽无值行为

既有证据确认版本 `>8` 时槽 `+0x1b4` 由 IEnvelope LoadObject 填充；样本 11 个
FeatureLayer 全为 20 字节空标记。本轮补充：

- `+0x1b4` Save 位于 `0x6f6ebd13..0x6f6ebd1f`：无空值检查，恒调用
  `SaveObject([this+0x1b4])`。
- AfCore `StrmUtil::SimpleSaveObject 0x68a7af70` 开头显式判断对象指针为空：
  构造 16 字节零并调用 `IStream::Write(...,0x10,...)`，随后返回。这闭合了
  “空对象指针会被写为 16 零字节”的通用写侧机制；样本中后续 4 字节键 0 与
  `read_object_entry_10_1` 的 20 字节空标记分类一致。

但这里的“无值行为”不能进一步推成已证事实：

1. [后续指令复核](mxd-terra-follow-feature.md)确认版本 >8 时，`Load` 在调用
   `LoadObject` 前释放非空旧引用并清零 `+0x1b4`。成员的完整运行时生命周期
   仍未验证。
2. 静态证据未证明空值保存前后没有任何副作用；`SaveObject` 空指针路径只表明
   不调用成员对象方法。
3. 非 IEnvelope 新类在该槽的 Load 行为未知；不能因为接口指针为空就说对象
   一定无值。

取证路径：在 ArcMap 10.1 中于 `0x6f6ebd13` 只读断点采样多个保存动作，记录
`[this+0x1b4]`、版本 short、调用栈；若指针非空，进一步记录 Envelope 值和成员
生命周期；若恒为空，至少在打开旧文件、缩放/平移、显隐图层、修改数据范围、
重开文档与保存后各采样一次。运行观察不应只看流输出，还要检查成员指针；两者
分别证明序列化输出与对象状态。

## Map+0x298 coclass 身份

本轮闭合静态身份。Map 惰性助手链为 `0x793fdfd0 -> 0x793fdcc0 ->
0x793a2ee0`：

- `0x793fdfd0` 在 docVer 分支处调用 `0x793fdcc0`，成功后在 `0x793fe3d6` 把返
  回指针写入 `Map+0x298`，必要时 AddRef 并释放旧值。
- `0x793fdcc0` 调用 `0x793a2ee0` 创建对象；`0x793a2ee0` 调
  `AfCreateInstance(DAT_79481570, NULL, 0x17, IID_IUnknown, &unknown)`，成功
  后运行对象并 QI `DAT_794815c4`。
- REA `read_bytes` 显示 `DAT_79481570` =
  `{30707212-52D5-11D0-A8F2-00608C85EDE5}`；与既有 Geometry.dll Envelope
  CoClass、GetClassID 和样本流内类标识逐字节一致。
- REA `read_bytes` 显示 `DAT_794815c4` =
  `{1B077914-38C5-11D0-92D2-00805F7C28B0}`，即既有类型库记录的 IEnvelope。

因此 Map+0x298 的静态 coclass 身份为 Geometry Envelope
`{30707212-52D5-11D0-A8F2-00608C85EDE5}`。该结论是静态创建与 QI 链证据，不是
运行时行为验证；未运行 ArcMap，也未在 ArcMap/QGIS 人工检查。后续如需升强，
可在同一目标上用只读运行断点记录 `Map+0x298` 的 QI 来源或 GetClassID 输出。

## 证据与未解决问题

- 本轮新增关键 REA Evidence：Map 创建链 `ev_8180a34881f99f9509dda243f3ee5d
  66063b248c4e43788af5adc1b7c9d8b4f2`、`ev_692d0de3f55968a59033ca08727dd9045
  40bd20207673eb232d4b5db86d7d3de`；FeatureLayer Load
  `ev_464203a579bef93aa3d0d61f5acdd1f3b04a8386cfb1db1e942a8dc4cf7226e4`；FL
  Save `ev_5798c6fee46cd1049d2b8994a9e9b5e2d441053f1a45b3d387af1960f8c92346`；
  GroupLayer Load/Save 与 AfCore 空保存路径分别在上述静态章节绑定。Evidence
  IDs 在当前 REA 会话内存中保留；若需要跨进程复查，应先导出证据包。
- 未运行 ArcMap、QGIS 或 COM 目标；没有目标软件人工检查。
- GroupLayer 架构版本分支、`+0x1f4` 成员命名、`+0x1b4` 运行时空成员状态与副
  作用仍未闭合。
- Map+0x298 结论限于上述 10.1.0.3035 静态映像；不能外推到 10.8 或其他版本。
