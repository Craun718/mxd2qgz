# mxd2qgz

面向 ArcMap MXD 到 QGIS 工程转换的实验工具。目前处于 MXD 格式研究阶段。

## 使用

需要 Rust 工具链。运行：

```sh
cargo run -- inspect 'map.mxd'
cargo run -- strings 'map.mxd'
cargo run -- extract 'map.mxd' /Maps maps.bin
cargo run -- compare 'before.mxd' 'after.mxd' /Maps
cargo run -- hex 'map.mxd' /Maps 0x860 96
```

`inspect` 列出 OLE 容器的数据流和字节数；`strings` 扫描 `/Maps` 中两种对齐方式的 UTF-16LE 候选字符串，并输出相对于该流的十六进制偏移。扫描不是对象解析，可能存在乱码、截断或误报，不能据此判断图层数量、完整路径或坐标系。`extract` 导出指定数据流，不会覆盖已有文件。`hex` 查看指定流的字节范围，偏移和长度支持十进制或 `0x` 十六进制，范围超出流时返回错误。`compare` 比较两个文件中指定流的相同偏移，输出不同字节的半开区间及流长度；插入或删除字节会使后续数据错位，差异区间不表示对象边界。研究记录见 [docs/maps-analysis.md](docs/maps-analysis.md)。

制作对照样本、记录流内偏移和脱敏的方法见研究记录中的“对照样本采集流程”。目前尚不能识别 MXD 版本或转换工程。

## 后续工作

1. 构造内容可控的多个 MXD 样本，比较数据流，验证对象边界、类型及版本差异。
2. 解析地图、图层树、数据源、坐标系与显示范围，建立有明确未知字段的中间模型。
3. 使用 QGIS 支持的接口生成项目，逐步覆盖符号、标注和布局，报告不支持的对象与断链数据源。
4. 对照 ArcMap 原始工程和 QGIS 输出，验证数据、图层顺序及视觉效果。

MXD 通常引用外部空间数据；仅有 MXD 时，数据源未必可访问。原始路径可能包含敏感信息，分享诊断结果前请检查。
