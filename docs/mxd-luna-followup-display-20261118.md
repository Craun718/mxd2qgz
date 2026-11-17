# display 组续研：RasterLayer 第二串至首个内嵌对象头

## 范围与文件身份

本记录只追踪 ArcMap Desktop 10.1 `RasterLayer.dll` 的 `IPersistStream::Load`
`0x7c73bd40` 中第二条 String、其后的定长短值、版本相关数量和首个嵌套对象头。
样本只作辅助交叉验证。未连接图层模型或共享 dispatcher；未编辑 `TODO.md`、
`README.md` 或公共模块声明。

目标映像为 10.1.0.3035 安装介质 `RasterLayer.dll`，SHA-256
`9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6`，PE32/i386，
ImageBase `0x7c710000`。身份与哈希同 `binaries/manifest.json`、本地二进制及
既有 10.1 基线核对；`logs/RasterLayer.log` 有 `EXPORT_DONE`（3258 成功、1 失败）。
使用既有导出 `RasterLayer/7c73bd40.c`、`RasterLayer/7c73b380.c`，并对原始映像
以 `objdump -d -Mintel` 复核下述关键指令。

## 原生顺序与证据

- `0x7c73bd40` 读版本，`cmp [esp+0x3c],0x11; jg 0x7c73c211`（指令
  `0x7c73bdf0..0x7c73bdfa`）按有符号比较拒绝大于 17 的值。
- 随后调用 AfCore `String::Read` 到对象 `this+0x88`。成功后在栈上构造临时
  String，并再次调用 `String::Read`（导出伪代码紧接第一条 String；汇编位于
  `0x7c73be12..0x7c73be28`）。这确认第二串存在且是临时接收对象，不确认其业务
  含义。
- 接着顺序读取三个 2 字节值，目的位置分别为 `this+0x90`、`this+0x92`、
  `this+0x96`（指令区间 `0x7c73be2d..0x7c73be90`）；成功后把版本传入
  `0x7c73b380`（调用点 `0x7c73be91..0x7c73bea2`）。本实现保留三个原始 `u16`，
  不命名字段。
- `0x7c73b380` 在版本大于 8 时从流读取一个有符号 4 字节数量（入口
  `0x7c73b3b5..0x7c73b3d8`）；其他版本使用局部初值 1，不消费数量字段。数量
  大于零时进入循环，首个迭代调用 `LoadObject`，请求 IID 地址
  `0x7c76c8a4`（反编译 `0x7c73b380.c` 的循环体，原始指令约
  `0x7c73b3ef..0x7c73b410`）。本轮边界定在这个辅助循环首个 `LoadObject` 的
  20 字节流内对象头之后，不跨入其对象体，也不消费循环余项。主 Load 在
  `0x7c73b380` 返回后还读取 `this+0xbc` 起的两组 8 字节值、后续标量，并在
  `0x7c73beda` 处另调一次 `LoadObject`；该主 Load 的后续固定段和对象不在本次
  读取器覆盖范围。

## 两个有界样本体交叉

从仓库参考工程 `samples/测试底图.mxd` 的 `/Maps` 流直接提取字节；下列体范围由
图层列表 ExtraSafe `A` 长度所限定，并与条目下一起点吻合。每个体内完整按原生
顺序读到首个嵌套对象头：

| `/Maps` 体范围 | 版本与两串 | 三个短值 | 数量及首对象头 | 续段终点 |
| --- | --- | --- | --- | --- |
| `0x4fda..0x145a3` | 版本 17；首串 22 字节（含终止零）、第二串 `.` | `0000, 0000, 0000` | 数量 1；类字节 `6f 7c 13 f2 75 90 af 4b 9b 38 e2 f3 60 83 fb f4`，键 260 | `0x501e`，对象体仍有界未读 |
| `0x145c1..0x16fb6` | 版本 17；首串 50 字节（含终止零）、第二串 `.` | `ffff, 0000, 0000` | 数量 1；类字节 `e2 e6 bc 75 f5 8a 8e 47 88 92 fa 45 ca 50 af 4d`，键 1545 | `0x14621`，对象体仍有界未读 |

区间末端均落在各自 ExtraSafe 体界内。首对象头先记录流内类字节与键；对象体仍未
解析。第二串在两个体中恰为 `.`，
仅是样本值，不据此赋予字段语义。第二条目边界按 `/Maps` 原始字节和
`前缀后对象头起点 + A` 复算为 `0x16fb6`（该处是 `ff ff ff ff` 活动索引）；
`0x16fb4..0x16fb6` 两个零字节仍在前一体的 A 范围内。仓库既有地图对象表将
该索引位置记为 `0x16fb4`，与这里的字节和长度复算相差 2 字节；本记录不改写
地图模块或该既有文档。

**主审补充核对**：RasterLayer.dll `0x7c76c8a4` 的 IID 为 IName
`{4BA33331-D55F-11D1-8882-0000F877762D}`。按安装介质 `ecfg` 的类名核对，
样本一首对象头（键 260）为 `FgdbRasterDatasetName`，样本二首对象头（键 1545）
为 `RasterDatasetName`。这补充了对象头的接口和类身份；对象体仍未解析，本轮未据此
推断或添加 RasterLayer 成员字段。

## 实现与限制

`src/layers/raster_layer.rs` 新增 `RasterLayerContinuation10_1` 和
`read_raster_layer_continuation_10_1`。调用方先用现有前缀函数消费版本及第一串，
再从第一串之后调用新读取器。读取器保留第二串、三个 `u16` 原值；版本大于 8
时读取数量；只有原生 `0x7c73b380` 循环会进入（正数量，或版本不大于 8 的隐含
初值 1）时，才读取该循环首个对象头。零或负数量不读循环对象头。其体、后续循环、
主 Load 在辅助函数返回后的固定段与 `0x7c73beda` 对象均未读；错误事务性回滚。

没有获得 RasterLayer 的 `ILayer::Name` 或 `ILayer::Visible` getter 对应体偏移的
证据。导出中 `get_Name 0x7c760cc4` 是 `BasicRasterDataset` 跳板，不是本类属性
归属证据。首串和第二串均未重命名为 Name；三个短值仍不推断字段名。

独立单元测试位于栅格模块内，覆盖版本 17 与 ≤8 的首对象头边界、零数量、截断
回滚。本文结论来自 10.1 静态映像及一个仓库样本的离线字节交叉，没有 ArcMap
实机运行，也没有 ArcMap/QGIS 人工目视验证；不能据此宣称 RasterLayer 体已解析
完整。验证：`cargo fmt --check` 通过；`cargo test --offline --lib
raster_layer::tests` 的 10 项通过。全量 `cargo test --offline` 有一个工作区既有
失败：`tests/map_surround.rs::legend_reads_full_version_five_body` 的 GUID 期望值与
实际值不符；本轮没有修改该测试或 MapSurround 模块。

## 主审：ILayer 成员与中间模型

10.1 esriCarto 的 MSFT TLB（本机提取的 `carto.tlb`，SHA-256
`8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`）：
TypeInfo 5 的 NameOffset `0x36c` 解为 ILayer，其 `cElement=18`。
从 TypeInfo 记录 `0xe30dc` 的成员数组及名称表 `0x4d3c4` 解出
`Name` getter/setter 槽为 `+0xc/+0x10`，`Visible` 为 `+0x2c/+0x30`。
RasterLayer.dll 主 QI `0x7c73aab9..0x7c73aad4` 比较 ILayer IID
`{34C20002-4D3C-11D0-92D8-00805F7C28B0}`，命中返回完整对象 `+0x90`，
构造器最终在该处安装 `0x7c76c17c`。

- Name getter `0x7c74dda0` 将子接口 this 减 4，转 `0x7c7374c0` 从
  相对 `+0x98` 取 BSTR（完整对象 `+0x124`）；setter `0x7c7374e0` 写
  `ILayer this+0x94`，落到同一位置，与 Load 首串的 `+0x9c+0x88` 一致。
- Visible getter `0x7c737640` 取 `ILayer this+0x9c` 的 16 位原值，
  setter `0x7c737670` 比较相同位置（完整对象 `+0x12c`），与 Load
  第二串后第一个 short 的 `+0x9c+0x90` 一致。

因此有界续段成功时，`parse` 以这两个字段构造 RasterLayer 节点；仍保留所有
未解析字节，未以剩余属性猜测数据源或栅格渲染。仓库参考工程的两个栅格条目
分别在 `0x501e`、`0x14621` 到达各自首个 IName 对象体；保留 Map 尾部停点
`0x253be`。栅格短读不会借用下一条目的字节。未运行 ArcMap 或进行 QGIS 目视核对。
