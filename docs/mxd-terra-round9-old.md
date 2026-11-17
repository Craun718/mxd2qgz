# 第九轮：旧版本对象和 SDK 候选复核

2026-11-17。本轮承接 [第八轮](mxd-terra-round8-old.md)，只研究 FieldInfo v0/v1、GroupLayer <11 和 Envelope v0 的真实字节证据与到达路径；没有修改 TODO 或产品读取器。静态反编译与宿主读取是不同证据，均不是 ArcMap 运行结果。SDK 的 ArcToolbox 流尚未建立宿主结构、引用表或对象边界；扫描值不能当作读取结果。

## 新发现：ArcToolbox 的 Envelope 候选可能错位

10.1 SDK .NET `Continents.mxd` SHA-256 `d20e7f7fea729c1ae9afc7d742745a2cdf16a0472ccedea2a2f46058758685e3`，`/Version` 文本 9.2、short 14；不把介质版本当文件保存版本。`/ESRI ArcToolbox` 的 `0x306a6` 和 `0x5b73c` 有 Envelope 类标识，之前以“16 字节类标识 + 4 字节键”猜测体首，两处 +20 都是 `00 00`。本轮核对连续字节，发现另一种更符合既有 Envelope 结构的解释：

| 类标识偏移 | +16 short | +18 起四个小端 double | +50 int32 | +54 起 16 字节 |
| --- | ---: | --- | ---: | --- |
| `0x306a6` | 3 | -180、约 -89.9、约 180.000009、约 83.6236 | 0 | GeographicCoordinateSystem 类标识 |
| `0x5b73c` | 3 | 四个 NaN 位模式 | 0 | 相同类标识 |

按 +20 为 v0 解码时，第一处的坐标得到极小的非地理数值，所谓标志 `0xa6a87a80` 又跨入后续类标识。两处相同的 +16 版本、32 字节坐标、四字节标志和后接的空间参考类标识强烈提示此前 +20 的 0 是**错位候选**。但无法仅由形态确定实际宿主是否采用无键格式、是否确为 Envelope 新对象或确认对象终点；也不能把四个 NaN 赋予业务语义。未将两处统计为 v0 或 v3 的宿主实证。测试独立断言上述原始字节及两种偏移，不调用对象条目分类器来制造宿主证据。

## 10.1 原生旧 Map 列表与 8.1 样本

本轮重核 Map.dll PE：x86、文件版本 `10.1.0.3035`、SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`。这只确定资源声明与文件身份，不证明签名或实际运行。隔离 pyghidra/Ghidra 12.2 反编译结果（忽略目录 `temp/terra-round9-old/native.json`）补足旧入口：

- `0x794035f0` 对有符号架构版本 `<23` 调用 `0x794038a0`；后者对版本 17 直接返回错误。其余版本先读 int32 计数并循环：版本 `<17` 使用普通 `LoadObject`，版本 `>17` 使用 `ExtraSafeReadObject`；再读第二个 int32 计数并循环普通 `LoadObject`；最后读取 int32 活动索引。两个循环中的对象和应用层插入副作用不能单凭计数概括。
- `Map::Load` `0x793ff4d0` 在列表后、基本图形层前，对版本 `<22` 调 `0x79403ec0`；该函数对于版本 19 不消费流字节。故本样本中以下边界与原生静态路径相容。
- 10.8 Desktop CAB 里的 `calibrate.mxd` SHA-256 `f883510f90d05a01c4ae152aa008a74b14cab0e8469177b98ef7e3b40fc533b4`，`/Version` 原文 8.1、short 10，非 10.8 保存档。独立测试从 `/Maps` 根计数、Map 对象头和既有 Map 前缀顺序读取：Map 架构版本 19，前缀止于 `0x52`；两旧列表计数 `0x52=0`、`0x56=0`，活动索引 `0x5a=-1`，`0x5e` 接 CompositeGraphicsLayer 类标识及键 32。没有遍历旧列表元素，也不声明整个 Map 完整。独立表从本流根建立，跨流先前键未知，故键 32 的新对象分类仍受会话范围限制；类标识和偏移是独立可核对的字节观察。

旧列表的分支和零计数真实字节可独立记为**静态规则与空集合对位**；该样本没有由此到达任何 FieldInfo、GroupLayer 或 Envelope 旧版本。非空集合、版本 17 错误路径的实档行为及其余旧档均未验证；不据此接入产品解析器。

## 独立测试及可勾范围

`tests/terra_round9_old.rs` 新增三个合成边界测试：FieldInfo v0/v1 空格式槽与尾部对位、每个前缀截断时游标和引用表回滚；GroupLayer v10 普通空子层、后续字段和逐字节截断；Envelope v0 无附加维度的字段及尾界。另有两个依赖本机介质的 ignored 测试，分别核对上面的两个 ArcToolbox 局部候选和 calibrate 旧 Map 宿主路径。合成数据是回归测试，不是旧档；ArcToolbox 测试是错位排查，不是宿主解析。两种 ignored 测试路径与诊断输出仅为本地研究，分享前检查敏感路径。

- **FieldInfo v0/v1：不可勾选实档分支。** 新增合成正常与截断测试；第八轮 v2 证据仍只覆盖 v2。
- **GroupLayer <11：不可勾选真实普通子层。** 新增 v10 合成回归；旧 Map 空列表无法提供 GroupLayer 实例。
- **Envelope v0：不可勾选真实旧档。** 新增合成无维度边界测试，先前完整截断与 Z/M 组合测试仍见 `tests/envelope_v0_round2.rs`；SDK 两处 +20=0 不再作为有意义的 v0 候选。

执行 `TMPDIR="$PWD/temp/terra-round9-old/tmp" cargo test --offline --test terra_round9_old -- --include-ignored`：5 项通过；`cargo fmt --check` 通过；完整 `cargo test --offline`：794 通过、17 忽略；`git diff --check` 通过。独立 REA 6.9.0 CLI 使用 JDK 25、Ghidra `/opt/ghidra`、仓库内 TMPDIR，55 秒本轮时限到达后被取消，输出标记 `cancelled`，没有产出函数证据，不能说是 socket 失败或 provider 自身超时。随后采用独立拷贝的 Ghidra 工程和隔离配置，通过 pyghidra 获取上述静态结果，没有切换共享 REA 目标。

ArcMap 当前不可用；未在 ArcMap 10.1 或 QGIS 中人工打开源工程与转换结果，不能声称旧版本兼容、转换结果或目标软件运行已验证。后续需取得能顺序到达对应旧对象体的可控旧档，或先闭合 ArcToolbox 宿主及对象引用机制；必要时追加旧介质。更多来源的全流扫描只能缩小候选范围，不能代替顺序读取。
