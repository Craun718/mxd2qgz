# Terra 第十轮：FeatureDatasetName 第四串和标志

## 结论

10.1 静态实现中，第四串与标志分别对应同一个接口视图的前两对访问器；成员名称和业务用途仍未证实，因此不能闭合 TODO 的「定性 FeatureDatasetName 第四个字符串和标志」。本轮未更改 TODO。Rust 继续保存原始字符串和字节，仅增加访问器偏移注释，公共 API 和解析行为不变。

## 目标身份与方法

本轮重算 `temp/agent-dataset-name/GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA`：SHA-256 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`；PE32 x86，ImageBase `0x708d0000`，版本资源 FileVersion/ProductVersion `10.1.0.3035`、OriginalFilename `GdbCore.DLL`、CompanyName Esri。10.1 `esriGeoDatabase.olb` 的版本资源亦为 `10.1.0.3035`，原件 SHA-256 `09b741f2627633d36b67985e175632055fe6c5b8cd1000484acc3007f5917f9d`，提取的类型库 SHA-256 `92634979daa3ed0008b36d152e1fdf39cba04944f6bc7bbb5ea4c786bb44bb9a`。哈希和版本资源不是签名认证。所有地址均为静态 VA。

先复查 [上轮调用链](mxd-terra-chain-dataset.md) 和既有反编译。既有独立 REA CLI 对同一 DLL 遇到 Unix socket `Operation not permitted`，见 [续查](mxd-terra-follow-names.md)；按允许的回退方案使用 Ghidra 12.2、JDK 25 和隔离 pyghidra，对既有工程副本反编译，独立项目、配置、TMPDIR 及输出在 `temp/terra-round10-dataset/`；未切换共享 REA 目标。未执行目标 DLL。反编译器对若干复杂函数报告删除不可达块，其结果不能作为该些分支完整行为的证据。

## 接口与持久化对位

`0x709b6560` 的 QI 比较 IID `{3C5BBE8C-C9C3-11D1-AA7C-00C04FA33A15}`，返回完整对象 `+0x20`；该视图虚表 `0x70ad0498`，虚表前 RTTI COL 指示偏移 `0x20`。构造函数中的反编译器标签将该视图写为 `IFeatureDatasetNameImpl`，但这是类型标签，单独不足以证明 IID 或方法的正式成员名称。10.1 GeoDatabase 公开类型库提取物未发现此名称；不能把没有成员定义当作不存在其他私有类型信息的证明。

| 虚表相对偏移 | 实现 | 对象成员与动作 | 持久化对位 |
| --- | --- | --- | --- |
| `+0x0c` | `0x708d8750` | 非空输出指针时，从接口 this `+0x24` 构造 BSTR；即完整对象 `+0x44` | Load `0x709b7c90` 和 Save `0x709b7d50` 的第四 String，IPersistStream this `+0x18` |
| `+0x10` | `0x709a8f40` | 将输入 BSTR 赋给接口 this `+0x24`；即完整对象 `+0x44` | 同一第四 String |
| `+0x14` | `0x709b74d0` | 从接口 this `+0x28` 读字节；恰为 1 时返回 `VARIANT_TRUE`（-1），否则返回 0；空输出指针返回错误 | Load/Save 的 1 字节，IPersistStream this `+0x1c`，即完整对象 `+0x48` |
| `+0x18` | `0x709b7510` | 仅当输入 short 为 -1 写入 1，否则写入 0 | 同一标志字节 |

IPersistStream 接口位于完整对象 `+0x2c`；所以 `0x20+0x24 = 0x2c+0x18 = 0x44`，`0x20+0x28 = 0x2c+0x1c = 0x48`。访问器规范化行为不等于流字节的约束：Load/Save 直接读写 1 字节，保留非 0/1 值仍有必要。字符串 getter/setter 的地址也用于其他接口视图，必须结合相应 this 基址，不能把其他视图的属性名直接套到此视图。

上轮还确认第二串为 `IName.NameString`，第一串为 `IDatasetName.Name`，第三串为 `IDatasetName.Category`，工作空间槽为 `IDatasetName.WorkspaceName`。第四串与标志虽有完整访问器和流字段对位，但接口成员名、生产者写入含义及消费场景缺少可靠证据，不能推测为特定路径、名称或业务开关。Africa 模板 v1 的局部对象链是结构辅助证据，不证明一般取值意义。

## TODO 与验证边界

- `TODO.md`「定性 FeatureDatasetName 第四个字符串和标志」：**不可勾选**；静态访问器对位有进展，成员名及业务含义仍开放。可在未来取得对应 10.1 私有接口声明、可靠调用方或受控运行数据后复核。
- `TODO.md`「FeatureDatasetName 版本 0 分支」：**不可勾选**；本轮没有真实 v0 字节或运行证据。
- 本轮没有其他原 TODO 可因这些新证据整项勾选。没有修改 TODO 或提交。
- ArcMap 10.1 暂不可用；未做目标运行、源工程目视检查，也未做 QGIS 转换结果人工验收。静态分析和合成测试不能替代这些验证。
- 验证：`cargo fmt --check`、`cargo test`、`git diff --check` 均通过；全量测试中依赖介质的既有用例有忽略项。本轮仅修改注释，未新增格式规则或测试；既有 `tests/dataset_name.rs` 覆盖第四串和非规范标志字节原样保留。

主审补证：`0x70ad0498[-1] → 0x70af5160` 的 MSVC Complete Object Locator 记录完整类 `FdbFeatureDatasetName` 和偏移 0x20。类层级中的 BaseClassDescriptor `0x70af52dc` 记录同一非虚成员偏移，类型名原文 `.?AUIFeatureDatasetNameImpl@@`；同偏移的 IUnknown 是其基接口。由此可将 IID 确认到私有接口名 `IFeatureDatasetNameImpl`。四个方法的正式名称和字段的业务用途仍未取得。
