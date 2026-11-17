# Version 续研：10.8 保存切流与 10.1 介质兼容头（2026-11-18）

## 10.8 切向 Maps 之前的间接保存调用

接续[第十轮未定调用](mxd-terra-round10-version.md)。PE x86，固定版本均为 10.8.0.12790。ArcMap.exe SHA-256 `a9a963b9dffbf0cb346c67c98bce0880b7e64363410a788dccb9597cff3c75fe`；PageLayout.dll SHA-256 `6d432cb4ee65fc3f73f20c2270db1f10e0292935dfec466fe74e8922ff2413ee`。地址为静态映像 VA，以下结论限定正常构造和成功查询路径。

- ArcMap `0x46f060..0x46f081` 初始化文档 `+0x210`，以 CLSID `{5012EFC9-6D2C-4F0E-8513-5349010CBCE3}` 创建 DocumentInfo；助手 `0x40b920` 请求 IDocumentInfo2（`0x55ec88`）。写入口 `0x48ae9b..0x48aead` 从该成员经 `0x40c7f0` 查询 `0x55eb38` 的 IID `{E8E5589D-215A-4322-BA68-E52F187363EE}`。此前将 `0x48b0b0` 记为未绑定的写调用，本轮确认其对该接口的 `+0x10` 传入文档存储、ObjectStream、保存索引；返回负值在 `0x48b0b4` 分流。
- PageLayout 的 `0x10002400` 输出同一 DocumentInfo CLSID；该类构造 `0x100024e0` 将 `+0x10` 子对象的最终虚表写成 `0x10088ca4`。`0x100027f0` 对上述 IID 返回对象 `+0x10`；实读虚表 `0x10088cb4` 处指针指向 `0x10002f10`，绑定到 DocumentInfo 的保存实现。
- REA 6.9.0 / Ghidra 12.2 补导 `0x10002f10`，证据 ID `ev_81c2d31a1a9b73cef5dea7b2723ab76e2a28bebead70931884137c3cbfbd178d`。原始指令 `0x10002f53..0x10002f8f` 以模式 `0x1012` 调用 IStorage 的 CreateStream 槽，流名的 UTF-16 原字节在 `0x100889a0`，为 `Metadata`。创建失败直接在本函数返回失败；成功时于 `0x10002fb4..0x10002fca` 通过 ObjectStream `+0x38` 接入新流。后续才按保存索引小于 4 分派旧体或写当前元数据。故该具体调用点在流连接成功后的对象流写入属于 `Metadata`，不能算作 `/Version` 尾部。REA 把 `+0x38` 错标为别的方法，实际槽位以指令和调用参数为准。

仍没有证明整个 10.8 保存链上其它间接分支、被替换的 COM 实现、保存失败后的流长度，或 10.1 与 10.8 的实机互读。TODO 原 114/115 整项保持开放。

## 10.1 Desktop 与 SDK 补充介质的 /Version

在[既有 134 个介质模板条目](mxd-install-media-samples-20261117.md)之外，对已提取的 10.1 Desktop 补充、SDK Java、SDK .NET 文件重新用本机 `version` 命令读取 OLE 流。10、17、20 份有效工程均读完已知头部，尾部长度为 0。另三个 `.mxd.xml` 是 XML 并被正确拒绝。有效工程内部各组以 SHA-256 去重。以下是额外出现且不属于 10.1 静态保存候选表的实际字节组合：

| 介质工程 | 文本 | short | 对象计数 | SHA-256 |
| --- | --- | ---: | ---: | --- |
| Java World | 9.3 | 15 | 351 | `04b6086b15e694bea1e33a978b57265313cc4c44d3603488491b2062a7e90109` |
| Java Yellowstone | 9.0 | 10 | 无 | `05ff0f36d97048eef5f241659664a57e24b3ce822212205ad0bbdbf9a6307e73` |
| .NET GISForXMLSample | 9.4 | 18 | 250 | `63afbe61413b80a26944b7b063c151d3b49399867c0741f0bd31c0f57611d4f5` |
| .NET PrintMultiPage | 9.4 | 18 | 98 | `931634d87ef82835fb7360cb2dfd20368947f8cbe196d604fc139fd82cfd022d` |

这四个工程均未命中保存索引；短值相同不能证明磁盘文本或实际保存版本相同。没有把它们加入静态索引表。新增样本亦未提供 10.1 头部未知尾字节，不关闭其语义条目。本轮未运行 ArcMap/QGIS；REA dossier 在忽略目录 `temp/luna-followup-20261118/pl108/`。
