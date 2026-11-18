# ArcMap 10.1 RasterLayer：渲染器后的图层尾段

依据为 x86 Desktop 10.1.0.3035 的 RasterLayer.dll（ImageBase
`0x7c710000`，SHA-256 `9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6`）。
复用仓外导出的 Load `0x7c73bd40`、Save `0x7c73d0a0`、扩展列表 Load
`0x7c73ded0`，并用 objdump 核对 Save 版本表 `0x7c7678a8`：
`[7,11,13,13,16,17]`。表不是加载器接受版本的全集。接口身份按同一介质里
esriGeoDatabase、esriDataSourcesRaster 类型库的 GUID、TypeInfo 与名称表
对位。RasterIO 记录头和结束对沿用 [RasterCoreLib 取证](mxd-luna-deep-fgdb-raster-20261118.md#rastercorelib-导出与-rasterio-记录)。

## 字段顺序

两条参考工程 v17 图层在渲染器体后均剩 298 字节，尾段内容逐字节相同。
下表偏移相对于渲染器之后第一个字节。色表图层此处为 `/Maps:0x14479`，
RGB 图层为 `/Maps:0x16e8c`。

| 相对偏移（参考工程） | Load / Save 序列 | 当前读取范围 |
| --- | --- | --- |
| `0x00` | double×2、int32、short、int32（最后一项版本 ≥2） | 原样保留 |
| `0x1a` | 扩展列表，版本 ≥3；≥8 调 `0x7c73ded0`，int32 计数 + ExtraSafeReadObject 循环 | 零计数 |
| `0x1e` | double（≥4）、内联 WKSEnvelope（≥5） | 四个 8 字节坐标原样读入；参考档首项为 NaN，不映射为有效范围 |
| `0x46` | ≥6：int32 选择子；非零再读一个 IName。之后 int32 数量 + IRelationshipClassName 循环（IID `311C2EE2-…`） | 复用 RelationshipClassName 和 MemoryRelationshipClassName；未闭合体停止 |
| `0x4e` | ≥7：IEnvelope（IID `1B077914-…`） | 空、回引或已支持的 Envelope 类 |
| `0x5e` | RasterIO：magic `0x00ff00ff`；可选 tag 30 对象与 tag 31 宽字符串；tag 35/36/37 三个 short；`(-1,-1)` | tag 30 的非空类尚未接入 |
| `0x8c` | AfCore String（≥10）、v10 专用 short、double×2（≥11） | 当前 13–17 分支读取，不含 v10 专用 short |
| `0xa2` | 一对对象：IGeodataXform（IID `5E6CF525-…`）、IEnvelope；v12 依赖原生数据集状态，≥13 总是读取 | 地理数据变换支持空、回引和[三个已确认类](mxd-10.1-geodata-xform.md)；Envelope 复用已有读取器 |
| `0xc2` | ≥14：IColor、int32 数量、压缩索引列表 | 支持现有颜色类及零索引数 |
| `0xd6` | ≥15：TimeData → TimeDataDisplay → TimeDimensionDefinition | 复用 LayerLib 同名段读取器；没有 FeatureLayer 的 TimeTableDefinition，且显示段在维度段之前 |
| `0x116` | ≥16：IPixelFilter（IID `90E3DE9D-…`） | 空或回引 |
| `0x126` | ≥17：int32 数量 + IStatsHistogram（IID `2B9062E8-…`） | 空、回引和 StatsHistogram；与 RGB 的无键 RasterIO 对象共用体读取器，此处使用 ObjectStream 键 |
| `0x12a` | 对象体结束 | 两个条目都与外部 ExtraSafe 边界一致（`0x145a3` / `0x16fb6`） |

IGeodataXform 也是 ImageServerLayer v20 的请求接口；RGB 的 RasterIO tag 18
另经 `0x7c99da10` 调 QueryInterface（IID `A9D01E68-…`，
esriDataSourcesRaster 中的 IPansharpeningFilter）。前三个已接入的 IGeodataXform 类见[对象体记录](mxd-10.1-geodata-xform.md)；
RGB 过滤器的已确认范围见[全色锐化](mxd-10.1-raster-renderer.md#rgb-tag-18全色锐化)。
普通 RasterLayer 的 tag 30 是另一条路径：Load `0x7c73bd40` 取对象后经
`0x7c731840` 得名称接口，读 tag 31，并经 ConvertToFullPath 和 IName::Open
打开数据；不能按 RGB tag 18 的过滤器对象体分派。

实现 [raster_layer_tail.rs](../src/layers/raster_layer_tail.rs) 仅在父条目已有长度
且渲染器完整消费时启动；尾段顺序读完且与条目边界一致才提交游标和引用键。
未支持的 v7/v11 等历史输出、ExtraSafe 扩展的非空列表、RasterIO tag 30、
压缩索引、变换和像素过滤器类仍待补充。合成 v17 尾段含非空 IEnvelope 与
IStatsHistogram；逐字节截断及越过条目边界时均验证回退，不保留新键。
参考工程仅用于字节和边界对照，未做 ArcMap/QGIS 软件验收。
