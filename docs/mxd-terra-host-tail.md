# Map 变换循环出口（ArcMap 10.1 静态取证）

2026-11-17。接续 [Map 变换循环](mxd-terra-chain-map.md)。所有地址是映像 VA；静态分析和辅助样本均不等于 ArcMap 运行。

## 身份与方法

本轮重算安装介质提取的 `Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` SHA-256 为 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`；PE machine 为 `0x14c`，版本资源 FileVersion / ProductVersion 为 `10.1.0.3035`，InternalName 为 `Map`，OriginalFilename 为 `Map.DLL`。摘要和资源声明不证明签名真实性。基于前轮隔离 Ghidra 12.2 / JDK 25 的 Load `0x793ff4d0`、Save `0x79401850` 反编译，本轮重新读取原 PE 字节、反汇编 `0x794002c3..0x794003df`，并解码两个 IID。新证据在仓库内 `temp/terra-host-tail/`；没有切换共享 REA，也未运行目标软件。已有伪代码足以定位本段，本轮无需重启 REA 分析。原生函数地址为静态证据，不是源代码。

## 循环之后的顺序

| 序 | Load 原生分支 | 普通 Save 顺序 | 当前实现 |
| --- | --- | --- | --- |
| 1 | `0x794002c3..0x79400300`，版本 >= 13：读 8 字节后调用接口 | 变换循环后取值并写 8 字节 | 保留 double 原始位 |
| 2 | `0x79400301..0x7940032a`，版本 >= 15：读 2 字节临时值 | 取得接口值并写 2 字节 | 保留原始位 |
| 3 | `0x7940032b..0x7940035d`，版本 >= 16：`LoadObject`，IID `{E4240309-DD41-11D2-B868-00600802E603}`，对应已有研究中的 IBackground | 写 `+0x280` 的对象 | 仅消费空引用 |
| 4 | `0x794005ed` 后版本 > 22：`LoadObject`，IID `{E265C5B5-4513-4BB9-B280-27B03AAE5957}` | 普通新版分支写 `+0x254` 的对象；另一保存分支写 `+0x258` | 消费空引用及已完整解析的 BasicOverposterProperties 新对象 |

旧记录 [Map 对象](arcmap-10.1-map-object.md) 将第四项 IID 的 `4BB9` 写为 `4BB4`；本轮按 `0x79488050` 的 16 字节重新解码，前者为原 PE 结果。这里不从 IID 猜测接口名。Load 版本 19..22 在第四项之前走另一旧对象分支；当前读取器仅允许 23..=51，故不读取旧分支。Save 的版本选择及后续字段不能据此推定全部历史版本对称。字段业务含义和非空对象体均未闭合。

实现按字段提交：短读停在该字段起点；两个对象槽遇非空或短读均不消费、不登记引用键。第二槽的 BasicOverposterProperties 新对象及颜色体现已可整体消费；不支持的非空对象、未闭合彩色体或截断仍在第二槽起点停止。第二槽之后仍停止，不宣称 Map 完整。既有变换循环内部的具体类分支未改动。

## 样本与可勾范围

参考样本从根和键表连续读取到 `/Maps:0x2524f`；其后 double 的原始位对应 5000，short 为 0，`0x25259..0x25269` 为空背景槽；下一对象在 `0x25269` 起非空；本轮续查并消费该对象后，读取器在 `0x252be` 停止，保留 687 字节。

可独立勾选的研究缺口：**确认 Map 10.1 支持版本中变换循环后的 double、short、背景对象槽、下一对象槽的 Load 顺序及普通新版 Save 对位**。可单独记录的实现成果：**保留前两个原始标量、消费两个空槽并对短读及非空槽安全停止**。非空背景、非空下一对象、字段业务语义、后续版本字段和完整 Map 尾部仍开放；若现有条目与这些缺口合并，不能整体勾选。本轮未修改 TODO。

## 验证与限制

构造测试覆盖四个字段的每个短读位置、原始位、两个空槽和两个非空槽回退，并核对引用表键集合；参考样本从根验证实际停止点。`cargo fmt --check`、完整 `cargo test --offline`（758 通过、11 忽略）、定向 Map 测试及 `git diff --check` 通过；按 README 执行 `cargo run --offline -- parse`，报告未知尾部从 `0x252be` 起 687 字节。未在 ArcMap 10.1 或 QGIS 打开源工程或转换结果，未运行 ArcMap，不能以静态反编译或构造测试代替运行取证与目视验收。

## 续查：BasicOverposterProperties

根据 `temp/agent-fl-tail/ecfg/esri.clsid.xml`（SHA-256 `71179ed661d9baa9366d603eb561141711ab0520b10c3c3a506a3f9003803e01`），`{E0C73D56-6C88-498C-B77C-78606E688C97}` 注册为 BasicOverposterProperties。用户提供的 esriCarto 10.1 类型库枚举结果为 type 289 IOverposterProperties、type 305 BasicOverposterProperties；本轮另从 LabelPlacement.dll QI 和构造函数交叉确认 `0x7857b28c` 的 IID `{E265C5B5-4513-4BB9-B280-27B03AAE5957}` 与该对象的接口子对象对应。类型库提取物 `temp/terra-integrate-page/carto.tlb` SHA-256 为 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`；本轮未重新枚举类型库。

LabelPlacement.dll 的 SHA-256 `8415f63c1889e269ba6ae585538f601130c20cef02e4556e875a52217d35ea3d`，PE x86、映像基址 `0x78530000`，资源 FileVersion / ProductVersion `10.1.0.3035`，InternalName `LabelPlacement`、OriginalFilename `LabelPlacement.dll`。这些是字节身份与版本声明，不是签名或运行验证。独立 REA 6.9.0 CLI、JDK 25 和仓库专用 TMPDIR 的运行 `ef60a148-4f12-41be-b901-2e398b872d01` 遇 UNIX socket `Operation not permitted`，未取得反编译证据；随后复制既有 LabelPlacement 工程到本轮目录，以隔离 pyghidra/Ghidra 12.2、JDK 25 和仓库内临时目录重新读函数，未切换共享 REA。

- CLSID 常量 `0x7857b2bc` 被 GetClassID `0x7853e700` 输出；构造函数 `0x7853e880` 的虚基表确定持久化接口虚表 `0x7857b254`。QI `0x78570d10` 对 IPersist / IPersistStream 的 IID 返回该虚基子对象；虚表 Load thunk `0x7853f3a0` 跳到 `0x7853f0e0`，Save thunk `0x7853f350` 跳到 `0x7853f1b0`。
- Load 首先读 2 字节版本，有符号大于 2 拒绝；版本 2 再依次读 2 字节、`LoadObject` 请求 IColor `{20CD40B0-8F3D-11D0-8590-0800091A2A72}`、2 字节和 8 字节 double。版本 0、1 不读取后续体字段。Save 在选定保存版本后写相同顺序；保存版本选择及历史版本含义仍开放。读取器安全地只接受无符号 0..=2，比原生有符号比较更严格。
- 样本对象头 `0x25269`、键 2180；体从 `0x2527d` 开始，版本 2，内嵌 RgbColor 键 2181，体结束 `0x252be`。颜色槽复用既有五类颜色读取器；非空未知宿主类、未知颜色、非法版本或任何截断均回退到宿主对象头并撤销其键和内嵌键。颜色回引遵循现有引用表模型，不能独立核对被引对象类。

可独立勾选 **BasicOverposterProperties 在 10.1 的 CLSID、IPersistStream 读写槽及版本 2 字段序静态对位**，以及 **Map 后续槽对该类已支持颜色的有界消费与失败回退**。其他对象类、引用目标类身份校验、业务属性、Save 版本策略、后续 Map 尾部及实机验证仍开放。
