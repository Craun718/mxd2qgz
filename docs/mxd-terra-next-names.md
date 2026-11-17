# DatasetName、FDO 与 IID 续研（2026-11-17）

行号固定为本次指派的 TODO 行号。未修改 TODO、公共入口或 `fdo_legacy.rs`。读取器现状与静态反编译不等同真实旧档或 ArcMap/QGIS 运行结果。本轮重新核对 GdbCore.dll SHA-256 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`，与 10.1.0.3035 既有基线一致。

| 行 | 状态 | 证据、缺口 |
| --- | --- | --- |
| 184 | 推进，未闭合 | TableQueryName 候选 Load/Save `0x708f5070/0x708f5280` 及字段序有 [Ghidra 桥接工具 记录](mxd-table-query-name-variant-20261116.md)，但 `Write_exref`、调用关系、虚表归属及 FeatureQueryName 完具体未闭合，不能安全实现。 |
| 224 | 推进，未闭合 | FeatureDatasetName 四个 String、标志与工作空间槽已有 10.1 读写结构；业务名尚缺访问器及成员偏移绑定，见 [名称记录](mxd-terra-names-20261117.md)。 |
| 225 | 推进，未闭合 | TopologyName 两 String 及 IDatasetName 槽的 Load/Save 已定位；字段属性及槽后虚调用仍待定性，见 [数据样本研究](mxd-parse-agent-sample-evidence-data-20261117.md)。 |
| 228 | 推进，未闭合 | FDO 旧加载器版本门和接口调用序已分析，接口方法到业务属性缺逐槽绑定，见上述数据样本研究。 |
| 245 | 阻塞 | `{97D5B914-FB22-11D1-94A2-080009EEBECB}` 在已枚举 79 个 10.1 类型库中未命名；FeatureLayer 镜像的有界字面搜索未命中，不能据此证明接口不存在，见 [IID 记录](mxd-iid-research-20261116.md)。 |
| 246 | 推进，未闭合 | 既有取证定名 IPublishLayer、IAnnotationSublayer；FDO QI 表还有四个未定名 IID，整个其他 IID 范围亦未穷尽。 |
| 247 | 推进，未闭合 | 六类的结构与 IName、IQueryDescription、IQueryFilter、ISelectionSet、ISpatialReference 名称已有；其标量、字符串与实际调用用途仍需访问器交叉绑定，见 [第四轮](mxd-round4-names-format-20261117.md)。 |
| 255 | 推进，待样本或运行证据 | FCN 非零 TopologyName 选择子分派、对象体读取器和合成测试已存在；尚无宿主链定位到的真实非零选择子。 |
| 256 | 推进，待样本或运行证据 | FDO 旧体版本 0..=3 字段序和读取器已有；参考样本版本为 11，无法验证旧分支。已有工作树的后续改动未由本组修改。 |
| 285 | 推进，待真实旧档或运行 | FeatureDatasetName v0/v1 静态读侧同布局，10.1 Save 恒写 v1；v0 目前仅构造测试。 |
| 286 | 推进，待真实旧档或运行 | TopologyName v0/v1 静态读侧同布局，10.1 Save 恒写 v1；v0 目前仅构造测试。 |

`cargo test --test dataset_name --test fdo_legacy --test terra_fdo_legacy` 共 34 项通过；这些为合成测试，不能证明旧档互操作。未作目标软件人工验证。未发起新 Ghidra 桥接工具 会话；复用既有版本化静态证据及本机导出，避免共享 `/tmp` 的空间压力。
