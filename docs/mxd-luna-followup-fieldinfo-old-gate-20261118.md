# FieldInfo v0/v1 旧门专项复核（2026-11-18）

本轮只复核 TODO「FieldInfo v0/v1 实档分支」中的一个窄点：旧版本体是否仍含 INumberFormat 对象槽。没有修改 TODO、README、公共模块声明或产品代码；仅新增专用测试与本报告。此复核没有找到可顺序到达的 v0/v1 实档，因此不能把旧门升级为真实 MXD 证据。

## 10.1 原生调用与版本门

- **实现模块：** ArcMap 10.1 `Layer.dll`，PE32 x86，固定文件版本 `10.1.0.3035`；既有身份复核和 SHA-256 前缀 `fe52bd9988b66f9d` 记录于 [FeatureLayer 尾段研究](arcmap-10.1-featurelayer-tail.md)。本机 10.1 反编译记录定位 `FieldInfo::Load` 实体为 `0x7865b520`，GetClassID 为 `0x7865b4e0`，保存实现为 `0x7865b660`。现有反编译与虚表/RTTI 对位见该研究记录“FieldInfo 元素体”一节。
- **宿主模块：** `FeatureLayer.dll`，固定文件版本 `10.1.0.3035`，ImageBase `0x6f280000`；现有本机导出中 `FeatureLayer::Load` 为 `0x6f6e9940`。伪代码显示 FieldInfo 集合经 `LoadObject` 装载；到 FieldInfo 体之前，宿主 FeatureLayer 版本门已控制是否进入该尾段。本轮未重新盘点尾段字段。
- **旧体布局窄结论：** `0x7865b520` 读 short 版本并以有符号 `version > 4` 拒绝；随后无条件读一个 short、一个 String，再通过 `LoadObject` 请求 INumberFormat。第二个 short 只在 `version > 1` 时出现。因此 v0/v1 **省略的是第二个 short，不是 INumberFormat 槽**。Save `0x7865b660` 的对应门与该旧版本字段分组一致。地址是 10.1.0.3035 静态映像地址。

该规则已由既有静态反编译和读取器确认。本轮新测试把旧版槽形态做成专项回归：分别构造 v0、v1 的“版本 + 首 short + alias + INumberFormat 回引”，断言第二 short 缺席、回引键不被错误登记、游标停在下一个字段，并对每个截断点检查游标与登记表回滚。测试属于合成输入，不是 MXD 样本证据。

## 宿主实档复核状态

已知 ArcMap 10.1 SDK .NET `Continents.mxd` 的 Map→图层→FeatureLayer→FieldInfo 宿主链到达 5 个 FieldInfo 新对象，均为 v2；10.1 安装介质 `SouthernUSA.mxt` 的 FeatureLayer 宿主链到达 11 个对象，均为 v3。相应对象头、键和体版本见 [介质旧档记录](mxd-terra-round8-old.md) 与 [FieldInfo 实档记录](mxd-layer-label-cgl-20261117.md)。这两条链验证的是 v2/v3，不能证明 v0/v1。

本轮没有建立任何 FieldInfo v0/v1 对象边界；不以全流 CLSID 命中、对象头扫描或类名候选代替宿主遍历。因此 FieldInfo 旧版本实档缺口仍开放。没有启动 ArcMap；也未在 ArcMap 或 QGIS 中打开或目视检查工程。

## 测试记录与限制

- 新增 `tests/luna_fieldinfo_old_gate_20261118.rs`，覆盖 v0/v1 的必需对象槽、回引、字段边界和所有前缀截断。
- 主工作树直接运行因其既有 `src/objects/property_set.rs:302` 非穷尽匹配而无法编译：`VariantObjectBranch10_1::Object(_)` 未覆盖。本轮未修改该公共文件。
- 为隔离该工作树错误，将同一专用测试复制到本轮隔离副本 `temp/luna-20261118/dataset-name-isolated/`，运行 `TMPDIR="$PWD/temp/luna-20261118/tmp" cargo test --offline --manifest-path temp/luna-20261118/dataset-name-isolated/Cargo.toml --test luna_fieldinfo_old_gate_20261118`：1 项通过。专用测试文件 `rustfmt --check --edition 2024` 通过。
- 静态反编译和合成测试不等于 ArcMap 运行验证；旧版本实际保存形态和其内部 INumberFormat 实现分布仍未验证。
