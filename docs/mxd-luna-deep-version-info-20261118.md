# 10.1 AfDoc 文档信息与 /Version 的边界（2026-11-18）

本轮只核对 ArcMap 10.1.0.3035。本机 ArcMap.exe SHA-256
`1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97`；
AfUILib.dll SHA-256 `35d41cf850cbe13c202cca84441cee05db487bc54421ab9d5012dc0b1cad7da4`。
版本资源核对为 10.1.0.3035，地址为 x86 PE 静态映像地址。

- ArcMap 保存入口 `0x466fb0` 在写 Version 前缀之后，于
  `0x467813..0x467823` 将 IStorage 传入 `AfDoc::SaveInfo`（IAT
  `0x54282c`，直接调用时 AfDoc 来自 CMxDoc `+0x1a4`）。AfUILib 按名
  导出的真实入口 `0x1006cca0` 创建的是 **Document Info**，字符串
  在 `0x10129a5c`；接着向返回的独立 IStream 写自己的 16 位版本和成员
  AfDoc `+0xcc`，不通过 ArcMap 保存者的 Version 对象流。REA 6.9.0/
  Ghidra 12.2 证据 `ev_bec2778df39e8e3ff1fd1a05ad0e81aeda51797d7c29075b4cb50f393f8a7517`；
  原始指令复核 `0x1006cd01` 流名和 `0x1006cd4f` 对该流的 short 写入。
- 对称的 `AfDoc::LoadInfo` `0x1006d310` 调 IStorage `+0x10` 打开同一
  **Document Info**，读该流的 16 位版本以及 AfDoc `+0xcc` 和版本门控字段。
  REA 证据 `ev_ff48fda8c06078de6a9eb378cf624a2d9950c7fe48a3fbeb5555f3e7628a3297`；
  核对函数 `0x1006d37d` 的流名和 `0x1006d3d5` 的读取 IAT。某条件路径将值
  写至 `*(AfDoc+0x34)+0x128`，接收者是 AfApp，不能混同 AfDoc 保存索引
  `+0x14c`。该函数直接体未见写入保存索引；被调用者和其它打开路径仍待审计。
- 另一路是 CMxDoc `+0x190` 的 DocumentInfo：保存者在 `0x46795e..0x467971`
  查询既有记录的 IID，再于 `0x467b3a` 取其 `+0x10` 保存槽；PageLayout
  虚表 `0x7b71ca9c` 上该指针为 `0x7b691b50`，实现创建 **Metadata**，
  与 Document Info 不是同一流。另复核 ArcMap `0x472e30`：成功创建
  **Templates**（`0x54c754`）后于 `0x472eef..0x472ef3` 将返回流交给
  ObjectStream 的 `+0x38`，不能将这里的写入归给 /Version。

因此三处写入各自归属于已确认的独立命名流。本轮并未穷尽 Version 的全部
间接写入、错误路径或实际旧档保存索引。未新增推测字段，不勾选完整尾部、
旧档索引或运行验证项。REA 证据位于忽略目录 `temp/luna-deep-20261118/afui/`；
没有运行 ArcMap 或做 QGIS 人工验收。
