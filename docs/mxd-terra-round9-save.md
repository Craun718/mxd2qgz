# 第九轮：旧档打开时 DocumentInfo 的数据流（2026-11-17）

接续[第八轮](mxd-terra-round8-version.md)，本轮只研究 10.1 静态实现，不改 TODO。地址为 PE 静态映像 VA；正常构造、类注册和虚派发均是条件，不等于运行中已经观察到。

## 身份和方法

本轮重新计算并以 pefile 核对 PE x86 固定版本：ArcMap.exe `10.1.0.3035`、SHA-256 `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97`；PageLayout.dll `10.1.0.3035`、SHA-256 `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f`。与[原生基线](arcmap-10.1-native-baseline.md)一致；资源版本是声明，摘要标识字节，不证明签名或运行身份。

本轮利用已有独立 Ghidra（经桥接工具） 12.1.4 的 PageLayout 全函数导出定位代码，其索引中的 target SHA-256 与本轮 DLL 一致；另重新读取 PE GUID、虚表字节并用 objdump 检查构造、接口调用及关键分支。已有第八轮隔离 Ghidra 12.2/JDK 25 伪代码用于 `0x47a3f0` 辅助定位。伪代码不是源码，PDB 加载状态未知，自动推断的库函数名不作为证据。本轮未切换共享 Ghidra 桥接工具 目标。忽略目录 `temp/terra-round9-*.asm` 等输出含本机路径，分享前须脱敏。

## 从 CMxDoc 成员绑定到 DocumentInfo

1. ArcMap `0x462676` 写 CMxDoc 虚表 `0x54c294`；在同一构造例程 `0x46288d..0x4628ba`，对象 `+0x190` 先清零，随后以 `0x555588` 的 CLSID `5012efc9-6d2c-4f0e-8513-5349010cbce3`、空聚合指针及上下文 `0x17` 调 `0x409bd0`。该助手经 AfCreateInstance/IUnknown、OleRun 后 QI `d3e5c54f-2a84-11d4-9ff2-00c04f6bc78e`，成功接口写入该成员；参见 `0x409bf2..0x409c31`。此前 PageLayout 类表已将该 CLSID 关联 **DocumentInfo**，其类自身 `0x7b691010` 同样输出该 CLSID。失败路径和本机注册或工厂被替换的情况不由此排除。
2. `0x476c31..0x476cb0` 把该成员地址交 `0x40a8d0`，解引用后经 QI 获取 IID `e8e5589d-215a-4322-ba68-e52f187363ee`；成功时带存储、ObjectStream 和 `0x47a3f0` 结果调用接口 `+0x0c`。构造与读取之间成员未做全程序别名穷举：**如果创建成功且该成员仍指向此 DocumentInfo**，静态路径绑定如下；不能声称运行时对象已观察确认。调用返回后的 `0x476cb2` 直接进入临时接口清理，本调用点没有局部检查返回值。
3. PageLayout `0x7b691390` 对该 IID 返回完整对象 `+0x10`；DocumentInfo 构造 `0x7b6910f0` 将此处最终虚表设为 `0x7b71ca9c`，表 `+0x0c`（`0x7b71caa8`）的实读指针为 **`0x7b691650`**。`0x7b691650` 的 `ret 0x10` 对应 this 加三个显式参数；这将第八轮候选确切绑定到 DocumentInfo 的读取槽，而不是根据偏移猜测 put。该 IID 的正式名称本轮未独立核实。
4. 对照另一实现：已有类表和工厂将 `0x7b69ecb0` 绑定 MapDocument；其 QI `0x7b69fe70` 命中同 IID 返回 `+0x20`，构造 `0x7b69ed6f` 设置虚表 `0x7b71decc`，其 `+0x0c` 指向 `0x7b69ec80`。后者仅返回 `0x80004001`（E_NOTIMPL）并 `ret 0x10`。因此相同 IID 不保证相同实现；该对照本身不证明 CMxDoc 实际创建了 MapDocument。

## 第三个参数和流的去向

- 已有隔离反编译 `0x47a3f0` 显示默认结果 5，依据存储状态、Build Number 解析或 short 门控可能给出 3；其反编译报告移除不可达块。它的精确全部边界和业务命名仍未用运行验证，不能把它直接等同保存版本属性。
- DocumentInfo `0x7b691650` 先通过 `+0x14` 清理自身信息；存储指针为空返回 `0x80070057`，打开名为 `Metadata` 的流失败返回 1；流或对象流为空返回失败。成功时 `0x7b6917ac..0x7b6917c3` 标记对象状态，再调用传入 ObjectStream `+0x38` 连接 Metadata。故此处随后的读取属于 Metadata，不应算作 `/Version` 尾部。
- 汇编 `0x7b6917c5..0x7b6917e1` 比较第三个参数是否小于 4：小于 4 按对象现有状态分派 `0x7b692a70` 或 `0x7b692ba0`，后者读取一系列 String，且根据对象内部 short 决定是否继续读取两个四字节字段；不小于 4 走另一套先读 short 的路径。这里参数**控制元数据布局分支**；相关字段位于 DocumentInfo，自身清理槽 `0x7b691550` 清理的是该对象的信息。并未在这些直接语句中看到对 AfDoc `+0x14c` 的赋值。
- 以上是局部静态数据流，不是整个加载链的全局无写入证明：ObjectStream 和其他接口是间接调用，读取器后续还有其他委派；第八轮未解决的 AfDoc 版本 put、别名和其他加载路径仍须审计。尤其不能从 `0x47a3f0` 返回 3 推断保存索引变成 3，也不能推断打开后必然仍为构造初值 5。

## 可勾范围与验证

| 独立缺口 | 状态 |
| --- | --- |
| 10.1 CMxDoc 正常构造的 DocumentInfo CLSID → 目标 IID → 构造后虚表 `+0x0c` → Metadata 加载器的条件静态链 | 可单独落账；受注册、创建、成员替换和运行类型限制。 |
| 两个 PageLayout 实现对该 IID 的槽位差异 | 可单独落账：DocumentInfo 读取，MapDocument 槽返回 E_NOTIMPL。 |
| 分类结果对 DocumentInfo 旧版 Metadata 读取的分支作用 | 可单独落账为局部静态作用；不等于完整元数据格式或所有异常路径闭合。 |
| 旧文档打开后 AfDoc 保存版本索引是否改写 | 不可勾：尚未穷举间接调用及其他路径，也无打开前后属性和保存结果的运行证据。 |
| 10.8 `/Version` 完整写入规则及跨版本兼容性 | 不可勾；本轮未研究 10.8 完整写入，既有未知尾部与失败路径仍开放。 |

这些结果不足以新增 MXD 版本流规则，未修改 Rust 实现；没有用合成数据冒充软件验证。`cargo fmt --check` 与 `cargo test --offline` 通过（部分依赖介质的测试按现有配置 ignored）；`git diff --check`、`git diff --cached --check` 通过。测试运行时工作区存在其他并行修改，结果反映当时的工作区，不证明各并行修改已完成。ArcMap 当前不可用，也未在 QGIS 打开或目视核对，尚无软件人工验收。

主审核对 10.1.0.3035 的 `ESRI.ArcGIS.Carto.dll`（SHA-256 `e638bbe014f6c39d3ec4792c3f410b3bc909d77ae6fe1178e8376c75f69745b4`）CLR 元数据：`{D3E5C54F-2A84-11D4-9FF2-00C04F6BC78E}` 是 `ESRI.ArcGIS.Carto.IDocumentInfo2` 的 COM GuidAttribute。该结果定名了构造成员所持接口；不能用于定名另一个加载接口 `{E8E5589D-…}`。

## 另存临时索引与加载状态的区分

对同摘要的 ArcMap.exe 10.1.0.3035 用 objdump 复核，保存对话框
路径 `0x47cec0` 在 `0x47d26b` 通过 CMxDoc 主表 +0x14c
读取原保存索引到栈局部 `[ebp-0x24]`。选择值经同一对象的
+0x148 写入后，`0x47d34d` 调用校验。校验返回零时在
`0x47d376` 写回该栈局部。通过校验则在 `0x47d407`
置位 `[ebp-0x51]`，后继两个清理路径在 `0x47d662`、
`0x47db83` 根据该标志调用同一写槽，实参也来自
`[ebp-0x24]`。按照 CMxDoc 和 AfDoc 的已对位虚表，
这些调用恢复本次保存前读取的属性值。

旧工程加载的 Serialize 分支在 `0x47a974` 调用
`0x4764d0`，在 `0x47a98b` 将结果写入 **CMxDoc**
的 +0x148 内存成员。这里没有取虚表，也没有经 CMxDoc
+0x1a4 取得 AfDoc。另有保存入口 `0x46705c` 清零
**CMxDoc** 的 +0x14c 内存成员，该地址同样与 AfDoc
+0x14c 保存索引分属不同对象。上述写点限定了几个容易混淆的
状态路径。旧工程加载时其他间接调用是否改写 AfDoc 保存索引
仍待查，未在 ArcMap 10.1 中运行观察。
