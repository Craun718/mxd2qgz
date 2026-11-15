# ArcMap 10.1 Save 写侧：版本表机制、FeatureLayer/FDO/ETM 写出链与 FDO 版本 4 探测裁决

记录日期：2026-11-16（E 轮，写侧专项）。对应 TODO「MXD 解析」→「Save 写侧（反编译闭
合写出链）」五项开放条目：FeatureLayer Save 归因、FDOGraphicsLayer Save 委派写出、
FDOGraphicsSublayer Save 版本表逐条目展开、EditTemplate 版本表前四档「不写任何字节」
的读侧交互、AfCore `AfStreamGetSaveVersion` 版本表提取与内部映射；顺带闭合 FDO 版本 4
探测三依赖定位与「Map 架构版本 9」历史布局解读（见
[arcmap-10.1-fdo-open-points.md](arcmap-10.1-fdo-open-points.md) 开放项）。全部为静态反
编译取证，未运行 ArcMap 10.1，未做人工目视验收；样本字节仅作读侧交叉验证（辅助观察）。

## 方法与文件身份

- 工具：本机 pyghidra（Ghidra + JDK 25，venv 沿用 `temp/pyghidra-venv`）。Ghidra 工程
  为既有工程副本（/tmp 下各轮缓存复制到本轮独享目录），只读打开，不与他人共享工程路径。
  分析脚本与产物在本轮目录 `temp/agent-e-saveside/{scripts,out}/`（脚本与反编译文本
  未入库，本文记录全部结论与地址）。
- 二进制开工前逐一核对 SHA-256（与既有各轮登记一致；10.1.0.3035，pefile 版本轮既往
  各轮已复核，本轮未重复）：

| 文件 | SHA-256（前 16） | 既有登记 |
| --- | --- | --- |
| AfCore.dll | `031858c414c0105b…` | 基线一致 |
| FeatureLayer.dll | `2baf870615bb75b6…` | 第一轮一致 |
| Map.dll | `1c0675cc3a996926…` | 第四轮一致 |
| SystemCore.dll | `f63cf9494acc155d…` | 基线一致 |
| AnnoLayer.dll | `b487f3e0dc0e70db…` | 第六轮一致 |
| LayerLib.dll | `55514f054c9e4827…` | 本轮登记（第八轮提取物） |

- 地址均为对应 PE 10.1.0.3035 静态映像 VA。样本交叉验证用仓库参考样本
  `samples/测试底图.mxd` 的 /Maps 流（`cargo run -- hex` 导出），偏移为流内偏移。
- 关键函数全部经本轮自身反编译或汇编逐指令核对，不沿用旧产物结论；对旧记录的修正
  在本文第八节集中列出（不回改旧文档）。

## 一、AfStreamGetSaveVersion / AfStreamGetDocumentVersion（证据：确定的）

AfCore 导出两个版本查询函数（本轮反编译一手证据；与
[arcmap-10.1-mapsurround-bookmark.md](arcmap-10.1-mapsurround-bookmark.md) 第四轮的
`AfStreamGetSaveVersion` 语义记录一致，本轮以自身反编译复核确认）：

- **`AfStreamGetSaveVersion(esriArcGISVersion docVer, short N, short const* table)`
  → short**（`0x68a847f0`；Ghidra 由符号修饰名恢复完整签名
  `?AfStreamGetSaveVersion@@YAFW4esriArcGISVersion@@FPBF@Z`）：
  1. `table == null || N == 0` → 返回 −1；
  2. `N - 1 < (int)docVer` → 返回 `table[N - 1]`（**超表尾截断到末项**）；
  3. 否则返回 `table[docVer]`。
  原生不检查负 `docVer`（负值按有符号整数与 `N-1` 比较，落 `table[docVer]` 为越界读）
  ；Rust 侧封装对此显式拒绝（见第七节）。
- **`AfStreamGetDocumentVersion(IStream*) → esriArcGISVersion`**（`0x68a84760`）：
  结果初值 **5**（esriArcGISVersion101）；输入流非空时经 QI 辅助以 IID
  `{ECC43C55-0148-4EC1-BF87-B9A183C5DC98}`（数据常量 `0x68c6dcb8`，即 IDocumentVersion
  族）查询，成功则调用返回接口虚表 +0x10（get_DocumentVersion）回填。即**不携带文档
  版本接口的流按 10.1（docVer=5）处理**。

`docVer` 的取值域＝保存索引（esriArcGISVersion 枚举 0..5，对应 8.3/9.0/9.2/9.3/10.0/
10.1；见 [arcmap-10.1-native-baseline.md](arcmap-10.1-native-baseline.md)「磁盘版本映
射」节）。各 Save 的版本表第二参数即表项数 N。

### 本轮提取的版本表（字节逐一读出；证据：确定的）

| 类 | DLL | 表地址 | N | 表内容（i16 LE） | docVer→版本映射 |
| --- | --- | --- | --- | --- | --- |
| Map | Map.dll | `DAT_79480dac` | 6 | `[22,29,30,31,45,51]` | 0→22、1→29、2→30、3→31、4→45、≥5→51 |
| FeatureLayer | FeatureLayer.dll | `DAT_6f767d24` | 6 | `[17,22,23,25,34,34]` | 0→17、1→22、2→23、3→25、4→34、≥5→34 |
| EditTemplateManager | FeatureLayer.dll | `DAT_6f77bf74` | 6 | `[-1,-1,-1,-1,1,2]` | ≤3→**FFFF（不写）**、4→1、≥5→2 |
| EditTemplate | FeatureLayer.dll | `DAT_6f77c02c` | 5 | `[-1,-1,-1,-1,2]` | ≤3→**FFFF（不写）**、4→2、≥5→2 |
| FDOGraphicsLayer | AnnoLayer.dll | `DAT_6b1ab0ac` | 2 | `[5,11]` | 0→5、≥1→11 |
| FDOGraphicsSublayer | AnnoLayer.dll | `DAT_6b1ab0a8` | 2 | `[-1,2]` | 0→**FFFF（不写）**、≥1→2 |

`0xFFFF`（−1）表项＝该文档版本不写此类（版本表先于任何字段写出检查，命中即整类不写，
见下文各 Save 的 `if (ver == 0xffff) return 0;`）。注：FDOGraphicsLayer 与
FDOGraphicsSublayer 两张表在 .rdata 中**背靠背**（`0x6b1ab0a8` = sublayer、
`0x6b1ab0ac` = FDO 本体），第一轮曾只记录前者的地址。

### 样本核对

仓库样本（10.1 保存，docVer=5）各对象体首 short：Map 51 ✓（样本 0x33）、FeatureLayer
34 ✓（样本 0x22）、EditTemplate 2 ✓（ETM 体首版本 2）、FDOGraphicsLayer 11 ✓（样本
0xb）——全部与上表 docVer=5 列一致（读侧样本对位为辅助观察）。

## 二、FeatureLayer::Save 归因与字段序（证据：确定的）

**归因闭合**：`0x6f6eb1b0` = FeatureLayer IPersistStream 虚表 `0x6f7acfb0` 槽 +0x18
（本轮直读虚表：+0x0c = GetClassID `0x6f6ec450`、+0x14 = Load `0x6f6e9940`、
+0x18 = Save `0x6f6eb1b0`）。此前 spatial-reference 轮以「槽[6]+GetClassID 同虚表」
锚定但标注「归因仍开放」，本轮以虚表三槽连读 + 全字段与 Load 逐段对称闭合。
签名 `(this, IStream*, int fClearDirty)`；版本表见第一节。

字段序（写出顺序＝流内顺序；成员偏移为对象基相对，与 Load 同一坐标系；`docVer` =
AfStreamGetDocumentVersion 结果）：

| # | 字段 | 门/备注 | 读侧对应 |
| --- | --- | --- | --- |
| 1 | short 版本（版本表→34） | 无 FFFF 档（表全有效） | Load ① |
| 2 | String::Write(+0x120)、String::Write(+0x124) | 恒有 | 双名称 |
| 3 | short +0x128、+0x12a、+0x12c | 恒有 | 三个 int16 |
| 4 | `FUN_6f6eaf70(this−0xd0, stream, fClearDirty)` | 恒有 | 数据源段（FCN+WorkspaceName，`FUN_6f6e94e0` 对偶） |
| 5 | SaveObject(+0x140) | 恒有 | 渲染器槽（Load `{40A9E884}` IFeatureRenderer） |
| 6 | 原样 16 字节 GUID ← +0x144 | 恒有 | +0x144 原样 GUID |
| 7 | String(+0x154)、double(+0x158)、double(+0x160)、short(+0x168) | 恒有 | 后继段 2-5 |
| 8 | int32(+0x178) 计数 + 循环 SaveObject(+0x174 数组) | 恒有 | IHyperlink 循环 |
| 9 | SaveObject(+0x1b8) | 恒有 | IColor 槽 |
| 10 | String(+0x1a8) | 恒有 | +0x1a8 |
| 11 | **docVer <4 且 `*(+0x188)==3` → 写空 String + int32 0**；否则 String(+0x184) + int32(+0x188) | **写向专用降级分支**（读侧无条件读两字段，无对应门） | +0x184/+0x188 |
| 12 | int32 计数（+0x1d0 集合 +0xc 槽取 Count）+ 循环 [String::Write（BSTR 经 +0x14 槽）+ SaveObject（QI +0x20 槽取条目）] | 恒有（条目名无写侧版本门；10.1 全部表项 ≥17 > 5，与读侧「版本 >5 门」在 10.1 一致） | FieldInfo 循环 |
| 13 | SaveObject(+0x1cc) | 恒有 | ALPC 槽 ⑪ |
| 14 | short +0x1bc | 恒有 | ⑫ |
| 15 | SaveObject(+0x1dc) | 恒有 | FeatureIDSet ⑬ |
| 16 | int32 计数（−0x8c 成员经虚表 +0x18 取）+ count×int32 **单块原样写**（枚举填充数组后一次 AfStreamWrite） | 恒有 | 选择集 ⑭ |
| 17 | short +0x1fe | 恒有 | ⑮ |
| 18 | SaveObject(+0x218) | 恒有 | IDisplayFilter ⑯ |
| 19 | short +0x21c、+0x220、+0x21e | 恒有 | ⑰ |
| 20 | int32 +0x22c | 恒有 | ⑱ |
| 21 | SaveObject（IName 槽；成员空时经 `FUN_6f72cc20` 构造缺省对象再写） | 恒有；docVer==4 路径另有 `DowngradeToVersion10QueryLayer`/`UpgradeVersion10QueryLayer` 往返（仅对象状态调整，不改字段序） | ⑲ |
| 22 | int32 计数 + 循环 SaveObject（关系类集合；反编译降级段，条目内部未逐条展开） | 恒有 | ⑳ |
| 23 | String(+0x18c) | 恒有 | ㉑ |
| 24 | SaveObject(+0x1b4) | 恒有 | IEnvelope ㉒ |
| 25 | int32 计数 + **负数游程编码** ID 序列（连续段以「段尾 ID 取负」收尾，如 {5,6,7,9} → `5, −7, 9, −9`；计数＝总元素数而非写出 int 数） | 恒有 | 范围计数 ㉓（读侧逐元素读，负值为游程标记） |
| 26 | `ExtensionHelper::SaveExtensions(+0x26c, stream, fClearDirty)` | **无 docVer 门**（10.1 全部表项 ≥17 ≥ 0xb，与读侧 ≥0xb 门一致） | 扩展段 ㉔ |
| 27 | double +0x1e8 | 恒有 | ㉕ |
| 28 | SaveObject(+0x1a4) | 恒有 | ISymbol ㉖ |
| 29 | short +0x12e、int32 +0x1f0 | 恒有 | ㉗ |
| 30 | SaveObject(+0x1f4) | 恒有 | IEnvelope ㉘ |
| 31 | —— | **docVer==0 → return 0（体止于此）** | —— |
| 32 | String(+0x28c)、short +0x1d8、double +0x290、+0x298、short +0x1da、int32 +0x244 | 恒有（docVer≥1 域） | ㉚-㉞ |
| 33 | —— | **docVer==1 → return 0** | —— |
| 34 | short +0x1be（值经 +0x1c 子接口 +0xc 槽取） | docVer ≥2 | ㉟ |
| 35 | —— | **docVer==2 → return 0** | —— |
| 36 | `CInternalLayerBase::WritePreJoinDefExpressions(this−0xa8, stream)`（LayerLib 导出，IAT `PTR_WritePreJoinDefExpressions_6f74e510`，thunk `0x6f73fec2`）→ int32 +0x2c0 → short +0x2c4、+0x2c6、+0x2c8 → String::Write ×4 | docVer ≥3 | ㊱（读侧 >0x17 门）/㊲（≥0x19 门） |
| 37 | —— | **docVer==3 → return 0** | —— |
| 38 | SaveObject(−0x84)（DEP 槽）、short +0x300、String +0x304/+0x308/+0x30c、TimeData 族四段（`SaveTimeData`/`SaveTimeTableDefinition`/`SaveTimeDimensionDefinition`/`SaveTimeDataDisplay`，−0x68/−0x80/−0x58/−0x48）、SaveObject(+0x2f4)、short +0x2ca | docVer ≥4（版本 34） | ㊴-㊸ |

要点：

- **降级截断阶梯**（docVer 0/1/2/3 各档提前收尾）与版本表 `[17,22,23,25,34,34]` 逐档
  对齐：17=0x11 止于 ㉘（读侧 ㉙ 门 >0x10 本应含 17——**静态不对称**，见第十节开放
  项 1）；22=0x16 止于 ㉞（读侧 ㉟ 门 ≥0x17 不含 22 ✓）；23=0x17 止于 ㉟（读侧 ㊱ 门
  >0x17 不含 23 ✓）；25=0x19 止于 ㊳（读侧 ㊴ 门 >0x19 不含 25 ✓）；34 全量。
- 读侧版本 0x1c..0x1f 专分支（round 9 记录）无写侧来源：版本表跳过 26..31（0x1a..0x1f），
  10.x 写侧不产该档。
- 第 11 步降级分支与第 25 步负数游程编码为**写向专用逻辑**（读侧无对应分支/读侧逐元素
  解释负值），此前各轮未记录。

## 三、FDOGraphicsLayer::Save 委派写出（证据：确定的，汇编逐指令）

AnnoLayer.dll Save `0x6b195b90` = IPersistStream 虚表 `0x6b1ab984` 槽 +0x18（本轮直读
虚表确认；+0x14 = Load `0x6b1955a0`）。签名 `(this, IStream*, int fClearDirty)`，其中
`this` 为内层对象基（外层 FDO 接口 = this+0x54）。汇编级字段序：

1. short 版本 ← `AfStreamGetSaveVersion(AfStreamGetDocumentVersion(stream), 2,
   DAT_6b1ab0ac=[5,11])`。**无 FFFF 档**（两表项均有效，8.3 也写 FDO 层，版本 5）；
2. int32 ← this+0xe8、short ← this+0xe0；
3. `SaveObject([this+0xec])`（关联要素图层，**带对象头**，fClearDirty 原样透传）、
   `SaveObject([this+0x11c])`（IElementCollection2，带对象头）；
4. **委派写出：`(*(this+0xbc))` 虚表 +0x18 = 聚合 FeatureLayer 实例的
   IPersistStream::Save(stream, fClearDirty)——无对象头内联写出**（汇编
   `0x6b195c3c..0x6b195c4a`：`MOV EAX,[EBX+0x68]; MOV ECX,[EAX]; MOV EDX,[ECX+0x18];
   PUSH fClearDirty; PUSH stream; PUSH EAX; CALL EDX`）。第七轮「Save 侧委派按对称性
   推定」升级为逐指令证据；聚合 FeatureLayer 体字段序即第二节；
5. **`CMP docVer,0; JZ → return 0`（版本 5 体止于此）**——与读侧「版本 <6 写默认值后
   提前返回」逐段对称（版本 5 读侧只有步骤 1-6）；
6. docVer ≥1（版本 11）尾段（汇编无进一步版本门——版本恒 11 ≥10，与读侧各门一致）：
   int32(+0x130)、short(+0x1a8)、int32(+0x1b0)、SaveObject(+0x1b4)、SaveObject(+0x1b8)、
   SaveObject(+0x1ac)（两 IColor + ISymbolCollection 三槽）、short(+0x13c)、
   int32 元素计数（**取自成员 this+0x190**，非集合实时 Count）→ 元素循环（
   `FUN_6b19b510`/`FUN_6b19b550` 迭代 this+0x188 集合；每元素经 `FUN_6b19e210` QI 后
   调其虚表 +0x18 = **FDOGraphicsSublayer::Save 内联写出，无对象头**）→
   short(+0x1bc) → SaveObject(+0x1c0 ILineSymbol) → SaveObject(+0x1c4 IMarkerSymbol)
   → SaveObject(+0x140 IColor 终结)。任一写失败返回 0x80004005 或透传 HRESULT。

## 四、FDOGraphicsSublayer::Save 版本表逐条目展开（证据：确定的）

Save `0x6b1a1c40`（IPersistStream 子虚表 `0x6b1ac880` +0x18，第七轮定位，本轮反编译
复核）。版本表 `AfStreamGetSaveVersion(docVer, 2, DAT_6b1ab0a8=[0xFFFF, 2])`：

- docVer = 0（8.3）→ 0xFFFF → **`return 0`，整类不写任何字节**；
- docVer ≥1 → 2（docVer=1 取 `table[1]`，≥2 截断到末项同值）。

写出字段（**版本 short 之外无任何版本门**；偏移为 IPersistStream 子对象 this 相对，
对象基 = this−0x40）：

| # | 写出 | 对象偏移 | 读侧对应（第七轮表） |
| --- | --- | --- | --- |
| 1 | short 版本（0 或 2） | — | ① int16 版本（>2 拒绝） |
| 2 | int32 ← this+0xc | +0x4c | ② int32（构造默认 −1） |
| 3 | String::Write ← this+0x28 | **+0x68** | ③ String（第七轮「成员槽位未恢复」→ 本轮补全） |
| 4 | short ← this+0x40 | +0x80 | ④ int16（默认 0xFFFF） |
| 5 | double ← this+0x38 | +0x78 | ⑤ double |
| 6 | double ← this+0x30 | +0x70 | ⑥ double |
| 7 | String::Write ← this+0x10 | **+0x50** | ⑦ String（同上补全） |
| 8 | double ← this+0x18 | +0x58 | ⑨ 前一个 double（版本 ≥2 门） |
| 9 | double ← this+0x20 | +0x60 | ⑨ 后一个 double（版本 ≥2 门） |

读写不对称闭合（第七轮开放项）：

- **版本 1 专属 short（读侧 ==1 字段 ⑧）无写侧对应**：10.1 写侧只产出「无字节」或版本
  2，**任何文档版本都不产出版本 1**——读侧 ==1 分支只服务更老写入器产生的流（兼容
  遗留），不是读写 bug。
- **尾随两个 double 无版本门**：写侧写时版本恒 2（≥读侧 ≥2 门）恒写，读侧 ≥2 门在
  10.1 可达域内恒真——两侧一致，无不对称。

## 五、EditTemplate 前四档「不写任何字节」与读侧交互（证据：确定的；闭合）

**宿主链（全部本轮反编译/汇编）**：

1. **EditTemplate::Save `0x6f6ce3c0`**（FeatureLayer.dll）：版本 ←
   `AfStreamGetSaveVersion(docVer, 5, DAT_6f77c02c=[-1,-1,-1,-1,2])`；
   **`if (ver == 0xffff) return 0;`（写 0 字节，返回成功）**；否则 short(2) →
   String(+0x08) → String(+0x0c) → 16 字节 GUID(+0x18) → int32 计数(+0x30) → 循环
   [String 键 + `Variant::Write`]（迭代器 `FUN_6f6d9ac0`/`FUN_6f6cfc30`，与读侧
   `Variant::Read` 对偶）→ String(+0x10)（迭代耗尽后恒写）。
2. **EditTemplateManager::Save `0x6f6d0bc0`**：版本 ←
   `AfStreamGetSaveVersion(docVer, 6, DAT_6f77bf74=[-1,-1,-1,-1,1,2])`；
   **`if (ver == 0xffff) return 0;`——docVer ≤3 时连模板计数都不写，根本到不了模板
   循环**；否则 short → int32 计数(+0x10) → 循环 `SaveObject(模板)` → 版本 ≥2 尾
   1 字节(+0x1c)。
3. 调用链静态闭合：EditTemplate::Save 在 FeatureLayer.dll 内的**唯一直接引用**是其
   IPersistStream 虚表槽（`0x6f7ab8d8`）——只经 `SaveObject` 的 +0x18 槽虚调用到达，
   唯一宿主 = ETM::Save 的模板循环；ETM::Save 同理（唯一直接引用 `0x6f7abb5c` =
   其虚表槽），由扩展段写出调用（第二节第 26 步）。
4. **`ExtensionHelper::SaveExtensions`**（LayerLib.dll `0x786c1cf0`）：**无 docVer
   门**。条目格式：`[int32 计数][每条 [int64 尺寸][16 字节 CLSID][体]]`——写侧逐条
   `StrmUtil::FindCurPos` 记位 → 写 8 字节零占位 → 写 CLSID → 记位 → 调扩展对象虚表
   +0x18（IPersistStream::Save）写体 → 记位 → **回填尺寸 = 体字节数**。 tenth 轮的
   「尺寸 = 体字节数」由写侧差值计算直接证实。
5. **`ExtensionHelper::LoadExtensions`**（LayerLib.dll `0x786c22e0`）：读 [int32 计
   数] + 每条 [int64 尺寸][16 字节 CLSID]；按 CLSID 建实例失败（未知类）→ 按尺寸
   `IStream::Seek` 跳过；**已知类且尺寸非零才调 Load（虚表 +0x14）；尺寸 ==0 → 完全
   跳过（不调 Load）**。

**交互后果（静态论证，三重闭合）**：

- docVer ≤3（另存为 8.3/9.0/9.2/9.3）时：ETM::Save 先于模板循环返回（0 字节）→
  扩展条目体 = 0 字节 → **尺寸字段 = 0** → 读侧 LoadExtensions 对该条目不调 Load，
  原地跳到下一条目——**不产生任何读侧垃圾解析**。「无外部长度体」的读侧难题在扩展段
  不存在：扩展条目是 /Maps 对象图中唯一自带外部长度的区域，`size==0 跳过`语义恰好
  覆盖「不写任何字节」路径。产物文件可正常打开（内容 = 无模板，与 8.x/9.x 原生行为
  一致——那些版本不存在 EditTemplate）。
- **EditTemplate::Save 的 FFFF 分支实际不可达**：唯一宿主 ETM 在 docVer ≤3 时已提前
  返回，循环内 SaveObject 不会对 docVer ≤3 发生。即「前四档不写任何字节」在
  EditTemplate 自身是死分支，真正的「不写」发生在宿主 ETM 层。
- docVer ≥4（10.0/10.1，可达域）恒写版本 2 满形态（10.0 经表项 4→2，10.1 截断到末项
  →2），与读侧「版本 2、`2 < ver` 拒绝」一致；样本 ETM 体版本 2 ✓。

实现含义：Rust 侧 `read_edit_template_body_10_1`（`edit_template.rs`）与 ETM/扩展段
读取器**无需任何改动**；本节结论全部为写侧/宿主链证据。

## 六、FDO 版本 4 探测三依赖定位与修正（证据：确定的，汇编逐指令）

探测函数 `FUN_6b198420`（AnnoLayer.dll；入口自定义寄存器约定 `EDI`=流，返回 AL）。
本轮汇编逐指令复核，对第七轮记录做三处修正（见第八节）。

**公共序言**（`0x6b19844a..0x6b198494`）：两次 `IStream::Seek`（虚表 +0x14）——
`Seek(0, SEEK_CUR, &P)`（当前位置记 `[EBP-0x40]`）+ `Seek(0, SEEK_SET, &old)`（**回
卷到流绝对 0**）→ `AfStreamRead`（IAT `0x6b1aa03c`，GUID 变体）读 16 字节 G0 → 与
`DAT_6b1ac0e0`（FDOGraphicsLayer CLSID）比对。

**分支 A（G0 == FDO 本类 CLSID，`0x6b19849a..`）——流根即注记层对象**：

1. `FUN_6b19e860(stream, &out)`：QI **IID `{18A45BA7-1266-11D1-86AD-0000F8751720}`**
   （`0x6b1ac130`，完整 GUID 本轮录得；SystemCore 同值两份 `0x80070fb0`/`0x8007123c`
   ——即 SystemCore 侧 IObjectStream）。
2. QI 成功 → 从**主流**读 int32（对象键，IAT `0x6b1aa118` int32 变体，`0x6b1984af..0x6b1984bd`；
   **修正 1：第七轮「从返回接口读 int32」系误记，分支 A 读的是主流**）；随后从主流读
   short（`0x6b1984c0..0x6b1984d0`）——该位置即调用者已消费的同一体版本 short（探测只
   在版本==4 时被调）→ **恒 ==4，≠4 路径为死分支**。
3. ==4（恒真）→ 读 int32、short（`0x6b1984f9..0x6b19850f`，即旧加载器字段序第 6/7 步
   的 +0xe8/+0xe0 位置）→ `LoadObject`（IAT `0x6b1aa11c`）两次：IID `DAT_6b1abe60`
   （ILayer 族）→ `[EBP-0x2c]`、IID `DAT_6b1abf60`（IElementCollection2 族）→
   `[EBP-0x28]` → `Seek(P, SEEK_SET)` 复位（`0x6b19853f..0x6b198555`）→ 两对象均非空
   → `JMP 0x6b1986bf`（`XOR AL,AL`，**假**）；**任一为空（空标记/装载失败）→
   `MOV AL,1`（真）**（`0x6b1985a8..0x6b1985e8`）。
   **修正 2：第七轮「两对象皆非空 → 真」与汇编相反——真条件是「至少一槽为空」。**
4. ≠4 路径（死分支）不复位流位置、返回假。

**分支 B（G0 ≠ FDO CLSID，`0x6b1985ed..`）——检查文档根 Map**：

1. flag（`[EBP-0x23]`）= G0 ≠ `DAT_6b1ac100`（Map coclass CLSID `{E6BDAA76-…}`）；
   再次 `Seek(0, SEEK_SET)` 回卷（`0x6b198605..0x6b198613`）。
2. flag ≠0（根首 16 字节既非 FDO 也非 Map，即带 int32 计数前缀的常规根）→ 读并丢弃
   int32（计数）→ 读并丢弃 16 字节（Map 头）；flag ==0（根首即 Map 头，无计数前缀的
   老布局）→ 跳过这两步。
3. `FUN_6b19e860` QI IObjectStream → 成功则**经返回接口**读 int32（键；
   `0x6b19865d..0x6b198668`，AfStreamRead＝虚表 +0x0c，即把返回接口当 IStream 用）。
4. 从主流读 short（`0x6b19866d..0x6b198676`）→ `Seek(P, SEEK_SET)` 复位
   （**`0x6b198684..0x6b198693`，真假两路都在比较之前执行——修正 3：第七轮「分支 B
   真/假均不恢复位置」系误读，分支 B 恒复位**）→ short ==9 → 真（`MOV AL,1`），
   否则假。

**三依赖的定位与现状**：

| 依赖 | 定位 | 现状 |
| --- | --- | --- |
| (a) 流绝对位置 0 访问 | 序言两处 Seek（`0x6b19844a..0x6b19846f`、`0x6b198605..0x6b198613`） | **成立**：探测需要整流根（子对象读取器只有对象体窗口），单独已足以支撑「不实现」 |
| (b) IObjectStream QI 的运行时结果与游标效应 | `FUN_6b19e860`（IID `0x6b1ac130`）；分支 A 以 QI 成败门控主流键读；分支 B 经返回接口读键（`0x6b19865d..0x6b198668`） | **定位完成，语义部分开放**：分支 B 的键读经返回接口的 +0x0c 槽（IStream 兼容 Read）；其读转发是否推进同一底层流游标未取证（SystemCore 侧应答子接口本轮未静态锁定）。由于两分支事后均复位 P，游标效应不外溢——残余影响仅为分支 B 后续 short 读落在哪两个字节（探测正确性，非流状态） |
| (c) 分支 B 不复位 | ~~（不存在）~~ | **撤销**：分支 B 恒复位（修正 3）；唯一不复位路径是分支 A 的死分支 ≠4 |

**「Map 架构版本 9」历史布局解读（闭合）**：Map::Save 版本表 `DAT_79480dac` =
`[22,29,30,31,45,51]`——**10.1 可写的最小 Map 架构版本为 22（8.3 档）**。分支 B 的
`short == 9` 因此只可能命中比 8.3 更老的文档（ArcInfo/ArcMap 8.0-8.2 时代，10.1 读侧
上限 51 的历史体），与 FDO 版本表 `[5,11]`（8.3 已是 FDO v5）互证：**FDO v4 = 8.3 之
前的注记层布局**（legacy 0..3 布局 + 收尾 String——读侧在探测真后按旧加载器读并丢弃
一个 String，即 v4 相对 v3 恰多一个尾随 String）。分支 B 的两种根前缀解读随之闭合：
flag ≠0 = 带 int32 计数的多数据框根（现行 `/Maps`）；flag ==0 = 无计数前缀、根首即
Map 头的单数据框老布局。「Map 架构版本 9 对应哪个具体产品版本」超出 10.1 二进制可证
范围（候选：8.0-8.2 某档），保持开放。

## 七、实现（src/version/save_version_table.rs）

按任务约束产出**纯常量模块**（不接任何调用点；`src/version/mod.rs` 仅追加 mod 声明
行）：第一节六张版本表以 `pub const` 编码（元素类型 i16，附 DLL/表地址/证据注释），
并附原生查表语义的纯函数封装 `save_version_for(table, doc_ver) -> Option<i16>`
（负 docVer 与空表返回 `None`——较原生的越界读/−1 更严，函数文档注明）。测试全部为
模块内 `#[cfg(test)]`：逐表字节回归、docVer 0..=7 的映射（覆盖截断）、FFFF 档不写语
义、负 docVer 拒绝。样本核对值（Map 51、FL 34、FDO 11、ET/ETM 2）一并断言。

## 八、对既有记录的修正（本轮裁决；不回改旧文档）

1. **第七轮（arcmap-10.1-fdo-open-points.md）「分支 B 真/假均不恢复位置」撤销**——
   汇编 `0x6b198684..0x6b198693` 在 ==9 比较之前无条件 `Seek(P, SEEK_SET)`（本文第六
   节修正 3）。探测三依赖由三项降为两项（(a) 整流根访问 + (b) IObjectStream 通道），
   「不实现」结论不变（(a) 单独已足）。
2. **第七轮「分支 A 探测真条件 = 两对象皆非空」反转**——汇编证明真条件是「至少一槽
   为空」（第六节修正 2）；「分支 A 从返回接口读 int32」修正为「从主流读、QI 成败仅
   作门控」（修正 1）。
3. **第十轮（arcmap-10.1-featurelayer-tail.md）「㊱ 计数后 6 个零字节归属未决」闭合
   （边界错位，不存在保留字节）**：写侧汇编证明 `WritePreJoinDefExpressions`（仅
   `[计数][计数×[String][String]]`，0 计数即 4 字节）之后**直接**写 int32 +0x2c0，
   中间无任何写；样本 0x14d4..0x151a 按写侧字段序零残差对位：
   `String(+0x28c)=@0x14d4、short(+0x1d8)=0、double(+0x290/+0x298)=0.0、
   short(+0x1da)=−1、int32(+0x244)=0、short(+0x1be)=0、PreJoin 计数=0、
   int32(+0x2c0)=0、short(+0x2c4/6/8)=[−1,0,−1]、String×4、DEP 头 @0x151a`。
   既有读取器把该 14 字节段按「计数 + 6 保留字节」消费，**字节消费量与样本值完全
   一致**（0x14ee..0x14fb 全零），行为无需改动；字段命名差异留给整合轮裁量。
   新样本观察：`short +0x1da = −1`（round 9 表该字段无样本值）。
4. **legend-tails 轮 EditTemplate 表注「docVer 83..92 时 Save 直接返回」的记法修正**
   ——比较对象是**保存索引** docVer（0..5），「83..92」应读作「枚举 83/90/92/93 四档
   （索引 0..3）」；本表 N=5，10.0（索引 4）→2、10.1（索引 5）截断到末项 →2。
5. **第一轮 spatial-reference 记录「FeatureLayer::Save 归因仍开放」闭合**——归因与
   字段序见本文第二节。

## 九、样本交叉验证清单（辅助观察）

- 版本表 docVer=5 列 × 样本体首 short：Map 51 ✓、FeatureLayer 34 ✓、FDOGraphicsLayer
  11 ✓、EditTemplate/ETM 2 ✓（第一节表）。
- FL::Save ㊳-㊴ 段（0x14d4..0x151a）按写侧字段序零残差对位（第八节修正 3），同时
  反向锚定 ㉚-㊲ 各字段样本值。
- FDO::Save / ET::Save / ETM::Save 的 FFFF/门行为在样本中不可达（样本 docVer=5 走
  满形态），无字节佐证，全部为反编译证据。

## 十、未解决事项

1. **FL::Save docVer==0 与读侧的静态不对称**：写侧版本 17 体止于 ㉘（SaveObject(+0x1f4)
   后 return），读侧 ㉙ UID 槽门为版本 >0x10（17 命中）——10.1 自产的「8.3 兼容」FL
   体按 10.1 读侧会在 UID 槽处错位。仅影响 docVer==0 存档的往返，无样本；是否属原生
   兼容缺陷未验证。
2. **docVer <4 时扩展段中的 ETM 条目**：静态推断产物为 `[尺寸 0][CLSID]` 且可被读侧
   安全跳过（第五节）；「另存为 8.3 时 ETM 是否仍在扩展数组中」未运行取证——若上层
   在降级保存前移除该扩展，则连零尺寸条目也不产生。两条路径读侧行为均安全，不影响
   实现约束。
3. **分支 B 经 IObjectStream 接口读键的转发语义**：返回接口的 +0x0c 槽是否将 4 字节
   消费转发到同一底层流（影响其后 short 读的落点，即探测正确性）未取证；SystemCore
   侧应答子接口未静态锁定（构造/QI 转发布局未解）。
4. **FL::Save 第 22 步（关系类循环）与第 21 步 IName 槽**的反编译降级段：计数源成员
   （+0x25c/+0x268 集合）与缺省对象构造（`FUN_6f72cc20`）未逐条展开；第 25 步负数游
   程编码的完整状态机以反编译语义为准，未做样本对位（样本计数 0）。
5. **FDO::Save 元素计数取自成员 this+0x190**：读侧无 +0x190 槽位记录（计数在流内、
   集合在 +0x188），该成员与集合的同步关系未展开。
6. 「Map 架构版本 9 ↔ 具体 ArcGIS 产品版本」与「FDO v4 体内各字段在 pre-8.3 布局中
   的准确形态」保持开放（需旧版本样本或运行取证）。
7. 全部结论为静态反编译取证与样本辅助对位；未运行 ArcMap 10.1，未人工目视验收，不
   声称已验证。
