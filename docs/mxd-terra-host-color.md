# 10.1 色带与图片填充持久化字段续查（2026-11-17）

范围仅为 RandomColorRamp 的读即弃 int32、PictureFillSymbol 的两个读即弃 double 和 MultiPartColorRamp 权重数组及版本门。结论为静态路径，不能代替 ArcMap 运行。对照 [G 轮](arcmap-10.1-display-slots-round3.md) 和 [Display 续查](mxd-terra-next-display.md)，本文更正其中 PictureFill 两字段「Save 恒写 0.0」及 Random 字段「直接读入对象槽」的说法。

## 身份和方法

重新计算 `temp/agent-fl-tail/DisplayCore.dll.33A02325_2485_4623_B2CE_71B5736B51DF` 的 SHA-256 为 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`，与既有基线一致；PE32 x86，ImageBase `0x6eb30000`，版本资源 FileVersion/ProductVersion `10.1.0.3035`、InternalName `DisplayCore`、OriginalFilename `DisplayCore.dll`。版本资源是声明，哈希标识字节，并非签名或运行证明。

在仓库专用 `temp/terra-host-color/` 的 TMPDIR、配置及缓存中，以独立 REA 6.9.0 CLI、Ghidra 12.2、JDK 25 取得 Random Load 的 Evidence `ev_ae3ca61e0fd052e0ef424c2a654e8d08ddd4dc55b0ef824411195d25b962dfa1`；新核对原 DLL 的虚表、常量及 objdump 指令。没有切换共享 REA 目标。类型库使用既有介质提取的 esriDisplay TYPELIB（SHA-256 `6052021b77c05ddb6b37b5373a04ada586a6664a121733ddf0651297106229ab`，见 [类型库身份记录](mxd-terra-follow-display.md)），在独立 Wine 前缀中枚举；Wine 是元数据读取，不是 ArcMap 执行。64 位 Wine 的 `oVft` 除二对位 x86 虚槽；成员名输出正常；本次只用于选定接口，不主张完整枚举类型库。地址均为静态映像 VA。分析文件可能含本机路径，分享前须脱敏。

## RandomColorRamp：请求 Size 的持久化槽

`0x6eb3bc90` 对 IColorRamp IID `0x6ec0b524` 匹配后返回对象基址 O+0x18；构造函数 `0x6eb3ba60` 将虚表 `0x6ec0c4d4` 安装到 O+0x18。类型库的 IColorRamp 槽 +0x14 是 `put_Size`；对应虚表项 `0x6eb3bd30` 写接口 this+8，即 O+0x20。其 `get_Size` 槽 `0x6eb3adf0` 读取 this+0x2c，即 O+0x44，**并非同一成员**；不能把下面的清零直接解释为 getter 一定返回零。

持久化接口位于 O+0x54（COL 偏移），Load `0x6eb3c190` 在 `0x6eb3c210..0x6eb3c237` 将流内 int32 读入栈局部，读成功后将 P−0x34 = O+0x20 清零；Save `0x6eb3c3bc..0x6eb3c3cd` 取同一槽写出。此流字段与 Size **setter 所写的请求值**对位；加载成功后该对象槽清零，流值本身不回填。代码继续保留原始 int32，不能把 setter 或 getter 的取值语义简化成流值。创建色带、生成后的 getter、失败分支和宿主重设仍需运行验证。

## PictureFillSymbol：Save 的图片尺寸与 Load 丢弃

版本 8 Load `0x6eb714af..0x6eb714d2` 两次各读 8 字节到栈局部，并在成功路径上继续读其他字段，没有将这两个局部值存入对应对象成员；读取器保留位模式。

Save `0x6eb71746..0x6eb71764` 先将两个局部 double 清零，再把它们的地址传给 `0x6eb73580`；`0x6eb71879..0x6eb7189a` 将这两个局部写入流。旧版 Save `0x6eb748d1..0x6eb7490c` 也调用同一函数后写两个值。**初始化为零不等于恒写零**：函数 `0x6eb73580` 经图片接口虚槽 +0x18/+0x1c 获取宽高，普通路径按 `尺寸 / 2540.0 × 72.0` 写出；位图尺寸超过阈值时有取像素尺寸、上限 128、按 `尺寸 / 96.0 × 72.0` 换算的分支（`0x6eb735d3..0x6eb736a7`）。这是持久化时计算的图片宽高，单位换算与点数相符；并非已证明的显示宽高属性或历史来源。失败路径可保留初始化值，Save 调用点未检查该函数返回值。尚未将它们与某个 PictureFill setter 的直接持久化成员绑定，不能据相似类型赋名为 XScale/YScale。

## MultiPartColorRamp：版本门控制子渐变权重

QI `0x6eb389a0` 将 IColorRamp/IMultiPartColorRamp 返回 O+0x18；类型库成员和 `0x6ec0c124` 虚表对应 `CreateRamp=0x6eb38be0`、`AddRamp=0x6eb39370`。Load `0x6eb3969f..0x6eb396c4` 每读一个子对象，就给配套 double 数组追加 `0x6ec18998` 的 1.0；仅对象版本≥2 在 `0x6eb396d2..0x6eb396f9` 以流数据覆盖该数组。Save `0x6eb39720` 的文档版本≥4 时写 count 个 double；对应版本表前五档 `[1,1,1,1,2]`，更高版本的截断规则见 [保存侧记录](arcmap-10.1-save-side.md)。

该数组不是图层数或任意填充：`CreateRamp` 在 `0x6eb38d63..0x6eb38d9f` 以元素、数组合计及 Size 请求值计算各子渐变分配数量，然后在 `0x6eb38e05..0x6eb38e2a` 对子渐变调用 Size setter 和 CreateRamp。`AddRamp` `0x6eb3942b..0x6eb3945c` 根据已有权重均值或 1.0 追加权重。故可定性为参与颜色数分配的**子渐变权重**；不声称其完整校验、边界算法或所有极端 double 行为。版本 0/1 对已加载子项默认 1.0；版本≥2 保留流值。新增 `loaded_weight_bits` 仅在解析完整时给出相应位模式，未知子对象导致停止时不构造默认结果。

## 可勾边界与后续验证

可独立落账的静态子项：Random 的 Size setter／Save 槽与读后清零关系；PictureFill 两字段的图片尺寸计算来源及读即弃关系（包含纠正恒零结论）；MultiPart 数组作为分配权重、旧版本默认值与读写版本门的关系。它们不构成原 TODO 整项、所有属性语义或软件运行验收的闭合，本轮未改 TODO。

`cargo fmt --check` 和完整 `cargo test` 通过；构造测试覆盖旧版默认权重、版本 2 非默认权重、停止与越界，以及图片尺寸非零位模式保留。构造测试只检查读取器边界及值保留，不证明原生可达性。ArcMap 10.1 环境暂缺：未打开、保存、重开或检查图层和图片显示；QGIS 也未进行目视验收。需后续在原生宿主中分别设置 Size、图片及权重，比较保存字节和重开结果，覆盖空图片、不同类型和失败路径。
