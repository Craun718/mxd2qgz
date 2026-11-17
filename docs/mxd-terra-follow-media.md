# 介质历史分支：从首个 Map 追踪宿主（2026-11-17）

本轮独立执行文件级遍历，目标是寻找 FieldInfo v0–2 和 Envelope v0 的可归属实例，兼记首层 GroupLayer、FDO。读取 [主审交接](mxd-terra-next-review.md)、[FeatureLayer 分项](mxd-terra-next-feature.md)、[Map 分项](mxd-terra-next-map.md)、[第四轮图层记录](mxd-round4-layers-20261117.md)和[介质清单](mxd-install-media-samples-20261117.md)。没有修改 TODO、公共入口或已有改动。

## 方法及覆盖

两套已提取的原始介质文件分别是 `temp/arcmap101-media-mxd/`（66 份）和 `temp/arcmap108-media-mxd/`（68 份）；来源及 CAB 摘要见介质清单。临时程序 `temp/terra-follow-media/src/main.rs` 对每个 OLE 文件读取 `/Version` 文本和 `/Maps`；从数量字段开始，创建新的引用表，读取首个 Map 新对象头后调用现有 `read_map_suffix_10_1`，逐条检查已解析的首层图层体、直接 FeatureLayer 的 FieldInfo 循环和 Map 的 IEnvelope 槽。另检查首层 FDO 的关联/委派 FeatureLayer 与首层 GroupLayer；没有遍历所有可能的嵌套宿主。结果是现有 10.1 读取规则对文件的结构化解释，不是目标程序执行结果。

所有 134 份均进入首个 Map 读取并返回结果；其中 20 份声明多个 Map，本次只检查第一个。读取器遇到未知体可以安全停止或依据图层前缀 A 推定边界跳过有界尾部；所以返回结果不表示整个 Map 已解析，也不能据未到达的字段断言对象不存在。两套介质存在相同内容，本表计的是文件及解析实例，**不是互不相同的工程数量**。

| 介质 | `/Version` 文本 | 文件数 | 可达直接 FeatureLayer 体 | 可达 FieldInfo 后继段 | 完整体 FieldInfo 版本及数量 | Map IEnvelope 完整体 |
| --- | --- | ---: | ---: | ---: | --- | --- |
| 10.1 | 9.2 / 9.3 / 10.0 / 10.1 | 3 / 34 / 11 / 18 | 0 | 0 | 未到达 | 未到达非空体 |
| 10.8 | 8.1 / 9.2 / 10.0 / 10.1 | 1 / 12 / 11 / 26 | 0 | 0 | 未到达 | 未到达非空体 |
| 10.8 | 9.3 | 18 | 92 | 56 | v3：368 | v3：18 |

统计仅包含 FieldInfo 新对象且解析体未标记内部停止；回引、空槽及未到达体不算作完整体。18 个 Map IEnvelope 均来自解析器的 Map 后缀对应槽，不含图层或其他宿主内的 Envelope。首层未解析出 GroupLayer 或 FDOGraphicsLayer，不能推广到有界未知体、嵌套图层或后续 Map。9.3 组的首个 Map 均在书签对象体处停止；图层列表中部分条目保留未知尾部。其他组实际停止点及每份结果保存在 `temp/terra-follow-media/inventory.txt`，其中可能含敏感本地信息，分享前需检查。

## 独立实例核对

10.8 介质的 `africa.mxt.2319C344_B117_4BE0_8B98_93D6E4E94502` 原文件 SHA-256 为 `39180b9d95cf1ade7fad3d4314b5157b9a58cfeced98ee54decb877659e8e884`；其 `/Maps` 流 SHA-256 为 `9568135816f004803db2c81a4f3d0af8e4e7d6b856cd583b7243924e89e43b2e`。`/Version` 原字节为 `08 00 00 00 39 00 2e 00 33 00 00 00 10 00 7b 02 00 00`：文本 9.3、short 16；介质来源不等于文件写出软件的运行版本。

解析路径是 `/Maps` 根计数 → Map 新对象及引用表 → 图层列表 → 起点 `0x202` 的直接 FeatureLayer 新对象（键 40）→ FieldInfo 计数 6 的循环。读取器报告六个新对象均完整，键 59、60、61、63、64、66 均为版本 3。以这些**已由宿主循环给出的键和类**复核 `/Maps` 原始字节，类标识 + 对应小端键 + `03 00` 的位置依次为 `0x86b`、`0x8c1`、`0x911`、`0x97b`、`0x9d3`、`0xa53`，位于该图层条目范围内；匹配仅作为原始字节交叉检查，偏移由键模式查找，非独立仪器记录的逐对象游标。列表边界的 A 前缀仍是观察推定，且宿主解析依赖既有读取器，不能把该检查升级为原生运行验证。

FieldInfo 静态版本门的目标二进制仍为 Layer.dll x86 10.1.0.3035；本轮重算 `temp/agent-element-coll/work/Layer.dll` SHA-256 `fe52bd9988b66f9d6f906b014e5684f28fc20e9b97b53c90909fa3d44e9efed8`，与[第四轮](mxd-round4-layers-20261117.md)的版本资源和 REA 静态反编译证据一致。哈希本身不证明签名、目标运行状态或文件实际由该版本写出。本轮没有更改共享 REA 服务器目标，也没有新建原生反编译或声称新运行证据。

## 可闭合性与复现

本轮没有新增可勾选的开放缺口：FieldInfo v0–2、Envelope v0 和 GroupLayer 版本分支仍缺能从真实宿主链归属的相应版本；FDO 旧体亦未在已检查的首层宿主中到达。已有 TODO 中 SouthernUSA v3 实例已单独记账，本轮 Africa 及批量 v3 是补充证据，不等于 v0–2。应优先继续未解析的图层体、嵌套对象和多 Map 文件的后续 Map，或寻找可确认来源的更早旧档；不得将这些文件中未到达的分支判为不存在。

复现：`TMPDIR="$PWD/temp/terra-follow-media/tmp" CARGO_TARGET_DIR="$PWD/target" cargo run --offline --quiet --manifest-path temp/terra-follow-media/Cargo.toml > temp/terra-follow-media/inventory.txt`。程序与输出均在仓库忽略的专用临时目录；需要两套已提取文件和 Cargo 离线依赖。运行成功，134 份均取得首 Map 结果；逐文件和版本数量由该输出统计，Africa 的 SHA-256、版本字节和键定位另以独立文件读取复核。没有修改 Rust 产品代码，故本轮未重跑全量 Rust 测试；本次遍历是研究动作，不替代截断测试或互操作测试。

未在 ArcMap 10.1 或 QGIS 打开样本，未进行人工目视验收，也没有目标程序运行观察。
