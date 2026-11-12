# Desktop 10.8 VariantStream 已知前缀

## 目标与证据来源

本文只记录静态逆向观察，不把规则推广到其他 Desktop 版本。目标是
ArcGIS Desktop 10.8 安装包中的 `AfCore.dll`：

- SHA-256：`560be16e0a99b9de4599f8241c421bddf0dcc8b26509f584074dea53b0ac4aae`
- 文件版本：`10.8.0.12790`
- 分析：REA 6.8.0 / Ghidra 12.1.4
- 分析配置摘要：`e763201e522a4b123fb330eede09834c44ddf2bb90702e50eea219801be95539`

REA 会话关闭时写入的本地分析快照是
`temp/arcgis/afcore-variant-snapshot.json`。该目录被忽略且可能包含目标软件
路径，不进入提交。

下表地址都是该 DLL 的进程内虚拟地址，不是 MXD/OLE 流偏移。

| 过程 | 地址 | REA parent evidence |
| --- | --- | --- |
| 全局 `Read` | `0x100581d0` | `ev_b63d9262a034ab1eb3bddd4c353dc71d5ba8705c51a77879171a27385fbda81b` |
| `ReadString` | `0x10058010` | `ev_2fa268a6dd24845dd2bc905f3b9d3595fbc6682cfd0cf2abedc5406aaaebbbc7` |
| `WriteString` | `0x10058070` | `ev_fb4844ce44125eb837d986d9696e2e9d00381e8ddb6b762e8a58dd32b4fb47ff` |
| `LoadObject` | `0x10057e10` | `ev_c42fc0e40c615ba1ae227e42db321e2a5226271cd6359f738eec1b3f249675f0` |
| `SaveObject` | `0x10057d70` | `ev_59f4ec4550479348570688b785b74c8ba139012a98ffb89552712d1ef5e05a5f` |
| `VariantStream::Read` | `0x100584c0` | `ev_031651cf66d426af6da4a3d8bd7ff48d056c074bb954136abcf089f87d801e5e` |
| `VariantStream::Write` | `0x100584e0` | `ev_4e6a2240fa55317ca3da02ef0eac10e1fcde0a1489bb95df1ce6ae015750a4f3` |
| `VariantStream` 构造 | `0x1000f1e0` | `ev_5b27291219011577fe55bc65e902aa78d582d59359e3028a602994eb5e872d27` |

过程名搜索的 parent evidence 是
`ev_4ddfe5ea76321a0454b55000489dbdb853b6735aca894ec838e7a797e5249be0`；
搜索视图是
`ev_5e2b4d8bbfe01ced695b71e3b0fabeb3f2d9ff76e6b6cb797654161940704abf`。

## 字节观察

Ghidra 伪代码中的全局 `Read` 先读 2 字节小端 `VARTYPE`，再按类型读取：

| `VARTYPE` | 后续读取 |
| --- | --- |
| `0`、`1` | 无载荷 |
| `2`、`11` | 2 字节 |
| `3`、`4` | 4 字节 |
| `5`、`7` | 8 字节 |
| `8` | 调用 `ReadString` |
| `9`、`13` | 调用 `LoadObject` |
| `0x11` | 1 字节 |
| 其他 | 返回 `E_FAIL`（`0x80004005`） |

`ReadString` 和 `WriteString` 的伪代码互相支持同一个布局：4 字节长度 `n`；
`n != 0` 时随后读/写 `n * 2 + 2` 字节；`n == 0` 时不读/写字符串载荷。
因此空字符串总长是 6 字节：2 字节 `VARTYPE=8` 加 4 字节零长度，没有终止零。
由此推断 `n` 是 UTF-16LE 内容码元数，末尾 UTF-16LE 终止零不计入 `n`。
`n > 0` 时最后两个零字节是写入侧的终止符。

原生全局 `Read` 成功后还会把值写入输出 VARIANT。若输出已有类型，代码会尝试
类型转换。Rust 读取器不模拟这部分 COM VARIANT 语义，也不猜测字段应有类型。

## 对象分支未闭环

`9` 和 `13` 进入 `LoadObject`。伪代码显示该入口先对输入 `IStream` 做
`QueryInterface`，识别到特定上下文时走不同路径；否则读取 16 字节类 GUID。
该读取依赖 COM 上下文、对象创建和对象体持久化。这里观察到的 16 字节
**不是** `object_stream::read_header_10_8` 所表示的“16 字节 GUID + 4 字节键”
对象头，不能把对象分支接入该读取器。

对象体边界、引用登记、替换对象、失败恢复和引用表生命周期仍未知。因此 Rust
实现遇到 `9/13` 时返回显式不支持错误，并保持传入游标不变。

## Rust 严格子集

`src/variant_stream.rs` 只实现上述显式输入起点的读取：

- 固定宽度分支保留原始小端位模式；不解释有符号数、浮点数、布尔值或时间。
- 字符串用独立的 VariantStream 规则，不复用 AfCore `String::Read` 的
  “含终止零的字节长度”规则。后者见 `native-strings.md`。
- `max_string_units` 限制前缀声明的内容码元数；`0` 允许空字符串。
- 载荷长度 `n * 2 + 2` 用 checked arithmetic 计算，读取前拒绝溢出和超过限额。
- 非空字符串必须完整读取载荷，最后两个字节必须是 UTF-16LE 终止零；内容用
  Rust 严格 UTF-16 解码，非法代理序列被拒绝。
- 载荷中的嵌入 NUL 不当作终止符。静态布局支持嵌入 NUL，但尚未用同版本运行
  行为验证所有原生读取路径对它的处理。
- 所有读取错误、未知 `VARTYPE`、超限、非法字符串和对象分支都不推进游标；
  调用者可在原边界重试或向上诊断。

实现不包含写入器、隐式 VARIANT 转换、对象体、引用表、MXD 字段归属、CLI
接入或 10.1 规则。

## 未解决问题

- 尚未在 ArcMap 或 QGIS 中人工打开和检查源/转换工程；本文结论不是转换验收。
- 没有确认 `9/13` 对象体在磁盘上的完整边界，也没有确认对象引用表读写。
- 没有核对 `ReadString` 对嵌入 NUL 的同版本运行时行为。
- 没有确认原生 `IStream` 短读、COM 错误映射和所有调用点的错误恢复策略。
- 没有 10.1 或其他 Desktop 版本证据；参考样本的版本候选不能证明兼容。
