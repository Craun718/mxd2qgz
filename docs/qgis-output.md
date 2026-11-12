# 独立 QGS/QGZ 输出基准与写入合同（18–21）

记录日期：2026-11-12。这里只研究构造模型到 QGIS 工程的输出，不依赖 MXD
解析，不证明任何 MXD 字段、图层边界或坐标系归属。没有修改 `model.rs`，
没有增加 `convert` 命令。源 ArcMap 和输出 QGIS 的人工目视检查均为**未验证**。

## 官方资料与适用版本

以下官方页面于记录日期实际浏览，不能把在线主干版本当成本机版本：

| 来源 | 核对内容与边界 |
| --- | --- |
| [QGIS 3.44 格式附录](https://docs.qgis.org/3.44/en/docs/user_manual/appendices/qgis_file_formats.html#qgs-qgz-the-qgis-project-file-format) | QGS 是 XML，QGZ 是 ZIP；工程包括 CRS、图层树、画布范围与数据源引用。该手册描述 QGD 辅助数据库，不是所有版本归档成员的强制清单。 |
| [官方 PyQGIS 工程读写教程](https://docs.qgis.org/3.44/en/docs/pyqgis_developer_cookbook/loadproject.html) | 用 `QgsProject.read()` / `write()` 检查 API 读写结果；返回成功仍需进一步核对图层有效性。已通过 Context7 检索其官方文档仓库内容。 |
| [QgsProject API](https://api.qgis.org/api/classQgsProject.html) | `write(filename)` 的成功返回值与文件名行为。浏览时文档为 4.3.0-Master（91d4a333d5d）。 |
| [QgsPathResolver API](https://api.qgis.org/api/classQgsPathResolver.html) | 路径以工程文件为基准读写；本实现独立进行跨平台词法改写，不声称复刻其全部 URI/附件处理。浏览时为 4.3.0-Master（664acece34d）。 |
| [官方 qgsproject.cpp](https://api.qgis.org/api/qgsproject_8cpp_source.html) | 浏览时 4.3.0-Master（91d4a333d5d），`readProjectFile` 2314–2355 行：只有 `SpatialRefSys/ProjectionsEnabled` 启用才读取工程 CRS；2698–2701 行读取 `ProjectViewSettings`。这些主干观察又在本机 4.2.3 API 中核对。 |
| [官方 qgsmaplayer.cpp](https://api.qgis.org/api/qgsmaplayer_8cpp_source.html) | 同一主干标识，`readLayerXml` 640–645 行只接受长度大于 10 的持久化 ID。本机 API 确认短 ID 被替换，因此输出使用长且确定的层级 ID。 |
| [官方 qgscoordinatereferencesystem.cpp](https://api.qgis.org/api/qgscoordinatereferencesystem_8cpp_source.html) | 浏览时 4.3.0-Master（e0370aa57ef），`readXml` 2170–2295 行展示 `spatialrefsys`、authority/WKT 读取顺序。写入的两份完整 CRS XML 直接取自本机 QGIS，未推测内部 `srsid`。 |

本机实测完整版本：

- QGIS `4.2.3-Belém do Pará`，`QGIS_VERSION_INT=40203`，`QGIS_DEV_VERSION=exported`。
  安装包未提供源码提交号，不能将它等同于上述主干提交。
- Qt `6.12.0`，PyQt `6.12.0`。
- `/usr/bin/python3`：`3.14.10 (main, Nov 11 2026, 08:10:30)`，
  GCC `16.2.1 20260810`。
- Linux `7.3.3-1-cachyos-x86_64`，glibc `2.44`。
- GDAL `3.13.3 "Iowa City"`（2026/08/13），PROJ `9.9.0`。
- 所有 QGIS 调用均设置 `QT_QPA_PLATFORM=offscreen`，使用 API，没有操作 GUI。

## 无敏感数据基准

`tests/fixtures/qgis/generate_baseline.py` 在临时目录新建工程，以
`QgsProject.write()` 生成 QGS/QGZ，设 EPSG:4326、标题 `Minimal baseline`、
默认视图 `[0,0,10,10]`，不包含真实数据源。API 自动写入的 `saveUser`、
`saveUserFull`、`projectMetadata/author` 已替换为合成账户标记；为此重新序列化
XML 和打包 ZIP，其余节点语义未主动修改。它们是脱敏的 API 生成基准，
不是原始逐字节输出。QGIS 自动时间、UUID 和随机样式库文件名未固定，因此
重新生成基准不保证字节相同；Rust 写入结果则具有确定性。

| 基准 | 身份 / 观察 |
| --- | --- |
| `minimal.qgs` | 13,611 字节；SHA-256 `844d2623b6ca62242b7a5be2573cac3df6f69139339781cb6b3f11f3e243e376` |
| `minimal.qgz` | 4,118 字节；SHA-256 `f489bf72b5087bbb96b9bf81d992a529e6d973faf394a69c1fe5276ed7248b87` |
| QGZ 成员 | `minimal.qgs`、`rkRUyT_styles.db`，本次没有 `.qgd`；官方 3.44 概述不能用来断言 4.2.3 必须生成 QGD。 |
| `minimal_attachments.zip` | QGIS 写普通 QGS 时产生的样式库附件，随基准保留。 |
| `epsg4326.xml` / `epsg3857.xml` | API 写出的 `projectCrs/spatialrefsys`，作为 Rust 编译时嵌入的定义。因此分文档/功能集成时，功能提交必须包含这两份 XML。 |
| `data/points.*` | API 创建的 Shapefile，只有两个合成点 `(1,2)`、`(3,4)` 和整数属性 0、1，无真实路径/业务内容。 |
| `baseline.json` | 版本、归档成员、合成数据与脱敏步骤的机器可读记录。 |

检查过基准文本、ZIP 解压成员和 Shapefile，不含本机账户名、用户主目录或
连接凭据。生成脚本的临时路径不写入公开基准。

重建基准会替换现有 fixture，应仅在明确要更新基准时执行：

```bash
QT_QPA_PLATFORM=offscreen /usr/bin/python3 tests/fixtures/qgis/generate_baseline.py
```

## Rust 接口和输出合同

```rust
use mxd2qgz::qgis::{WriteOptions, to_qgs, write_project};
use std::path::Path;

fn example(map: &mxd2qgz::model::Map) -> std::io::Result<()> {
let options = WriteOptions::new("/synthetic/source"); // 源 MXD 所在目录
let preview = to_qgs(map, &options, "/synthetic/output")?;
let diagnostics = write_project(map, Path::new("/synthetic/output/map.qgz"), &options)?;
// 调用方展示或保存 preview.diagnostics / diagnostics。
Ok(())
}
```

`to_qgs` 只生成 XML 与结构化诊断；两个目录参数均要求绝对目录。
`write_project` 根据大小写不敏感的 `.qgs` / `.qgz` 扩展名选择格式，允许宿主
相对输出文件名，但父目录必须已经存在。库不运行 QGIS 或 MXD 解析器。
IO/语义错误返回 `io::Error`；不支持的数据不会偷偷替换成其他数据源。

- QGS 是 UTF-8 XML；文本/属性转义 `& < > " '`，以字符引用保留换行、制表符、
  回车，拒绝 XML 1.0 非法控制字符。
- QGZ 为 Deflate ZIP，只含固定安全成员 `project.qgs`，不打包源数据。
  当前无辅助存储或样式数据库引用，因此不伪造空 SQLite/QGD；这种最小归档
  已通过本机 QGIS API 加载。未宣称其他 QGIS 版本兼容。
- ID 形如 `mxd2qgz_frame_0_0_1_0`，来自树中索引，长度满足 QGIS 读取条件。
  重名不冲突，相同模型重复写入相同目录，XML/ZIP 字节稳定；重命名和切换
  可见性不改变 ID。插入、删除、移动节点可能改变后续 ID，不表示跨编辑的源
  对象身份；模型尚无此字段。模型顺序直接映射树顺序，禁用自定义渲染排序；
  没有声称模型的源顺序已确认等于 ArcMap 叠放顺序（`source-order` 诊断）。
- `overwrite=false` 为默认：先检查已有目标（含悬空符号链接），最终仍用
  `persist_noclobber` 防并发覆盖。显式开启覆盖时，完整临时文件写完、ZIP
  完结且 `sync_all` 成功后才替换目标；覆盖的是链接项，不跟随目标符号链接。
  临时文件位于目标同目录，正常错误返回时自动清理，错误不会留下半成品。
  不保证进程被强杀后无临时残留，也未做父目录 fsync 的断电持久性保证。
- **多个数据框直接 fatal 拒绝**（`InvalidInput`），即使其他框为空也不会静默
  选择第一框。零框允许输出带明确诊断的空工程。单框名称保留到工程属性。
- 层级可嵌套，保存组和叶子的独立勾选状态。父组不可见时仍保存子节点勾选，
  交由 QGIS 合成有效可见性。深度大于 128、叶图层超过 100000 时返回错误。
- 只支持 `.shp` 文件，provider 为 `None`、`ogr` 或 `Shapefile`（后两者不区分
  大小写）。缺源、缺路径、其他 provider/扩展名或非空连接字符串均 fatal。
  缺主文件、缺同名小写 `.shx/.dbf`、异平台路径分别诊断，断链仍保留。
  文件存在检查不是数据有效性检查；编码、字段、驱动与几何有效性由 QGIS 再验。
- `unknown_fields` 在各模型层级非空时诊断，不复制未知原始字节及其中的敏感
  内容；原模型应另存。诊断位置为结构 ID，消息不嵌入连接和源路径。正常工程
  仍必须包含调用方提供的数据源路径，分享前需检查。

## 路径语义

`source_path::resolve(path, source_directory, output_directory)` 不使用宿主
`Path::is_absolute` 识别源格式，也不依赖文件存在。它先按源目录解释相对路径，
再相对于输出目录改写：同根时输出 `./...` 或 `../...`，跨盘、跨 UNC 共享或
跨平台根时保留绝对路径。正反斜杠都作分隔符，输出统一用 `/`。

| 输入情况 | 策略 |
| --- | --- |
| Windows `C:\data\a.shp` | 保留盘符绝对语义，不在 Linux 上拼到当前目录。 |
| UNC `\\server\share\a.shp` | 保留服务器/共享根；不把 `..` 提升到共享之外。 |
| 相对 `..\data\a.shp` | 相对于源 MXD 所在目录，再对输出目录计算相对路径。 |
| Windows 不同盘或不同共享 | 保留绝对路径，不生成无意义的相对路径。 |
| POSIX `/data/a.shp` | 按 POSIX 绝对路径解释；不是 Windows 当前盘根。 |
| `C:a.shp`、单反斜杠根路径 | 需要当前盘/盘内工作目录，信息不足，明确拒绝。 |
| URI、OGR `|` 选项、`?`/`#`、NUL、非盘符冒号 | 基础文件路径接口保守拒绝；不执行或猜测连接语法。 |

Windows 盘符、服务器、共享及公共目录前缀比较忽略 ASCII 大小写；POSIX 区分
大小写。`.`/`..` 作词法归并，越过根返回错误。不解析符号链接、网络映射或
非 ASCII Windows 大小写折叠；含符号链接及 `..` 的路径需要调用方先明确目录
身份。异平台文件能否在目标 QGIS 中加载仍需目标系统验证。

## CRS 与范围

目前只将明确的 `EPSG:4326`、`EPSG:3857` 定义（忽略大小写和外侧空白）写成
受支持 CRS。其他 EPSG、自定义 WKT、ESRI 文本、PROJ 字符串均发出
`unsupported-crs`，不从名称或文本猜 EPSG。原定义保存在工程
`mxd2qgz/sourceCrs` 属性或图层同名自定义属性，QGIS API 可读取；工程属性已
验证经 QGIS 再保存后仍保留。缺失 CRS 发出 `missing-crs`。

工程使用单框 CRS，图层使用各自 CRS，二者不互相覆盖。主画布使用
`ProjectViewSettings/DefaultViewExtent`，范围附带框 CRS；没有范围时诊断
`missing-extent`。不对坐标重新投影，不用图层范围反推主画布。
所有给出的范围及宽高必须有限且 `xmin<xmax`、`ymin<ymax`，否则 fatal；点状/线状
退化范围未支持。CRS 未支持时不应用范围（`extent-without-crs`），原值仍保留
到 `sourceExtent` 属性。未知工程 CRS 经 API 确认无效，没有变成默认 EPSG。
图层缺 CRS 时，QGIS 可能使用提供者读出的 `.prj`；这是目标软件行为。

图层 `<extent>` 是保存的范围元数据，不是要素裁剪或缩放指令。API 实测正常
OGR 加载会重算为数据范围 `[1,2,3,4]`，即使模型/输出 XML 为 `[0,0,10,10]`；
主画布范围仍保持 `[0,0,10,10]`。未开启“信任图层统计”等选项强行覆盖真实
数据范围，原模型范围可通过自定义属性读取。

## 验证结果与复现

```bash
cargo fmt --check
cargo test
cargo test --test qgis qgis_api -- --ignored --nocapture
```

普通测试不要求安装 QGIS；API 测试默认忽略，第三条命令显式运行，测试进程
自行给 Python 设置 offscreen。API 不可用时应报告测试未运行，不把忽略算通过。

本次最终检查（包含共享工作区已集成的 native_string 测试）：
`cargo fmt --check` 通过；`cargo test` 43 项通过、1 项 API 测试
默认忽略；显式运行该 API 测试后另有 1 项通过。Cargo 曾输出全局缓存自动清理
权限警告，但构建和测试返回成功；无需修改共享缓存或提升权限。

本次独立输出测试包含 XML 解析与文本还原、ZIP 成员/CRC 解压、输出确定性、
重名 ID、组顺序/勾选、CRS/范围、未知信息诊断、Windows/UNC/POSIX 重定位、
非法 XML/范围/数据源/多框、无效目录/扩展名、默认防覆盖、悬空链接、8 个并发
写入者仅 1 个发布成功，以及临时文件已写完但最终发布失败时的清理。

QGIS API 测试在本机 4.2.3 中通过，核对：

- 脱敏最小 QGS/QGZ 可读，无数据图层，EPSG:4326、默认视图 `[0,0,10,10]`。
- Rust 输出 QGS/QGZ 各 3 个图层；2 个有效，每个读取出 2 个合成点；1 个缺失
  文件的图层无效但仍在工程中；加载路径指向 fixture，无错误拼接。
- 两级分组、同名叶子、ID、树序和独立/有效可见性均保持预期；隐藏父组中的
  已勾选叶子仍勾选，但 `isVisible()` 为 false。
- 工程 EPSG:3857、图层 EPSG:4326、主画布及范围 CRS 正确，工程/图层原始属性
  可读。图层范围按上述提供者重算行为记录。
- 未知 CRS 工程保持无效 CRS，默认视图为空；空 QGZ 没有图层和有效 CRS。
- 未知 CRS 的原始定义经 QGIS 再保存和读取仍保留。

范围缺口：仅单框 Shapefile 和两个已核对 CRS；不含样式、标注、布局、其他
数据源、多框主题/布局映射及广泛 WKT/CRS 支持。没有证明 ArcMap/MXD 到模型的
映射正确，没有测试 Windows QGIS、其他 QGIS 版本、真实项目或 GUI 视觉效果。
每个图层均有 `default-style` 诊断：QGIS 默认样式不能代表源样式还原。
**API 加载核对通过不等于人工目视检查通过。**
