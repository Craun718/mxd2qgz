# ArcMap 10.1 研究成果使用指南

后续 MXD 逆向研究优先使用本机目录 `/home/gxrs/文档/source/arcmap10.1/` 中已有的成果，再按需要使用 Ghidra、pyghidra 或 objdump 补充静态分析，或进行目标软件运行观察。该目录不在本仓库中，其他环境需要自行提供；其内容是反编译伪代码、索引和研究材料，不是原始源码，也不是完整的 MXD 解析器。

## 检索流程

1. 先读该目录的 `README.md` 和 `已有对象流分析.md`，结合仓库内相关研究记录确定模块、版本和候选函数。
2. 按模块查 `functions.tsv` 中的函数名、地址和状态，再读该模块以地址命名的 `.c`；查 `calls.tsv` 追踪已恢复的直接调用。`AfCore/` 使用 `decompile-index.json`、`decompile-journal.jsonl` 和 `functions/`，索引格式不同。
3. 查模块日志中的 `EXPORT_DONE` 和索引中的失败状态；目录可能正在增长，进程退出或文件存在不能证明该模块全部导出成功。目标缺失或失败时，可针对原始二进制补充分析。
4. 根据 `binaries/manifest.json` 与原文件核对 SHA-256，结合 PE 版本资源、文件身份和已有 [10.1 原生文件基线](arcmap-10.1-native-baseline.md) 确认研究对象。哈希确认字节身份，版本资源是声明，均不等于签名认证或运行验证。
5. 在研究记录中注明模块、函数地址、使用的成果和版本、交叉核对方法、推断与未解决问题；关键类型、分支和间接调用需要时用原始指令、虚表及其他证据核对。

## 范围与限制

- `AfCore` 涵盖通用流和对象持久化；`SystemCore` 涵盖对象流与引用表；`PageLayout` 涵盖文档和布局；`Map`、`MapLib`、`MappingCoreLib`、`FeatureLayer`、`RasterLayer`、`DisplayCore`、`DisplayLib` 分别提供地图、图层和显示相关候选函数。具体职责以函数证据为准。
- `binaries/` 的清单还列有 `MappingCore`、`MxTools`、`ArcMap`、`Geometry` 和 `GdbCore` 等文件；清单存在不代表对应模块伪代码已导出。2026-11-17 检查时 `MappingCore` 日志记录空间不足导致导入失败，`MappingCoreLib` 尚未看到 `EXPORT_DONE`；后续以实际日志和索引为准。其余已出现完成标记的模块也有个别函数导出失败。AfCore 的既有导出及限制见 [对象体记录](arcmap-10.1-object-body.md)。
- `calls.tsv` 仅包含 Ghidra 已恢复的直接调用；COM 类创建、虚函数、跨模块调用及插件依赖可能不在其中。伪代码的类型、控制流和函数边界可能有误，不能据此断言完整调用链或格式。
- 静态研究与构造测试不能代替 ArcMap 10.1 的运行取证，也不能代替 ArcMap 和 QGIS 的人工目视验收；未执行的验证须明确标注。
- 原始二进制、样本和诊断可能含敏感路径；分享前检查脱敏，遵循项目对本地研究材料的处理约定。
