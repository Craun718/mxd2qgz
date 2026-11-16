# MXD DatasetName：FgdbFeatureClassName 取证（2026-11-16）

## TODO 行 170：FgdbFeatureClassName 体

**状态：实现已接入，静态字段体已闭合；样本越过旧停止点，目标软件目视验证未做。** 本轮纠正了虚表槽位转录错误，按真实 Load/Save 及其内嵌对象构造证据实现解析与事务测试。

### 10.1 原生身份与虚表定位

- 检查对象来自 ArcGIS 10.1 安装介质 `Setup1.cab` 的 FileGDB.dll，版本 `10.1.0.3035`、PE x86、SHA-256 `73cef6bac97b0bdba8e4b26e2052cefab0b55a32e3a4141c01701d7cbc5f1230`。提取文件：`temp/agent-dataset-name/filegdb/FileGDB.dll.687DEE04_CE68_4C5E_9861_D1987DAF6242`。注册映射及 10.1 类型库均确认 CLSID `{75E10086-4226-42AC-AFEC-CBB9B748F847}` 对应 `FgdbFeatureClassName`。
- 初始误读把虚表字节 `90 32 a3 6f` 错录为地址 `0x6fa32a90`。原始连续读取 `0x6fafe0c0` 的 64 字节（Evidence `ev_c9f5c83308bdac7d2e2eccea8c72eac40e842712b6d7e894dbf8ecce9aaf2cf7`）表明：`0x6fafe0c0` 存储 COL 指针 `0x6fb0c6a8`，实际虚表从 `0x6fafe0c4` 起；小端逐项解码为 QI `0x6faa1f60`、AddRef `0x6faa1f40`、Release `0x6fa31e60`、GetClassID `0x6fa31600`、IsDirty `0x6fa31c80`、Load `0x6fa33290`、Save `0x6fa31ca0`。COL 原始读取 `0x6fb0c6a8`（Evidence `ev_fd8e9f6ca7e589f82b6e0509e04ad002c39634c934f5e1802582ada3d1615287`）确认次级基类偏移 `0x18`、TypeDescriptor `0x6fb40314`、ClassHierarchyDescriptor `0x6fb0c70c`；RTTI 名为 `.?AVFgdbFeatureClassName@@`。
- 数据交叉引用与槽位一致：GetClassID 函数 `0x6fa31600` 的 xrefs Evidence `ev_569ce286133688869938e3a2b98f22a297feea10e59affe74544b9048d165e15` 指向 `0x6fafe060` 和 `0x6fafe0d0`；后一地址正是虚表 `+0x0c`。Save `0x6fa31ca0` 的 xrefs Evidence `ev_b81dd07ee357df32ac9150b335b104a7382ee05c1f996859d9045049c781bff1` 指向 `0x6fafe0dc`，正是 `+0x18`。Load 的函数分析 Evidence `ev_261988028c2f25d2380c20549a9d0632faac6c22c96e71442a0490b7e58d9d78` 给出完整函数边界 `0x6fa33290..0x6fa3330a`，并确认该入口是有效函数，而非构造函数内部地址。

### Load/Save 字段序与内嵌对象

- Load `0x6fa33290`：检查非空 IStream；读 2 字节版本；对有符号 short 做 `version > 1` 拒绝；读 `this+0x10` 的 2 字节字段；随后调用 `this+0x0c` 所指对象的虚表 `+0x14` Load，传同一个 stream。这里的 `this` 是偏移 `0x18` 的 IPersistStream 子接口地址，因此两个成员对应完整对象偏移 `+0x28` 与 `+0x24`。
- Save `0x6fa31ca0`（分析 Evidence `ev_9a324df723cb81f91fa8ba652e009f8e8bc24b8c733c4b1852b3c285118a7b0a`）顺序相反对应：写常量 short `1`；写 `this+0x10` 的 short；委托 `this+0x0c` 对象虚表 `+0x18` Save。常量地址 `0x6fafa5e0` 的读取 Evidence `ev_28a4c2128bd8a2297d370d990c19322d530dea3616d354dee1ebd02518d2a507` 为 `01 00 ...`，确认写出值 1。
- `FgdbFeatureClassName` 构造函数 `0x6fa31640`（Evidence `ev_dc00ae9a7662238fecb098aca7a574e00fa059e6c773cd0ca40c876989942911`）创建内嵌对象 CLSID `DAT_6fafe14c`，在完整对象偏移 `+0x24` 保存指针，并查询 IID `DAT_6faf9048`。原始 IID 字节 Evidence `ev_f031070684ea1d5b489a3b16ffaf1474997e4aa8942960fa0851fb5c4b705649` 为标准 `IPersistStream` IID；原始 CLSID 字节 Evidence `ev_67b16e2c8ee72548913052e8bc46cb821390cf33eaec290b15c92a8814803d36` 为 `{198846D0-CA42-11D1-AA7C-00C04FA33A15}`，即 GdbCore `FeatureClassName`。因此外层字段之后直接委托同流读取既有 GdbCore FeatureClassName 体，不存在第二个对象头。
- 由此闭合体序为：`i16 outer_version(原生拒绝 >1)` → `i16 outer_unknown(+0x28)` → 内嵌 GdbCore `FeatureClassName::Load` 的完整已确认字段体（字段序、嵌套槽及其停止语义见 `src/layers/feature_layer.rs` 模块文档）。短字段的业务语义仍未知，按位保留。

### 实现、回滚与验证

- 在 `src/layers/feature_layer.rs` 新增 `FGDB_FEATURE_CLASS_NAME_CLASS_ID`、`FgdbFeatureClassNameHeader10_1` 和 `read_fgdb_feature_class_name_body_10_1`。解析器以 trial Reader / 暂存 RefTable 处理外层与内嵌体，仅成功后提交。为了兼容现有对 `DataSource10_1` 的穷尽匹配，保持枚举形状不变：结果复用 `FeatureClassName` 分支，`FeatureClassNamePrefix10_1::fgdb_wrapper` 标识并保留外层字段。
- FeatureLayer 数据源按精确 CLSID 分派到 Fgdb 包装解析器，嵌套 FeatureClassName 的已知停止原因继续传播到 FeatureLayer。
- 新增四项单元测试：完整外层+内嵌体；非法外层版本不推进游标/不改引用表；嵌套 WorkspaceName 截断时游标和新键登记回滚；FeatureLayer Fgdb 分派及内层停止传播。
- 验证：`cargo fmt --check` 通过；`cargo test` 全部通过（库 167 项，集成测试通过；1 项需 QGIS API 的测试按仓库配置忽略）。`cargo run -- parse 'samples/测试底图.mxd'` 正常完成；原先的未知起点/停止位置 `0x1622` 推进到 `/Maps` 偏移 `0x2c01`，剩余 141,676 字节。样本在 ArcMap 10.1 / QGIS 中未人工目视验证；当前更深处的具体字段语义或对象边界不由字符串扫描推断。
- 更新后的全量 REA Evidence 导出：`temp/agent-dataset-name/mxd-open-dataset-evidence-20261116.json`（77 条记录，0 unknowns）。目标会话保持打开，没有调用 close/reset。

### 剩余边界与主代理接线

- `src/objects/mod.rs` 无需新增模块声明；实现放在已注册的 `feature_layer` 模块中。`src/maps/mxd_maps.rs` 未修改，保留现有 `DataSource10_1` 枚举/API。
- 10.1 原生字段体已闭合到内嵌 FeatureClassName 的已知停止语义；样本当前深处停止处仍有 141,676 字节未知尾部，不能把其余数据视为已解析。主代理可审查共享文件改动及偏移 `0x2c01` 的结果，再决定是否记录相应 TODO 完成。

### 主代理最终复核

主代理复读 Load 与构造器 Evidence，核对字段宽度、接口基址偏移及内嵌类标识；保留外层 short 的原始值。补入 robustness 第 138 个入口，覆盖合法包装体的逐前缀截断与重复失败游标契约，并补测负版本拒绝。整库 665 项通过、1 项按既有配置忽略。参考样本在 `0x2c01` 的委派层状态为 `Successor(TailCompleted)`：该 FeatureLayer 体已完整消费，后续尚未接线的是 FDO 第 7–15 步，不是未知 Fgdb 体。相关诊断与 TODO 已同步。
