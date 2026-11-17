# FeatureIDSet 重复成员语义（2026-11-18）

## 可复核结论

FeatureLayer.dll 10.1.0.3035 的 `FeatureIDSet::Load`（静态映像地址 `0x6f6d97d0`）对每个流内 int32 调用 `0x6f6d9940` 插入内部集合。该插入器检查既有成员：相同值分支返回 1，不增加成员计数；新值才分配节点并增加计数。`FeatureIDSet::Add` 的包装函数 `0x6f6d96b0` 调用同一插入器。`Save`（`0x6f6d9880`）写出集合计数并通过迭代器遍历集合节点序列化成员。因此 Load 输入中重复整数会消费其流字节，但不会形成重复集合成员。

本轮将 `FeatureIdSetBody10_1.ids` 的含义改为去重后的集合成员，按流内首次出现顺序呈现；`count` 仍保留流内声明计数，游标仍消费所有 `count` 个 int32。该向量顺序不宣称等同原生 Save 的哈希桶遍历顺序。接口 `IFeatureIDSet` 与集合去重行为已绑定；元素是否为数据源 OID 仍未证实。

## 原生证据身份与范围

- 模块：`FeatureLayer.dll`，10.1.0.3035，既有介质提取及反编译成果 `/home/gxrs/文档/source/arcmap10.1/FeatureLayer/`。
- 目标类：`FeatureIDSet`；Load `0x6f6d97d0`、Add 包装 `0x6f6d96b0`、集合插入 `0x6f6d9940`、Save `0x6f6d9880`。函数索引见同目录 `functions.tsv`（地址记录为 `ok`）；伪代码文件与地址同名。
- 证据是静态反编译，未在 ArcMap 10.1 运行，也没有非空真实 FeatureIDSet 样本；参考 MXD 的该对象计数为 0。原生 Save 的成员遍历顺序未映射到 Rust 表示顺序。

## 修改与验证边界

- `src/layers/feature_id_set.rs`：读取所有输入整数，但只在结果向量中首次加入每个值。
- `tests/terra_host_label.rs`：覆盖带负值及重复值的可构造输入，验证原始计数、去重成员、完整消费和截断失败时游标回滚。
- 本记录不闭合 OID 含义、成员有效范围、原生保存顺序或 ArcMap/QGIS 人工验收。
