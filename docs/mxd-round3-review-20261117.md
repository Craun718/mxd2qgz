# MXD 解析第三轮复核（2026-11-17）

6 个 `gpt-6.1-terra` 子代理并发处理了上一轮列出的 48 个静态研究项，并单独追查 BasemapLayer。对缺失函数补做定点 REA 反编译，保留文件身份、证据 ID、适用版本和停止边界。原始 PE、伪代码及测试样本仍放本地，反编译结果不当作原始源码。

原 48 项闭合 3 项，剩余 45 项，其中 44 项仍有静态路径、1 项（239）在现有类型库和已核验二进制中未定位，可能需要其他类型库或真实 QI／调用链。原先的 10 项运行验证与 20 项样本任务本轮未改，MXD 解析 TODO 共剩 75 项。

## 原编号逐项勾选

- **179 ObjectClassName 实现定位**：10.1 GdbCore 的 CLSID 注册、工厂 RTTI 与构造函数的实例虚表，均指向 `FdbTableName`。本项只解除实现位置未知，不认定持久化体已解析。
- **233 Geometry 记录类型归属**：10.1 类型库和原生解析分支区分磁盘 `esriShapeType` 与公开 `esriGeometryType`。例如磁盘类型 5 是 Polygon，公开 Polygon 值为 4；0x34/0x35 是 GeneralPoint/GeneralMultipoint。导入和导出对所有变体的接受范围仍未验证。
- **262 LineElement 原宿主**：从 10.1 介质 `CentralUSA.mxd` 的 CFB `/PageLayout` 根校验类标识并顺序解析。首个 TextElement 在 `0x96e` 定义空间参考键 743，紧随的 LineElement 在 `0xb0f` 回引它，并完整消费到 `0xb29`。测试只登记实际读取的键，单独从空表读第二项会回滚失败。本项不代表整个页面或跨流打开链已经支持。

## BasemapLayer 和 Map 尾部

- 10.1 介质 `Africa.mxd` 的 BasemapLayer 键 40 能通过有界图层入口读取第一个准备结果内的 FeatureLayer（键 42），首个 HorizontalLegendItem 实际回引该键。余下 Basemap 体从 `/Maps:0x145d` 到原条目边界 `0x65fd` 仍保留；起点为未接入的嵌套 Map（键 91）。图例继续到第二项的 FeatureLayer 键 140（`0xa13d`）处停止，尚未走完剩余基图及正式子层循环。
- 10.1 DisplayCore 的照明属性读写布局与参考样本一致：参考文件在 `/Maps:0x24fb2` 开始读取 IlluminationProps，完整到 `0x24fe8`；后续字段继续保留。原 113 仍有成员语义和非空变换对象等缺口。

## 48 项分派及证据

| 分组 | 原编号 | 报告 |
|---|---|---|
| BasemapLayer 优先接线 | 额外优先项 | [基图](mxd-round3-basemap-20261117.md) |
| 页面、装饰和元素 | 174、213–216、220–221、242、262 | [页面与线元素](mxd-round3-line-element-20261117.md) |
| 版本与对象流 | 71、74–75、88、100–101、104–105、107、113–115 | [版本与对象流](mxd-round3-version-object-20261117.md)、[Map 尾部](mxd-round3-map-tail-20261117.md) |
| 图层与标注 | 153–155、222–226、272、290–291 | [图层与标注](mxd-round3-labels-featurelayer-20261117.md) |
| 数据集与数字格式 | 178–179、218–219、230–231、240–241 | [数据集与格式](mxd-round3-dataset-format-20261117.md) |
| 符号与空间 | 210、212、217、233、239、243–244、270 | [符号与空间](mxd-round3-display-spatial-20261117.md) |

`cargo fmt --check`、`git diff --check`、`cargo test --locked` 通过（705 项通过，9 项默认忽略），另显式执行 8 项本地介质测试，均通过。未在 ArcMap 10.1 或 QGIS 内打开工程进行人工验收。介质用例验证的是上述确切的持久化读取链。
