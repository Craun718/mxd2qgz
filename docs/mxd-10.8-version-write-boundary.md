# ArcMap 10.8 `/Version` 的写入边界（2026-11-18）

适用范围：内置 ArcMap 10.8.0.12790 的文档保存入口 `0x48a380` 与读取入口
`0x47a250`。使用 objdump 核对 x86 PE 指令，用 PE 版本资源、导入、导出和重定位核对身份
及调用。ArcMap.exe、AfCore.dll、SystemCore.dll 的版本及 SHA-256 与[第十轮记录](mxd-terra-round10-version.md)
一致；本轮另核对同版本 afUILib.dll，SHA-256
`4fe3cc961bd0a1f1c0d00b277649ba65b5977da78c2547722a8b910e40d12bf3`。PageLayout.dll 的身份与
反编译证据见 [DocumentInfo 调用续查](mxd-luna-followup-version-20261118.md)。

## 成功保存时的字节

| 顺序 | 字段 | 门控 |
| --- | --- | --- |
| 1 | 小端 u32，UTF-16LE 文本的字节长度，包含结尾的 U+0000 | 总是 |
| 2 | 由资源载入的文本，带 UTF-16LE 零终止符 | 总是 |
| 3 | 小端 i16，保存版本 short | 总是 |
| 4 | 小端四字节，对象写表条目数 | 保存索引 ≥ 1 |

- `0x48a47e` 设置创建模式 `0x1012`；`0x48a8c8` 以该模式调用
  `IStorage::CreateStream` 创建 `Version`。返回流由 `[ebp-0xe98]` 持有，
  `0x48a8df` 将其另存至 `[ebp-0xea4]`，`0x48a921` 还将它接入对象流。
  AfCore 的 `String::Write`（`0x10055190`，ArcMap 调用点 `0x48aaf6`）
  写长度和编码后的文本；`0x48abb0` 写 short。ArcMap `0x554ff0` 的 12 项 short
  表是 `10, 13, 14, 16, 18, 19, 20, 21, 22, 23, 24, 25`。AfCore
  `AfStreamGetSaveVersion`（`0x10007160`）对超出上限的索引用最后一项；
  文本走独立的资源分派，不用 short 推断本地化文本。
- 索引 ≥ 1 时，`0x48ac93` 调用 Seek 记录当前位置，`0x48acb3` 写四字节零占位。
  最后使用保留的 IStream 在 `0x48f3de` 定位，调用对象流 `+0x5c(0, &count)`
  取对象写表条目数（[SystemCore 字段绑定](mxd-terra-next-version.md)），并在
  `0x48f433` 覆盖四字节。索引 0 跳过占位和回填。
- 头部写完至第一次切流之间，`0x48ad54` 的 `AfDoc::SaveInfo` 在
  afUILib `0x100777ff` 创建独立的 `Document Info`。`0x48aded` 的
  `AfDoc::SaveTemplateData` 只接收模板文档和布尔值，使用模板自身的存储。
  `0x48ae31` 的虚表调用也只拿到文档存储：`CMxDoc+0x224` 的接收对象由
  `0x42a2e0` 构造，虚表 `0x54f0dc+0x2c` 经 `0x5328dc` 转发到
  `AfDoc::SaveVbaProject`（afUILib `0x10078040`），走 VBA 持久化和
  `UIControls` 命名流，不使用当前的 ObjectStream。
- 该段第一次再使用对象流是 `0x48b0b0`：已绑定的 DocumentInfo 实现先创建并
  切到 `Metadata`，然后才写对象。后续 `Maps` 等段创建各自的流，
  `0x48f43c` 后的 `DocumentDatabase` 同样独立。对整个 ArcMap 保存入口检查，
  `Version` 保留引用除此回填外只用于赋值和释放；其名称地址 `0x55e6d0`
  的两处重定位引用分别是创建和读取。对应的内置写入格式在 short 或回填计数后结束，
  没有额外的已定义尾段。此结论不为外部替换组件或外来非标准数据定性。

## 失败分支与读取边界

- CreateStream 返回负值时，`0x48a8cc` 跳到 `0x48ad44` 继续后续保存。
  保留引用未赋值；索引 ≥ 1 且中间步骤没有提前退出时，`0x48f3d9`
  将从该空引用取虚表。该分支没有显式恢复。文本、short、零占位任一写入返回负值时，
  分别走清理并返回 0 的路径。取占位位置的 Seek 没有检查返回值。
- 回填 Seek、查询写表计数和覆盖写入的返回值均不检查；紧接的 `0x48f43c`
  比较的是保存索引。后续正常路径在 `0x48fbe2` 设置成功返回值 1；
  最终结果仍可受其他保存步骤影响。以上是控制流结论，不代表观察到了短写、事务回滚
  或失败后的实际 OLE 字节。
- 读取入口拒绝有符号 short > 25；short ≥ 12 时，`0x47a9c4` 在原 IStream
  读取四字节，`0x47aa00` 在**对象流**上调用 `+0x54(-1, count)`。SystemCore
  虚表 `0x100ff724+0x54` 指向 `0x10070660`，非零选择器进入读表分支，
  用有符号计数计算桶数 `max(17, trunc(count/10))`，重建读表而不读取 IStream。
  此处此前误被 Ghidra 识别为 `IStream::CopyTo`。Rust 严格读取器仍保留边界后的
  未知字节，也不会按外部计数分配散列表。

Rust 的显式 10.8 编码器只构造成功回填后的流字节，测试覆盖版本索引、条件字段、
字节序、非法输入及读回边界。`version` 命令在 10.1 规则不匹配时尝试完整的
10.8 头部，会明确拒绝截断和超限 short，并从条件字段之后保留未知字节。没有运行 ArcMap 验证另存、故障注入或实际文件长度；
跨版本互读和 10.1 未知尾部的语义维持开放。

## 简体中文资源对照

本机简体中文安装包提取的 `resArcMap.dll` 固定版本同为 10.8.0.12790，SHA-256
`439fecbb454f0df1e9f00208ba6225590788649a8d3b6ba9d32a17e4fb4ed47f`。
逐个读取 PE 的 RT_STRING（语言 `0x0804`）：12 个写侧资源 ID 中，11 个数字文本和
[同版本英文资源](mxd-round3-version-object-20261117.md)逐字相同；索引 3 的
`0x11a` 在中文资源中为“打印文档时发生异常错误。”，也是打印错误模板。
因此不能把该资源 ID 直接写成 `9.3`，也不能由静态资源异常推定实际另存结果。
这份语言包属于 10.8，不能作为 10.1 的本地化取证。
