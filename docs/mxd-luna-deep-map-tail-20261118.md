# Map 遮罩后属性与 IMap TLB 槽位复核（2026-11-18）

后续已用具体虚表及 esriDisplay 类型库闭合首个 short 的语义为 [IScreenDisplay.IsFramed](mxd-luna-next-screen-display-20261118.md)。本文其余内容保留初次排查过程。

## 目标与边界

目标是 ArcMap 10.1 Map.dll 10.1.0.3035，静态映像地址。Map.dll 提取件 SHA-256 为
`1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`，对应
`/home/gxrs/文档/source/arcmap10.1/binaries/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`。
证据是本机反编译、原始 DLL `objdump` 和 10.1 `esriCarto` MSFT 类型库；不以仓库样本
`0x253bc/0x253be` 的内容推字段含义。

## Load/Save 与汇编对位

- `Map::Load 0x793ff4d0`：在架构版本 `>0x1c` 分支读取 2 字节到局部变量，随后调用
  子接口虚表 `+0x60`；版本 `>0x1d` 再 `String::Read` 到 Map 成员 `+0x398`。版本 `>0x1e`
  继续读取成员 `+0x3a0` 的 short，版本 `>0x1f` 读取成员 `+0x15d` 的 1 字节。
- Map 构造阶段 `0x793e285a` 在对象 `+0x28` 安装初始虚表 `0x7948662c`；该表
  首槽至 `+0x64` 均指向 `0x7946f166`，不能用来断言最终接口或实现。Load 中调用
  的子接口地址是 `param_1-0x40`。
- 构造函数反编译 `Map/793e2820.c` 显示：先把多个接口 vtable 写入对象子区域（包括
  `param_1[10] = IMap::vftable`），随后又把这些位置改写为 `Map::vftable`。因此初始表中
  的纯虚槽既不能用来给 Load 的 `+0x60` 命名，也不能证明 Load 时实际使用哪一张表；本轮
  未追到具体 Map 实例完成构造后的最终 vptr 和 `+0x60` 目标。
- `Map/7946f166.c` 与 Map.dll 函数表把 `0x7946f166` 标为 `purecall`。该目标身份支持
  “初始表是构造期/抽象接口状态”的解释，但不替代对最终虚表的追踪。
- `objdump -d -Mintel` 复核 Load `0x794008b0..0x794008dd`：`AfStreamRead(...,2, &local)` 后，
  从子接口虚表 `+0x60` 取函数，调用时传入 `this` 和 short 值。紧接的 String 读取位于
  `0x794008ea..0x794008f7`，目标地址为 `Map+0x398`。
- `Map::Save 0x79401850`：在对应书写链上调用子接口 vtable `+0x5c`，把输出 short 写入流；
  文档版本条件满足时，随后 `String::Write(Map+0x398)`。原始 DLL 汇编地址分别为
  getter 调用 `0x79402493..0x79402496`、随后 short 写操作从 `0x7940249e` 开始，以及
  `String::Write` 调用 `0x794024f1..0x794024f7`；反编译对应代码见
  `Map/79401850.c` 中 `+0x5c` 与 `String::Write(+0x398)` 的片段。

这些证据把 short 的读写关联到子接口调用，并把后续 String 关联到 `+0x398`；
它们本身不证明该 short 的业务字段名称。`+0x398` 可以称为持久化字符串成员，尚无依据
给它赋业务名称。

## esriCarto TLB 槽位结果

使用的 `carto.tlb` 为 MSFT 格式，文件头声明版本 10.1、`SYS_WIN32`、1749 个 typeinfo；
SHA-256 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`，与本地
esriCarto 类型库研究材料一致。原始成员表见
`temp/luna-followup-20261118/ilayer-msft.txt`；类型库来自
`/tmp/mxd-arcgis101-guiddocs/wine/carto.tlb`。使用 `ITypeLib::GetTypeInfoOfGuid` 读取
IMap 的 `FUNCDESC.oVft`、`invkind` 和参数数。MSFT typeinfo 清单中 IMap 为 typeinfo 179、IID
`E6BDAA75-4D35-11D0-98BE-00805F7CED21`。按该 TLB 得到：

| 槽偏移 | 名称 | 调用类型 | 参数数 |
| --- | --- | --- | ---: |
| `+0x58` | `MapSurroundCount` | property get | 1 |
| `+0x60` | `MapSurround` | property get | 2 |

因此 **TLB 将 IMap `+0x60` 标为 `MapSurround` 的 getter**，没有将其标为 setter。
这只给出 IMap 类型库的槽位名，与 Load 反编译的调用形状并不吻合：Load 只显式传入一个
short；而 TLB 的 `MapSurround` getter 有两个参数。Save 读取 `+0x5c`，该偏移也没有
对应的 IMap `FUNCDESC.oVft` 项。再结合构造阶段初始表为纯虚目标、构造中途 vptr 会被改写，
当前不能把 Load 的实际目标直接绑定到 IMap `MapSurround`，也没有 setter 实名证据。结论
保持 raw short；后续若要实名，需先取得具体对象最终 vtable，并核对最终 `+0x60` 实现及
调用约定。

## 本轮代码与测试状态

独立的 `read_post_masking_map_fields_10_1` 与专属合成用例覆盖版本门、short、字符串、
后续定长成员及截断回滚。工作树现有 Map 接线由主代理更新为 stage 38→39：保留
`masking_empty_short`，只在逐层遮罩范围已判定可越过后从 stage 38 读取后续字段。
该条件与引用解析的覆盖边界应由主代理按已知类和真实回引复核；本记录不将样本偏移推进
解释为字段名称或逐层遮罩行为的独立证明。

本轮未运行 ArcMap 10.1，也未做 QGIS 目视验收。构造测试只验证 Rust 安全读取子集，不能
代替 TLB/原生实现的成员绑定或目标软件运行验证。
