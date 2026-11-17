# 第九轮：逐图层 LayerMasking 的 QI 与递归边界（2026-11-17）

本轮从第八轮停止点继续，只新增本记录，没有修改解析器、TODO 或既有改动。结论均为静态证据；参考 MXD 仅作辅助观察，不代表 ArcMap 实际执行。

## 身份与方法

本轮重新读取 PE 固定版本资源：Map、FeatureLayer、AnnoLayer、LayerLib 均为 x86（Machine `0x14c`），FileVersion / ProductVersion 均为 `10.1.0.3035`；重算 SHA-256：

| 文件 | SHA-256 |
| --- | --- |
| Map.dll | `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8` |
| FeatureLayer.dll | `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60` |
| AnnoLayer.dll | `b487f3e0dc0e70dbc4b75f72eb6dcd8d5521eb422dbea6e3b2e201489a2d33a5` |
| LayerLib.dll（资源名 LayerUI） | `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02` |

文件位置及资源名的历史核对见 [前轮](mxd-terra-round8-mask.md)、[FDO 记录](mxd-terra-nextpass-fdo.md)。版本和哈希不证明签名或运行状态。独立 REA 6.9.0 CLI、JDK 25、`/opt/ghidra`、仓库专用 TMPDIR 对 Map `0x79403bc0` 的本轮 run `74a743dd-ea19-463a-9c46-8badc407ea56` 导入后因 Unix socket `Operation not permitted` 失败，没有返回反编译证据。随后在复制的独立工程中用 pyghidra/Ghidra 12.2 定位原始 IID 字节及地址引用，并以 `objdump` 反汇编和 PE 原始数据复核；未切换共享 REA 目标。以下地址均为这些文件的静态 VA。IID 搜索未穷举间接引用或跨模块聚合。

## Map 的实际控制流

- `0x79403d50` 调用 Map 自身的 `LoadLayerMasking` 后，从对象 `+0x150` 数组及 `+0x154` 数量遍历图层。对于每项，`0x7941a5b0` 向图层 QI 内存地址 `0x79488cf8` 的 IID `{F9D6AA98-5586-486F-8242-670AEDF7860E}`；结果非空且 Map 版本有符号小于 40 时，该项直接跳到释放及下一项。该接口名称未确认。
- 否则，`0x7941a620` 向该图层 QI `0x79488bb8` 的 IID `{536427DA-5354-491D-AE92-ABEBAA8BB05B}`，将实际返回指针传给 `0x79403bc0`。汇编在 `0x79403e09..23` 明确传入结果，纠正了先前反编译把参数化成常量空指针的假象。此 IID 即传给 LayerLib `LoadLayerMasking` 的接口；其正式类型名尚未从类型库单独核对。
- `0x79403bc0` 入口在接口指针为空时直接成功返回，**连子项查询也不进行**；非空时先读当前遮罩，再经 `0x7941a3e0` 对该接口 QI `0x7948237c` 的 IID `{BA119BC4-939A-11D2-A2F4-080009B6F22B}`。若成功，虚表 `+0x0c` 获取数量，`+0x10` 逐项取得对象，再经 `0x7941cb40` 查询遮罩接口，递归调用 `0x79403bc0`。因此遍历是条件递归而非固定“六个图层对应六段”；不能仅根据一个对象的复合接口存在就推断其遮罩不存在时仍会遍历子项。子项对象至遮罩 QI 的完整辅助函数尚未逐指令复核。
- 这些 IID 的标签以调用行为描述；尚未重新枚举类型库，不能把观察到的槽直接扩展为全部接口成员语义。

## 具体类与参考样本

- FeatureLayer.dll 的 GroupLayer 构造/析构使用的 IPersistStream 虚表 `0x6f7af244` 与既有 `0x6f711270` Load 锚点一致。在同一类的 QI 分支 `0x6f710211..240`，比较 `0x6f7af4e8` 的 `{536427DA-5354-491D-AE92-ABEBAA8BB05B}`，命中返回对象 `+0x70` 并 AddRef。该分支证明此类的静态接口实现，不能推断样本各类均相同。GroupLayer 不是参考列表中已识别的六类之一。
- AnnoLayer.dll 的 FDOGraphicsLayer 构造链设置 `+0x80` 虚表 `0x6b1abb1c`（`0x6b18dd38`）。其 QI 比较 `0x6b1abe90` 的 `{BA119BC4-939A-11D2-A2F4-080009B6F22B}`，`0x6b18ed1f..2d` 命中后跳至 `0x6b18ef2c` 返回 `+0x80`。虚表 `+0x0c/+0x10` 分别指向 `0x6b199190/0x6b1991c0`：前者输出计数成员，后者按索引查找并查询返回子对象。该静态链支持它是可枚举复合对象，但递归能否进入还取决于前述遮罩 QI。
- FDOGraphicsLayer 的 QI 对未在本对象表中直接处理的 IID，在 `0x6b18ef03..13` 将请求转交 `+0xb8` 持有对象的虚表 QI；既有构造证据表明该对象是聚合创建的 FeatureLayer。不能因为 AnnoLayer 中没有遮罩 IID 的直接常量就认定 QI 失败。FeatureLayer.dll 的上述 GroupLayer 命中也不是聚合 FeatureLayer 的 QI 证明。还须沿聚合对象的非委派 IUnknown、实际类 QI 和潜在基类/聚合分支逐项确认，避免循环委派与类身份混淆。
- 参考文件已有证据为 Map 版本 51、列表六项，第一项 FDOGraphicsLayer 含一个已解析子层，第二至四项 FeatureLayer（第二项回引），后两项 RasterLayer，其体按范围保留。列表顺序及内嵌子项见 `src/layers/layer_list.rs` 与现有解析记录；还没有对 RasterLayer 和该子层完成相关 QI 链，且有界跳过可能漏登记内部对象。
- 本轮 CLI 再次到 `/Maps:0x253be`，保留 431 字节；该偏移起前 8 字节为 `ff ff 02 00 00 00 00 00`。这仅是停点字节，不能据此将 `ffff` 强认作某个图层的遮罩版本、跳过首项或判定下一字段。首项接口状态和递归结果未闭合，不能安全逐层推进。

## 可勾范围、验证和后续

- 可单独落账：Map 版本小于 40 时的 IID 查询与跳过分支、逐项遮罩接口 QI 实参及非空后条件递归的上述**静态指令链**；GroupLayer 的遮罩 IID 命中及返回偏移；FDOGraphicsLayer 的复合 IID、子虚表和聚合 QI 转发链。各项均以已列静态范围为限。
- 不可勾：参考样本逐图层接口状态与逐项读取、任意图层或子层支持、完整 LayerMasking、Map 对象末尾及根循环。下一步先从 FDOGraphicsLayer 聚合 FeatureLayer 的真实 QI 入口追该 IID，再查其子层和 RasterLayer 的实际 QI；只有可以按原生规则确定第一项及其递归子项时，才接入逐项读取并覆盖未知状态、截断和回退。
- 本轮无证据充分的解析器改动；保留 `0x253be` 和既有引用表状态，不以试读成功充当接口证明。`cargo fmt --check` 和 `cargo test --offline --test terra_round8_mask` 通过（2 项），README 用法 `cargo run --offline -- parse 'samples/测试底图.mxd'` 可运行并重现上述停点。本轮未重跑完整测试套件，因为未改 Rust。测试只覆盖已有读取器，不验证新 QI 结论。
- ArcMap 10.1 暂不可用；未在 ArcMap 或 QGIS 中打开工程并目视验收。诊断输出在仓库忽略的 `temp/terra-round9-mask/`，可能含敏感路径，分享前须脱敏。

主审从 10.1.0.3035 的 `ESRI.ArcGIS.Carto.dll`（SHA-256 `e638bbe014f6c39d3ec4792c3f410b3bc909d77ae6fe1178e8376c75f69745b4`）COM GuidAttribute 补定名：`{F9D6AA98-…}` 为 IBasemapLayer，`{536427DA-…}` 为 ILayerMasking，`{BA119BC4-…}` 为 ICompositeLayer，后者的方法依次为 get_Count、get_Layer，与 `+0x0c/+0x10` 反汇编一致。这为旧版本 BasemapLayer 跳过分支和条件递归提供类型名，不代表参考样本各图层 QI 的结果已确认。
