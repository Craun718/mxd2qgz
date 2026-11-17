# RasterDatasetName 单类体续研

## 原生映像与入口

目标文件为主审从 ArcMap 10.1 CAB 提取的 RasterDB.dll，SHA-256
`97207d5cc5307bfe126e030792b5e92c085668953a736a6875d35e72d7e86400`，PE32/i386，
ImageBase `0x7bfb0000`。REA/Ghidra 12.2 成功反编译 Load，Evidence ID
`ev_6ec5e42e619cf0d91e0b60a0d561c694d5add3381684d1af2013c0e0e5d03a31`，本地 JSON
位于 `temp/luna-followup-20261118/raster-native/rasternameread.json`。关键入口：
`GetClassID 0x7c0ba570`、`Load 0x7c0ba5b0`、`Save 0x7c0ba790`。GetClassID
与 RTTI `RasterDatasetName@ESRI` → TD → COL `0x7c148bc0` → 虚表 `0x7c133608`
相连，完整 CLSID 为 `{75BCE6E2-8AF5-478E-8892-FA45CA50AF4D}`。

## IAT 与控制字

原始导入表把 Load 指令的间接调用解析为：

| IAT 地址 | 导入 | Load/Save 用法 |
| --- | --- | --- |
| `0x7c1211bc` | `AfStreamRead(IStream*, long*, ...)` | Load 起始读 4 字节到有符号 `local_18` |
| `0x7c12114c` | `String::Read(IStream*)` | Load 读取四条 String |
| `0x7c1211c4` | `LoadObject(..., const GUID&, IStream*, ...)` | Load 读取嵌套对象 |
| `0x7c121228` | `AfStreamGetDocumentVersion` | Save 获取文档版本输入 |
| `0x7c12122c` | `AfStreamGetSaveVersion` | Save 将所选保存版本写入局部值 |
| `0x7c1211b0` | `AfStreamWrite(IStream*, const long*, ...)` | Save 写 4 字节值 |

因此首字段确为 SaveVersion 导出的 4 字节门值；读取器仍原样保留为 `i32`，并
只按 Load 的有符号比较 `>=3`、`>=4` 选择后续两个 int32。门值为零时原生
`Seek(-4, CUR)`，再从原位置进入第一条 `String::Read`；这是旧零门分支，不应把
这个回退误读为一般字段顺序。当前严格 String 子集拒绝零长度编码，读取器在该
分支停在体起点，保留原值和停止原因。

## Load 字段与嵌套对象

REA 伪代码与原始指令相符：非零门值后顺序读取四条 String 到对象 `+0x10`、
`+0x14`、`+0x18`、`+0x4c`，随后 `LoadObject`，门值 `>=3` 读 `+0x6c` 的 4 字节，
门值 `>=4` 再读 `+0x70` 的 4 字节。字段保留顺序名，不从字符串内容猜属性名。

`p_Var7 = &DAT_7c131138` 的 16 字节为
`5c 97 dd fa 6f e3 d1 11 aa 81 00 c0 4f a3 3a 15`，GUID
`{FADD975C-E36F-11D1-AA81-00C04FA33A15}`，即 IWorkspaceName 请求 IID。参考样本
RasterDatasetName 键 1545 的体起点 `/Maps:0x14621`：

- 门值 4；四条 String 的字节长度依次为 50、168、30、2，第四条为空串。
- IWorkspaceName 对象头从 `0x1472f` 开始，类标识是现有
  `WORKSPACE_NAME_CLASS_ID` `{5A350011-E371-11D1-AA82-00C04FA33A15}`，键 1546；
  对象体起点为 `0x14743`。
- 复用 `read_workspace_name_body_10_1` 在父栅格对象 ExtraSafe 界内解析该体，
  WorkspaceName 体终点为 `0x1485a`。之后 RasterDatasetName 的两个门控 int32
  位于 `0x1485a`、`0x1485e`，本类 Load 读到 `0x14862` 停止。外层有界体终点为
  `0x16fb6`，本类停止后仍有 10,068 字节属于父 RasterLayer 后续内容，故不能
  把父体边界当作 RasterDatasetName 体终点。

## 独立实现范围

新增 `src/layers/raster_dataset_name.rs`，暂未在 `mod.rs` 声明或接入分派。该模块：

- 提供由 GetClassID 复核的类标识常量；四条 String 作为无语义名称的有序数组保留。
- 在 IWorkspaceName 槽中消费空标记/回引，或仅对新 WorkspaceName 类调用现有体读取器；
  其他新类在对象头前成功停止。
- 按有符号门值读取 `+0x6c`、`+0x70` 原值，不猜字段名。嵌套体停止、未知类及零门
  分支均保留停止状态；错误不推进 Reader 或引用表。
- 调用方必须传入已由父条目长度切出的有界 Reader；不读取 FgdbRasterDatasetName
  wrapper，不消费 RasterLayer 的后续字段。

独立探针临时引入该源文件后运行模块 4 项合成测试，并读取键 1545 的真实样本体；
5 项均通过。样本解析确认完整消费至 `0x14862`，嵌套 WorkspaceName 登记键 1546，
且未越过父条目界。格式检查使用 Rust 2024 rustfmt。未在 ArcMap 10.1 或 QGIS
实机验证；REA/静态结果不是原生运行或转换验收。
