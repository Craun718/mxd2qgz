# Terra 第十轮：MXD 解析复核（2026-11-17）

接续 [第九轮](mxd-terra-round9-review.md)。六名 GPT‑6.1 Terra 子代理分工查证，主审复核了原始指令、类型信息和实现。

| 范围 | 主审结论 |
| --- | --- |
| [StdPicture](mxd-terra-round10-picture.md) | 闭合安装介质中 OLEAUT32 2.40.4275.1 的第二字及解码器首层分派：写侧恒零，读侧不使用该值。Rust 仍保留原始载荷；测试不是解码验证。 |
| [Legend](mxd-terra-round10-legend.md) | 闭合 Legend 自身前缀的三个标志、保留 short 和条目计数；MapSurround 基类仍另列。独立读取 PE 地址 `0x79480d38`，修正旧版 MinFontSize 内存默认值注释为 4.0；没有把它补成流字段。 |
| [私有 IID](mxd-terra-round10-iid.md) | 主审把 AnnoLayer 和 MappingCore 的 QI 命中偏移与 MSVC RTTI 基类对位，定名 IDocumentVersionSupport 和 INoPersistInArcGISDocument；完整保存委派链仍开放。 |
| [FeatureDatasetName](mxd-terra-round10-dataset.md) | 确认第四串和标志对应 IFeatureDatasetNameImpl 的两对访问器。只补注释；成员业务用途仍未知。 |
| [10.8 版本流](mxd-terra-round10-version.md) | 确认 Version 流的独立引用、切流到 Maps 和部分错误分支；切流前间接调用及其全部副作用仍未闭合，暂不改读取规则。 |
| [逐图层遮罩](mxd-terra-round10-mask.md) | 从本地 10.1 的 QI 及 RTTI 确认 FeatureLayer、FDOGraphicsSublayer、RasterLayer 的本类入口不支持 ILayerMasking；FDOGraphicsLayer 聚合返回的确切非委派入口未证实。参考底图仍读 6 条图层记录，`/Maps:0x253be` 起保留 431 字节。 |
