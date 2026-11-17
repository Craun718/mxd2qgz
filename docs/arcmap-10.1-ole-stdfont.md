# 10.1 OLE StdFont / StdPicture 持久化体（oleaut32）

研究记录：OLE 标准字体（StdFont `{0BE35203-8F91-11CE-9DE3-00AA004BB851}`）与
标准图片（StdPicture `{0BE35204-8F91-11CE-9DE3-00AA004BB851}`）在 MXD 对象流
内的 IPersistStream 持久化格式，2026-11-15 轮。实现落位 `src/objects/ole_font.rs`
（读取器）与 `src/objects/element_body.rs`（TextSymbol IFontDisp 槽类分派 +
版本门尾段样本对位）。

## 背景

IFontDisp（`{BEF6E002-A874-101A-8BBA-00AA00300CAB}`）与 IPicture
（`{7BF80980-BF32-101A-8BBB-00AA00300CAB}`）槽的流内实现类是 OLE 系统组件
oleaut32.dll 提供的 StdFont/StdPicture，横跨 TextSymbol、CharacterMarkerSymbol、
Picture 系符号等多处槽位。仓库参考样本 /Maps 流 37 处 StdFont 新对象、0 处
StdPicture。样本解析此前停在 /Maps `0xd6c`：TextSymbol 体内 IFontDisp 槽的
StdFont 新对象（键 68）体未闭合且对象体无外部长度不可跳过（停止原因链
`NestedObjectNotClosed slot "IFontDisp 字体槽"`），是样本推进的第一阻塞环。

实现类不在 ArcGIS DLL 内：esri.clsid 表（11348 项）查无 `{0BE35203}`/
`{0BE35204}`，属预期——两者是 OLE 系统类，不是 Esri coclass。

## 证据来源分级

- **A 一手样本字节（本轮最强证据）**：样本 /Maps 流 37 处 StdFont 新对象按
  本轮字段序逐字节全部吻合（37/37；走查脚本复制于本 worktree
  `temp/agent-olestd/stdfont_walk_all.py`，其中 21 处 TextSymbol 字体槽语境、
  16 处 CharacterMarker 系符号字体槽语境；TextSymbol 语境续读键严格 +1 递
  增，构成边界正确性的独立旁证）。StdPicture 样本 0 实例，无字节佐证。
- **B 原生反编译（介质分发件）**：10.1 安装介质 `Desktop/Setup1.cab` 携带
  MS 运行库条目（`cabextract -l` 命中 5 条 OLE 系条目），其中
  `Global_System_OLEAUT32_f3.8C0C59A0_7DC8_11D2_B95D_006097C4DE24` 经 PE 版
  本资源核对为 **Microsoft OLEAUT32.DLL 2.40.4275**（产品名 Microsoft OLE
  2.40 for Windows NT/Windows 95，PE x86，下称 oleaut32 本体；ImageBase
  `0x65340000`，.text 0x81E90）。提取物与全部反编译脚本在本 worktree
  `temp/agent-olestd/`（cab-extract、ghidra_*.py、*_out.txt），未写主检出区。
  定位链：CLSID 常量字节在 .text（`{0BE35203}`@`0x65346680`、
  `{0BE35204}`@`0x65346d68`，另有 `{46763EE0-CAB2-11CE-8C20-00AA0051E5D4}`@
  `0x65346670`）→ `DllGetClassObject` 分派 `0x653a565d` 以 16 字节比对认类 →
  `OleCreateFontIndirect`（导出 ord 420，RVA `0x583c6`）→ OleFontImpl 构造器
  `0x653981bd`（对象 0xb0 字节、六接口虚表）→ IPersistStream 虚表
  `0x65379d48`（接口位于对象基址 +0xc）。
  - **版本适用性（如实标注）**：2.40.4275 是介质为旧系统分发的 OLE 2.40 组
    件（1999-2000 年代文件时间戳）；ArcMap 10.1 运行时实际调用操作系统自带
    的 oleaut32（XP 为 5.1.x 系列），本轮未提取 5.1 系列本体。「OLE 字体/图
    片持久化格式长期稳定」是假设，不是本轮独立证据；该假设由 A 级 37 处样
    本字节支撑（样本由目标软件在含 OS 版 oleaut32 的系统上写出——写侧即目
    标运行环境，对 5.1 的格式是直接字节证据）。
- **C 公开文档**：MS-OAUT（learn.microsoft.com「OLE Automation Protocol」，
  rev 22.0 PDF 可下载）公开描述字体/图片序列化；本轮未逐节核对其中小节编
  号与字段表，不作为独立证据源（网络检索未命中可直接引用的小节正文，仅在
  此记录其存在）。
- **非原生参照（仅记录，不作证据）**：本机 wine 内建 oleaut32 存在但未使用；
  wine/ReactOS 源码未核对。同 cab 条目 `Global_System_OLEAUT32_f2.*` 经版本
  资源复核实为 **ASYCFILT.DLL**（InternalName，条目名有误导性，后缀 `_f2/_f3`
  是平台变体标记）；`Global_System_OLEPRO32_f0.*`（5.0.4275）为 Win9x 时代
  图片对象载体，本轮未反编译（oleaut32 2.40 已含同等图片实现， OleLoadPicture
  等导出在 oleaut32 本体内）。
- 工具与纪律：全部静态分析用本机 pyghidra（Ghidra headless 工程放系统临时
  目录以避开路径限制），未动 Ghidra 桥接工具 MCP 二进制工具；Ghidra 反编译函数列表与输
  出存 `temp/agent-olestd/*_out.txt`。全部为静态观察，未运行目标软件。

## StdFont 体字段序（对象头 20 字节之后；确定的）

| # | 类型 | 字段 | 原生证据 | 样本观测（37 处） |
| --- | --- | --- | --- | --- |
| 0 | BYTE | 版本。Save 恒写 1；Load 无符号 `>1` 拒绝（`E_FAIL`
0x80004005）；`==0` 合法且跳过字段 1 | Load `0x65399520`：`Read(buf,1)` 后
`CMP AL,1` `JA` 拒绝、`TEST AL,AL` 条件读 | 37/37 = 1 |
| 1 | SHORT | charset（仅版本 ≥1；IFont `get_Charset`） | Load
`Read(this+0x48,2)`；`this` = 对象基址+0xc，+0x48 = 基址+0x54 charset 槽
（`OleCreateFontIndirect` 从 FONTDESC.sCharset 写入同槽） | 134
（GB2312_CHARSET，宋体）×21 / 0（ANSI_CHARSET，ESRI Default Marker）×16 |
| 2 | BYTE | 标志位包：bit1=Italic、bit2=Underline、bit3=Strikethrough
（bit0 与 bit4-7 保留，写侧 `旧值&0xf1` 保护） | `OleCreateFontIndirect`
`((fUnderline&1\|(fStrikethrough&1)<<1)<<1\|fItalic&1)<<1\|旧值&0xf1` 写基
址+0x58；Save `MOV DL,[this+0x4c]`（= 基址+0x58） | 0（全部非斜体/下划
线/删除线） |
| 3 | SHORT | weight（IFont `get_Weight`） | Save `MOV CX,[this+0x4a]`
（= 基址+0x56 weight 槽） | 400（FW_NORMAL）/ 700（FW_BOLD） |
| 4 | INT32 | 字号 CY 低 32 位（点数 ×10000；CY 高 32 位不在流内，Load 恒
置 0） | Save `MOV EDX,[this+0x38]`（= 基址+0x44 = CY.lo 槽；构造器默认
80000 = 8.0pt）；Load `MOV dword ptr [ESI+0x3c],0`（清 CY.hi） |
120000=12.0、82500=8.25、97500=9.75、80000=8.0、54000=5.4、142500=14.25 等 |
| 5 | BYTE | 名字 ANSI 字节数。Load 无符号 `>0x20` 拒绝（`E_FAIL`） |
Load `CMP AL,0x20` `JA` 拒绝 | 4（宋体）/ 19（ESRI Default Marker） |
| 6 | BYTE[n] | 名字 ANSI 字节，无结尾终止符；Load 后 MultiByteToWideChar
转入 0x20 wchar 内联缓冲（基址+0x5a 名字槽） | Load
`MultiByteToWideChar(0,0,buf,n+1,this+0x4e,0x20)`（调用结果不检查——溢出
行为未定义，本读取器不模拟） | GBK「宋体」（`cb ce cc e5`）/ ASCII「ESRI
Default Marker」 |

体长 = 11 + n 字节；体不含任何对象条目/键（无嵌套 LoadObject），键登记表
不参与。

Save（`0x65399626`）逐指令核对：`buf[0]=1`；`MOV CX,[this+0x48]`→
`buf[1..2]`；`MOV DL,[this+0x4c]`→`buf[3]`；`MOV CX,[this+0x4a]`→
`buf[4..5]`；`MOV EDX,[this+0x38]`→`buf[6..9]`；strlen(ANSI 名字)→
`buf[10]`；单次 `Write(buf,0xb)` + `Write(name,n)`。Load（`0x65399520`）
对称：`Read(buf,1)` 版本 →（版本 ≥1）`Read(this+0x48,2)` charset →
`Read(blk,8)`（blk[0]=flags→基址+0x58、blk[1..2]=weight→+0x56、
blk[3..6]=size→+0x44、blk[7]=名字节数）→ `Read(name,n)` + 零终止 +
MultiByteToWideChar。Save/Load 往返与 37 处样本三方一致。

对象字段身份旁证（同一构造器/创建函数闭环）：IPersistStream 接口 `this` =
基址+0xc，故 this+0x48/+0x4a/+0x4c/+0x38/+0x4e 恰为基址+0x54/+0x56/+0x58/
+0x44/+0x5a；这五个槽分别被 `OleCreateFontIndirect`（`0x653983c6`）从
FONTDESC.sCharset/sWeight/(fItalic,fUnderline,fStrikethrough 位包)/
cySize.lo/名字写入，构造器 `0x653981bd` 置默认值（+0x44=80000、+0x54=1
DEFAULT_CHARSET、+0x58=0、+0x5a=0）。IFont 属性序旁证：属性袋 Save
（`0x653999e3`，IPersistPropertyBag 虚表 `0x65379d28`）按 IFont 虚表
get_Name(0xc)/get_Size(0x14)/get_Italic(0x24)/get_Underline(0x2c)/
get_Strikethrough(0x34)/get_Weight(0x3c)/get_Charset(0x44) 取值，与
IFontDisp 属性表一致。

## StdPicture 体字段序（对象头 20 字节之后；Load 路径确定的）

| # | 类型 | 字段 | 原生证据 |
| --- | --- | --- | --- |
| 0 | WORD | 魔数 0x746c（LE `'lt'`）。Save 头恒写；Load `!=0x746c` 走旧格
式分支（无头，直接按图像数据首 WORD 魔数嗅探）——本读取器拒绝非 0x746c
形态（MXD 写侧恒为 Save 头形态） | Load 读头循环 `0x653a0191` 读 2 字节比
对；SaveHeader `0x653a1071` 写入 |
| 1 | WORD | 第二字。Save 写 0；语义未定；Load 不校验，原样保留 |
SaveHeader 第二个 `Write(4)` 的高半 |
| 2 | DWORD | 内嵌图像数据声明字节数；`==0` = 空图（Load 置 PICTYPE 0 并早
退） | Load 读 4 字节后零值分支 |
| 3 | BYTE[n] | 图像数据。Load 按首 WORD 魔数分派解码器：`0x4D42`（'BM'）
BITMAP、`0xFFD8` JPEG、`0x4749`/`0x6967` GIF、`0xCDD7`、`0x0001`、`0x0000`
；未知魔数拒绝 `0x800a01e1`。本读取器原样保留字节、不解释格式（解码超出
MXD 解析范围，未知魔数保留而非拒绝——与原生差异如实标注） | Load 内嵌分支
魔数 switch；BITMAP 分派 `0x6539f482`、GIF `0x653a3162(&0x6537a0c8)`、
JPEG `0x653a3162(&0x6537a0d8)`、增强元文件 Save `0x653a0cab` |

定位链：OlePictureImpl 构造器 `0x6539d4bc`（对象 0x9c 字节；由
`OleCreatePictureIndirect` ord 419 `0x60623` 调用），IPersistStream 虚表
`0x6537a168`（接口位于对象基址 +0x8），Load thunk `0x653a0764`→`0x653a0722`
→读头循环 `0x653a0191`（+重渲染 `0x6539e24a`），Save thunk `0x6539f34e`→
`0x653a1183`（头 `0x653a1071` + 按 PICTYPE 分派 1=BITMAP `0x653a0ba8`、
2=METAFILE `0x653a0d1d`、3=ICON `0x653a104a`、4=ENHMETAFILE `0x653a0cab`）。
样本 StdPicture 0 实例：读取器仅反编译证据 + 合成测试（`tests/ole_font.rs`），
样本对位无从进行。

## 样本对位（辅助观察）

- **StdFont 37/37**：候选布局（字段序见上表）对 /Maps 流全部 37 处 StdFont
  新对象逐字节解析，版本恒 1、标志恒 0、名字节数与名字字节全部落位（GBK
  「宋体」/ ASCII「ESRI Default Marker」——后者是 ArcGIS 自带 marker 字体
  名，非异常数据）。无异常实例。
- **首例（键 68 @`0xd6c`）链路**：FDOGraphicsLayer → ElementCollection →
  标注属性集合（ALPC 键 59）→ LELP（键 60）→ TextSymbol（键 61）→ IFontDisp
  槽 StdFont（键 68）：体 15 字节（`01 86 00 00 90 01 c0 d4 01 00 04 cb ce
  cc e5`）逐字节吻合；TextSymbol 版本门尾段随后可达：尾 short `00 00`
  （0x0000）、尾 GUID `{D7099B91-E298-499E-9757-72BB39E55CF3}`（TextSymbol
  内置默认 TextPath CLSID，与 DisplayCore 常量 `0x6ec15a94` 旁证互证）、尾
  short `00 00`。此前元素体轮文档注记的「样本 0x8601」实为 StdFont 体前两
  字节（`01 86` LE）的误读，本轮更正。
- **键递增旁证**：TextSymbol 语境 21 处 StdFont 闭合后，下一对象头键均严格
  +1（如 68→69、1597→1598、1617→1618），边界的独立旁证。
- **两种宿主语境**：宋体 21 处为 TextSymbol 字体槽；「ESRI Default Marker」
  16 处为 CharacterMarker 系符号字体槽（渲染器→LegendGroup 链，闭合后下一
  对象头为 LegendGroup `{167C5EA2-AF20-11D1-8817-080009EC732A}`）。后者宿主
  体的槽接线属 display_symbol 轮，见「接线建议」。
- **样本重跑**：接线后 `parse samples/测试底图.mxd` 停止点由 `0xd6c` 推进至
  `0xda3`（+55 字节；StdFont 头 20 + 体 15 + TextSymbol 尾段 20），TextSymbol
  体完整消费（含 StdFont 与尾段，键 68 登记）；新停止点为 LELP ⑰
  IBasicOverposterLayerProperties 槽的未识别类新对象
  `{C9EE5352-D141-11D1-880A-080009EC732A}`（键 69，esri.clsid 表查无；同
  `880A-080009EC732A` 族但非 `{EE535289}` BasicOverposterLayerProperties），
  LELP 条目键与其后字段不可达。其余 36 处 StdFont 位于停止点之后，未参与本
  轮解析。

## 接线落位（本轮）

- `src/objects/ole_font.rs`（新建）：`STDFONT_CLASS_ID`/`STDPICTURE_CLASS_ID`
  常量、`read_std_font_body_10_1`/`read_std_picture_body_10_1` 事务性叶子读
  取器（无嵌套对象条目，登记表不参与；`max_bytes` 约束名字与图像数据声明长
  度；版本/魔数/名字节数超限 `InvalidData`、截断 `UnexpectedEof`，失败不推
  进游标）。
- `src/objects/mod.rs`：追加 `pub mod ole_font;`（仅此一行）。
- `src/objects/element_body.rs`：IFontDisp 槽（TextSymbol 第 15 字段）由仅分
  类改为类分派——新枚举 `FontSlot10_1 { Null, BackReference, StdFont }`，
  `read_font_slot` 助手：StdFont 新对象整体解析（键按原生「登记先于持久化」
  语义保留；体自身错误上抛即整体失败，与 Polyline 几何体语义一致）；其余实
  现类（含 StdPicture 类标识出现于字体槽的异常形态）维持停止回退（游标停在
  该槽对象头起点、键不登记、停止原因仍为 `NestedObjectNotClosed { slot:
  "IFontDisp 字体槽" }`）。版本门尾段代码不变（既有静态结论），样本对位值
  落账于字段注释。
- `tests/ole_font.rs`（新建，11 例）：样本首例体字节原样、marker 字体形态、
  bold 形态、版本 0 跳 charset、版本/名字节数原生拒绝、逐前缀截断游标不动、
  StdPicture 头/空图/魔数拒绝/限额/截断、TextSymbol 槽级 StdFont 全消费 + 尾
  段、未知类与 StdPicture 类停止回退、回引/空标记、体截断整体不提交。
- `tests/element_body.rs`：IFontDisp 空标记断言由 `NestedEntry10_1` 改
  `FontSlot10_1`（既有用例小幅适配）。

## 接线建议（写给整合轮）

1. **display_symbol.rs IPicture 槽**：PictureMarkerSymbol（图像类型 3 /
   历史版本 <4 / 版本 ≤6 / 版本 ≥7 空串分支）与 PictureFillSymbol 的
   IPicture 槽可按类分派接入 `read_std_picture_body_10_1`：类标识 ==
   `STDPICTURE_CLASS_ID` → 整体解析（体无停止点）；其余实现类维持
   `read_unclosed_slot_10_1`（`DisplaySlotKind10_1::Picture`）停止回退。样
   本 0 实例，接线后建议用合成测试固定契约（可仿 `tests/ole_font.rs` 槽级
   用例）。现有 `UnclosedSlot10_1` 结构体字段如需携带体，建议仿本轮
   `FontSlot10_1` 模式新增槽位枚举而非复用结构体。
2. **CharacterMarkerSymbol 字体槽（`font_slot_v3`/`font_slot_v5`）**：样本
   16 处「ESRI Default Marker」StdFont 位于该符号字体槽，闭合后其宿主体续
   读字节（单实例观测，供布局核对）：槽体后为 short=1、16 字节零 GUID、
   short=0、int32=0、int32=2、short=0，随后 LegendGroup `{167C5EA2}` 对象
   头——与 display_symbol 轮 CharacterMarkerSymbol 字段序（font 槽 v≤3 /
   v>3 重写门 + font_name String + int32 ×4 + short ×2 + int32 ×2）的互证
   与取舍由该轮裁决；建议分派方式与 TextSymbol 字体槽一致（StdFont 解析、
   其余类停止回退）。
3. **robustness 建议入口**（供并入现有清单）：`read_std_font_body_10_1`
   （截断逐前缀、版本 >1、名字节数 >0x20、max_bytes 限额）、
   `read_std_picture_body_10_1`（截断、魔数非 0x746c、声明长度 u32::MAX 溢
   出安全、max_bytes 限额）、`element_body::read_text_symbol_body_10_1` 的
   IFontDisp 槽三分支（StdFont 体截断整体失败不提交、未知类停止回退、回
   引）。全部入口已在 `tests/ole_font.rs` 覆盖，robustness 轮可按其既有矩
   阵形态并入。

## 未解决事项（开放项）

- **版本适用性假设**：介质 oleaut32 为 2.40.4275（Win9x/NT4 时代分发件）；
  ArcMap 10.1 运行时用的 OS oleaut32（XP 5.1.x）未提取反编译。「格式长期稳
  定」目前仅由 A 级 37 处样本字节（目标环境写侧）支撑；若取得 5.1 系列本体
  （XP 安装目录或 update 包），应复核 `OleFontImpl::Load/Save` 与
  `OlePictureImpl` 读头循环的指令级一致性。
- **StdPicture 样本 0 实例**：读取器仅反编译 + 合成测试；头第二字语义未定
  （Save 写 0、Load 不校验）；图像数据格式分派表未逐一反编译各解码器入口；
  与样本字节的对位无从进行。
- **StdFont 版本 0 的字段默认值**：Load 对版本 0 跳过 charset，其余字段照
  读；写侧恒写 1，版本 0 的完整语义（何时产生）未定性。
- **flags 位包仅 bit1-3 定性**（Italic/Underline/Strikethrough，源自
  FONTDESC 打包式）；bit0 与 bit4-7 在 37 处样本恒 0，原生是否使用未定。
- **charset/weight/size 与 QGIS 字体模型的映射**未做（属转换语义，非格式）；
  名字 ANSI 字节按写侧系统 ANSI 代码页编码（样本为 GBK），解码策略属转换
  轮。
- **新停止点类未识别**：`{C9EE5352-D141-11D1-880A-080009EC732A}`（键 69，
  LELP ⑰ 槽）不在 esri.clsid 表；是否为另一 IBasicOverposterLayerProperties
  实现未定。
- CharacterMarker 语境的宿主续读字节为单实例观测，未成布局结论（见「接线建
  议」2）。
- 全部结论为静态反编译与字节走查，未在 ArcMap 10.1/QGIS 中人工目视验证；单
  元测试不代表目标软件验证。

## 落账

- 2026-11-15：StdFont 体闭合（37/37 样本对位 + 介质 oleaut32 2.40.4275
  Load/Save 反编译）、StdPicture 读取器（反编译 + 合成）、TextSymbol
  IFontDisp 槽类分派接线、版本门尾段样本对位（更正 0x8601 误读）；样本停止
  点 `0xd6c`→`0xda3`。Rust 改动经 `cargo fmt --check` 与全量 `cargo test`
  绿；未声称运行验证。

## 接线轮落位（整合接线轮，2026-11-15）

「接线建议」1、2 已落位：

- **display_symbol.rs IPicture 槽**（建议 1）：PictureMarkerSymbol 四处
  IPicture 槽（图像类型 3 / 历史版本 <4 / 版本 ≤6 / 版本 ≥7 空串分支）改经
  新 `StdPictureSlot10_1` 槽枚举与 `read_std_picture_slot_10_1` 分派——
  StdPicture 新对象整体解析 `read_std_picture_body_10_1`（体无停止点；回退
  语义仿 `read_unclosed_slot_10_1`），其余实现类维持停止回退。IMarker
  Background/IPictureFillSymbol/IColorRamp 槽维持 `UnclosedSlot10_1` 停止语
  义不变。样本 0 实例，接线契约由 `tests/display_symbol.rs` 新增合成用例固
  定（类型 3 StdPicture 内联解析 + 未知类停止回退）。
- **CharacterMarkerSymbol 字体槽**（建议 2）：`font_slot_v3`（v<3）与
  `font_slot_v5`（v>3 重写门）改经新 `StdFontSlot10_1` 槽枚举与
  `read_std_font_slot_10_1` 分派（StdFont 解析、其余类停止回退，与
  TextSymbol 字体槽一致）。槽后宿主续读字节按本轮「接线建议」2 的单实例观
  测与 display_symbol 轮字段序（int32 ×2 + short_10c + 字体描述符段）由既有
  读取器继续闭合——合成用例验证 v<3/v>3 两形态的 StdFont 内联解析后宿主尾
  段可达；16 处样本实例位于图层条目 2..6 渲染器链（当前样本停止点之后），
  接线不改变样本停止点，样本内推进无从观察（如实标注）。
- **robustness**（建议 3）：`read_std_font_body_10_1`/`read_std_picture_body
  _10_1` 并入 `tests/robustness.rs` 集中清单（截断扫描/游标/限额矩阵）；
  IFontDisp 槽三分支维持 `tests/ole_font.rs` 槽级用例覆盖（槽读取器为模块
  私有，不单列入集中清单）。
- 样本重走：停止点维持 `0x155a`（DEP 尾段，见 featurelayer-tail 文档接线轮
  节）；本轮接线不改变样本行为。`cargo fmt --check` 与全量 `cargo test`
  绿；未声称运行验证。
