# 第八轮：TableName 的跨模块持久化与 SDK 样本

日期：2026-11-17。静态原生证据与读取器结果均不是 ArcMap 执行观察；ArcMap
10.1 和 QGIS 未作人工目视验证。本轮限定独立表及此记录，没有修改 TODO 或
`terra_round8_old` 测试。

## 目标身份和方法

- SDK Java `LinearRef.mxd`：SHA-256
  `5eb1835d15610b6b56c853153360476400f6d55cf0693dfb72305ded3ce2715d`。
  介质提取与 `/Version` 9.2/14 见 [旧档记录](mxd-terra-round8-old.md)；
  文件保存版本不等于应用 DLL 版本。
- GdbCore.dll：PE32 x86，ImageBase `0x708d0000`，SHA-256
  `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`；
  GdbCoreLib.dll：PE32 x86，ImageBase `0x70b90000`，SHA-256
  `10e0af85e3e3825647733f99d573c969d69ccaf47203365e51323fce07008baa`。
  两者文件资源均声明 FileVersion/ProductVersion 10.1.0.3035、原名分别为
  GdbCore.DLL/GdbCoreLib.DLL。这是静态字节身份及版本声明，不是签名认证。
- GdbCore 以 JDK 25、`/opt/ghidra` 和仓库内独立临时目录对既有 Ghidra 工程
  副本进行 pyghidra 反编译；用 PE 导入/导出表和 objdump 指令核对跨 DLL
  跳板及字段序。未切换共享 REA 会话。临时产物在
  `temp/terra-round8-tablename/`；避免分享其中可能含本机路径的诊断数据。

## 旧结论的明确更正

[旧记录](arcmap-10.1-dataset-name-slots.md)称 TableName 没有 GetClassID 写出、
GdbCore 内没有流持久化并据此推断新对象无法加载；该结论对 TableName **不成立**。
`0x709b94b0` 是析构入口，不是构造入口。旧扫描没有正确跟踪类族虚表的跨 DLL
导入跳板：没有直接调用 AfStreamRead 并不表示没有持久化。本次结论只针对
TableName，不据此推断 UnknownDatasetName 或 ObjectClassName 的行为。

## 原生类、接口与读写链

1. GdbCore 注册代码 `0x70a8084e` 将 TableName CLSID 常量地址 `0x70ac318c`
   放入注册记录，`0x70a8086e` 安装工厂虚表 `0x70ac2208`；该工厂的创建槽
   `0x7090a240` 进入构造 `0x709b9330`。这与相邻 ObjectClassName 的注册记录
   不应混为一条。
2. 构造 `0x709b9394` 调 GdbCoreLib 的 `FdbDatasetName` 构造，随后完整对象
   +0x18 安装虚表 `0x70ad076c`。QI `0x709b96b9..0x709b96db` 比较标准
   IPersistStream `{00000109-0000-0000-C000-000000000046}` 和 IPersist
   `{0000010C-0000-0000-C000-000000000046}`，都跳到 `0x709b98dc` 返回完整
   对象 +0x18；同一 QI 的 `0x709b9647..0x709b9664` 将 IName 映射到 +0x10。
   这是静态接口路径，未观察实际创建或 QI 执行。
3. +0x18 虚表的 +0x0c 为 `0x709b9d40`，两次 MOVQ 写入
   `{06783DB1-E5EE-11D1-B0A2-0000F8780820}`；+0x14 为跳板 `0x70a63662`，
   `jmp [0x70a8759c]`，导入名 `FdbDatasetName::Load(IStream*)`；+0x18 为
   `0x70a63668`，跳至导入的 `FdbDatasetName::Save(IStream*, int)`。
   GdbCoreLib 对应导出 VA 为 `0x70d4e980` 和 `0x70d4ea10`。跳板和导入表
   共同建立跨模块绑定，而非只按函数名或样本猜体。
4. GdbCoreLib Load 检查流指针，读取 2 字节版本；`0x70d4e9ab..b0` 有符号比较，
   大于 1 拒绝。接着对持久化接口 this+4、+8、+0xc 依次调用 AfCore
   `String::Read`，然后 `LoadObject` 请求 IID
   `{FADD975C-E36F-11D1-AA81-00C04FA33A15}`（IWorkspaceName）。Save 写固定
   short 1、对应三次 `String::Write`、然后 `SaveObject`。字符串各自的业务
   名称本轮未确认。`LoadObject` 结果的原生错误折叠和底层短读行为不由本读取器模拟。

## 实现与样本边界

`src/maps/standalone_table.rs` 对两个 IName 槽新增 TableName 类分派：版本、
三字符串、IWorkspaceName 空槽/回引/新 WorkspaceName；未知工作空间对象在其
头前停止且不登记该键，WorkspaceName 内部停止则向外传递。错误时游标和引用表
整体回滚，资源限额沿用调用者值。读取器接受 0..=1 并拒绝负值位模式，较
原生有符号版本门更严；不模拟 QI、对象创建或访问实际数据源。当前只接入
StandaloneTable 名称槽，不能推广为所有 IDatasetName 宿主均支持 TableName。

独立测试从 `/Maps` 根开始、共享引用表读取，按已有读取器到达首个表；其中部分
上游未知体采用了已有有界跳过，所以该宿主链仍需原生运行验证。表头
`0x6a50` 键 331、版本 9；首 IName 新对象头 `0x6a66` 为 TableName 键 332，
体从 `0x6a7a` 开始，版本 1；随后实际读取新 WorkspaceName 键 333，未遇其
内部停止点。继续到首个命名对象的新对象头 `0x6bed`（键 334）并在该头前停止；
表本身和第二个表尚未完整消费。测试不输出数据源字符串或本机路径。

## 验证与可勾范围

新增独立测试覆盖空工作空间、已有键回引、未知工作空间头前停止、版本超限、
字符串限额和逐字节截断回滚；SDK 样本测试单独显式启用。`cargo fmt --check`、
`cargo test` 与独立样本测试的具体执行结果以本轮最终报告为准。保留已有旧档
测试，其依赖原停止点的忽略用例需要主线整合时同步更新。

可单独落账：TableName 的 **10.1 GdbCore → GdbCoreLib IPersistStream 读写链**，
以及 StandaloneTable 中上述可安全读取的对象体分派。不能勾选非空独立表完整
解析、其他 TableName 宿主、UnknownDatasetName/ObjectClassName，或 ArcMap/QGIS
人工验收。IPersistVariant 在其他聚合/调用情形是否介入、不同版本运行兼容性
仍未由本次静态取证排除或验证。
