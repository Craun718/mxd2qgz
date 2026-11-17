# 第三轮 Display / Spatial 定点研究

日期：2026-11-17。编号为 `mxd-round2-needs-20261117.md` 第三个表第一列的原 ID。本轮只更正 `ole_font.rs` 的解码分派注释，未修改解析行为、TextSymbol 或共享 TODO。静态分析不是 ArcMap/QGIS 运行和显示验证。

## 身份、方法和边界

- DisplayCore.dll：Setup1.cab 提取物，PE x86，文件版本资源 10.1.0.3035，SHA-256 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`。
- Geometry.dll：同介质，PE x86，文件版本资源 10.1.0.3035，SHA-256 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`。复用 Ghidra 桥接工具 全函数伪代码，分析配置摘要 `af4d30c9d76866092ce084421f113cb34393a9703542b46a5c290a493c77a8c4`；`0x7197fc50` 先前 Evidence `ev_cbecb6b16ba35e765699cadc9a8d83b8d7fc563278c3bb35194cf7153fa72304`。
- esriGeometry.olb：既有介质提取物 SHA-256 `d45fb2105c869cda034fa80101cf2d0b932e08ea6f31eca8d4322fe7f7cd3e8e`；提取的类型库资源 SHA-256 `341646ec9ef602e82aa75072a116bd3468a6b023475b827dbd418481e97d9409`。用 Wine `LoadTypeLibEx` 和 `ITypeInfo::GetVarDesc` 输出枚举成员；接口和枚举元数据非运行行为。
- 介质 OLEAUT32：PE x86，文件版本资源 2.40.4275.1，SHA-256 `baeb2f7c1b8be56738d34e1d1ddf8e0eebd3a633215dc1575e14656be38b939d`；这是介质分发件，不是已经核验的 ArcMap 10.1 目标系统运行库。
- 新 Ghidra 桥接工具 CLI 6.9.0 / Ghidra 12.2 独立进程定点反编译，未更改共享目标：ArrowMarker Load `ev_d1d90cc9ea36b4907a7721ac0299419f3a3d9ad10a695170b436f0cd97abb596`；OLE Load `ev_af151d9f6677e00845aa730bc7436e73f78328e21cd3606ad4ba745d55d68d09`；OLE SaveHeader `ev_6ac39a425f185ffc2b5c2576b1b15f0452458df87c8d5c5bbe9bdbe0c5635b4a`。原始输出和类型库 dump 只在 `temp/round3-display-spatial/`；地址均为相应文件的静态映像地址。第一次 Ghidra 桥接工具 因 `/tmp` 无空间失败，重试改用仓库临时目录；本地 Unix 套接字在沙箱内被拒后使用独立授权进程。

## 233：记录类型和公开几何类型

类型库的 `esriShapeType` 与 `esriGeometryType` 是两套不同枚举。结合 Geometry 解析器已观察的类型分支，可建立下表的有限对应；这是类型名称与解析分支的静态对照，不等于证明所有输入都合法或导出器一一转换。

| 记录 shape 基本类型及变体 | 对应几何概念 | 公开 esriGeometryType |
| --- | --- | --- |
| 0 Null | 空 shape | 0 Null；宿主对象仍决定几何类 |
| 1 Point、9 PointZ、11 PointZM、21 PointM、52 GeneralPoint | 点 | 1 Point |
| 8 Multipoint、18 MultipointZM、20 MultipointZ、28 MultipointM、53 GeneralMultipoint | 多点 | 2 Multipoint |
| 3 Polyline、10 PolylineZ、13 PolylineZM、23 PolylineM、50 GeneralPolyline | 折线 | 3 Polyline |
| 5 Polygon、15 PolygonZM、19 PolygonZ、25 PolygonM、51 GeneralPolygon | 面 | 4 Polygon |
| 31 MultiPatchM、32 MultiPatch、54 GeneralMultiPatch | MultiPatch | 9 MultiPatch |

尤其记录值 5 是 Polygon，而公开枚举 5 是 Envelope；记录值 9 是 PointZ，而公开枚举 9 是 MultiPatch。General 类的高位标志还由 `esriShapeModifiers` 指定：Z `0x80000000`、M `0x40000000`、曲线 `0x20000000`、ID `0x10000000`、基本类型掩码 `0xff`。已有 Point/Multipoint 解析器根据自身分支处理，不应把表格用于绕过原有校验。`0x7197fc50` 读取记录值，0 走空记录并按宿主回填 3/5；其余非零值并非在该处逐一验证。完整的 Geometry 导入/导出组合和 MultiPatch 调用尚未逐项核验。主审复核后按原待办范围闭合记录类型所属枚举及其与公开类别的对照；不把类型库对照视为运行支持。

## 270：StdPicture 头与分派

介质 2.40 SaveHeader `0x653a1071` 写 4 字节整数 `0x0000746c`，所以第二个 WORD 写零；`0x653a0191` 循环读 `0x746c` 后读第二个 WORD，再读 DWORD 长度，未见第二字校验或作为分支条件，但不能推断它的设计名称或跨系统含义。长度为零时置空图并返回；非零时再次读取首 WORD 并分派，因而不可把 `0x0000` 魔数分支称为空图。直接复核的**小端数值**：BMP `0x4d42` → `0x6539f482`；GIF `0x4947`/`0x6967` → `0x653a3162`（目标 `0x6537a0c8`）；JPEG `0xd8ff` → 同一入口（目标 `0x6537a0d8`）；`0xcdd7` → `0x6539f8ad`；`0x0001` → `0x6539f7dc`；`0x0000` → `0x653a0137`；其他值返回 `0x800a01e1`。JPEG 头字节 `ff d8` 作为小端 WORD 是 `0xd8ff`，GIF 头字节 `47 49` 是 `0x4947`。旧 `arcmap-10.1-ole-stdfont.md` 和原源码注释写作 `0xFFD8`/`0x4749`，前者保留给文档所有者修正；本轮已更正后者。本解析器保留载荷，不实现或验证图像解码，且未知类型并不按原生拒绝。

读取伪代码的调用约定及部分栈变量恢复不完整；第二字实际功能、三种未定名分派器完整语义、目标 Windows 运行库的差异以及 JPEG/GIF/图元文件受控样本仍缺，故 PARTIAL。上一轮真实 BMP 局部样本仅证明第二字为零及其长度边界，不能推出其他类型；详见 `mxd-round2-symbol-ole-20261117.md`。

## 原 ID 状态及交接

| ID | 状态 | 证据与还缺什么 |
| --- | --- | --- |
| 210 | PARTIAL | `arcmap-10.1-display-symbols-round2.md`、`arcmap-10.1-display-tails.md` 已记录颜色/符号读写字段和偏移；未逐类把类型库 getter/setter 绑定到成员槽，缺少全量业务语义。继续做接口槽、调整器和属性访问器对照，无法定性的用受控运行验证。 |
| 212 | PARTIAL | 新 Ghidra 桥接工具 ArrowMarker `0x6eb3ec40`：版本≥2 读 short 至持久化接口 this−0xc4，成功后无条件调用 DisplayLib `Symbol::GetDefaultRotateWithTransformValue(this−0x11c,-1)` 并覆盖同槽；版本<2 不读但同样赋默认。既有记录中 CharacterMarker `0x6eb9b150` 版本>1 的 this−0x10c、PictureMarker v9 `0x6eb7a650` 的 this−0x134 及旧版本 `0x6eb7d750` 的对象+0x70 均记录同型；另有 SimpleMarker `0x6ebbefa0` 的独立同型证据。保存值及属性对应、DisplayLib 默认函数实现和历史意图尚未闭合；尤其不能假定返回值总为固定布尔值。须取得并核验 DisplayLib 10.1 后反编译默认函数、属性 getter/setter，必要时运行受控存取。 |
| 217 | PARTIAL | `element_body.rs` 文档给出 TextSymbol Load `0x6ebd0a60` 字段顺序和成员偏移，字体槽有既有 StdFont 静态及样本证据；尚未将每个标量绑定相应属性。请主代理修改 `element_body`，本轮不改该文件；需 TextSymbol 各属性访问器与偏移逐个对照。 |
| 233 | CLOSED（类型域及类别对照） | 10.1 类型库明确两枚举及基础类别，已知 Ghidra 桥接工具 分支互证。全部导入/导出分支对变体的接受行为未验证。 |
| 239 | BLOCKED | `{97D5B914-FB22-11D1-94A2-080009EEBECB}` 未定名；`mxd-iid-research-20261116.md` 的 79 类型库未命中和 FeatureLayer 该模块字面搜索未命中均为有界负证据，不能证明不存在。需包含该接口声明的其他类型库/二进制或 10.1 实际 QI 与调用链。 |
| 243 | PARTIAL | `arcmap-10.1-pcs-geometry-slots.md` 已有 PCS/GCS/Unknown 读写及 Unknown 第二串被第三串覆盖、ISRPersistStream slot 8 恒写 `0x38`、GCS 对象+0xac 非零拒绝等静态事实；域串/四串的名称和填充、slot 名称、守卫业务含义及 pe/sg 容差数值推导尚缺完整访问器与调用链。 |
| 244 | PARTIAL | 同版本 DisplayCore RandomColorRamp Load `0x6eb3c190` 读 int32 后置零、Save `0x6eb3c350` 写槽；PictureFill Load `0x6eb71110` 两 double 读后不用、Save `0x6eb71580` 写零；MultiPart 的 Load≥2 与 Save docVer≥4 的门依版本表对应。依据 `arcmap-10.1-display-slots-round3.md`；这些观察尚不解释字段原用途或历史兼容意图，需访问器、调用链或受控对象。 |
| 270 | PARTIAL | 新 Ghidra 桥接工具 验证读写头及魔数分派，纠正两处小端常数与空图描述；第二字命名、未定名解码分支和目标系统实际运行库仍未闭合，详见上节。 |

## 验证

`cargo fmt --check`、`cargo test` 均通过；额外单独运行 display_color、display_symbol、geometry、ole_font 四个集成测试共 75 项通过。Cargo 报全局缓存自动清理权限警告，不影响测试。本轮改动仅注释，未新增运行时行为；未在 ArcMap 10.1 或 QGIS 中人工打开并目视验证，也没有声称图像解码或转换结果已经验证。
