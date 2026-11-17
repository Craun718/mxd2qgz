# Terra 第十轮：ArcMap 10.1 Legend 前缀标量对位

2026-11-17。结论仅适用于所核对的 ArcMap 10.1 静态实现；没有运行 ArcMap，也没有在 ArcMap 或 QGIS 中打开或目视检查工程。流中的 short 保留全部位模式，接口标志类型和 setter 约束不能用于限制持久化输入。

## 身份与取证

重新计算 `temp/agent-map-surround/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` 的 SHA-256 为 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`；PE Machine `0x14c`、ImageBase `0x793a0000`，固定文件版本 `10.1.0.3035`。既有 esriCarto 类型库 `temp/terra-integrate-page/carto.tlb` 重新计算 SHA-256 为 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`，其来源和 10.1 身份见 [前轮](mxd-terra-depth-legend.md)。版本资源是文件声明，哈希识别字节，不证明签名或运行状态。

本轮先复用此前独立 Ghidra 桥接工具 CLI 分析相同 Map.dll 的结果：`temp/terra-round9-bookmark/rea.json` 中 run `d4dda94a-5c97-4f39-b047-a2736cc97f45` 的 Ghidra 12.2 桥接 Unix socket 因 `Operation not permitted` 失败，未返回函数证据；该失败不是本轮新运行。随后复制已有 Map Ghidra 工程至 `temp/terra-round10-legend/`，使用 JDK 25、Ghidra 12.2/pyghidra 和该目录独立项目、TMPDIR、Java home、配置及缓存，反编译共同 Load/Save、ItemCount getter、集合清理函数和一个内部 getter；并直接检查 PE 虚表与 objdump 指令。未连接或切换共享 Ghidra 桥接工具 目标。反编译是静态伪代码，不是源码或运行证据；临时输出可能含本机路径，分享前应脱敏。类型库使用既有隔离 Wine `LoadTypeLibEx` 导出，64 位报告的 oVft 除以二为目标 x86 字节偏移；成员名与 IID 使用可读输出，宽字符串类型名乱码的局限见前轮。

## Legend 自身字段

令 Legend 对象起点为 O、IPersistStream 指针 P=O+0x1f4。Legend Load `0x793da250` 先读自身版本，再调用 MapSurround Load，之后读 Legend 自身数据。字段的已知名称和版本门：

| 流中字段 | 静态语义及依据 |
| --- | --- |
| 版本 short | Load 有符号比较 `>5` 拒绝；Save 的版本表在 10.1 写 5。这是格式版本，不是界面属性；当前 Rust 读取器额外拒绝负值。 |
| 三个 short：P−0xf0、−0xee、−0xec | 分别为 AutoAdd、AutoReorder、AutoVisibility 原始位模式。类型库、QI、x86 虚表槽、thunk 和 getter 的完整偏移对位见 [前轮](mxd-terra-depth-legend.md)。 |
| 三 short 后一个 short | `0x793da427..0x793da436` 读入栈临时变量，`0x793da441..0x793da458` 调集合清理函数且不传该值；Save `0x793da74e..0x793da760` 写低 16 位 `0xffff`。因此是该实现读后未使用、写固定值的保留字段；不把它命名为集合选择属性，不要求输入等于 `0xffff`。详见 [集合续查](mxd-terra-chain-legend.md)。 |
| 后续 int32 | Load 读到局部计数并以其控制 `ILegendItem` 的 `LoadObject`/添加循环，不直接把磁盘计数写入对象成员。Save `0x793da76d..0x793da781` 从 P−0xdc（即 O+0x118）取集合当前计数写出，之后按该数循环保存对象。类型库 `{15ABE7F6-49E8-4012-BE72-D14E5A755ACE}` 的 ItemCount getter oVft=200，折算 x86 `+0x64`；QI 返回 O+0xdc，对应虚表 `0x794857cc` 的该槽为 thunk `0x793dee70`，减 4 后跳 `0x793d7520`。getter 从调整后接口指针的 `+0x40` 返回 int32：`O+0xdc−4+0x40=O+0x118=P−0xdc`。因此这是 Legend 条目数量，读侧是循环控制值，写侧取实际集合计数；不能假定失败或非规范输入时两者始终相等。 |
| 版本 ≥3 short：P−0xe8 | 流向内部反向标志；FlowRight getter 将该内部值为零映射为真，不能将原始位直接当作 FlowRight。 |
| 版本 ≥4 short：P−0xe6 | ScaleSymbols 原始位模式。 |
| 版本 ≥5 三 short 与 double | 依次为 FixedFrame、AutoColumns、AutoFit 和 MinFontSize；AutoColumns getter 可能校正内存值。访问器、指针调整及版本门见 [前轮](mxd-terra-depth-legend.md)。 |

版本 1 的两个旧 double 读入局部变量且未存入对象，当前 Save 无对应写段；具体历史业务名称不明。版本 <5 的默认分支 `0x793da5fe..0x793da629` 从 `0x79480d38` 载入 double，其既有反编译值是 `0x4010000000000000`，即 **4.0**。`src/maps/map_surround.rs` 两处和 `docs/arcmap-10.1-mapsurround-bookmark.md` 一处旧注释误写 4.5；主审直接读取 PE 常量复核后修正。读取器不把默认值补成流字段。

## MapSurround 基类字段与边界

Legend 的 P−0x118 传给共同 Load `0x794413d0`，即 O+0xdc。版本 ≥2 的 short 读入该指针 −0x8c，即 O+0x50；共同 Save `0x79441596` 写同一地址。内部函数 `0x793a1170` 读取传入 this+0x50 并返回 16 位值，其地址在 Map.dll `.rdata` 多处出现，但尚未证明 Legend 实际虚表安装和该调用点的接口/业务属性，因此这只是待追候选，不足以给 short 命名。版本 ≥3 的 32 字节 Load 读入栈临时区，Save 取共同指针 −0x84；读写不同位置的业务意义仍未闭合，不能只凭长度称为范围。名称和 IMap 槽的结构及其限制见 [共同前缀记录](arcmap-10.1-mapsurround-bookmark.md)。

## TODO 与验证

- `TODO.md` 第 239 行“Legend 共同前缀各 short/int32 字段业务语义”：若按本类别中 **Legend 自身**各 short/int32（不含另列 MapSurround 基类字段）解释，可按上述静态业务含义勾选；保留 short 以“读弃、写固定值”闭合，而非赋予未知历史属性。版本 1 两个 double 不属于该条的 short/int32 范围。真实工程和目标运行验收仍须单独进行。
- 第 244 行“MapSurround 共同前缀字段业务语义”不能勾选：short 的业务名、32 字节块以及读写之间的关系仍未闭合。若将第 239 行理解为连同基类字段一起要求闭合，则第 239 行也不能整体勾选，宜拆分职责后分别落账。
- 第 238 行 HorizontalLegendItem 的各布局模式值仍未闭合；第 245 行书签共同前缀也不在本轮范围。未修改 TODO。

本轮只新增研究文档，没有修改 Rust 或解析行为；没有新测试。指令、类型库和隔离反编译为静态证据，不能替代 ArcMap 10.1 受控工程的载入、保存、重开及 QGIS 目视验收。
