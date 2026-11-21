# 空间参考增量取证（2026-11-17）

本记录只记本轮新增或更正；既有布局及未闭合项见 [PCS/GCS 记录](arcmap-10.1-pcs-geometry-slots.md)。全部是静态证据和合成测试，未运行 ArcMap，不能替代实机验证。

## 对象身份与方法

目标为介质提取的 x86 Geometry.dll，SHA-256 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`，与既有 VS_FIXEDFILEINFO 10.1.0.3035 身份记录一致；介质 Desktop 配置声明 10.1.3035，见 README。以 JDK 25 和独占 pyghidra 工程分析，另以 objdump 核对控制流。首次尝试因 JVM 使用 `/tmp` 导致空间不足失败；设置 `java.io.tmpdir` 到仓库独占目录后完成。未连接或修改共享 Ghidra 桥接工具 服务器。反编译及查询记录在 `temp/terra-integrate-sr/`，该目录为临时研究产物，不作为交付依据。

## 版本 2 域串及 PCS 垂向单位段

- PCS Load `0x719d343a..0x719d3452` 和 GCS Load `0x719beeca..0x719beee5` 都对版本与 2 做有符号比较，`jl` 跳过四串读取，即 **版本 >=2** 才进入 `0x719babf0`。这是对既有记录中“版本 >2”的更正；版本 2 并非仅 WKT。GCS 范围的另一判断在 `0x719bec60` 反编译中是 `2 < version`，仍为版本 >=3。原生有符号负版本的行为不等同本读取器，本读取器仍拒绝负位模式。
- PCS 的标志非零路径 `0x719d3481..0x719d34a8` 调用 `0x719ea140`，之后才进入精度段。读取器此前仅消费标志，会误把单位段当精度字段；现复用 GCS 的单位段读取，保留版本及原始文本，失败回滚游标。单位 WKT 的含义不由此推断。
- 原生 `0x719babf0` 对每串读 u32 长度和指定字节，写宽 NUL 到 `buffer + (length & ~1)`，再以 NUL 结尾构造 BSTR；奇数长度末字节会被覆盖，内嵌宽 NUL 也会影响所得 BSTR。`0x719bad00` 写 BSTR 字节长和字节。读取器有意保留完整原始字节而不模拟这些对象重建结果；业务名称和填充来源仍未闭合。

## GCS 守卫与槽位

- `0x719beca5` 在第一个 `AfStreamRead` (`0x719bece0` 附近) **之前**检查持久化接口 this+`0x98`，非零走错误 `0x80004005`/`0x1ca`。构造器 `0x719c345c` 在完整对象 +`0x2c` 安装持久化虚表，因此该成员是完整对象 **+`0xc4`**，并非旧记录的 +`0xac`；旧表格把守卫置于读 WKT 之后的顺序亦不准确。此处不推断失败后流和对象的其他行为。
- 构造器 `0x719c3091` 将 EBX 清零，`0x719c32a7` 写到对象 +`0xc4`；析构路径 `0x719c3985..0x719c3995` 非零时仅清零这一成员。QI `0x719bc4b7..0x719bc4e3` 对 IID `{3B219F24-D3FF-11D1-BC9B-0000F875BCCE}` 检查同一成员，非零返回 `0x80004002` 和空接口。同介质 esriGeometry.olb（SHA-256 `d45fb2105c869cda034fa80101cf2d0b932e08ea6f31eca8d4322fe7f7cd3e8e`）的 MSFT 类型记录将此 IID 定名为 IGeographicCoordinateSystemEdit，GeographicCoordinateSystem coclass 也直接声明该接口。由此确认非零成员同时拒绝持久化 Load 和对地理坐标系编辑接口的查询。成员类型及产生非零状态的路径仍待取证。
- slot 8 `0x719bf440` 取第二个栈参数并直接写入 `0x38`，返回 0，未见空指针检查；静态引用仅发现 GCS/PCS 两虚表项 `0x71a26838`/`0x71a29cec`。这不证明无间接调用，也不足以确定接口成员名或调用协议。

## 验证范围与剩余工作

定向合成测试检查 PCS/GCS v2 四串边界、GCS v2 无范围、PCS 非零垂向段后精度对齐、逐字节截断回滚和限额；仅验证 Rust 读取器，不是原生运行结果。Unknown 四串业务名称、slot 8 名称、守卫置位来源及含义仍待进一步取证；GCS/Unknown 未在参考 MXD 中确认实例。ArcMap 10.1 和 QGIS 人工目视检查均未执行。

### PCS 与 GCS 域串的属性映射

用 objdump 复核同一份 x86 Geometry.dll 10.1.0.3035。Load 辅助函数
`0x719babf0` 按序装入独立结构的 `+0x04/+0x08/+0x0c/+0x10`，
Save `0x719bad00` 按相同顺序取值。PCS 从持久化接口 `+0x20`
传入该结构（`0x719d344e`），GCS 从持久化接口 `+0x8c`
传入（`0x719beede`）。

两类的 ISpatialReferenceInfo 都位于完整对象 `+0x18`，分别由
QI `0x719d0967` 与 `0x719bc485` 的 IID 比对确认。按同版
esriGeometry.olb TypeInfo 4 对位，PCS 使用虚表 `0x71a297ac`，
GCS 使用 `0x71a26614`。以下 getter 均取同一结构中的 BSTR。

| 流序号 | 结构偏移 | 属性 | PCS getter | GCS getter |
| --- | --- | --- | --- | --- |
| 1 | `+0x04` | Alias | `0x719d39b0` | `0x719bf520` |
| 2 | `+0x08` | Abbreviation | `0x719d3a00` | `0x719bf570` |
| 3 | `+0x0c` | Remarks | `0x719d3a30` | `0x719bf5b0` |
| 4 | `+0x10` | Usage（类型库参数拼写为 `useage`） | | |

辅助结构构造器 `0x719baaa0` 与 `0x719bab40` 用 AfCore
BString 构造器初始化四个空串。PCS 复制入口中的 `0x719d053d–0x719d0574`
将四个槽逐项复制。

同版类型库 TypeInfo 75 和 108 的 `+0x0c` 槽分别是
`IGeographicCoordinateSystemEdit::Define` 与
`IProjectedCoordinateSystemEdit::Define`。二者的前五个显式参数均为 BSTR，
依次是 Name、Alias、Abbreviation、Remarks、`useage`。

- GCS 主 QI 在 `0x719bc4b7` 比对编辑 IID，成功路径在
  `0x719bc530` 返回完整对象 `+0x28`。虚表 `0x71a26800` 的 Define
  为 `0x719c02d0`。其 `0x719c04db–0x719c0525` 路径采用 AfCore
  BString 赋值（IAT `0x71a102fc`），将四个非空指针参数分别写到
  `+0x04/+0x08/+0x0c/+0x10`。最后一个参数从 `[ebp+0x1c]`
  经局部变量 `[ebp-0x78]` 转入第四槽。
- PCS 主 QI 在 `0x719d08f9` 比对编辑 IID，成功路径在
  `0x719d0953` 返回完整对象 `+0x30`。虚表 `0x71a29c28` 的 Define
  为 `0x719d5d50`。其 `0x719d60a2–0x719d60e6` 以同一顺序赋值，
  最后一个参数从 `[ebp+0x1c]` 经 `[ebp-0x78]` 转入第四槽。

这两条 Define 路径确认第四串为 Usage，并确认四串可由编辑接口填充。
其他赋值入口仍待核查。ArcMap 10.1 和 QGIS 尚未人工验证。
