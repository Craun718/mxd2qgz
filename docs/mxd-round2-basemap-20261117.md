# BasemapLayer 10.1 原生静态取证

安装介质 `Desktop/Setup1.cab` 条目 `BasemapLayer.dll.93A93CDE_D64F_4B7A_A2BA_829DB1A9C7E4`，PE FileVersion 10.1.0.3035、ImageBase `0x6b630000`、SHA-256 `c8f4692970c5311e5d591f317c5e6c0379e9e6b9ad62a2383c71ecb6c8f64aad`。配置 `esri.clsid.xml` 将 `{DA4122BF-7B07-4158-88B0-19D342BED8BA}` 映射到 BasemapLayer；本轮另以原生 GetClassID 和虚表核对。提取文件、Ghidra 项目、伪代码及 objdump 汇编保存在 `temp/round2-basemap-native/`，未运行 ArcMap。

## 原生身份及字段序

RTTI `.?AVBasemapLayer@@` 的 TypeDescriptor `0x6b6c0860`，COL `0x6b6b0934`，IPersistStream 候选虚表 `0x6b6ac1a8`。槽 +0x0c 为 `0x6b6468a0`，汇编从 `0x6b6ab750` 复制 16 字节，PE 内容解码为上述 BasemapLayer CLSID；槽 +0x14 为 Load `0x6b646260`，+0x18 为 Save `0x6b6465d0`。Load 伪代码为 `temp/round2-basemap-native/0x6b646260.c`，汇编 `load-asm.txt`；Ghidra 12.2 反编译输出不等于原始源码。

Load 读取 short 版本，原生有符号 `>11` 拒绝；依次读 AfCore String、short×2、版本 <8 再读 short、double×2、short。版本 >1 读 16 字节 GUID，>2 读 `LoadObject`（请求 IID `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`），>3 再读 `LoadObject`（IID `{00A527CF-28DA-4043-9CF7-E9911172F05F}`）；版本 5、6 或 >8 读 short；版本 >5 调用 `CPrepareResultsContainer::LoadResults`，版本 >8 读 short。其后 `0x6b6464a9` 读取 int32 计数，正数循环中 `0x6b6464eb` 调 AfCore `ExtraSafeReadObject`，请求 IID `{34C20002-4D3C-11D0-92D8-00805F7C28B0}`（ILayer）；成功时调用集合接口槽 +0x54。版本 >9 在循环后读扩展、String 和两个 double；版本 >10 再读 8 字节。IAT 地址 `0x6b6aa0e4` 和 `0x6b6aa0e8` 经 PE 导入表分别确认 AfStreamRead(int32) 和 ExtraSafeReadObject。

## Africa 已证前缀与停止点

Africa `/Maps` 顶层 ExtraSafe 图层头在 `0x202`，标记 3、有界体长度 25585；BasemapLayer 对象键 40，体从 `0x220` 开始。按上述 Load 顺序和样本字节：版本 11 在 `0x220..0x222`，String `Basemap` 在 `0x222..0x236`，两个 short 在 `0x236..0x23a`，两个 double 在 `0x23a..0x24a`，short 在 `0x24a..0x24c`，16 字节 GUID 在 `0x24c..0x25c`。版本 11 不读旧版第三 short；下一个对象槽从 `0x25c` 开始。未知标量和 GUID 均不赋业务含义。

未走完两个前置 LoadObject 与 `LoadResults`，故尚未证明子层计数字节位置，也未证明键 42 的首次登记或从 Basemap 到 Legend 的回引。不能通过扫描候选 GUID 或预登记键跳过这些对象。最小后续工作是定位两个请求 IID 的实际实现并解析其体，再分析 MappingCoreLib 10.1 的 `LoadResults`，从真实对象流上下文到达计数并逐项按 ExtraSafe 边界读取；最后才可核实键 42。未新增 Rust 读取器或接入共享入口。
