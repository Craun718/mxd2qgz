# Terra 第十轮：Map 逐图层遮罩接口（2026-11-17）

本轮继续 [第九轮](mxd-terra-round9-mask.md)，以静态类实现和 RTTI 交叉检查 `ILayerMasking`，未修改解析器或 TODO。ArcMap 10.1 暂不可用，以下均不是运行观察。

## 身份与方法

PE 固定版本资源和 Machine 复核：FeatureLayer、AnnoLayer、RasterLayer 均为 x86 `0x14c`，版本 `10.1.0.3035`。SHA-256 分别为 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`、`b487f3e0dc0e70dbc4b75f72eb6dcd8d5521eb422dbea6e3b2e201489a2d33a5`、`9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6`，与既有基线一致；哈希及资源声明不验证签名。

隔离 Ghidra 桥接工具 6.9.0 CLI 以 JDK 25、仓库专用 TMPDIR 对 AnnoLayer `0x6b1a1890` 发起 run `301df0d3-e94e-4ebd-9eb4-625d1589202a`；Ghidra 12.2 导入后 Unix socket `Operation not permitted`，无反编译结果。随后使用仓库隔离 pyghidra/Ghidra 12.2、JDK 25：FeatureLayer 和 AnnoLayer 复制第九轮工程后打开，RasterLayer 独立导入；对指定入口建函数、反编译，并以 PE 数据和 `objdump` 复核比较常量与控制流。没有切换共享 Ghidra 桥接工具 目标。原始日志、汇编及伪代码在忽略目录 `temp/terra-round10-mask/`，可能含本机路径。

RTTI 按最终虚表前一指针取 COL，COL `+0x10` 取类层级，再取层级 `+8/+0xc` 的数量和 BCD 数组；BCD 类型名在类型描述符 `+8`，同时检查 `mdisp/pdisp`。这只核对编译器记录的基类，不排除聚合或 QI 委派。目标 IID 为 `ILayerMasking {536427DA-5354-491D-AE92-ABEBAA8BB05B}`。

## 类级证据

- **FDOGraphicsSublayer**：主虚表 `0x6b1ac764`、COL `0x6b1af338`、层级 `0x6b1af34c`，24 条 BCD 中无 `ILayerMasking`；`+0x44` 的 BCD 是 `IDocumentVersionSupport`，`pdisp=-1`，修正旧文档的未定名。主 QI `0x6b1a1890` 的直接比较均不是目标 IID；未命中转 `0x6b1a3e90`，再到 `0x6b1a3b00`，后者只比较 IUnknown 和另外两个 IID，最终 `0x6b1a3bea` 返回空及 `0x80004002`。因此此本地类的该 QI 静态结果为不支持；不推广到其他子类或运行时替换。
- **RasterLayer**：RTTI 类型 `.?AVRasterLayer@@` 的最终主虚表 `0x7c76c004`、COL `0x7c774178`、层级 `0x7c77418c`，103 条 BCD 无 `ILayerMasking`。主 QI `0x7c73aaa0` 经其直接 IID 比较，目标不命中；`0x7c73b321..324` 转 `0x7c75d9b0`，再转 `0x7c75d680`，其余三个比较也非目标，最终返回 `0x80004002`。因此该本地 RasterLayer 类静态不支持该接口。RTTI 给出的 `IDocumentVersionSupport` 偏移为 `+0x120`；不能以未搜索到 IID 字节单独作为结论。
- **FeatureLayer**：构造链 `0x6f6db953` 设置主虚表 `0x6f7ace14`，同一对象 `+0xd0` 虚表 `0x6f7acfb0` 的 Load 槽指向既有 FeatureLayer Load `0x6f6e9940`。主虚表 COL `0x6f7b6fac`、层级 `0x6f7b6fc0`，178 条 BCD 无 `ILayerMasking`。主 QI `0x6f6e7a40` 的直接比较与基类 `0x6f73c440 → 0x6f73c0d0` 均未匹配目标，后者返回 `0x80004002`。IPersistStream 子接口 QI 经 `0x6f709aa0 → 0x6f6d8210` 转到控制未知接口，因此不能把这个委派入口当成非委派入口；主虚表 QI 和委派入口须区分。
- **FDOGraphicsLayer 聚合**：最终主虚表 `0x6b1ab748`、COL `0x6b1adc48`、层级 `0x6b1adc5c` 的 74 条 BCD 无目标接口。第九轮已确认外对象未直接处理 IID 时向 `+0xb8` 的聚合对象 QI；上述 FeatureLayer 本类 QI 的负结果提供强支持，但本轮没有从 `AfCreateInstance` 工厂及返回的 `+0xb8` 指针完整逐指令证明其非委派入口就是 `0x6f6e7a40`，也未排除工厂运行时替代实现。因此聚合结果仍保留为**高度可能不支持，未闭合**；不能仅用 RTTI、模块全局 IID 扫描或内部 IPersistStream 的委派 QI 断言。

## 参考样本边界与可勾范围

参考样本 Map 版本 51，FDOGraphicsLayer 内联子层计数为 1（`src/layers/fdo_graphics_layer.rs` 记录 `/Maps:0x2c91`），与某些旧记录中“计数 0”不一致，须以当前读取链和字节为准；该子层在其父层遮罩接口为空时不会被 `0x79403bc0` 递归检查。其余顶层项包含 FeatureLayer、RasterLayer 和回引；有界未知体可能遗漏引用登记。因聚合返回指针的身份尚未闭合、列表中有跳过的对象体，未把具体样本的类级推断固化为读取策略，未试读停点后字段。

本轮实际 CLI 仍停在 `/Maps:0x253be`，保留 431 字节；另有图层体未知范围 `0x4fda` 和 `0x145c1`。**没有新的样本消费边界**，该偏移的 `ffff` 仍不能凭试读成功赋义。可单独落账本轮三个本地类的上述 QI/RTTI 静态检查及聚合转发的部分证据；不能勾选完整聚合 QI、逐图层读取、完整 Map 属性或后续 Map 根循环。现有 TODO 中没有仅覆盖这些本轮静态子结论的独立未勾条目，因此本轮无原 TODO 可整项勾选；尤其第 137 行 Map 未定属性和第 330 行后续 Map 循环仍开放。

下一步应追 FeatureLayer 的聚合工厂、返回非委派 IUnknown 与 `+0xb8` 的虚表关系，确认目标环境的 CLSID 工厂实现，并按对象实际身份和引用关系建立保守的接口判定；之后依据 Map Load 原生字段顺序逐项推进，并对未知类、回引、截断、引用表回退添加测试。不得以合成测试代替 ArcMap/QGIS 人工验收。

验证：`cargo test --offline --test terra_round8_mask` 两项通过，`git diff --check` 通过；按 README 执行 `cargo run --offline -- parse 'samples/测试底图.mxd'`，结果如上；本轮未改 Rust，未声称完成转换或软件内目视检查。
