# TableQueryName 变体流体布局核验（未闭合）

记录日期：2026-11-16。目标为 ArcMap 10.1.0.3035。GdbCore.dll 原件身份为
SHA-256 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`，
PE32 x86，ImageBase `0x708d0000`。本文只记录本次重新取得的 REA 反编译观察，
不修改共享模块入口、TODO 或解析器。

## REA 证据

REA/Ghidra 12.1.4 对上述 10.1 GdbCore 原件成功反编译以下入口：

- Load `0x708f5070`，Evidence `ev_692562cbf1e7d3f7180662b9c83de903e10293297ab9531dd5970cd2e49162d8`。
- Save `0x708f5280`，同一 Evidence。

二者为完整反编译函数体，调用 IStream 的读取/写入结果均有错误传播。Load 观察到：

1. `AfStreamRead` 读 2 字节版本；有符号比较仅拒绝 `version > 2`。
2. 依次 `String::Read` 三次；由 `wchar_t*` 参数地址换算为对象偏移 `+0x08`、`+0x10`、`+0x18`（伪代码的 `param_1+2/+4/+6`）。
3. `LoadObject`，请求 IID 位于 GdbCore `.rdata` `0x70ab8554`，对象槽由 `FUN_708ed690()` 提供。
4. 再依次 `String::Read` 三次，对象偏移 `+0x28`、`+0x2c`、`+0x30`（伪代码 `+0x14/+0x16/+0x18`）。
5. 执行依赖对象状态和指针的接口调用；部分调用在伪代码中为间接虚调用，流消费量无法从该段静态反编译唯一确定。
6. `version > 1` 时另读一个 String（对象 `+0x18`）和 2 字节值（对象 `+0x1c`）；版本 1 不执行该尾段。

Save 观察到：

1. 写出位于 `.rdata` `0x70ab7b6c` 的 2 字节版本常量。
2. 依次 `String::Write` 对象偏移 `+0x04`、`+0x08`、`+0x0c`。
3. `SaveObject` 写对象槽 `+0x10`。
4. 对象槽 `+0x14` 非空时，调用其三组虚表方法取得三个 BSTR；随后对三者分别构造 String 并调用未解析的 `Write_exref`。对象槽为空时，三值保持空，仍执行对应写出路径。
5. 再调用一次 `Write_exref`，最后将对象 `+0x1c` 的 2 字节值写出。

## 未闭合点与实现裁决

目前不能把以上观察作为完整、可靠的线性体布局实现：Load 中的第二组三字符串与
Save 中的三个虚接口取值能数量对应，但它们的具体流写入路径未由已解析 helper
闭合；Save 还有第四次 `Write_exref`，Load 仅在版本大于 1 时出现 String+short
尾段，二者如何对应尚未证明。版本常量的数值、`Write_exref` 的具体目标及其
序列化格式、接口槽 `+0x14` 的运行时对象类型/Load 行为，也未由该函数证据确定。
因此不新增读取器或合成布局测试，以免把未解释的调用当作普通字符串或标量。

这次反编译地址属于 GdbCore 中此前按 RTTI/GetIDType 线索定位的 TableQueryName
候选持久化路径；但当前 Evidence 只直接证明两个函数的指令行为，未重新验证
其虚表归属链或类标识写出。FeatureQueryName 对应完整 Load/Save 仍未定位，保持
开放。仓库现有 `QueryTableName`（CLSID `{793873DE-C0BB-48F4-A6BF-F6F83FA3C1BC}`）
读取器是另一类，不代表本文的 TableQueryName（CLSID `{81CAA5D8-90C5-11D3-9F7B-00C04F6BC886}`）。

本轮未取得 ArcMap 运行时或真实 MXD 字节对这两类的验证；本文为静态 REA 观察，
不宣称人工软件验收。
