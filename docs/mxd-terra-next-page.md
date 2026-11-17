# 页面、图例和书签续查（2026-11-17）

范围为本轮 TODO 当前行 219–222、226–227、248、266；未修改 TODO、公共入口或书签代码。`temp/agent-map-surround/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` 本轮 SHA-256 为 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`，与 [第四轮](mxd-round4-page-20261117.md)中 PE x86、Esri Map DLL、10.1.0.3035 身份一致。`temp/pseudocode/pagelayout` 是 Ghidra 12.1.4 的静态反编译导出，其索引 SHA 为 `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f`；不能当作原始源码。此目录不含 Map.dll、Layer.dll 和 LayerLib.dll 的完整伪代码，不能仅凭命名把它代替这些目标。因 `/tmp` 空间告急，本轮没有启动大型 Ghidra 缓存，关闭了本轮只完成身份绑定的 REA 目标。

## 条目状态

| 行 | 状态 | 证据与剩余工作 |
| --- | --- | --- |
| 219 | 推进 | [LegendClassFormat 读写](arcmap-10.1-legend-tails.md)定位四对象槽及版本 >1 的两个 double（+0x1c/+0x24）；缺接口访问器逐偏移对位。 |
| 220 | 推进 | 同文定位 Horizontal 封装 int32 和 BaseLegendItem 字段组，旧 Africa 体有边界证据；业务名仍需逐槽访问器绑定。 |
| 221 | 推进 | [MapSurround/Legend 记录](arcmap-10.1-mapsurround-bookmark.md)给出版本门与短整数字段序；不能从集合或布尔外观推断所有字段含义。 |
| 222 | 推进 | [元素体](arcmap-10.1-element-bodies.md)给出 ElementProperties 的双字符串、旧版遗留 int32、Variant、short、double 和 int32 及版本门；名称等仍为候选，需接口访问器对位。 |
| 226 | 推进 | MapSurround 前缀名称、IMap 槽、short 和 32 字节块已有读写证据；后者读后丢弃，不据长度命名范围，short 含义未定。 |
| 227 | 推进 | AOI 与 Feature 书签分别具有不同的体布局；名称 String 与 IName 槽已有证据，但 Feature int32 业务语义仍未知。仅文档研究。 |
| 248 | 推进 | 装饰物族各类结构与共同前缀调用者已有 [第四轮](mxd-round4-page-20261117.md) 静态证据，标量尚缺逐类访问器绑定，ScaleLine 三调用者类身份仍未知。 |
| 266 | 阻塞 | Legend v0/v1 已有静态版本门和构造测试，介质候选筛查未发现对应体；需要真实旧档及从 Map 根建立的对象引用表，不能以构造输入闭合真实验证。 |

介质筛查只覆盖既有 66 份安装介质文件的 `/Maps` 流，候选搜索不能独立证明对象边界、图例数量或其他文件不含旧版。`cargo test --test round4_page` 两项通过，只验证构造输入边界和回滚；并未在 ArcMap 或 QGIS 打开文件，也没有声称运行或人工验证。
