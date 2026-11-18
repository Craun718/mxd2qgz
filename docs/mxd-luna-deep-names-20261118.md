# MXD 深挖：FeatureDatasetName 第四串与标志

日期：2026-11-18。目标为 ArcMap 10.1 `GdbCore.dll`，PE32 x86，ImageBase
`0x708d0000`。分析文件
`/home/gxrs/文档/source/arcmap10.1/binaries/GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA`
的 SHA-256 与 `binaries/manifest.json` 一致：
`ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`。
该文件版本资源在既有 10.1 基线记录为 `10.1.0.3035`；哈希和版本资源仅用于
文件身份/声明核对，不代表签名认证。本记录地址均为静态 VA。

## 结论

- `FdbFeatureDatasetName::Load 0x709b7c90` 和 `Save 0x709b7d50` 按相同顺序
  读写版本、四个 AfCore String、一个字节以及 WorkspaceName 对象槽。第四串
  落在 IPersistStream 视图 `this+0x18`，标志字节落在 `this+0x1c`。
- 完整对象的 IPersistStream 视图偏移为 `+0x2c`，`IFeatureDatasetNameImpl`
  视图偏移为 `+0x20`。第四串访问器读写接口视图 `this+0x24`，即完整对象
  `+0x44`；标志访问器读写接口视图 `this+0x28`，即完整对象 `+0x48`。
- 字符串 getter `0x708d8750` 将接口 `this+0x24` 交给 BSTR 构造函数；setter
  `0x709a8f40` 将输入字符串赋给同一字段。标志 getter `0x709b74d0` 仅在
  字节恰为 1 时返回 `VARIANT_TRUE`（-1），否则返回 0；setter `0x709b7510`
  仅在输入 short 恰为 -1 时写入 1，否则写入 0。
- Load/Save 直接把标志字段作为 1 字节读写，不经过规范化访问器；Rust 继续
  保存任意原始字节是必要的。当前证据能定性访问器的字段位置和转换规则，不能
  定性私有属性正式名称、真实业务含义、生产者的具体赋值来源或消费者用途。
  不应把它命名成路径、别名或业务开关。

## 原始指令核对

用 `objdump -d -Mintel` 对上述同一 DLL 复核以下范围，转储保存在仓库临时目录
`temp/luna-20261118/`，不含 MXD 样本属性：

| 地址 | 指令证据 | 结论 |
| --- | --- | --- |
| `0x709b7c90` | `lea ecx,[edi+0x18]` 后调用 AfCore String reader；其后 `lea edx,[edi+0x1c]`，长度参数为 1，调用字节 reader；最后以 `edi+0x20` 作为对象槽接收位置 | 第四 String、标志字节、WorkspaceName 槽的偏移及顺序 |
| `0x709b7d50` | 对 `edi+0x18` 调用四次 String writer；对 `edi+0x1c` 调用长度 1 的 writer；之后取 `edi+0x20` 写对象槽 | Save 与 Load 字段对称，标志流宽为 1 字节 |
| `0x708d8750` | 检查输出指针，再对接口 `this+0x24` 调用 BSTR 构造函数并写出结果 | 第四串 getter 的成员偏移 |
| `0x709a8f40` | 将入参传给以接口 `this+0x24` 为目标的字符串赋值函数 | 第四串 setter 写同一成员 |
| `0x709b74d0` | `cmp BYTE PTR [ecx+0x28],1`；相等写 short `-1`，否则写 0 | 标志 getter 的字节判定与 VARIANT_BOOL 输出 |
| `0x709b7510` | 比较输入 short 与 `-1`，将相等结果写入接口 `this+0x28` | 标志 setter 的规范化规则 |

接口身份及视图位置参照同版本 RTTI/QI 记录：`0x709b6560` 返回完整对象
`+0x20` 处接口，IID 为 `{3C5BBE8C-C9C3-11D1-AA7C-00C04FA33A15}`；接口
虚表为 `0x70ad0498`，槽 `+0x0c/+0x10/+0x14/+0x18` 对应上述方法。
完整对象偏移由视图偏移加接口内成员偏移得到；IPersistStream `+0x18/+0x1c`
与接口视图 `+0x20` 的 `+0x24/+0x28` 分别汇合到 `+0x44/+0x48`。

## 代码与测试

只更新了 `src/objects/dataset_name.rs` 的中文字段注释：标明完整对象偏移、
访问器槽和标志规范化规则，并保留语义未知的界线。读取器没有按访问器规则
归一化字段。`tests/dataset_name.rs` 已有合成覆盖：第四串读为 `string_d`；
`non_canonical_flag_byte_is_preserved_verbatim` 断言 `0x5A` 原样保留。没有新增
测试，因为这些行为已有断言。

## 证据边界

本轮未获取私有接口 IDL/正式成员名，也未从外部调用者追到确定的业务赋值点或
消费场景；访问器 setter 只说明调用方传入值如何存储，不能证明值的业务来源。
REA CLI 因当前沙箱不能绑定 Unix socket 未启动。本轮通过现有 10.1 DLL 与原始
指令复核字段和访问器，因此没有依赖补导结论；如需继续追调用者，可补导
`GdbCore.dll` 的 `0x709b7c90`、`0x709b7d50`、`0x708d8750`、`0x709a8f40`、
`0x709b74d0`、`0x709b7510` 及 QI `0x709b6560`，并从接口虚表槽追到其调用点。
没有在 ArcMap 运行或人工打开 MXD；合成测试不等于目标软件验收。

## 续查：要素类类别与单类枚举（2026-11-18）

第四串是**要素类的类别描述**；标志为真时，**按数据集名称对象的自身名称枚举
一个要素类**。这是根据生产者和消费者描述的语义；未取得私有接口的原始 IDL
成员拼写。前几节是此前只确定成员偏移的阶段记录。

从 10.1 Desktop `Setup1.cab` 补提取以下 x86 PE，固定版本资源均为
10.1.0.3035，地址为静态 VA。另复核上文 GdbCore 的版本、架构和哈希一致。

| 文件 | SHA-256 |
| --- | --- |
| FileGDB.dll | `73cef6bac97b0bdba8e4b26e2052cefab0b55a32e3a4141c01701d7cbc5f1230` |
| SdeFDB.dll | `6b56d69e3e32736f08716c11f402bb90e9f67c74ed4e7cc017de1f9f3e6aeed0` |
| GdbCoreLib.dll | `10e0af85e3e3825647733f99d573c969d69ccaf47203365e51323fce07008baa` |

1. **类别字符串的生产者。** FileGDB 的 FeatureDataset、SdeFDB 的 SdeDataset
   分别将 `0x6fa21c10`、`0x7e7c5940` 放入 IDataset 虚表 `+0x10`
   （`FullName`，10.1 esriGeoDatabase 类型库 TypeInfo 9）。创建名称对象后先
   对其 `IDatasetName +0x1c` 设置数据集 Category，FileGDB 加载字符串资源
   `0xfa2`（`File Geodatabase Feature Dataset`），SdeFDB 用 `0x578`
   （`SDE Feature Dataset`）。随后通过 `0x6fa238a0` 或 `0x7e7cb610`
   查询同一个私有接口 IID `3C5BBE8C-C9C3-11D1-AA7C-00C04FA33A15`，
   在 `0x6fa21db9` / `0x7e7c5af1` 向该接口 `+0x10` 另写
   `0xfa1` / `0x589` 的文本。PE 英文资源（语言 1033）分别为
   `File Geodatabase Feature Class`、`SDE Feature Class`。因此第四串属于
   要素类类别，和第三串所在的要素数据集 Category 分开，也不用于替代名称或路径。
2. **标志的生产者。** 两个 FullName 方法均在 `0x6fa21dc4..0x6fa21dd9` /
   `0x7e7c5afc..0x7e7c5b11` 读 IDataset 视图 `+0x165` 的单字节，
   转成 VARIANT_BOOL，设置私有接口 `+0x18`。方法所在 IDataset 视图位于完整
   对象 `+0x24`；同一对象的 FdbDataset 基类从 `+0x10` 开始，
   所以这是该基类的 `+0x179`。`FdbDataset::Open 0x70bf9010` 在走常规
   要素数据集初始化以外的 `OpenFeatureTable` 或 `OpenFeatureQuery` 路径后，
   于 `0x70bf914e` 把此字节设为 1。另一生产者是
   `FdbFeatureDatasetEnum::Next(IDatasetName**) 0x70c12ca0`，在
   `0x70c12e00..0x70c12e19` 根据当前枚举条目对应的字节是否为 1
   设置同一标志。
3. **标志的消费者。** GdbCoreLib 导出的
   `FdbFeatureClassEnum(IDatasetName*, const String&, bool) 0x70d4ef50`
   在 `0x70d4f0cd` 查询同一私有接口（`0x70d56270`），在
   `0x70d4f0e3` 调 `+0x14` 取标志。返回 `VARIANT_TRUE (-1)` 时，
   `0x70d4f10b` 调原名称对象的 `IDatasetName::get_Name(+0x0c)`，
   把得到的一个名称插入枚举数组并加一计数。否则转 `0x70d4f180`，
   调用符号可核验的 `AddESRIFeatureClasses`，再调用
   `RemovePrivateExtensionFeatureClasses`，使用正常的子要素类枚举路径。

类型库来自同一介质的 `esriGeoDatabase.olb`（SHA-256
`09b741f2627633d36b67985e175632055fe6c5b8cd1000484acc3007f5917f9d`），
PE TYPELIB 1/1033 资源 SHA-256
`92634979daa3ed0008b36d152e1fdf39cba04944f6bc7bbb5ea4c786bb44bb9a`；
直接按 MSFT 的类型及成员记录解析，TypeInfo 9/11 分别定位 IDataset 和
IDatasetName。补充逆向用 `objdump -d -Mintel`（代码）、PE 导入导出和 RTTI
（调用角色）及 RT_STRING（生产者传入文本）互证。

已有介质工程 Africa 的 `/Maps` 在 `0x3a8` 处有键 44 的版本 1 体：
第三串 `0x3d4` 为 `File Geodatabase Feature Dataset`，第四串 `0x41a` 为
`File Geodatabase Feature Class`，`0x45c` 的标志为 0，工作空间对象头紧随于
`0x45d`。这是两种类别在实际文件中共存的字节证据；真分支仍由原生指令及合成
输入检验。

Rust 添加语义访问器，保持原字段与解析宽度；真值必须是原始字节 1，其他位模式
仍保留。未在 ArcMap 或 QGIS 实际打开工程验证，也未以此推断类型库未公开的
私有成员拼写。
