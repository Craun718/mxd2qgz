# GdbCore 查询名称续查（2026-11-17）

范围：FeatureQueryName、TableQueryName 的 10.1 身份、接口和持久化路径；不修改 TODO 或公共入口。结论均为静态分析，尚无原生运行记录或宿主链定位的真实对象体。

## 原件与方法

- GdbCore 原件为 `temp/agent-dataset-name/GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA`，重新计算 SHA-256 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`；PE32 x86、ImageBase `0x708d0000`，PE 版本资源 FileVersion `10.1.0.3035`、CompanyName `Esri`、OriginalFilename `GdbCore.DLL`。以下地址只对应此文件的静态映像，不是运行地址；版本资源不构成签名认证。
- 既有 10.1 类型库提取物 `esriGeoDatabase.olb.392FE3A8_65C9_4463_A15B_89F00F8F3FE8` 本轮重算 SHA-256 `09b741f2627633d36b67985e175632055fe6c5b8cd1000484acc3007f5917f9d`，PE 版本资源为 `10.1.0.3035`，FileDescription 和 OriginalFilename 均为 `esriGeoDatabase.olb`。既有 LoadTypeLibEx 枚举转储的索引 675、677 为两个 typekind=5 类，分别是 TableQueryName `{81CAA5D8-90C5-11D3-9F7B-00C04F6BC886}` 和 FeatureQueryName `{81CAA5D9-90C5-11D3-9F7B-00C04F6BC886}`；转储并非本轮重新枚举。10.1 `esri.clsid.xml` 对应声明与之相符。类型库本身并不证明调用发生。
- 独立 Ghidra 桥接工具 6.9.0 CLI、Ghidra 12.2、JDK 25 以仓库内独占 TMPDIR 分析 `0x708f4ab0`；run `38615cf7-7e7f-41ed-993b-1456e3278bde` 因 Unix socket `Operation not permitted` 失败，没有产生函数证据，也未改变共享服务目标。改为本机 pyghidra/Ghidra 12.2、JDK 25，打开**复制**的既有 GdbCore 分析工程，对所列函数重新定点反编译；再用 PE 原始数据、导入表和 objdump 指令交叉核对。脚本、伪代码和失败诊断在忽略目录 `temp/terra-follow-names/`，可能含本机路径，分享前脱敏。反编译不是原始源码，部分类型传播和间接调用仍有不确定性。

## 归属链及重要更正

| 类 | RTTI TypeDescriptor / 主表前 COL / 主表 | 次级持久化表 | 该表 +0x0c / +0x14 / +0x18 |
| --- | --- | --- | --- |
| TableQueryName | `0x70b2d81c` / `0x70ae052c` / `0x70aba628` | `0x70aba6dc`，前一指针 `0x70ae04b4` | `0x708f46c0` / `0x708f5070` / `0x708f5280` |
| FeatureQueryName | `0x70b2d7f8` / `0x70ae03e8` / `0x70aba700` | `0x70aba7b4`，前一指针 `0x70ae0370` | `0x708f5760` / `0x708f57a0` / `0x708f5800` |

主表前 COL 的 TypeDescriptor 指针分别指向上表所列 RTTI；构造函数 `0x708f4710` 写 Table 主表及完整对象 `+0x20` 的 `0x70aba6dc`，派生构造 `0x708f55a0` 先调用前者，再改写主表和 `+0x20` 次级表为 Feature 的对应地址。两个标识函数对空输出返回 `0x80004003`，非空分别写出上述对应 CLSID。

**接口更正**：`0x708f4960` 的 QI 分支在 `0x708f49cb`/`0x708f49df` 比较 `0x70a8840c` 和 `0x70a883fc`，原始 GUID 分别为标准 `IPersistStream` `{00000109-0000-0000-C000-000000000046}` 和 `IPersist` `{0000010C-0000-0000-C000-000000000046}`；匹配后于 `0x708f4a75` 返回完整对象 `+0x20`。故次级表的 +0x0c 标识函数是 **IPersist::GetClassID**，+0x14/+0x18 分别是 **IPersistStream::Load/Save**。Feature 的 QI `0x708f5700` 未匹配其新增接口时委托该 Table QI。旧文档依据主表 +0x14 同构将它称作 `IPersistVariant::GetIDType`，不足以成立；相同函数指针同时出现在主表不能单凭位置定接口。亦不能仅凭该分支断言其他 QI 后备路径不支持 IPersistVariant，需单独追证。`0x708f4ab0` 是另一接口的对象打开相关方法，不能作为 Save 候选。

## Load/Save 与 helper

下面偏移相对于传入 IPersistStream 接口的 this，即完整对象 `+0x20`，不等同完整对象偏移。Table 的 `0x708f5070` 依次：读取 i16 版本（有符号 `>2` 拒绝）；`String::Read` 三次（this `+4/+8/+0xc`）；`LoadObject`（请求 IID `0x70ab8554`，即 `{FADD975C-E36F-11D1-AA81-00C04FA33A15}`，既有研究定名 IWorkspaceName）；再读三次 String（`+0x28/+0x2c/+0x30`）；依对象状态可能构造对象并对该对象调用三个 setter；版本至少 2 时再读 String（`+0x18`）和 2 字节（`+0x1c`）。这些 setter 调用不是已证明的额外流读取，但其异常、状态和间接接口副作用仍需完整核查。

Save `0x708f5280` 通过 `0x70ab7b6c` 写出的版本常量经原始字节核对为 `02 00`。先写前三个 String，再 `SaveObject`；取得三个 BSTR，构造临时 String 并分别写出，最后写 this `+0x18` String 和 `+0x1c` 两字节。此前伪代码中的 `Write_exref` 由本轮汇编消歧：`0x708f52b6` 将 IAT `0x70a873a8` 装入 EBX，导入名是 AfCore `?Write@String@@QAEJPAUIStream@@@Z`；`0x708f5381`、`0x708f53a7`、`0x708f53cd` 和 `0x708f540c` 均调用同一 EBX，第四次先以 `lea ecx,[edi+0x18]` 设置 this。故这四次均为 `String::Write`，不是另一个未知格式 helper。空对象槽时前三个 BSTR 初始化为空，但空 String 的实际字节仍须按已验证的 String 编码处理。

Feature `0x708f57a0` 先调用 Table Load，成功后读 this `+0x38` 的 String，以及 `+0x3c`、`+0x40` 各 4 字节；`0x708f5800` 对称调用 Table Save 后写这三项。汇编确认读写各 4 字节；业务语义及实际版本兼容性未确定。保留值为原始位模式较安全。两包装函数对中间错误返回；成功路径是否接受短读仍取决于 AfStreamRead 等 helper 的实际契约。

## 实现与可勾项

这次闭合了**10.1 静态定位子项**：两类 CLSID/RTTI、IPersistStream QI 与 GetClassID、Table Load/Save、Feature 追加段及四次 String 写调用归属。它们可以作为独立研究成果记账，但不能勾选原 TODO 的「FeatureQueryName/TableQueryName 的 IVariantStream 体」：已定位的是 IPersistStream，不能把两种路径混为一谈；仍需核查 QI 后备与实际 ObjectStream 分派、嵌套对象和流上下文、短读错误契约，并用受控 10.1 工程或目标软件运行验证完整体和边界。

本轮不新增解析器或合成测试：尚未确认原 TODO 所称 Variant 路径及其宿主，也未闭合嵌套对象和边界；构造测试不能证明流实际到达或互操作。没有运行 ArcMap 或 QGIS，未进行人工目视验收。`QueryTableName`（`{793873DE-C0BB-48F4-A6BF-F6F83FA3C1BC}`）仍是不同类，不能以其解析器冒充上述两个类。
