# 共同前缀调用者身份续证（2026-11-17）

## 取证边界与身份

本轮针对 [上轮待查](mxd-terra-next-page.md)的三个地址重新追索持久化虚表，不修改 TODO。`temp/agent-map-surround/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` 重算 SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`；PE32 x86，ImageBase `0x793a0000`，版本资源声明 Esri Map DLL、FileVersion/ProductVersion `10.1.0.3035`、OriginalFilename `Map.DLL`。这是文件自述身份与摘要，不是签名或执行验证。地址均为该映像 VA。

仓库独占 `temp/terra-integrate-page/` 中使用 JDK 25、Ghidra 12.2 / pyghidra 新建工程，重新反编译三个函数及三个跳板，产物为 `0x*.c`，再以 PE 指针、RTTI、objdump 指令和介质 `esri.clsid.ecfg` 交叉核对。未连接或修改共享 Ghidra 桥接工具 服务；这是静态伪代码，不是原始源码或目标程序运行记录。脚本及输出可能包含本机路径，分享前需脱敏。

## 三个调用者：更正 ScaleLine 候选

三个既有候选地址是各自持久化 Load 的跳板目标，**不是 ScaleLine 类的已证入口**。定位方法：从跳板的相对 `jmp` 反查其地址在虚表的 `+0x14` 槽，检查虚表前 COL、COL 的类型描述符及类名，再核对类注册项和类标识。`this` 调整不可省略。

| Load 实体 | 跳板及虚表 | COL → RTTI | 独立类身份 |
| --- | --- | --- | --- |
| `0x793c8990` | `0x793c9820` → `0x79483fb0` | `0x79491324`（位移 `0x144`）→ `0x794b316c`，`ImageInset` | identity 虚表 `0x79483ce4` 的 `+0x14` → `0x793c6920` 写 `{D4F32594-65B5-45DC-BC6F-B3464B1D8373}`，与介质注册项一致 |
| `0x7943b2f0` | `0x7943b780` → `0x79489fa0` | `0x79496de8`（位移 `0x134`）→ `0x794b4b5c`，`MapInset` | `0x79439d50` 写 `{7A3F91E3-B9E3-11D1-8756-0000F8751720}`，与注册项一致 |
| `0x79448500` | `0x79448d50` → `0x7948bb24` | `0x794979e8`（位移 `0x10c`）→ `0x794b4c94`，`Overview` | `0x79448490` 写 `{7A3F91DE-B9E3-11D1-8756-0000F8751720}`，与注册项一致 |

三个跳板均先将接口指针减去其前四字节所指示的动态位移，再跳到实体；具体指令分别在 `0x793c9820..982b`、`0x7943b780..b78b`、`0x79448d50..8d5b`。两个 Inset 的持久化虚表 `+0x0c` 都指向 `0x793c9870`，再跳至写 MapInset CLSID 的 `0x79439d50`；ImageInset 自己的 identity 虚表另有上述 `0x793c6920`。这说明静态虚表中存在共用 GetClassID 实现与独立 identity 两种观察，不能把前者直接当作 ImageInset 的独立类标识，也不能据此推断实际保存时一定使用哪个接口。

## 可勾子项与未完成项

- 可勾独立子项：**确认 `0x793c8990`、`0x7943b2f0`、`0x79448500` 共用前缀调用者的静态类归属**。原条目把这三个地址称为 ScaleLine 族候选，应改为上述类；本结论不证明真正的 ScaleLine 实现不存在或已被定位。
- 装饰物族 Scalebar、MarkerNorthArrow、MapTitle、LegendFormat、LegendItem、Patch 的标量业务属性**未闭合**，原综合条目不能整体勾选。已重新提取 `GraphicElements.dll`（SHA-256 `f24451c8bf25cce7c1906ea67e5fa3ce351b8f9918c6ddcd2a40083151f77ae9`，版本资源 `10.1.0.3035`），也提取 esriCarto 类型库资源（SHA-256 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`）；隔离 Wine 的类型库读取工具启动失败，没有成员名与偏移对位证据，不据类名或数值大小推断属性。该 GraphicElements 复核也不替代上述 Map.dll 族的成员绑定。
- 真正 ScaleLine 的入口、三个类所读标量与对象调用的业务语义、对象流中具体版本与实际使用接口均需另行查证。静态身份不能替代真实 MXD 对象体或运行观察。

未使用 ArcMap 10.1 或 QGIS 人工打开、目视检查，也未执行 ArcMap；无实机授权环境，不能把静态分析视为运行验收。本轮仅改研究记录与源码注释，没有改变解析规则或新增构造测试。

`cargo fmt --check` 通过；`cargo test` 在并行修改的 `src/maps/map_object.rs:999`、`:1013` 因 `PostIlluminationFields10_1` 与 `Option<ObjectSlot10_1>` 返回类型不匹配而编译失败，未运行测试。本轮不修改该文件。源码注释的 `git diff --check` 通过。
