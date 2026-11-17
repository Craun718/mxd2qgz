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
