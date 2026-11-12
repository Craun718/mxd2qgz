# MapDocument 原生入口（2026-11-12）

本轮为 REA 静态分析，未执行 ArcMap。除明确标为流内位置者外，以下地址均为
所属 DLL 在 Ghidra 中的虚拟地址，不是 MXD 偏移。REA 6.8.0、Ghidra 12.1.4，
分析配置摘要 `e763201e522a4b123fb330eede09834c44ddf2bb90702e50eea219801be95539`。

## 身份和注册

同一 Desktop 10.8 CAB 的 `esri.clsid.ecfg`（ZIP）SHA-256 为
`a7a53b6a248f481e68746126e2906aa21869d5c7ad52e77eded2c38c53dce897`；
其 `esri.clsid.xml` 摘要为
`71179ed661d9baa9366d603eb561141711ab0520b10c3c3a506a3f9003803e01`。
本地静态 ZIP/XML 检查观察到：Module `PageLayout.dll`、progIDPrefix `esriCarto`
下注册 Class `MapDocument`、CLSID `2FE5E68E-3DA5-4BDA-BAF1-103C55115A1C`。
这是一项安装配置观察，实际实现再由下述 REA 证据核对。

提取 `PageLayout.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`，SHA-256
`6d432cb4ee65fc3f73f20c2270db1f10e0292935dfec466fe74e8922ff2413ee`，
x86 PE，VS_FIXEDFILEINFO 文件版本 10.8.0.12790。
同 CAB 托管 `esri.arcgis.carto.dll` SHA-256 为
`12bf0b0de0b4d5ec1c44b371dbae02e881c4ed8ccffbfdf1dcef8d8e5e182bd4`。
用 .NET PEReader/MetadataReader 静态读取 GuidAttribute 和方法顺序（不加载目标）：
IMapDocument token `0x02000424`、IID `5366451C-A001-4179-B8CB-E8F014628A44`；
MapDocumentClass token `0x02000429`、CLSID 同上。IMapDocument 为 IUnknown 接口，
Open 是第 9 槽（字节 `+0x24`），GetVersionInfo 第 22 槽（`+0x58`）。
10.1 托管元数据同 GUID 和方法顺序，但这不证明两版文件格式相同。
REA 托管方法证据 `ev_c9446a9bbeae375b4db02863ca1a0ca04b4fe244df0d799f1f77f41332977fce`
只给方法和 token，GuidAttribute 的解码来自上述独立本地静态检查。

## 类工厂、QI 与最终虚表

| 观察 | REA 证据 |
| --- | --- |
| CLSID 在 `0x10089a84`，引用含全局初始化 `0x100015db` | `ev_24e3e24be98316f7159759c16fde3cd0163a196cf12d7406ab39fad665588c1c`、`ev_c402d076c62c675945c6de4f720f406007f2b6d272dec44185ec3b5f48300f40` |
| 初始化函数 `0x10001000` 将 CLSID、MapDocument 名称与 ClassFactory<MapDocument> 关联 | `ev_6b7981407165260461d6049aa6a84981dfb9693eb14c4af6505063cc7c027ab0` |
| 工厂虚表 `0x100897f4` 的 `+0xc` 为 `0x1000cb00`；分配 100 字节后调用构造器 `0x10010cc0` | `ev_1abb9fccba0232bee2e5401f3e6d78ceb2b3202d04fac725baea5a99a82e6a07`、`ev_3b281062549d5ef6e7c2b8838e9349f593ddc67436826e05f4a5a16643b91787` |
| QI `0x10011b60` 比较 `0x10089fcc` 处的 IMapDocument IID，返回完整对象 `+0x10` | `ev_72c9df3138beb296737b0e2f27ae44d5c17b9ddb702504b0dd75369d9d1868dc`、`ev_1ff7a8e931ff150390a76f25cc2bedb3d0174af377bc1cb1d91f3595f24d562b` |
| 构造器先设基类虚表 `0x1008a000`，最终将对象 `+0x10` 改为 `0x1008a1a4` | `ev_77ebc7c759165f2e8d41d5df1703edd718cde96c07b363faba6d55939c650fc3`、汇编 `ev_126c17fc4e676bbf4e91d5839c412a1e63a22f506f53136c40c807ee9b089b31` |
| 最终虚表 `+0x24 = 0x10012ed0`（Open），`+0x58 = 0x10013e40`（GetVersionInfo） | `ev_347307ecc1945af6e30604921fe7260d539582f80fbef63ea7a2236a8d330369` |

**类型恢复限制：** Ghidra 将 `0x100105a0` 误标为 ATL GetConnectionInterface；
实际反编译只是向输出写上述 CLSID（证据
`ev_055b0b3330ac1282947281f3188825d8826fde63206a1d2add67474dc00e362f`）。
不能因这个库匹配名称推定对象行为。虚表中多个相同 RTTI 名称也不能互换。

## Open 与 GetVersionInfo 的不同路径

Open `0x10012ed0`（`ev_a210c791df6c2b7ca9294b06d8c36524ce154dbddf5d723d4172aa688cf78712`）
先处理包/Web Map 等分支，检查文件和密码；普通路径按 `0x1000f4d0` 的结果
选择后端。结果 0 时创建 `0x60` 字节后端，经 `0x10075190` 打开存储，
`0x10077160` 读取文档信息。未将所有扩展名和分支的对应关系验证完毕。

GetVersionInfo `0x10013e40`（`ev_8da0e5c495b58f56898d955314bb05346208218959fa5b728f2ef84fba93251b`）
调用 `0x10011200`，再拷贝完整对象 `+0x50..+0x60` 的四个整数。
`0x10011200`（`ev_8f193dbecbd0048f29d2f063dd3279afe36bc03a64ddf1ce81e89bf35d3ea7c8`）
在普通后端调用 `0x10076920`，取属性 `Build Number` 并交给 `0x10011080` 分解。
`0x10076920`（`ev_18dbed67f1da0e03cdb33ca2e8e813e62e83e3239854dded58b4dc7a23e71217`）
打开 `Mx Document Version Info`，读 4 字节后连接 ObjectStream，再加载对象。
所以 **GetVersionInfo 不是 `/Version` 字符串的直接读取入口**。
另一个后端分支使用常量 `10.8.0.12790`，也不能推广到普通 MXD。

## 已找到的 /Version 局部读取

后端 `0x10076ef0`，反编译证据
`ev_802ad4e215adfe29d537f35a827084e9cd51a7085912f448e4058593e45e70e0`，
汇编 `ev_28b0043bf0ce322ea26d7f53f0b517f209899601fba6d3c576e877716ab27056`：
成功取得存储和文档信息接口时创建 ObjectStream（CLSID 在 `0x10089120`，
字节为已知 `043731d0-a7cf-11d1-8bd1-080009ee4e41`），
IStorage `+0x10` 打开 UTF-16LE 名称 `Version`；连接后调用 AfCore String::Read，
再从底层流调用 AfStreamRead 读 2 字节 short。short 有符号值 `<0x12` 时
将输出值改为 3，否则保留初值 `0xb`。随后调用文档信息接口 `+0x28(0x12)`。

该函数未检查 String::Read 和 AfStreamRead 的返回值；短读影响须继续核对
AfCore 帮助函数，不能仿照此处忽略错误。它不解析字符串文本以选择版本，
也没有消费其后的 4 字节。因此 `/Version` 的 `10.1` 候选不是已证实的完整
版本标识；尾部字段和完整读写规则仍未知。

## 未解决与保存

AfCore `0x1025cc40` 的版本 IID 实现者尚未核对；String::Read/Write 的长度和
编码、AfStreamRead 的短读、完整 Version 写入及尾部、实际版本映射仍待追踪。
本轮没有足够依据自动识别 MXD 版本或接入对象头读取器。
MappingCore 和 MappingCoreLib 的有边界 GUID 搜索保存在忽略目录的
`mappingcore-guid-snapshot.json`、`mappingcorelib-guid-snapshot.json`。
DLL、注册清单、临时元数据检查程序均仅保留在 temp/；没有提交原始提取物。
