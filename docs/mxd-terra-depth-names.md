# FeatureDatasetName 与 TopologyName 的属性对位（2026-11-17）

本轮只讨论 10.1 静态实现，不修改 TODO；此前的结构、版本门和样本边界见 [原始结构记录](arcmap-10.1-dataset-name.md)、[名称续查](mxd-terra-follow-names.md)。下表“Load 偏移”均相对于 **IPersistStream 接口指针**，不是完整对象；两类该接口均位于完整对象 `+0x2c`。这一点纠正旧记录中把这些偏移称作 `obj` 偏移的写法。

## 证据与定位

GdbCore 原件 `temp/agent-dataset-name/GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA` 本轮重算 SHA-256 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`；PE x86，版本资源 `10.1.0.3035`、CompanyName Esri、OriginalFilename GdbCore.DLL，ImageBase `0x708d0000`。地址为该文件静态映像地址，不是运行地址。版本资源与哈希不是签名认证。

10.1 类型库提取物 `esriGeoDatabase.olb.392FE3A8_65C9_4463_A15B_89F00F8F3FE8` 重算 SHA-256 `09b741f2627633d36b67985e175632055fe6c5b8cd1000484acc3007f5917f9d`；对应已提取的 `big.tlb` 哈希 `92634979daa3ed0008b36d152e1fdf39cba04944f6bc7bbb5ea4c786bb44bb9a`，关联提取过程见 [基线](arcmap-10.1-native-baseline.md)。本轮以隔离 Wine 前缀运行本地 LoadTypeLibEx 成员枚举工具，得到 `IDatasetName`（IID `{89E75917-C287-11D1-AA77-00C04FA33A15}`）在 IUnknown 之后依序为 Name get/put、Type get、Category get/put、WorkspaceName get/putref、SubsetNames get；`ITopologyName`（IID `{505B6FDD-9E6E-42F9-B705-53355B2F1DEF}`）依序为 DirtyAreasName get、TopologyErrorName get、FeatureDatasetName get/putref。枚举进程为 64 位，输出 `oVft` 为 8 字节槽；本记录按**成员序**转换成目标 x86 的 4 字节槽，而非直接使用 64 位偏移。类型库只提供接口契约，属性归属还需下面的 QI、虚表及函数交叉证据。

原研究已用 REA 取得两个 Load 的证据；本轮复用 [前次独立 REA CLI 失败](mxd-terra-follow-names.md)（Unix socket `Operation not permitted`，未切换共享目标）的记录，采用隔离 pyghidra/Ghidra 12.2、JDK 25，复制既有工程，仓库内独立 TMPDIR 与配置进行新函数取证。原始结果和成员枚举脚本在忽略目录 `temp/terra-depth-names/`，可能含本机路径；分享前需检查脱敏。汇编核对 QI 与 GUID，伪代码可能有类型传播和间接调用限制，未执行 GdbCore。

RTTI COL 显示两类的 IPersistStream 虚表前 COL 偏移均为 `0x2c`；FeatureDatasetName QI `0x709b6560` 比较 IDatasetName IID 后返回完整对象 `+0x14`，TopologyName QI `0x7095e160` 同样返回 `+0x14`，其 ITopologyName IID 分支返回 `+0x18`。对应虚表分别为 FeatureDatasetName `0x70ad0430`、TopologyName `0x70aca3ac`（IDatasetName）以及 `0x70aca3dc`（ITopologyName）。

## FeatureDatasetName

| 流字段 | Load 偏移 → 完整对象 | 对位证据与边界 |
| --- | --- | --- |
| 第 1 String | `+0x0c` → `+0x38` | IDatasetName +0x0c/+0x10 为 `0x708d8750`/`0x709a8f40`；getter/setter 分别访问接口 this+0x24，即完整对象 +0x38。**Name**。 |
| 第 2 String | `+0x10` → `+0x3c` | 尚未建立实际接口属性映射，保留原值。 |
| 第 3 String | `+0x14` → `+0x40` | IDatasetName +0x18/+0x1c 为 `0x70a401e0`/`0x70a40210`，访问接口 this+0x2c，即完整对象 +0x40。**Category**。 |
| 第 4 String | `+0x18` → `+0x44` | 尚未建立实际接口属性映射，保留原值。 |
| 1 字节 | `+0x1c` → `+0x48` | 完整对象 +0x20 的次级表 `0x70ad0498` 的 +0x14/+0x18 分别为 `0x709b74d0`/`0x709b7510`，访问该接口 this+0x28，即同一字节；getter 仅在字节恰为 1 时返回 VARIANT_TRUE，setter 仅将输入 -1 写为 1。QI 比较 IID `{3C5BBE8C-C9C3-11D1-AA7C-00C04FA33A15}` 返回该接口，但本轮未确认接口和属性名；不能把访问器的规范化结果等同磁盘原值。 |
| IWorkspaceName 槽 | `+0x20` → `+0x4c` | IDatasetName +0x20/+0x24 为 `0x7091b960`/`0x70962020`，访问接口 this+0x38，即完整对象 +0x4c。**WorkspaceName**。 |

IDatasetName +0x14 的 `0x7092cc40` 返回常数 4（Type），不是另一个已读取的字符串；类别值与界面展示含义不作推断。第二、第四字符串及字节属性未定，故原「FeatureDatasetName 字段业务语义」整项**未闭合**。

## TopologyName

| 流字段 | Load 偏移 → 完整对象 | 对位证据与边界 |
| --- | --- | --- |
| 第 1 String | `+0x0c` → `+0x38` | IDatasetName +0x0c/+0x10 的 `0x7095e4f0`/`0x7095e510` 分别转调完整对象 +0x10 接口的 +0x10/+0x0c；目标 `0x708d7600`/`0x70a401c0` 访问该接口 this+0x28，即完整对象 +0x38。**Name**。 |
| 第 2 String | `+0x10` → `+0x3c` | IDatasetName +0x18/+0x1c 为 `0x708d7600`/`0x70a401c0`，访问接口 this+0x28，即完整对象 +0x3c。**Category**。 |
| IDatasetName 槽 | `+0x14` → `+0x40` | ITopologyName +0x14 getter `0x70933a80` 和 +0x18 setter `0x7095e9d0` 访问接口 this+0x28，即完整对象 +0x40。**FeatureDatasetName**；getter 返回槽接口并 AddRef。setter 对非空输入先调用其 IDatasetName +0x20 获取 WorkspaceName，再更新该槽与完整对象 +0x44 的工作空间指针。 |

`0x7095e5c0` 装载该槽后对其 IDatasetName 虚表 `+0x20` 调用，按类型库成员序为 **get_WorkspaceName**，并非已证实的“通知”。其参数/结果和错误传递依静态函数可见，但被调对象实现可能有副作用，不能从间接调用本身证明不读流或证明运行时行为。TopologyName 自身 IDatasetName.WorkspaceName 访问的是完整对象 `+0x44` 的缓存指针（`0x70a49440`/`0x7093b0b0`），与持久化的 FeatureDatasetName 槽不同。

由实际 QI、访问器及 Load 对位，TopologyName 两字符串和嵌套槽的**静态属性命名子项可闭合**；整体运行语义、异常和真实拓扑对象仍需另行验证。原生在 Load 成功后无可见空指针检查即调用槽对象，解析器的空槽处理不等同原生成功路径。

## 验证与剩余边界

Rust 公共字段名保持兼容，只更新 `dataset_name.rs` 的已证实语义注释；保留未定字段原始值，不修改解析行为或 TODO。`cargo test` 通过（有依赖本地介质的忽略测试）；`rustfmt --check --edition 2024 src/objects/dataset_name.rs` 通过。全仓 `cargo fmt --check` 因已有共享文件 `src/maps/mxd_maps.rs` 的格式差异未通过，本轮没有修改该文件。已有 Africa v1 局部对象链仅为辅助；TopologyName 无经宿主链确认的真实对象，两个类的 v0 仍无真实旧档或受控运行证据。合成测试只能验证 Rust 解析行为。未运行 ArcMap 10.1 或 QGIS，未做人工目视验收。
