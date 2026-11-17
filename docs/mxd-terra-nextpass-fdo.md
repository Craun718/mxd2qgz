# FDO 旧加载器接口槽续查（2026-11-17）

范围是 10.1 `FUN_6b197dc0` 的接口身份和调用目标；字段顺序和版本门沿用 [既有记录](arcmap-10.1-fdo-open-points.md#旧加载器-fun_6b197dc0-字段序版本-03证据确定的反编译)，此处只记新的对位和纠正。静态分析未执行 ArcMap；第 240 行「FDO 旧加载器接口槽业务语义」整体仍未闭合。

## 身份与方法

重新计算 `temp/agent-fdo-open/cab1/AnnoLayer.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` 的 SHA-256 为 `b487f3e0dc0e70dbc4b75f72eb6dcd8d5521eb422dbea6e3b2e201489a2d33a5`；PE Machine `0x14c`、ImageBase `0x6b180000`、固定版本资源 `10.1.0.3035`，与 [既有 10.1 基线](arcmap-10.1-fdo-open-points.md)一致。静态 VA 仅适用于该文件。esriCarto 类型库提取物 `carto.tlb` 重算 SHA-256 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`，与 10.1 介质基线一致；利用既有 `ITypeLib` 类型名/IID 清单，未能重新枚举其方法。GeoDatabase 的既有类型清单所用 `.olb` 和提取类型库重算 SHA-256 分别为 `09b741f2627633d36b67985e175632055fe6c5b8cd1000484acc3007f5917f9d` 和 `92634979daa3ed0008b36d152e1fdf39cba04944f6bc7bbb5ea4c786bb44bb9a`；本轮未独立重新核对其 CAB 来源，该类型名仅作辅助。哈希和版本资源不构成签名认证或运行证明。

复用先前隔离反编译，重新以 PE 字节、objdump 指令及隔离 pyghidra/Ghidra 12.2、JDK 25 反编译核对调用点、查询辅助和目标函数。独立 Ghidra 桥接工具 6.9.0 CLI 指向上述 DLL，超时取消，未取得可用反编译结果；按要求改在仓库 `temp/terra-nextpass-fdo/` 独立 TMPDIR、配置及新项目中用 `GHIDRA_INSTALL_DIR=/opt/ghidra` 的 pyghidra，没有切换共享 Ghidra 桥接工具 目标。独立 Wine 在默认沙箱因套接字 bind 被拒；隔离前缀重试未获得类型库成员输出，不把它当作成功枚举。反编译变量类型和不可达块警告不作为调用约定的唯一证据，关键处按汇编复核。

## 可确认的接收者与实际操作

以下 `O` 是旧加载器收到的 FDO 对象指针，槽偏移均为 x86 字节偏移，字段号对应既有二十字段表。

| 字段 | 新对位与依据 | 目前可说的作用 |
| --- | --- | --- |
| 1–5 | 构造时 `0x6b18dfc9..d5` 在聚合 FeatureLayer 上查询 `{34C20002-4D3C-11D0-92D8-00805F7C28B0}` 存 `O+0xc4`；类型清单为 `ILayer`。`0x6b197e17..77` 和 `0x6b197f5c..88` 分别调用其 +0x10、+0x30、+0x38、+0x20、+0x28。 | 将字符串、两个 short、两个 double 分别传入聚合对象的 ILayer 方法。+0x20/+0x28 发生在字段 8–11 之后，流内 double 顺序不等于立即调用顺序。具体成员名及参数语义未逐槽证实。 |
| 9 | `0x6b197f0b..19` 取的是 `O+0x60` 处虚表并以 **`O+0x60` 本身**作 this，调用 +0x4c；构造 `0x6b18dd00` 将虚表 `0x6b1ab9dc` 写入该处。该槽为 `0x6b18f680`：引用变化时保存输入于 `this+0x110`（即 `O+0x170`）并 AddRef/Release，再转发给 `[this+0x64]`（`O+0xc4`）的 +0x4c。 | 请求的 ISpatialReference 被赋给 FDO 的持有槽并转发 ILayer；不是把接口指针存入 `O+0x60`。不能据此认定投影类型或实际生效状态。 |
| 11 | `0x6b197be0` 中 `0x6b197c1e` 加载 IName，调用 `ConvertToFullPath` 后 `0x6b197cb0..c0` 调该对象 +0x14 得到对象；`0x6b197d2f` 调辅助 `0x6b19e740`，其 QI 常量 `0x6b1ac340` 为 `{D4803EE6-79F4-11D0-97FC-0080C7F79481}`，既有 GeoDatabase 类型清单命名为 IFeatureClass；成功后 `0x6b197d47..55` 调 FDO 的 `O+0x60` 虚表 +0x58（`0x6b190da0`）。 | 名称对象不仅做路径转换：还调用其对象返回槽，查询要素类接口并交给 FDO。该 setter 含类型检查与后续接线；伪代码出现不可达块警告，完整副作用和失败路径仍需逐指令与实现验证。这里并不证明任意 IName 都能打开。 |
| 12–14、17 第一 short | 字段 12 调 `O+0xc8` 的 +0x70；构造 `0x6b18dfe4..f3` 对聚合对象 QI `{40A9E885-5533-11D0-98BE-00805F7CED21}`，即 `IFeatureLayer`。字段 13、14 调 FDO 自身 `O+0x60` 的 +0x60、+0x68，虚表目标 `0x6b1910a0`、`0x6b1910e0` 分别转发 `[this+0x68]` 即 `O+0xc8` 的同号槽。字段 17 第一 short 在 `0x6b198219..24` 调 FDO +0x7c，`0x6b191190` 也转发到 `O+0xc8` 的 +0x7c。 | 这些值最终交给聚合 IFeatureLayer；两个 String 分别对应不同槽，不是旧表写的同一个 +0x20。该 short 单独传入此槽。成员名尚未由类型库方法或具体访问器闭合。 |
| 15、16、17–18、19 | `0x6b1980ea..101` 从 `O+0xc8` 调辅助 `0x6b19def0`；其汇编 `0x6b19deff..08` 明确以 `0x6b1ac470` 的 `{9336A280-C9A3-11D1-847E-0000F875B9C6}` 做 QI，10.1 esriCarto 名为 `IFeatureSelection`。随后的接收者为**查询结果**，而非 `O+0x60`：+0x18 收字段 15；+0x38 收 IFeatureRenderer；+0x28 只收字段 18 的 short（`0x6b19825f..69` 只有一个显式参数压栈）；+0x30 收 IColor。 | 可将这组数据归于聚合图层的要素选择接口及其渲染器、颜色输入；不能仅凭对象 IID 将每个选择参数命名为特定 UI 属性，也不能声称样式或选择行为已验证。 |
| 20 | `0x6b19837e..392` 对 `O+0xc8` 调 `0x6b19e7f0`；`0x6b19e7ff..808` 的 QI 常量 `0x6b1ac360` 为 `{6CA416B1-E160-11D2-9F4E-00C04F6BC78E}`，esriCarto 命名为 `IDataLayer`。`0x6b1983b5..bc` 用其 +0x20 传入 String。 | 版本 ≥3 尾随字符串被交给数据图层接口；具体属性和路径含义未闭合。 |

特别注意旧记录中第 9 项「存 +0x60」及第 13–19 项归属、偏移不准确；第 17、18 项分别调用不同接口，反编译器把残留值显示为双参数而汇编不支持该说法。以上汇编和虚表对位为本轮纠正；本轮不改原文档或读取器。反编译中 `IUnknownVtbl` 指针算术易使 `lpVtbl[8].QueryInterface/Release` 被误读成 +0x20：x86 下分别是 +0x60/+0x68。若要进一步确认各属性，应获取可用的 10.1 类型库成员级 `FUNCDESC`，按 x86 槽、put/get 方向、参数类型与聚合 FeatureLayer 的 QI/虚表/访问器逐一对位，不能把接口名当作成员名。

## 可勾边界与剩余工作

- 可独立落账：旧加载器 ILayer、IFeatureLayer、IFeatureSelection、IDataLayer 的**接收者、查询 IID 和上述槽级数据流**，以及 ISpatialReference 的持有与转发。可独立落账字段 11 的 IName 对象返回 → IFeatureClass QI → FDO +0x58 路径，限定静态调用链及 GeoDatabase 类型名来源。
- 不可整体勾掉当前语义项：ILayer 五个方法、IFeatureLayer 四个槽、IFeatureSelection 四个槽、IDataLayer 尾字段的**成员名及业务作用**未逐槽绑定；字段 6、7 本身的业务含义亦未在此研究。QI 失败、名称对象为空及被调用方法失败时旧加载器的实际结果也需要单独细查，不能从此表推导完整错误处理规则。
- 真实版本 0–3 对象体、版本 4 探测与目标软件运行、ArcMap/QGIS 对照仍需各自证据；本轮没有任何 ArcMap 或 QGIS 人工检查。无旧版本样本字节可据此推定字段值或显示行为。

## 验证及文件

本轮只新增本记录，未修改 Rust、TODO 或既有研究记录。`cargo test --test fdo_legacy --test terra_fdo_legacy` 通过 22 项；是现有合成测试，验证读取器契约，不证明目标软件行为。Cargo 的全局缓存自动清理因权限被拒并发出警告，不影响本次测试。`git diff --check` 通过；该命令不覆盖未跟踪文件。本轮隔离脚本、指令及伪代码位于 `temp/terra-nextpass-fdo/`，属于临时研究输出，分享前应检查本地路径。
