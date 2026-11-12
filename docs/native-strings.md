# Desktop 10.8 AfCore 字符串规则

2026-11-12，目标 AfCore.dll SHA-256
`560be16e0a99b9de4599f8241c421bddf0dcc8b26509f584074dea53b0ac4aae`，
文件版本 10.8.0.12790，REA 6.8.0 / Ghidra 12.1.4，分析配置与
[MapDocument 入口记录](mapdocument-native.md) 相同。以下地址均为 DLL 虚拟地址。

## 已观察到的实现

- `String::Write` `0x10055190`，反编译
  `ev_24b2ad650c077b5d77bf17149320907c84aa04677282aea11b2348d53f9536dd`、
  汇编 `ev_046792507eef6b423ba456342eb35ca2c6bb0d8a49b6e4bf8ed9128b0b5cae80`：
  从内存字符串取得码元数 n，计算 `cb = n*2+2`；写 4 字节
  `cb & 0x7fffffff`，再通过 OleAut32 Ordinal 4 复制字符串并写 cb 字节、
  Ordinal 6 释放。x86 小端，wchar_t 为 2 字节；普通字符串包含终止零。
  掩码仅应用于前缀，不是读函数中的压缩标志。极大长度计算存在回绕风险，
  不能将这种算术当作安全的格式能力。
- `String::Read` `0x10055200`，反编译
  `ev_be62720a778fd480c13f63c7cb9b336f1b0efb3d2c9da633b33dd90ed629f8fd`、
  汇编 `ev_10f5766848094dd8eef177c48f8805eabbbc96ead602d789b21b77007a928730`：
  先 IStream::Read 4 字节，读取量非零且长度非零才按该长度 malloc；
  再 Read 内容，HRESULT 非负且实际读取量非零才赋值。未验证实际量等于
  请求量，未验证偶数长度或缓冲区内终止符。零长/零读取量/分配失败返回 E_FAIL。
- 赋值 `0x10055040`，`ev_fdf2d3900c6b161093f2cd748b04aea724aa576704377d85130cdbd0b433d0bf`：
  扫描 wchar_t 到首个零，再复制。因此无终止符或短读可能令原生代码越过
  已初始化数据，不能把异常输入下的原生行为作为 Rust 必须模拟的行为。
- `AfStreamRead` `0x100071a0`，
  `ev_910fd0f18a8a6dea07f343985eb5147b8453d39a6c5a58f96bab865a92dfb6d0`：
  只转发 IStream::Read 及 HRESULT，没有 ReadExact，也不把 S_FALSE 变成失败。

## 与具体字段的关系

PageLayout.dll `0x10076ef0` 确认对 Version 流调用此 String::Read，随后读 short，
见 MapDocument 记录。它不依据文本匹配版本，也不读取其后全部数据。
该关联支持对 **10.8 的这个调用点** 解释字符串表示；不能据此宣布样本的
`10.1` 是识别结果，或认为全部 Maps 属性均用此规则。
PageLayout Metadata 的多个调用点也使用 String::Read，但属性字段归属和完整
版本分支仍须独立追踪。WKT 候选可能使用另一种编码，不纳入本规则。

## 严格 Rust 子集及未知项

`native_string::read_string_10_8` 仅供调用者已确认调用点与版本时使用。
要求完整 4 字节前缀、长度至少 2 且为偶数、长度不超过调用者资源限额、
内容完整、最后一个码元为零、无提前终止、UTF-16 合法；返回文本去除末尾零。
提前零的数据可能被原生截断，其后数据含义未知，本实现明确报错而非丢弃。
这些是安全的已知写入器普通输出子集约束，**不是原生读取器的验证行为**。
失败不移动 Reader 游标；所有位置相对于传入 Reader，子范围仍是子范围坐标。
调用者负责记录所属流和对象；函数不扫描、不定位字段、不识别版本。

合成测试覆盖空字符串、中文、代理对、末尾后续字段、全部关键截断、
非法长度、最大 u32 前缀、资源限制、非法 UTF-16、终止符及失败重试。
未执行 ArcMap；未确认 10.1 兼容性、Version 尾部或文本的版本含义。
