# ArcMap 10.1 MXD 未定名 IID 类型库复核

> [后续 PE 指令复核](mxd-terra-depth-iid.md)确认版本 ≥17 的 FeatureLayer 槽实际请求 `{1714D59B-…}`。以下旧候选 `{97D5B914-…}` 的槽位归属已撤销。

日期：2026-11-16。目标版本：ArcMap 10.1.0.3035。范围为 TODO「MXD 解析」中的 `{7453C805}`、`{DBCA59AC}`、`{97D5B914}` 及相邻 FDO 委派对象 QI 表里的未定名 IID。本文只新增研究记录；未修改 TODO、共享源码或已有研究文档。结论来自类型库静态元数据与原生 DLL 静态分析，不代表 ArcMap/QGIS 人工验收。

## 来源身份与方法

- 安装来源为 README「适配目标」所列 ArcMap 10.1 Desktop `Setup1.cab`。本轮用 `cabextract -l` 确认包内有 79 个 `.olb` 条目；其中 `esriCarto.olb.392FE3A8_65C9_4463_A15B_89F00F8F3FE8` 条目大小 1,618,944 字节，时间戳 2012-04-20。直接从该 CAB 条目提取后，SHA-256 为 `b95e4a043c5776c41cee5601be0400a77a61cb1f4475b8bd2a5c43f4893cc51c`，与 `docs/arcmap-10.1-native-baseline.md` 登记的安装介质哈希一致。该身份链将类型库绑定到本地 10.1 Desktop 载荷。
- 类型库枚举使用既有 `LoadTypeLibEx` 枚举转储（对 `ITypeLib::GetTypeInfoCount/GetTypeInfo/GetTypeAttr` 取得 typekind、GUID 与类型名），并与本轮从 Setup1.cab 新鲜提取的 esriCarto 哈希核对。`typekind=3` 是接口记录。esriCarto 枚举序号 101 为 `IPublishLayer`，序号 604 为 `IAnnotationSublayer`。完整行见旧转储 `/tmp/mxd-arcgis101-guiddocs/wine/dump_esriCarto.olb.392FE3A8_65C9_4463_A15B_89F00F8F3FE8.big.tlb.txt`；此临时转储本身未重建，但其输入 OLB 摘要与本轮 CAB 提取物相同。
- 使用 REA 12.1.4 / Ghidra 对从同一 Setup1.cab 提取的 AnnoLayer.dll 与 FeatureLayer.dll 作新鲜字节搜索。AnnoLayer SHA-256 `b487f3e0dc0e70dbc4b75f72eb6dcd8d5521eb422dbea6e3b2e201489a2d33a5`，与原生基线相同；FeatureLayer SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`，亦与既有研究相同。对 IID 搜索时按 GUID 的 COM 内存布局（Data1/Data2/Data3 小端，Data4 原序）编码。
- `{DBCA59AC…}` 在 AnnoLayer.dll 搜索命中 `0x6b1ac4e0`、`0x6b1ac8d8`；完整已加载区覆盖 256,392 字节，无跳过的未初始化块。REA Evidence：`ev_dd8e2910933e308f4ca9bcddf891350d37570b884844a22b69d0e22358b35c45`。其用途与 `docs/arcmap-10.1-fdo-open-points.md` 记录的 FDO 主 QI IID→子对象偏移表相符（请求/查询到 +0x28）。
- `{97D5B914…}` 在 FeatureLayer.dll 的已加载区全镜像搜索无命中，REA Evidence：`ev_e1d61444c8be8ec5b175c033ccbc13577075266465f6a462342778e8766a221a`；覆盖 1,375,628 字节，无跳过的未初始化块。无命中仅是该 DLL 静态字面常量搜索结果，不证明接口不存在或 GUID 未经间接/重定位方式形成。

## 裁定

| IID | 类型库结果 | 与原生调用点互证 | 裁定 |
| --- | --- | --- | --- |
| `{7453C805-9D64-44A6-BF2F-73E477DCE18F}` | esriCarto `IPublishLayer`，接口（typekind=3，枚举序号 101） | FDO Load 创建的委派 FeatureLayer 在其对象 +0xc0 位置以该 IID 查询；既有 10.1 反编译记录见 `docs/arcmap-10.1-fdo-open-points.md` 第 190–192 行 | 接口名确认；可将此前“未定名”改记为 IPublishLayer。该槽与 FDO 流中是否持久化该接口状态仍未由此证明。 |
| `{DBCA59AC-6771-4408-8F48-C7D53389440C}` | esriCarto `IAnnotationSublayer`，接口（typekind=3，枚举序号 604） | 新鲜 REA 搜索在 AnnoLayer.dll 命中两处；既有 10.1 QI 表把它映射到 +0x28 | 接口名确认；但 `IAnnotationSublayer` 名称不等于已证明 FDO 体内该偏移的字段业务语义或具体实现类。 |
| `{97D5B914-FB22-11D1-94A2-080009EEBECB}` | 现有 79 个安装类型库的枚举转储均未检出 | FeatureLayer 版本 >0x10 请求 IID；新鲜 REA 字面搜索无命中 | 仍未定名。类型库未命中和 DLL 搜索无命中均非接口不存在的证明。 |

## 其余 FDO QI 候选

`docs/arcmap-10.1-fdo-open-points.md` 主 QI 表还列有 `{34C20004-4D3C-11D0-92D8-00805F7C28B0}`、`{12E361F0-1907-11D3-80C7-0080C79F0371}`、`{EEF01B2D-8FE1-4759-8B93-51EE22DED70A}`、`{2425B413-1FFC-4407-B9F0-B4AC9ED50BDE}`，此外另有已定名的 ILayer、IPersistStream、IPersistStorage。该四项 IID 在该记录中仍未定名；本轮未取得可将它们逐一绑定到类型库接口定义的肯定证据，故不凭 GUID 家族、地址邻近或调用槽猜名。它们不应与本轮已确认的 `IPublishLayer`、`IAnnotationSublayer` 混为一类。

## 未解决与核验复现

1. 复现介质身份：对 Desktop `Setup1.cab` 列出 `.olb` 条目，按条目名提取 esriCarto OLB，计算 SHA-256 并与上述摘要比较；不得只依文件名认定版本。
2. 复现接口名：对提取后的 OLB 调用 Windows `LoadTypeLibEx`（或兼容实现）枚举所有 `ITypeInfo`，读取 `TYPEATTR.guid/typekind`，按 GUID 做精确匹配，并保存输入文件哈希、类型库名、typekind、索引与接口名。GUID 字符串需按字段规范格式化，避免把内存字节序误当作字符串顺序。
3. 复现代码互证：分别从同一 Setup1.cab 提取 AnnoLayer.dll、FeatureLayer.dll，先核对 SHA-256，再在 REA/Ghidra 中按 COM 内存序搜索 IID；查看命中点所在函数的反编译/汇编，确认是 QI 比较或 LoadObject 请求常量，而非孤立数据。搜索空结果应保留范围、未加载区和工具限制。
4. 未解决：`{97D5B914…}` 的类型库所有权、运行时实现/查询路径和接口名；FDO QI 表剩余四个 IID 的类型库身份及成员槽映射；本轮没有枚举每个接口的方法名与 vtable 槽，也没有运行时 QI 观察。故这里只确认两项 IID 名称，不声称槽级方法语义闭合。

验证说明：本轮未改 Rust 代码，未运行测试；通过安装包条目清单、提取哈希、类型库枚举记录和 REA 搜索范围核验以上结果。没有在 ArcMap 10.1 或 QGIS 中打开工程进行人工验证。
