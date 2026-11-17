# FgdbRasterDatasetName 与 RasterIO（ArcMap 10.1）

## 目标与证据范围

目标版本为 ArcMap Desktop 10.1.0.3035。`FgdbRasterDB.dll` 为 PE32/i386，ImageBase
`0x6f920000`，SHA-256
`571be820c79b217571ea8f902c091e44f6268566bd35880488e8c4ef74008fef`；`RasterCoreLib.dll`
为 PE32/i386，ImageBase `0x7beb0000`，SHA-256
`6d89601832b788177aac492e7fdfb0f1b012db86e2a75e6e480e6e647508eb1f`。反编译由
REA/Ghidra 12.2 完成，关键调用、槽地址、导出映射和条件分支已对原始 PE 指令
复核。RasterIO 的 REA 记录位于 `temp/luna-deep-20261118/rasterio/`；Fgdb Load
伪代码位于 `temp/luna-followup-20261118/fgdb-raster/load.json`。REA 对
`read_tag` 的证据 ID 为 `ev_3e5d7eabed1c429ad62f46b3b95af4b068443ca4d130051509fabe595e98f4be`，
对 `seek_tag` 的证据 ID 为 `ev_ff2c9e880b76e971aa3621141e8ced6dad4bf83801dd9f748a2b397446b5b5e6`。

## Fgdb 对象与内部委派

`FgdbRasterDB.dll` 的构造函数 `0x6f9339c0` 在 `0x6f933ade` 将普通
`RasterDatasetName` CLSID（数据 `0x6f947648`）传给 `AfCreateInstance`（IAT
`0x6f9450ac`）。对象的输出槽为完整对象 `+0x34`，由构造路径在 `0x6f933a57`
形成并传入工厂。`GetClassID 0x6f933960` 返回 FgdbRasterDatasetName CLSID
（数据 `0x6f947668`）。

内部接口视图偏移已闭合：构造函数 `0x6f933a6e` 将 vtable `0x6f947518` 安装到
`[esi+0x14]`；其 PE 原始表 `+0x14` 的 DWORD 指向 `0x6f934910`。该函数是
`IPersistStream::Load` 实现。Load 以接口视图 `+0x20` 检查委派对象，故该地址
等于完整对象 `+0x34`，与 `AfCreateInstance` 输出槽相同。Load 通过 QI 请求
`IPersistStream`（IID 数据 `0x6f947038`），调用其 `Load`，释放接口后才开始
Fgdb 自身的 RasterIO 读取。

因此对象体开头由内联的普通 `RasterDatasetName` Load 读取；它没有独立对象头、
不产生第二次键登记。代码直接复用
[`RasterDatasetNameBody10_1`](../src/layers/raster_dataset_name.rs)。传入的 `key`
表示外层已经登记的 FgdbRasterDatasetName 键；委派读取器仅将该键保留在结果中，
嵌套 WorkspaceName 新对象仍按流内键正常登记。

## RasterCoreLib 导出与 RasterIO 记录

PE 导入表确认 RasterIO 实现在 `RasterCoreLib.dll`，不是 GdbCoreLib。导入按名字
对应以下导出 RVA/VA；导入表中的数值是 Hint，不能当作 ordinal：

| 方法 | 导出 RVA / VA | 证据及读取行为 |
| --- | --- | --- |
| 构造 | `0x63ac0` / `0x7bf13ac0` | 接收 `IStream*` |
| `InitCheck` | `0x63b30` / `0x7bf13b30` | 检查 magic `0x00ff00fa` |
| `read_tag` | `0x63c80` / `0x7bf13c80` | 连续读取两个 4 字节整数：tag、length |
| `seek_tag` | `0x63ce0` / `0x7bf13ce0` | 逐对读取 tag/length，按 length 跳过；识别 `(-1,-1)` |
| `End` | `0x63bd0` / `0x7bf13bd0` | 读取模式定位结束 tag、清理状态并返回 |
| `skip` | `0x63eb0` / `0x7bf13eb0` | 以当前记录 length 作为流偏移 |
| `ReadBase` | `0x64360` / `0x7bf14360` | 按记录 length 读取负载 |
| `Read(long/wchar_t)` | `0x64420` / `0x7bf14420` | 导出 thunk 跳到 `ReadBase` |
| `ReadObj` | `0x643c0` / `0x7bf143c0` | 转入对象读取逻辑 |
| 对象读取 helper | `0x63d90` / `0x7bf13d90` | 读取 16 字节 CLSID；全零 CLSID 返回空对象 |

Fgdb Load `0x6f934910` 的调用顺序是 tag 1 `Read(long*)`、tag 2/3 `Read(wchar_t*)`、
tag 4 `Read(long*)`、tag 5 `Read(wchar_t*)`、tag 6 `ReadObj`、tag 7 `Read(long*)`，
最后调用 End。tag 1–6 失败会退出；tag 7 失败仍调用 End；End 失败则退出。这里
保留原始 tag 编号，不据接收成员偏移推测业务字段名称。

RasterIO 记录头明确是两个 little-endian i32。字符串记录的 length 是字节数；安全
子集要求正偶数字节数、唯一 UTF-16LE 尾零并受调用者限额约束。对象记录使用
length `-1`，随后为 16 字节 CLSID。空 CLSID 分支已由 `get_obj` 反编译和指令确认；
非空 CLSID 需要进一步恢复对象工厂及其对象体，不在本解析器中猜测。End 通过
`seek_tag(-1)` 扫描记录并识别 `(-1,-1)`；当前实现只消费已验证的立即结束对，遇到
额外记录保留其头部为停止点。

## 样本交叉验证

仓库样本 `/Maps:0x501e` 从 gate 4 开始，普通名称体四条字符串长度为 22、22、40、
2 字节（第四条为空）。随后 WorkspaceName 对象头含 GUID 和键 261。复用现有
RasterDatasetName 读取器后，游标恰停在相对偏移 `0x11d`，即绝对位置 `0x513b`；
此处 RasterIO magic 记录为 tag `0x00ff00fa`、length `0`。这与 Fgdb Load 的
`InitCheck(0x00ff00fa)` 调用对齐。

样本 RasterIO 记录边界（仅列结构，不列样本属性值）：

| 绝对偏移 | tag | length | 解析结论 |
| --- | ---: | ---: | --- |
| `0x513b` | `0x00ff00fa` | 0 | InitCheck magic |
| `0x5143` | 1 | 4 | 定长整数 |
| `0x514f` | 2 | 22 | UTF-16LE 字符串 |
| `0x516d` | 3 | 14 | UTF-16LE 字符串 |
| `0x5183` | 4 | 4 | 定长整数 |
| `0x518f` | 5 | 22 | UTF-16LE 字符串 |
| `0x51ad` | 6 | -1 | 对象槽；后续 16 字节 CLSID 全零，故为空对象 |
| `0x51c5` | 7 | 4 | 定长整数 |
| `0x51d1` | -1 | -1 | End 结束对 |

样本的解析边界与原生调用顺序及 RasterCoreLib 记录读法一致。只据此确认上述
前缀；不外推其他非空对象、尾随未知记录或整个父对象长度。

## 实现范围与验证

新增 `src/layers/fgdb_raster_dataset_name.rs` 提供两个可分开调用的读取器：
`read_fgdb_raster_dataset_name_prefix_10_1` 复用普通名称体并返回 RasterIO 起点；
主调用者可在委派结果 `stop == None` 时调用 `read_fgdb_raster_io_prefix_10_1`，
并分别保留两个结果。RasterIO 读取器覆盖 magic、tag 1–7、tag 6 空对象及结束对；
遇到 tag 6 非空 CLSID 时停在 CLSID 起点，tag 7 后其他记录停在记录头。格式错误
不会推进 Reader。主审将两个读取器接入 `layer_list.rs` 的有界 RasterLayer
IName 分派；只有委派体读完才试读 RasterIO。

单元测试覆盖委派结果携带外层键、委派截断事务性、RasterIO 正常安全子集、非空
对象停止位置及错误事务性。完整 `cargo test` 通过；
`cargo test --test luna_fgdb_raster_20261118` 通过，从 Map 根顺序解析并核对
WorkspaceName 键 261 与 RasterIO 结束位置 `0x51d9`。一次性 probe 曾独立验证
`/Maps:0x501e` 至 magic 的偏移和键登记，已删除。`cargo fmt --check` 未通过：差异
位于共享的 `layer_list.rs`、`raster_layer.rs`、`map_object.rs`、`mxd_maps.rs` 和
主审测试文件；本轮只格式化本模块，没有改动这些共享文件。尚未在 ArcMap 10.1
或 QGIS 中人工打开验证。
