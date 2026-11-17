# 第八轮：Save A Copy 命令与旧档加载状态（2026-11-17）

接续[保存链](mxd-terra-chain-version.md)和[旧档候选写入](mxd-terra-follow-version.md)，这里只记录新增证据；均为静态映像地址，不表示运行时实际调用。未修改 TODO。

## 身份与方法

从本机 Desktop 安装介质提取 `Setup1.cab`，本轮重算 SHA-256 `6833035af901b4b46fe508485cdd5d26534bc27630c512f8bf87c5db417ddfe3`，与[基线](arcmap-10.1-native-baseline.md)一致。重新核对下列 PE，均为 x86、固定版本和 FileVersion/ProductVersion 资源 `10.1.0.3035`：

| 文件 | SHA-256 | OriginalFilename |
| --- | --- | --- |
| ArcMap.exe | `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97` | `ArcMap.exe` |
| MxTools.dll | `d29677c8514e705c6585b1b5a27d4e5cf32f7326891ccfe6ed7beeb70ca51347` | `MxTools.DLL` |
| afUILib.dll | `35d41cf850cbe13c202cca84441cee05db487bc54421ab9d5012dc0b1cad7da4` | `AfUI.DLL` |
| PageLayout.dll | `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f` | `PageLayout.DLL` |

版本资源是文件声明，摘要是字节身份，均不证明签名或运行版本。数据源及工作输出在忽略目录 `temp/terra-round8-version/`，可能包含本机路径，分享前应脱敏。复核 PE RTTI、导入表、跳转表、GUID 字节和 objdump 指令，另用隔离 pyghidra / Ghidra 12.2、JDK 25，在先前 ArcMap 工程的本轮副本中反编译；伪代码不是源码。独立 REA 6.9.0 CLI 使用仓库专用 TMPDIR，任务 `7b387cf4-8f73-458f-8cda-97f5ca07762f` 在导入分析 330 秒后报告 `provider_timeout`，未产出反编译，不将超时误记为 Unix socket 失败；未切换共享 REA。既有 PageLayout REA 导出只作定位辅助。

## 命令到非零参数分支

1. MxTools 的 `0x10230fc8` 是 CLSID `{119591DB-0255-11D2-8D20-080009EE4E51}`，`0x1010f120` 写出该 CLSID。`MxFileMenuItem` RTTI `0x10295b38`、类层次 `0x1024d5b8` 将 `ICommand` 定位到对象 +0x10，其虚表 `0x1023136c` 的 +0x38 指向 `0x1010f300`。同一类的 `ICommandSubType` 虚表 `0x10231350` 的 `0x1010f7d0` 保存子类型；`0x1010f810` 报 13 项。Esri 后版[官方命令清单](https://desktop.arcgis.com/en/arcobjects/10.6/net/858a508c-b771-467c-81e6-c6aa72d88f9c.htm)将相同 CLSID、子类型 12 列为 `File_SaveCopyAs`；该资料标注 10.6，不单独证明 10.1 执行。
2. `0x1010f335..0x1010f363` 对子类型减一后查 13 项跳转表 `0x1010f6fc`；第 12 项确为 `0x1010f635`。该分支 `0x1010f65b..0x1010f671` 依次压入 `0xffffffff`、空指针和接收者，调用接收者虚表 +0x48，即路径为空、short 位模式 `0xffff` 非零。它不是标准 MFC Save As 的零参数路径。虚调用目标取决于实际对象，静态代码本身不证明运行已派发。
3. 该接收者来自 `ICommand` 子对象 +0x14。其 `OnCreate` 跳板 `0x101ebc90` 导入 afUILib 的 `AfMenuItem::OnCreate`，后者 `0x100887e0` 把 `IDispatch` 参数交给 `0x10046dc0`；后者向传入对象 QI `0x10123ce0` 的 `{8E52B9A5-307B-11D2-94C9-080009EEBECB}`，成功接口写入同一 +0x14。旧类型库转储中的 10.1 esriFramework `IApplication` IID 与之相符；转储对应 OLB SHA-256 `ee2b3d51b4b15070620c3d8f845515008e204bee61bc1dd26e77efd8e74444f3`，本轮重算，但没有重新生成转储。
4. ArcMap.exe 中 RTTI complete-object locator `0x556de8` 标明 `App` 的接口子对象偏移 +0x24；类层次描述符 `0x55668c` 将其标为 `IApplication`。对应虚表起点 `0x544024` 的 +0x48 条目 `0x54406c` 为 `0x414f60`。其参数形状和相邻 `0x414cd0` 单路径方法与 Esri 后版 [IApplication.SaveAsDocument](https://desktop.arcgis.com/en/arcobjects/10.7/net/IApplication_SaveAsDocument.htm) 的路径和副本标志一致；方法名是根据接口资料和 10.1 静态槽位的交叉推定，非运行捕获。
5. 连接既有证据：在正常初始化为该 ArcMap `App`、且路径不与当前路径相等的前提下，子类型 12 → `IApplication +0x48` → `0x414f60` → `0x465c40`，空路径到 `0x47cec0` 的非零对话框分支，再按[既有记录](mxd-terra-chain-version.md)的条件将过滤索引写入 AfDoc。相同路径的短路、嵌入状态、对话框取消、版本验证失败与回滚条件各自保留；这条静态链不证明每次点击都会完成保存或状态恢复，也不证明其他实现或外部调用者的行为。

## 旧档加载：候选接口不是已证实的版本 put

`0x476c31..0x476c43` 将 CMxDoc +0x190 成员交给 `0x40a8d0`；该函数复制引用后进入 `0x40afd0`，后者在 `0x40b00c..0x40b014` 对其 QI `{E8E5589D-215A-4322-BA68-E52F187363EE}`。成功时返回该接口；`0x476ca5..0x476cb0` 对其 +0x0c 传入存储、对象流及 `0x47a3f0` 结果。与仅凭相同槽偏移推断版本 put 不同，这是三个显式参数的接口调用。`0x47a3f0` 本轮隔离反编译可见结果默认为 5，按存储状态、Build Number 或 short 分支可能改为 3；伪代码报告删除不可达块，边界及业务含义仍需逐指令和动态复核。

该 IID 的身份、CMxDoc +0x190 运行时对象及其此槽实现仍未闭合。PageLayout.dll 的 QI `0x7b691390` 在比较同一 IID 命中后返回对象 +0x10，另一 QI 路径 `0x7b6a0010..0x7b6a006d` 返回 +0x20，说明不能从 IID 单独确定唯一实现和字段写入。没有对整个加载过程及跨模块间接写入做穷举；不能断言旧档加载后 AfDoc +0x14c 保持 5，也不能断言它被改写。需要绑定实际接收者、逐槽追踪实现，并在 10.1 打开旧档前后观察属性及保存结果。

## 可勾范围与验证

可作为独立静态研究成果落账：**10.1 `MxFileMenuItem` 子类型 12 的命令分支、IApplication 查询及 ArcMap 接收实现到非零保存分支的静态链**；另可落账 **旧档候选调用的 QI IID、接收来源和三参数调用形状**。如果 TODO 的「实际派发」包含运行观测，则整个条目仍不能勾选；旧文档打开后版本索引是否改写仍不可勾选。其他语言资源、运行时实际类型、对话框选择和异常路径仍待验证。

本轮未改变 Rust 解析规则：上述信息涉及应用命令和运行状态，不足以推出新二进制格式规则。`cargo test --offline version::` 16 项通过，`cargo test --offline --test round4_version_object` 两项通过；`git diff --check` 和 `git diff --cached --check` 通过。无 Rust 改动，未重复全量 Rust 测试。ArcMap 运行环境暂缺；未在 ArcMap 或 QGIS 打开、保存、重开或目视检查，不能将静态证据和测试当作人工验收。
