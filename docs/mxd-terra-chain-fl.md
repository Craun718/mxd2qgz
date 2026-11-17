# FeatureLayer 包络槽下游接口链（2026-11-17）

承接 [包络槽续查](mxd-terra-follow-feature.md) 的原 TODO 296。本文仅为静态研究，地址为 x86 映像 VA、成员偏移基于 FeatureLayer Load 的第一个显式参数；不代表 ASLR 运行地址。本轮未修改 TODO 或解析器，未运行 ArcMap、QGIS，未进行人工目视验收。

## 身份与方法

本轮重新计算并用 pefile 检查提取文件：FeatureLayer.dll 的 Machine 为 `0x14c`，FileVersion `10.1.0.3035`、InternalName `FeatureLayer`、OriginalFilename `FeatureLayer.DLL`，SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`；ArcMap.exe 为 `0x14c`，版本 `10.1.0.3035`、InternalName `ArcMap`、OriginalFilename `ArcMap.exe`，SHA-256 `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97`。与仓库 10.1 基线一致；版本资源是声明、哈希是字节身份，均不证明签名或运行。

独立 REA 6.9.0 CLI 使用 Ghidra 12.2、JDK 25 和仓库专用 TMPDIR 对 FeatureLayer.dll 发起反编译；导入和分析后桥接 Unix socket 报 `Operation not permitted`，未产生函数 Evidence。未切换共享 REA 目标。另用隔离 pyghidra/Ghidra 12.2、JDK 25 独占项目分析 ArcMap.exe，并以 objdump、PE 原始字节交叉核查。分析物及失败诊断位于忽略目录 `temp/terra-chain-fl/`，可能含本机路径，分享前应检查。伪代码不是源码，以下直接控制流以机器指令为准。

## 修正第二级 IID

`0x6f705d00` 使用 `0x6f7ae424`，原始 16 字节 `7d 85 0a 72 a9 45 bf 4d aa 06 73 71 5c 65 4b 01`，COM GUID 小端字段解码为 `{720A857D-45A9-4DBF-AA06-73715C654B01}`。

`0x6f705db0` 使用紧邻的 `0x6f7ae434`，原始字节 `61 62 e8 89 2d 2b a7 45 be 01 64 19 0d bc 9b db`，解码为 **`{89E86261-2B2D-45A7-BE01-64190DBC9BDB}`**。此前 [包络槽续查](mxd-terra-follow-feature.md) 将第二字段误写为 `2D2B`；该记录中的第二级 IID 应以本文为准。原始字节与 ArcMap.exe `0x552048` 一致。两个 IID 的正式接口名、继承声明和实现类仍未知，不能仅凭 GUID 邻接推断。

## 可确认的调用关系

1. FeatureLayer Load `0x6f6ea8ae..0x6f6ea8ce` 对 `+0x1f4` 预释放/清零，调用 LoadObject 请求 IEnvelope；失败进入错误路径。
2. 成功后 `+0x130` 非空时，先对其查询第一级 IID，检查结果非空，再从该接口查询修正后的第二级 IID；`0x6f6ea93a..0x6f6ea94b` 将 `+0x1f4` 指针传入第二级接口虚表字节偏移 `+0x10`，调用后 `0x6f6ea94d..0x6f6ea953` 对该接口执行 `Release`。两次 QI 的中间助手对失败 HRESULT 返回空指针；此处第二次 QI 后没有空指针检查。首个接口已 QI 成功不构成第二个接口一定成功的静态证明，实际可达性及异常处理须运行验证。
3. 本轮在同版 ArcMap.exe 发现独立使用该修正 IID：`0x507b80` 的 QI 助手由 `0x5030f0` 调用；其调用点 `0x4ffd34..0x4ffd77` 在取得非空接口后，将初始化为零的 16 位局部变量地址传给同接口虚表 `+0x14`，随后按该 short 是否为零分支。这提供了第二个方法槽和调用形态，但尚不足以命名该方法、解释 short 的业务意义或证明与 FeatureLayer 调用的是同一具体对象实例。
4. 同版 Map.dll `0x794671d9` 和 MappingCoreLib.dll `0x79bbef69` 也将第一级 IID 传给 QI；这两处是查询助手，不是接口定义或实现方法。MappingCoreLib.dll 哈希 `c2bf81dcfcb65473ce35b953c4f4b76392a594085460c299326cc20bed59986a`，Map.dll 哈希 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`；两者版本资源均声明 `10.1.0.3035`、x86，InternalName 和 OriginalFilename 与模块相符。

对本机 `temp/arcgis`、`temp/arcgis101-bin`、`temp/agent-fl-tail2`、`temp/mxd101-wine`、`temp/terra-depth-legend` 中大小不超过 60 MB、以 MZ/MSFT 开头或扩展名为 .olb/.tlb 的 3365 个文件做 GUID 字节有界扫描；修正后的第二级 IID 命中 FeatureLayer.dll 和 ArcMap.exe，第一级还命中上述 Map.dll 和 MappingCoreLib.dll 的部分本地副本。扫描含重复文件和非目标文件，并非全部安装介质；无命中不证明实现不存在，接口也可能用拆分比较、重定位或其他模块实现。

## 结论与后续

新证据将第二级 IID 修正并定位 ArcMap 中额外的 `+0x14` 调用，但仍未找到两个接口的正式名称、`+0x10` 方法声明或承接该方法的具体类和实现。现阶段只可称 `+0x1f4` 为传入该方法的持久化 IEnvelope，不能认定为实际绘制范围、缓存范围或数据范围；也不能从额外 short 分支给 Envelope 赋名。这不改变已确认的文件字段布局，不实施解析器语义变更。

可单独落账的已完成研究动作：**10.1 FeatureLayer 第二级 QI 的 IID 原始字节纠错，并交叉定位 ArcMap 同 IID 的另一方法槽调用**。原 TODO 296 的业务语义与具体实现仍未闭合，不可勾选；尚需找到能识别 IID 的 10.1 类型信息或 QI 实现，追踪其虚表 `+0x10` 目标及成员读写，并在可用环境中对可控图层、视图和数据源进行 ArcMap 10.1 运行观察。任何静态推断、合成测试或扫描结果均不能替代运行及 ArcMap/QGIS 目视验收。
