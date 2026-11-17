# 10.1 标注表达式引擎续研（2026-11-17）

本轮只覆盖 VBScript、JScript、Python 三个已登记的标注表达式引擎，不能据此穷尽其他实现类。这里的 DLL、注册表和反编译均是静态证据；没有运行 ArcMap，也没有使用 JScript/Python 的真实 MXD 对照样本。

## 文件身份与方法

- LabelPlacement.dll 来自既有 10.1 安装介质提取物（来源见 [10.1 基线](arcmap-10.1-spatial-reference.md)），本轮重算 SHA-256 为 `8415f63c1889e269ba6ae585538f601130c20cef02e4556e875a52217d35ea3d`；PE x86，ImageBase `0x78530000`。本轮用 pefile 读取资源：FileVersion/ProductVersion 均为 `10.1.0.3035`，InternalName `LabelPlacement`，OriginalFilename `LabelPlacement.dll`。身份声明与哈希不等于签名验证。
- esri.clsid 既有提取文件 `temp/agent-fl-tail/ecfg/esri.clsid.xml` 分别登记下表三类。类型库和前期范围见 [续研](mxd-terra-next-label.md)、[第三轮](mxd-round3-labels-featurelayer-20261117.md)。
- 独立 REA 6.9.0 CLI / Ghidra 12.2 对同一哈希启动分析成功，但桥接 UNIX socket 在沙箱中报 `Operation not permitted`，未取得函数 Evidence ID，原始失败信息位于忽略目录 `temp/terra-depth-label/js-getclass.json`。随后使用隔离 pyghidra/Ghidra 12.2、JDK 25、仓库内独占 TMPDIR 和新工程，对同一 DLL 反编译，结果在 `temp/terra-depth-label/0x*.c`；未切换共享 REA 目标。首次 pyghidra 因配置目录只读、第二次因 X11 不可访问失败；隔离 XDG 目录并使用 headless 后成功。结合 PE RTTI/COL、虚表原始字节和 objdump 汇编交叉检查；临时输出可能含本机路径，分享前需脱敏。

## 类、接口及持久化归属

| 类 | 注册 CLSID | RTTI TypeDescriptor | 主虚表 | +0x18 子对象虚表 | GetClassID |
| --- | --- | --- | --- | --- | --- |
| AnnotationVBScriptEngine | `{AA157207-E079-11D2-9F48-00C04F6BC6A5}` | `0x7858db40` | `0x7857a194` | `0x7857a200` | `0x78535ae0` |
| AnnotationJScriptEngine | `{AA157208-E079-11D2-9F48-00C04F6BC6A5}` | `0x7858db68` | `0x78579ebc` | `0x78579f28` | `0x785356e0` |
| AnnotationPythonEngine | `{BB6721A2-D81A-45E3-7FEF-884DB2B2A905}` | `0x7858db90` | `0x78579fcc` | `0x7857a038` | `0x78535790` |

COL 将每对虚表分别归属同一 RTTI，子对象偏移为 `0x18`。三个主虚表的第一槽均指向 `0x78535c80`：该 QueryInterface 比较 `0x7857a248` 中 `{AA157206-E079-11D2-9F48-00C04F6BC6A5}`，命中后返回完整对象 +0x10；比较 `0x7857958c`、`0x7857959c` 的 IPersist `{0000010C-0000-0000-C000-000000000046}`、IPersistStream `{00000109-0000-0000-C000-000000000046}` 时返回 +0x18。因此这三个类的静态实现支持两个宿主请求的同一 `IAnnotationExpressionEngine`，且该子对象为持久化接口。LELP 的请求见 [标注记录](mxd-label-luna.md)，DisplayExpressionProperties 的请求见 [DEP 记录](arcmap-10.1-dep-tail.md)。

子虚表 +0x0c 是表中各 GetClassID，JScript 和 Python 反编译分别拷贝 `0x78579f70`、`0x7857a070` 的 16 字节，恰与上表注册 CLSID 一致；+0x14、+0x18 对三类均分别为同一 Load `0x785361d0` 和 Save `0x78536240`。Load 读取有符号版本 short，拒绝大于 2；版本为 2 时再读一个 short 至持久化子对象 +0x0c；Save 写版本常量 2 和该 short。既有读取器对负版本额外拒绝，比静态原生有符号上限检查更严格。short 语义、实际表达式求值及其他引擎仍未知；静态接口支持不证明目标软件实际写出过相应槽。

## 实施边界

- `annotate_props.rs` 新增两类 CLSID 和统一体读取接口，以种类保留类身份并复用共享 Load 布局；LELP 和 DisplayExpressionProperties 的 `IAnnotationExpressionEngine` 槽均接入两类。原 VBScript 枚举分支保留兼容；其他未知类仍在槽前停止并回退登记。`mxd_maps.rs` 仅增加对应诊断分支以正确显示类名，不把其他并行修改归于本轮。
- LELP 解析后可继续读取 Envelope 槽及其后版本字段。DEP 的引擎槽就是其 Load 的最后一项，所谓后续字段属于外层 FeatureLayer 而非 DEP；测试用哨兵证明读取器停在宿主后继字节前，不声称已对新引擎完成整条外层真实样本验证。
- 引擎的短读、未知 CLSID、非法版本维持事务回退；仍不解释 short、表达式语义，也不声称其余表达式引擎全部支持。未修改 TODO。

## 验证

`tests/terra_depth_label.rs` 使用构造数据覆盖三类版本 0/1/2、两种新类在两个宿主槽中的分派与类身份、LELP 后续标量、DEP 后继哨兵、未知类回退和引擎体截断回滚。全仓 `cargo fmt --check` 和 `cargo test` 通过，后者含既有忽略项；Cargo 提示全局缓存自动清理因权限不足失败，但构建和测试成功。Rust 测试不是 ArcMap/QGIS 运行或目视验收。
