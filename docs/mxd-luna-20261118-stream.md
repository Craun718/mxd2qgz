# stream 组：持久化对象流与 VariantStream

## 研究范围与环境

本记录按本轮接手时 `TODO.md` 的行号列项。目标为 ArcMap 10.1，所有结论均为静态分析或合成输入结果；没有运行 ArcMap/QGIS，也没有用旧 MXD 的存在性或字符串命中代替结构证据。

- SystemCore 证据来自仓库外的 ArcMap 10.1 原生反编译材料：ObjectStream 对象读取器 `0x7ffcff70`、写出器 `0x7ffcf7e0`、SetStream `0x7ffcf7a0`、写表插入器 `0x7ffd0950`。调用链和指令复核见 `docs/arcmap-10.1-object-body.md` 与本机反编译记录《已有对象流分析》；相关结论限定于 ArcMap 10.1 的 SystemCore 实现。
- AfCore 二进制身份按现有基线核对：10.1.0.3035，SHA-256 `031858c4…cc47c17`。REA 6.9.0 CLI 首次请求 `0x68a7b530` 时启动 Ghidra 12.2 全量导入，90 秒限时结束于 `cancelled`；临时会话与 Ghidra 日志保留在 `temp/luna-20261118/rea-tmp/afcore-rea/`。随后 PyGhidra 3.2.0 / Ghidra 12.2 对同一二进制新建独立项目、完成分析，并定向导出 `0x68a7b530`、`0x68a7af70`、`0x68a7c1d0`、`0x68a7c240`、`0x68a7c0f0`、`0x68a7c160`；输出日志和脚本在 `temp/luna-20261118/pyghidra/`。这是伪代码而非原始源码，Ghidra 的缺失类型信息和 COM 间接槽位仍需谨慎解释。
- AfCore `SaveObject` 导出 `0x68a7b640` 调用 `StrmUtil::SimpleSaveObject` `0x68a7af70`。新鲜 PyGhidra 结果确认普通 `IPersistStream` 分支在可选头部后把传入流原样交给 `Save`，该分支本身不写 ObjectStream 键；`0x68a7b530` 展示 Variant、临时流及包装对象的多条分支，并递归调用 `SimpleSaveObject`。辅助 QI 函数的类型、接口槽位与每条异常/清理路径没有全部由独立原始指令重建，因此本轮不把局部委派提升为完整通用链闭合。

## TODO 原行号状态

| 原行号 | 状态 | 证据、实现与边界 |
| --- | --- | --- |
| 25：AfCore SaveObject/LoadObject 对象头分派 | 已静态闭合 | 后续原始指令复核确认两个导出先查询流的 IObjectStream，分别委派到 +0x40/+0x44。普通流保存回退到 SimpleSaveObject，只处理 GUID 和对象体。运行验证另见第 40 项。[读写委派证据](mxd-simple-object-dispatch-20261118.md#导出入口) |
| 31：纸张创建分支的 CLSID | 已纠正 | 后续复核确认 `0x7b70e819` 将 `0x7b7207ec` 压栈，随后调用创建函数。该地址是 esriOutput Paper 的 CLSID。原记录中的 `558c4e63` 为同一地址的误记。[原始字节复核](arcmap-10.1-native-baseline.md#纸张创建分支的-clsid-勘误) |
| 37：动态验证对象读取错误恢复 EH 展开 | 待验证 | 静态失败路径记录在 `docs/arcmap-10.1-object-body.md`；本轮未运行 ArcMap，也未获得运行期异常观察。 |
| 40：在 ArcMap 10.1 中运行验证 ObjectStream 调用链 | 待验证 | 未执行目标软件运行取证。静态 Ghidra/PyGhidra 证据不能代替此项。 |
| 124：跨流复用 ObjectStream 的对象体 | 部分 | ObjectStream 实例经 `SetStream 0x7ffcf7a0` 更换底流时不清引用表；`0x7ffcff70` 在读完 16 字节类 GUID 与 4 字节键后，登记新对象再调用对象类的 `Load(ObjectStream)` 或 `Load(VariantStream)`。体没有外层长度，结束位置由各类 `Load` 决定。仓库的 `object_refs` 只分类头并登记键，类体读取器在已支持的类上分别接续；未知/未实现类无法通用跳过，故不宣称跨流任意对象体闭合。 |
| 125：跨流复用 ObjectStream 的版本适用性 | 部分 | 10.1 的跨流共享由 SystemCore SetStream 与对象表证据闭合；10.8 既有对照记录支持相同的头/回引状态模型和两种持久化分支。Rust 入口仍显式区分 `read_header_10_1` 与 `read_header_10_8`。这只支持已确认的对象头/引用机制，不外推每个类对象体的版本布局或未知包装流。 |
| 128：VariantStream 对象分支的对象体 | 部分 | AfCore `VariantStream::Read 0x68a7bc50` 是向底层 IStream 的薄转发；10.1 全局 Variant 读路径的 9/13 分支进入 `LoadObject`。标准 ObjectStream 包装上下文可到带键对象读取器，但对象体随后由类的 Variant 持久化方法消费且无通用长度。当前 `variant_stream` 对非空对象明确返回 `Unsupported`，不吞掉未知对象体；仅空对象子集接受。 |
| 129：VariantStream 对象分支引用表 | 部分 | 标准 10.1 VariantStream 包装由 ObjectStream 创建，传入其对象上下文，引用表沿用同一 ObjectStream 会话；证据见 `0x7ffcff70` 的 VariantStream 构造/Load 调用序列及对象流包装分析。仓库 `RefTable10_1` 已支持跨独立 Reader 保持键状态，`object_refs` 测试覆盖跨流回引。非标准 VariantStream 包装、真实 MXD 对象键顺序与非空体的字段归属未在本轮验证。 |
| 131：SimpleSaveObject 委派链 | 部分 | `0x68a7b640` 到 `0x68a7af70` 的直接调用已确认；PyGhidra 新鲜反编译补查 `0x68a7b530` 及 QI 辅助 `0x68a7c1d0/0x68a7c240/0x68a7c0f0/0x68a7c160`，确认有普通持久化、Variant 值和临时 VariantStream 包装分支。辅助 QI 的 IID/槽位调用链及所有失败清理尚未逐条用指令闭合，故保持部分。REA 首次全量导入超时的日志保留，不把它当成成功反编译。 |
| 141：触发并验证重复键替换分支 | 待验证 | SystemCore 写表插入器 `0x7ffd0950` 对同键节点有覆盖分支；正常读路径先查 `0x7ffd0a50`，命中时直接 QI 并返回回引，不走插入器。没有找到能触发写侧同键替换及核对其与后续回引交互的受控运行证据。Rust 读表不是写表，也不保存对象指针；已修正其注释，避免把“重复登记计数”说成“替换”。 |
| 302：PropertySet 版本 0 分支 | 可勾选（静态规则及合成测试） | SystemCore `PropertySet::Load 0x7ffd3080` 对版本只做有符号 `>1` 拒绝，之后无版本分流；v0 与 v1 读取相同的计数和键值条目。`read_property_set_10_1` 已接受 0，并拒绝大于 1；模块测试覆盖空 v0 与截断版本，`tests/property_set_v0_independent.rs` 覆盖非空 v0、尾随宿主字节和所有体前缀截断。Save 恒写 v1，因此没有 v0 样本/运行证据；这项闭合限于安全读取子集，不声称实际旧 MXD 已验证。 |

## 修改与验证

- 修改 `src/storage/object_refs.rs`：明确读侧已见键命中为回引、不调用登记，写侧替换是另一表/另一插入路径；并为只含可比较键状态的登记表派生 `PartialEq/Eq`，以支持跨解析步骤的状态回滚断言。
- PropertySet v0 的实现及独立合成测试在本轮开始前已存在，未重复新增等价测试。
- `cargo fmt --check` 通过；`cargo test --lib` 的 186 项通过，`cargo test --test property_set_v0_independent` 的 1 项通过。全量 `cargo test` 中 `tests/fdo_legacy.rs` 的两个现有/并行测试断言失败，分别为 `associated_layer_unsupported_rolls_back_to_header` 与 `associated_layer_feature_layer_stops_inside_its_body`：实际引用表状态为 `None`，期望为 `Some(NewObject { … })`。失败断言不在本组写入范围，未改该测试。Cargo 提示全局 registry cache 的自动清理因权限拒绝而跳过；没有清理全局缓存。
- 全局 `/tmp` 容量不足时，普通沙箱命令启动失败；REA/PyGhidra 与 Cargo 的临时文件均改用仓库内隔离目录，没有清理 `/tmp`。
- ArcMap 10.1 运行取证、真实旧 MXD 的 PropertySet v0 结构佐证、QGIS 打开检查均未执行。
