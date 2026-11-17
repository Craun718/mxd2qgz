# MapSurround / HorizontalLegendItem 字段语义跟进

日期：2026-11-18。目标版本 ArcMap 10.1.0.3035。本记录仅闭合一个 MapSurround 共同前缀成员，不改 TODO、README 或共享模块声明。

## 已闭合：MapSurround 关联地图槽

- Map.dll 10.1 SHA-256：`1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`；类型库 esriCarto OLB SHA-256：`8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`。前者在本地导出索引的 Map 模块 `EXPORT_DONE` 记录中列出 8026 个成功、7 个失败函数。
- Map.dll `FUN_794413d0`（共同前缀 Load）调用 `LoadObject` 请求 IID `{E6BDAA75-4D35-11D0-98BE-00805F7CED21}`；10.1 esriCarto 类型库将其命名为 `IMap`。Load 随后把结果传给 MapSurround 接口子对象的 `+0x10` 虚函数。地址命名的反编译文件为 `/home/gxrs/文档/source/arcmap10.1/Map/794413d0.c`。
- 配对 `FUN_79441510`（Save）对 backing IMap 指针调用 `SaveObject`（伪代码参数偏移 `this−0xa4`）。两函数的 this 调整方式不同，因此本记录只用 IMap 接口身份和读写行为确认语义，不将伪代码偏移视为相同绝对地址。文件为 `/home/gxrs/文档/source/arcmap10.1/Map/79441510.c`。
- 因此该流成员语义可具体命名为“关联地图”。读取器字段注释与合成测试同步收紧；测试回引头使用 Map 的 CLSID。读取器仍原样保留回引头类标识，不把该语义扩展成额外的格式校验规则。

## 未闭合与验证边界

- 共同前缀版本 ≥2 的 short 与版本 ≥3 的 32 字节段仍无成员级 getter/类型库属性映射；不命名为边界、范围或标志。
- HorizontalLegendItem / BaseLegendItem 标量字段仍未闭合。本轮没有对 Renderers.dll 或 LayerLib.dll 做 getter 地址映射，也没有改变其解析器。
- 本结论来自 ArcMap 10.1 静态反编译、IMap 接口 IID/类型库名称及读写同槽交叉核对；没有 ArcMap 实机，未作运行取证、目标应用打开验证或 QGIS 目视检查。仓库参考样本的 MapSurround 计数为 0；合成测试只核对读取器行为，不构成真实样本字节证据。
