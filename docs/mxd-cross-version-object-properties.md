# 跨 MXD 版本的对象属性边界（2026-11-23）

本记录闭合 `TODO.md` 中「其他 MXD 版本的对象属性规则」。这里的闭合是格式边界
定性：不存在一个可绕开具体类的全局「对象属性」布局；10.1 目标读取器按已证明的
类分派和各类版本门处理对象体，未证明的类或版本继续保留为未知，不能猜测跳过。

## 已证机制

- ObjectStream 的对象头是 16 字节类 GUID 加 4 字节键；空 GUID 是空对象，已见键是
  回引。对象头没有通用体长字段。未命中对象时，`SimpleSaveObject`/`LoadObject`
  根据 `IPersistStream`、`IPersistVariant`、`IDocumentVersionSupport` 和
  `IDocumentVersionSupportGEN` 分派到对象类自己的持久化方法，未知类不能安全跳过。
  证据见 [10.1 ObjectStream](arcmap-10.1-object-body.md) 与
  [SimpleSaveObject 委派](mxd-simple-object-dispatch-20261118.md)。
- 各对象体的版本值不是全局文件版本，也不是类 CLSID。10.1 的 `Map`、
  `FeatureLayer` 等类通过 `AfStreamGetDocumentVersion` 取得保存索引，再用
  `AfStreamGetSaveVersion` 查自己的版本表；`0xFFFF` 表示该文档版本整类不写。
  已提取的表和查表语义见 [10.1 Save 写侧](arcmap-10.1-save-side.md) 与
  `src/version/save_version_table.rs`。
- 10.1 可写保存索引 0..5，对应 8.3/9.0/9.2/9.3/10.0/10.1。同一 10.1 二进制内的
  `Map`、`FeatureLayer`、`FDOGraphicsLayer`、`EditTemplateManager` 等版本门因此
  可直接覆盖这些目标存档档位；不同类仍可拥有不同表、不同字段序和不同提前结束点。
- 介质实际覆盖 8.1/10、9.2/14、9.3/16、10.0/18、10.1/19 等磁盘头。已有读取器从
  10.1 与 10.8 两组介质读取 `/Maps`、`/PageLayout` 时，按每个对象流内的版本字段
  选择分支；例如 8.1 的 `calibrate.mxd` 读到 Map v19，9.x 工程读到各自 Map、
  Layout 和渲染器版本。参见 [介质样本](mxd-install-media-samples-20261117.md) 与
  [旧 Map 读取](mxd-terra-round9-old.md#10.1-原生旧-map-列表与-81-样本)。

## 结论

「其他 MXD 版本的对象属性规则」不能实现为一条通用字段序。正确的完成条件是把
文件版本转成各组件的保存索引或对象体版本，再由具体类的读取器解释字段；遇到没有
原生证据的类、版本或选择子时停在对象头并保留未知字节。

ArcMap 10.1 目标范围内，8.1 至 10.1 已观察对象的版本门和字段序由现有逐类读取器
覆盖；10.8 介质中出现的 8.1 至 10.1 磁盘版本也按同一对象体版本分派读取。未覆盖的
是 10.1 二进制中尚无类级证据的外来组件、插件类和更老历史分支；这些不能从本结论
自动获得完整支持。完整 Map/图层家族的人工核对仍属于后续中间模型与人工验收项。

本轮未新增对象体格式实现，也不宣称所有对象类已闭合；新增的是对跨版本属性规则的
边界裁决定性和 TODO 核销。未运行 ArcMap 或 QGIS，未做人工目视验收。
