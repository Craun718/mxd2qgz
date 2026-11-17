# DisplayCore 与 OLE 指定条目复核（2026-11-17）

范围为接手时 TODO.md 第 216、218、223、250、269–270、276–277 行；未修改 TODO 或公共入口。静态反编译及局部试读不是目标程序运行或显示验收。

## 身份和取证条件

README 的目标为 ArcGIS Desktop 10.1.3035。本轮重新核对 `temp/agent-fl-tail/DisplayCore.dll.33A02325_2485_4623_B2CE_71B5736B51DF`：PE32 x86，版本资源 FileVersion/ProductVersion `10.1.0.3035`、OriginalFilename `DisplayCore.dll`，SHA-256 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`，与既有记录一致。本地 ArcMap 10.1 提取目录及 `temp/pseudocode/` 的完整反编译导出不含 DisplayCore；采用已有 Ghidra（经桥接工具） 证据和对该文件的独立字节、指令复核。OLE 介质库身份与静态证据见 [第三轮](mxd-round3-display-spatial-20261117.md) 和 [第四轮](mxd-round4-display-20261117.md)。本轮没有运行新的 Ghidra 桥接工具 会话：`/tmp` 已满，无法确认共享 Ghidra 桥接工具 服务的临时目录与并行目标隔离；未冒险启动 Ghidra。

## 分项结果

| 行 | 状态 | 本轮结论和未完成条件 |
| --- | --- | --- |
| 216 | 推进，未关闭 | [颜色与符号原生读写](arcmap-10.1-display-symbols-round2.md)及[第四轮](mxd-round4-display-20261117.md)已限定字段类型、成员去向。RandomColorRamp 布尔短整使用全部 16 位判断。其余大量字段与属性访问器尚未逐项绑定，颜色分量和符号属性不能凭类名或样本值定名。需将类型库接口槽、调整 thunk、getter/setter 与对象成员对位。 |
| 218 | 推进，未关闭 | 已存 Ghidra 桥接工具 Evidence `ev_d1d90cc9ea36b4907a7721ac0299419f3a3d9ad10a695170b436f0cd97abb596` 显示 ArrowMarker `0x6eb3ec40` 的版本≥2 输入 short 被读至相应槽，成功后调用 `Symbol::GetDefaultRotateWithTransformValue(this−0x11c, -1)` 并覆盖；本轮 objdump 复核 `0x6eb3ede2..0x6eb3edf7` 调用及写回 `this−0xc4`。CharacterMarker 与 PictureMarker 同类路径见 [第二轮](arcmap-10.1-display-symbols-round2.md)及[第三轮](mxd-round3-display-spatial-20261117.md)。这是成功路径的静态读取即弃，历史流值的业务语义、默认函数实现及属性关联仍未知，不能仅以函数名声称业务语义已闭合。 |
| 223 | 推进，未关闭 | TextSymbol `0x6ebd0a60` 的版本门、对象槽和标量存储位置见 [字段序](arcmap-10.1-element-bodies.md#textsymbolload-0x6ebd0a60-字段序证据确定的样本逐字节互证)，正确虚表归属见 [复核](arcmap-10.1-display-symbols-round2.md)。颜色、文本路径、字体槽由请求 IID 定名；其余标量尚未逐个与属性访问器绑定，不能依据样本值定名。 |
| 250 | 推进，未关闭 | [G 轮](arcmap-10.1-display-slots-round3.md)记录 RandomColorRamp 的 int32 读后清零、PictureFill 两个 double 读后弃用且写零。MultiPart Load 以对象版本≥2 决定读取数组，Save 以 docVer≥4 决定写入；本轮从 DisplayCore 文件 VA `0x6ec0b324` 重读十字节 `01 00 01 00 01 00 01 00 02 00`，即前五档 `[1,1,1,1,2]`，因此这五档的对象版本和写入门相配；对更高 docVer 的末项截断规则见 [Save 版本记录](arcmap-10.1-save-side.md)。此配套关系不能解释旧字段用途及设计意图。PictureFill 版本表 VA `0x6ec0b2ac` 复核为 `[3,3,3,4,7,8]`。 |
| 269 | 推进，未关闭 | Africa 两个不同磁盘版本模板的 MultiLayerMarker、CharacterMarker 和其嵌套 SimpleFill/Line 仅为局部引用表试读，见 [样本记录](mxd-symbol-ole-20261117-followup.md)。其余符号类及上层宿主路径仍未全覆盖，不能以 GUID 扫描次数表示对象数或关闭全族。 |
| 270 | 推进，未关闭 | RgbColor 存在样本对象级证据；HSV/CMYK 在有限介质流中存在 GUID 候选，但未从宿主路径确认，Gray/HLS 同样缺对象级证据。见 [第四轮](mxd-round4-display-20261117.md)。候选命中不等同可解析对象。 |
| 276 | 推进，未关闭 | 既存介质 OLE 2.40 Ghidra 桥接工具 Evidence `ev_af151d9f6677e00845aa730bc7436e73f78328e21cd3606ad4ba745d55d68d09` 与 `ev_6ac39a425f185ffc2b5c2576b1b15f0452458df87c8d5c5bbe9bdbe0c5635b4a` 支持 SaveHeader 写 `0x0000746c`、Load 读取第二字后未见分支检查；图像首 WORD 的 BMP/GIF/JPEG 等分派及未定名分支详见 [第三轮](mxd-round3-display-spatial-20261117.md)。旧版 Africa 模板局部 BMP 的第二字为零；未建立其宿主链、第二字业务名称或跨运行库行为。 |
| 277 | 阻塞 | 已知介质 oleaut32 为 2.40.4275.1，并非确认的 ArcMap 目标 Windows 运行库；缺身份确认的其他目标 Windows 库及双向受控运行。Wine DLL 不能证明 Windows 版本兼容。静态结构和样本均不能证明跨版本稳定。 |

## 验证和交接

本轮在仓库独占 `temp/terra-next-display-work` 设置 `TMPDIR` 后，`cargo fmt --check` 通过；`cargo test --test round2_symbol_ole_media -- --ignored` 两项通过，覆盖两份介质局部符号体、局部 BMP 读取及部分截断和限额。测试不证明宿主路径、解码器运行或显示效果。本轮未修改 Rust 代码，未执行全量 `cargo test`；也未在 ArcMap 10.1 或 QGIS 打开工程进行人工检查。

未修改其他代理的 `src/objects/ole_font.rs` 脏改动或其他已有改动；无公共入口接线。本轮未获得足够证据支持新业务字段名或新解析规则，因此不建议主代理勾选上述完整条目。若把 218 拆为独立的「10.1 成功路径静态读取后覆盖」子项，该子事实可关闭，但历史字段语义仍须保留。

## 第 218 行补充：RotateWithTransform 的属性和默认值（2026-11-17）

从 10.1 Desktop `Setup1.cab` 提取 `DisplayLib.dll.33A02325_2485_4623_B2CE_71B5736B51DF`：PE x86、ImageBase `0x6ed60000`；版本资源 FileVersion/ProductVersion `10.1.0.3035`、InternalName `DisplayLib`、OriginalFilename `DisplayLib.dll`；SHA-256 `e16c4c2610f20896005cf7ae56965dd60a61530b27daa3d566682f6ac11982c6`。为文件身份声明及摘要，不是签名或运行验收。DisplayCore 文件身份见上节。

Ghidra 桥接工具 6.9.0 CLI 独立进程在仓库专用 `temp/terra-next-display-work/tmp` 设置 `TMPDIR`、`JAVA_TOOL_OPTIONS`；默认 Java 21 因版本不符被拒绝，显式使用 JDK 25 后 Ghidra 12.2 因 Unix socket bind `Operation not permitted` 未完成（原始输出分别为 `rea-default.json`、`rea-default-jdk25.json`，均在该独占临时目录）。据 [Basemap 第三轮](mxd-round3-basemap-20261117.md) 的降级方式，改用隔离 pyghidra/Ghidra 12.2 项目反编译三个导出函数，并用 objdump、PE 导出表、导入表、数据字节交叉核对；伪代码是反编译产物，不是原始源码或运行证据。局部脚本和输出留在该目录，可能含本机路径，分享前检查。

1. DisplayLib 导出 `Symbol::GetDefaultRotateWithTransformValue(short)` 为 `0x6ed96410`。调用 `CRegKey::Open`，根句柄 `0x80000001`（HKEY_CURRENT_USER），子键 `SoftWare\\ESRI\\Symbology\\Settings`，权限参数 `0xf003f`；成功后 `CRegKey::QueryValue` 读取 `RotateWithTransform`。若 Open 或 QueryValue 返回非零，返回入参 short；成功则将读出的数值零映射为 `0`、非零映射为 `-1`。汇编 `0x6ed96434..0x6ed96479` 与伪代码互证。此判断不能证明目标机器注册表的实际值，且申请的权限可能影响能否打开键。
2. `Symbol` 无参构造函数 `0x6ed94d10` 在 Symbol 基址 `+0x08` 安装虚表 `0x6edb7c14`，该虚表 `+0x0c/+0x10` 指向 `get_RotateWithTransform` `0x6ed953b0` 和 `put_RotateWithTransform` `0x6ed95330`。它们分别读取、写入接口 this `+0x50`，即 Symbol 基址 `+0x58`；构造函数也初始化该位置。由实际虚表条目及成员偏移，可将这一槽定名为 RotateWithTransform，非仅凭默认函数名称推断。
3. 三类成功 Load 均以 `-1` 调用默认函数：Arrow `0x6eb3ede2..0x6eb3edf7`，参数 Symbol 基址为持久化接口 this−`0x11c`，写入 this−`0xc4`；Character `0x6eb9b58d..0x6eb9b5a2`，分别为 this−`0x164` 和 this−`0x10c`；Picture v9 `0x6eb7aabf..0x6eb7aada`，分别为 this−`0x18c` 和 this−`0x134`；Picture 历史路径 `0x6eb7dcc3..0x6eb7dcd2` 分别为局部对象基址+`0x18` 和 +`0x70`。四组差值均为 `0x58`，与上项属性槽严格对位。流中原值被成功读取以消费字节，随后原生内存中的同一属性由注册表值或失败默认值替换。具体版本门和 Save 路径见 [第二轮](arcmap-10.1-display-symbols-round2.md)。
4. Save 写出的是保存时的槽值；不能因为 Load 曾赋默认值，就断言中间经过属性 setter 后 Save 仍写默认值。读取器继续保留流中原始 short，不模拟环境注册表或把原值当成加载后属性。`src/objects/display_symbol.rs` 的对应字段注释已补充属性名，并更正 Save 描述，不改变解析行为或接口。

因此第 218 行可按 **ArcMap 10.1 静态读取即弃语义及对应属性**交主代理关闭；具体机器的注册表配置、实际运行路径及旧版 ArcMap 是否具有同样语义未验证，不能推广。216 的其他字段、223 等其余待办仍保持开放。本文此前「默认函数实现及属性关联仍未知」已由本补充更新。

补充验证：使用上述专用 `TMPDIR` 执行 `cargo fmt --check` 和完整 `cargo test` 均通过（默认忽略项未运行）。这些测试验证仓库当前解析器，不替代注册表受控实验或 ArcMap/QGIS 人工检查。
