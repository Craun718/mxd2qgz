# 第四轮 DisplayCore 与 OLE

2026-11-17。行号对应本轮接手时 TODO.md 的原行；未修改 TODO 或公共模块接线。静态证据不等同于 ArcMap/QGIS 中运行或显示验证。

## 文件身份与方法

- README 指向 ArcGIS Desktop 10.1.3035 安装介质。本轮重新计算提取件 `temp/agent-fl-tail/DisplayCore.dll.33A02325_2485_4623_B2CE_71B5736B51DF` 的 SHA-256 为 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`；PE machine `0x14c`、文件版本 10.1.0.3035。
- 介质分发件 OLEAUT32 SHA-256 `baeb2f7c1b8be56738d34e1d1ddf8e0eebd3a633215dc1575e14656be38b939d`，PE x86，版本 2.40.4275.1，不是已验证的目标系统运行时。
- Ghidra（经桥接工具） 12.1.4 批量反编译 `0x6eb35310`、`0x6eb3c190`、`0x6eb395b0`、`0x6eb71110`、`0x6ebd0a60`，Evidence `ev_bc70a38ee0a7fab898b527725a2614ede5388940e6461ab113c2c5b296a05e21`；另用 objdump 核对下述指令。静态 VA 属于上述文件，伪代码不是原始源码。

## 新增原生观察及实现

RandomColorRamp Load `0x6eb3c190` 在 `0x6eb3c230..0x6eb3c240` 读 2 字节，`0x6eb3c24b` 为 `cmp word ptr [ebp-4],0`，`0x6eb3c25a` 为 `setne al`，`0x6eb3c25e` 写入 `[edi-0x28]`。因此判断使用**整个 16 位**，不是已有记录称的低字节；例如 `0x0100` 也为真。`src/objects/color_ramp.rs` 保留原始 `bool_short`，新增 `loaded_flag()` 表达装载后的布尔值，并修正该文件中的错误注释；字段的业务名称仍未知。测试覆盖零、低字节非零、高字节非零、全位非零和截断回滚。旧 `docs/arcmap-10.1-display-slots-round3.md` 对该字段的低字节说法由本记录更正，未改共享旧文档。

## 原行状态

| 原行 | 状态 | 证据与余项 |
| --- | --- | --- |
| 214 | 部分 | 颜色核心 `0x6eb35310` 再次确认三 double / 四字节分支与尾字节取补；尚未把颜色及各符号全部标量与属性访问器对位。新确认梯度布尔载入规则不提供业务名。 |
| 221 | 部分 | TextSymbol `0x6ebd0a60` 重新反编译，支持现有字段序与版本门；未完成属性访问器和各成员偏移绑定，继续保留原始值。 |
| 248 | 部分 | RandomColorRamp 丢弃 int32、PictureFill 两个局部 double、MultiPart 版本门再次由上述 Ghidra 桥接工具 结果支持；新纠正同体布尔标志的整字判断。丢弃字段的历史用途和业务含义仍未知。 |
| 267 | 部分 | 既有 Africa 介质 MultiLayerMarker/CharacterMarker 局部对象测试见 `mxd-round2-symbol-ole-20261117.md`；其余符号类尚未逐个从已证实宿主路径读取，不能按候选命中闭合。 |
| 268 | 部分 | 检查现有 10.1 介质提取的 Maps/PageLayout：HSV 类标识 10 次、CMYK 306 次候选，HLS/Gray 0 次；例如 CentralUSA Maps `0x4227` 和 Africa Maps `0xeb4`。这是字节定位，不证明对象归属或边界；尚未从已证实宿主链走到这些候选。原行不闭合。 |
| 274 | 部分 | 第三轮 `mxd-round3-display-spatial-20261117.md` 核实介质 OLE 2.40 写第二字为零、读侧未见校验，并确认 BMP/GIF/JPEG 等分派；第二字业务语义及目标 OS 解码器仍未定。真实 BMP 局部测试见第二轮记录。 |
| 275 | 阻塞 | 介质只确认 2.40.4275.1；搜索既有提取介质、Wine 环境和文档目录未获得经身份确认的目标 Windows oleaut32 5.1.x。Wine 文件不能代替目标运行库，跨版本互读及受控写出仍待目标运行库和实际环境。 |

现有介质检索是有界检查，不证明其他样本、Windows 版本或类不存在。对象测试是解析器验证，不是目标软件的验收；未在 ArcMap 10.1 或 QGIS 打开工程目视检查。

## 本轮验证

- `cargo test` 通过（依赖介质的忽略项未包含在默认运行）；新增标志位测试包含在内。
- `cargo test --test round2_symbol_ole_media -- --ignored`：2 项通过，复核两份介质的既有局部符号及 BMP 边界；仍非完整宿主链或显示验收。
- `rustfmt --edition 2024 --check src/objects/color_ramp.rs` 和本组改动的 `git diff --check` 通过。执行 `cargo fmt --check` 时，其他并行改动的 `src/maps/map_object.rs` 和 `tests/map_object.rs` 有格式差异，本组未改动这些文件。

接线交接：`RandomColorRampBody10_1::loaded_flag()` 已在既有公开模块可用，无须修改 `mod.rs` 或 `lib.rs`；调用方只有在需要原生加载后的布尔解释时才需使用该方法，原始 `bool_short` 继续可用。其余原行仍未闭合，不建议勾选。
