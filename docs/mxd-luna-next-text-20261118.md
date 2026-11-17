# TextSymbol 属性访问器映射续查（2026-11-18）

## 范围与结论

本记录继续处理 TODO 中 TextSymbol 字段语义。目标为 ArcGIS Desktop 10.1.0.3035 的 DisplayCore.dll；复核提取件 SHA-256 为 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`，与既有 10.1 文件清单一致。结论来自类型库、既有 Ghidra 12.1.4 导出伪代码及同一提取件原始 x86 指令，不是原始源码，也不是运行取证。

本轮可靠闭合的是**接口属性名 → 10.1 TextSymbol 类虚表槽 → 实际访问器函数**。ITextSymbol 的 Font、Size、Color、VerticalAlignment、HorizontalAlignment、Angle、RightToLeft、Text 各访问器均可定位；IFormattedTextSymbol 的 ShadowColor、ShadowXOffset/YOffset、Position、Case、CharacterSpacing/Width、WordSpacing、Kerning、Leading、Direction、FlipAngle、Background、TypeSetting、FillSymbol 槽也可按类型库偏移与类虚表顺序定位。部分访问器直接读写其 `this` 相对成员，部分委派至内部对象。

这**没有**把上述全部属性与 `TextSymbol::Load` 中每个序列化目标逐字段配平。Load 使用的 this 调整与这些 COM 访问器的 this 基址尚未通过完整 thunk/虚基关系换算；近似偏移或流序相邻都不足以定名。因此本轮不声称 TODO 的整项字段业务语义已闭合，也不改 TODO。

## 方法和证据链

- 原生文件身份：`binaries/manifest.json` 中 DisplayCore.dll 项的摘要与本轮 `sha256sum` 一致；仓库已有记录注明其 PE 版本为 10.1.0.3035，ImageBase `0x6eb30000`。
- 类型库：`temp/terra-follow-display/ITextSymbol.txt`（IID `{A80B5E91-7F9C-11D0-9410-080009EEBECB}`）与 `IFormattedTextSymbol.txt`（IID `{B65A3E72-2993-11D1-9A43-0080C7EC5C96}`）列出属性及 `oVft`。原枚举经 64 位 Wine `LoadTypeLibEx` 得到；按既有记录的 x86 虚表换算，`oVft / 2` 为 x86 字节偏移，再除以 4 得槽序号。
- 类虚表身份：构造函数 `0x6ebc9da0` 在 `this+0x98` 安装虚表地址 `0x6ec1643c`；对应类构造/析构与 `TextSymbol` RTTI/CLSID 关系见 [元素对象体记录](arcmap-10.1-element-bodies.md)。直接读取 DisplayCore 原始 `.rdata` 可在 `0x6ec1643c` 起按 4 字节取得函数序列。前三槽为 IUnknown；ITextSymbol `Font` 类型库偏移 24/32（64 位 dump）换算后落在 x86 槽 3/4，实际值为 `0x6ebcb700` / `0x6ebcb620`。后续属性同样按槽逐项对应。
- 原始指令抽查：`0x6ebcb8d0/0x6ebcb840` 对 `this+0x2d0` 读写 double；`0x6ebcb750/0x6ebcb720` 对 `this+0x4c` 读写 int32；`0x6ebcc1d0/0x6ebcc260` 对 `this+0x284` 的 String 取 BSTR/赋值。`objdump -d -Mintel` 核对了这些机器指令；相应伪代码、所有槽值及解释见 [槽位证据表](../temp/textsymbol-20261118/i-text-symbol-vtable.tsv)。
- 外部调用方旁证：既有 `TextElement::Load`（CartoXProvider `0x6ce70e50`）对 ITextSymbol 槽调用虚表字节偏移 `+0x34` 并传入文本元素成员地址、再以 `+0x48` 传入其 BSTR。结合类型库槽表，前者为 Angle getter、后者为 Text setter。这确认两个接口成员在 TextElement 中的实际使用角色，但仍不把 TextSymbol Load 中相似位置的原始字段自动归给它们。
- TextSymbol Load 实体 `0x6ebd0a60` 的持久化字段序及样本边界仍按 [既有字段序记录](arcmap-10.1-element-bodies.md#textsymbolload-0x6ebd0a60-字段序证据确定的样本逐字节互证)；本轮没有重新解析样本，也未更改读取器。

## 可核对属性

属性、访问器 VA 及访问器内可见成员表达式已列在 TSV。以下含义限于类型库公开的接口契约与属性类型；它不推导枚举取值表、单位、显示效果，也不自动证明对应的持久化字段。

| 属性 | 类型库属性类型 | 本轮核实的访问器表现 | 持久化关联状态 |
| --- | --- | --- | --- |
| Font | IFontDisp | 虚表槽 3/4；访问内部字体对象 | Load 的 IFontDisp 槽存在，但与该 getter/setter 的成员/this 调整尚未闭环 |
| Size | R8 | 槽 5/6；直接读写 `this+0x2d0` double，setter 检查一个特殊值 | 未与 Load 的 double 逐项对位 |
| Color | IColor | 槽 7/8；getter 可走替代对象，默认成员在 `this+0x3c` | Load 有两个 IColor 槽，未判定哪一槽对应此属性 |
| VerticalAlignment / HorizontalAlignment | enum | 槽 9/10、11/12；分别访问 `this+0x4c`、`this+0x48` 的 int32 | 未与 Load 中多个 int32 逐项对位；枚举成员值含义未定 |
| Angle | R8 | 槽 13/14；经内部对象接口委派，相关成员偏移 `this+0x60` | 未与 Load 多个 double 之一闭环；单位/角度方向未定 |
| RightToLeft | BOOL | 槽 15/16；经内部对象委派并涉及 `this+0x55` byte | 未与 Load byte 组闭环；流值规范性及运行行为未定 |
| Text | BSTR | 槽 17/18；String 位于访问器 `this+0x284` | Load 确有 String 字段；尚缺两套 this 基址和 thunk 调整的精确换算，暂列候选而非已配平 |
| ShadowColor、阴影偏移 | IColor、R8 | IFormattedTextSymbol 槽 20–25；阴影颜色走内部对象，偏移由直接 double 成员访问 | 未与 Load 对位；现有 TextSymbol Load 字段序中也未识别出可证的独立阴影颜色对象槽 |
| Position、Case | enum | 槽 26–29；直接 int32 成员 `this+0x158/+0x15c` | 未与 Load 的 int32 组配平；枚举值表未定 |
| CharacterSpacing、CharacterWidth、WordSpacing、Leading、FlipAngle | R8 | 槽 30–35、38–39、42–43；直接 double 成员 | 未与 Load 的 double 组配平；单位与各字段含义不由槽位本身证明 |
| Kerning、TypeSetting | BOOL | 槽 36/37、46/47；分别涉及 byte 成员 `this+0x178`、`this+0x19c`，TypeSetting setter 还可能重置默认 parser GUID | 未与 Load 的 byte 组配平；布尔流编码语义未定 |
| Direction | enum | 槽 40/41；直接 int32 成员 `this+0x18c` | 未与 Load 的 int32 组配平；枚举值表未定 |
| Background、FillSymbol | 接口对象 | 槽 44/45 的 Background 成员在 `this+0x198`；槽 48/49 的 FillSymbol getter 从 `this+0x1a0` 查询接口，setter走受状态限制的内部更新路径 | Load 有 ITextPath 与 ISymbol 槽，但尚不能将其认作 Background/FillSymbol；不得按类型相似直接等同 |

## 明确未定项

1. Load 的首批 int32（`this-0x478/-0x470/-0x474`）、byte（`-0x468/-0x467`）、double（`-0x45c/-0x44c/-0x444`）和 int32（`-0x4fc/-0x4f4`）均未由本轮证据命名。
2. 第二组 IColor 槽后的 double×2、int32×2、double×3、byte、double、int32、double、byte，以及末尾各版本门字段，除原始类型/顺序外仍未完成业务语义映射。
3. Load 的原样 GUID 与 ITextPath/ISymbol/IFontDisp 对象槽沿用旧记录的分类；不能据本轮接口清单重新赋予背景、填充或文字路径语义。
4. 未确定枚举完整取值含义、距离/角度/字号单位、setter 对流值的接受范围，以及任一属性在 ArcMap 中的实际显示效果。
5. 精确关联仍需确认 TextSymbol 每个接口子对象的 this 调整、基类/虚基 thunk 及 Load 落点与访问器成员地址间的共同对象基址。届时才能将确定的行单独闭合；其余保留原始字段名和值。

本轮仅增加本研究文档与 `temp/textsymbol-20261118/` 证据表。未改解析代码、TODO 或共享工作树中的既有文件；未运行 ArcMap（当前无环境），未做 ArcMap/QGIS 人工验收。
