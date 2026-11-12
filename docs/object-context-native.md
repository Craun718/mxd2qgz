# ObjectStream 版本属性、引用表与错误分支补充

2026-11-12，REA 6.8.0 / Ghidra 12.1.4，默认分析配置摘要
`e763201e522a4b123fb330eede09834c44ddf2bb90702e50eea219801be95539`。
目标 SystemCore.dll SHA-256
`48cbd921972d12156119d524f9c3923225fd30795e33c22c65c87db3a6c89aca`，
Desktop 10.8.0.12790。下文是 DLL 虚拟地址和进程内对象偏移，不是 MXD 偏移。
未执行目标软件；这是静态证据。

## AfCore 版本 IID 的一个实现者

AfCore `0x1025cc40` 的 16 字节读回为
`553cc4ec4801c14ebf87b9a183c5dc98`，GUID
`ecc43c55-0148-4ec1-bf87-b9a183c5dc98`，证据
`ev_8a9010e2d86559d593ac36997a74ec7e94fd28bc7518a2df8e95591bf249bd94`。
SystemCore 字节搜索命中 `0x100f52e4` 和 `0x10103b0c`，
`ev_e6cfca31e387e3a17f7ad15fe1495cd7c76451e7e8022aab119a825bb89a5950`。
这不是全安装包实现者清单；第二处及其他类仍待单独解释。

ObjectStream QI `0x1006f570` 比较首处 IID，返回完整对象 `+0x14`，
`ev_a0e9d81c4aa8d3a57b84bfdf57f477438467916c1efe0e49e44faef042b9cfdf`。
构造器汇编 `ev_e4e22b0d36e8549778008ef8d038edbff8bbf900cb72b20c44cac5a8832799ae`
确认该位置最终虚表为 `0x100ff70c`，完整对象 `+0x54` 初始化为 `0xb`。
虚表读回 `ev_4efec77ac1de3c17113dc71f653deaf21f99c35f5b150bf5539032d1900d1d5d`：
`+0xc` 为 `0x10070920`，`+0x10` 为 `0x10070940`。
前者赋值接口相对 `+0x40`（完整对象 `+0x54`）；后者返回同一值，空输出返回
E_POINTER。对应证据 `ev_5833a37e2c60f36e2de2499b5d7f3e35917a824953052d6eadd9da262cfb4711`、
`ev_7d5e9cacdf44372f8d63baa71a82fd34ba9fbc2297a722ebf23302c94baa2a03`。

因此 AfStreamGetDocumentVersion 的 QI 与 `+0x10` 调用已连到一个真实实现者，
得到的是内存属性，不在此读 Version 流。尚未闭合谁从源文件设定该属性、
枚举完整含义及 10.1 文本的关系；不能自动选择解析版本。

## 表的作用域与持久化调用

`SetStream` `0x1006f8c0`（`ev_1a69d78b5faa809ff5f4b203b4e0d7f182eb9f45980a2731a537a488e83b5864`）
只更新接口相对 `+0x10` 的底层流并增减 COM 引用；没有清空对象表或重置计数。
结合 PageLayout 文档读取链复用 ObjectStream 处理 DrawingDefaults、Maps、
PageLayout、View，**引用表不能默认在 OLE 流切换时重建**。
这不排除其他显式重置入口，尚未穷尽所有调用者。

读入口 `0x10070020`（反编译
`ev_e66c6155653da0ba5901726fdff392d6a3011656f50ebd0968c42ce0717be65c`，
汇编 `ev_dd3d1e1b30da9706eb0e4ba54711f29334d71e40a8d5abdf2d4797f5dd456a48`）：

1. 按已有记录读 GUID、键；表命中直接查询接口，不读对象体，也不比较这次
   GUID 与原登记类。缺失键按新对象创建处理，不能独立解读为“坏引用”。
2. 新对象在读取对象体之前登记（`0x10070362` 调用插入），使自引用和祖先引用
   成为静态上可行的情况。但尚未通过运行验证循环对象的最终恢复行为。
3. `0x1006f160` 查询 IID `00000109-0000-0000-c000-000000000046`，
   对应证据 `ev_e0e5b31a0076f55df559c69a746eeba64cdfbc4a4792fe3e462272a64b80fbd1`、
   `ev_e828bff14259f297de900c467290db046aed3509602456372426782814b2d1c4`；
   `0x10070f80` 查询 `67a4d651-69e0-11d2-8500-0000f875b9c6`，
   `ev_16e96641bba74e26a7f9b2fce3bad39394bf7633dbf3f20aabdd0ddd64ab3cdb`、
   `ev_fe3df91917f2a56805a796f09cd421c0b479c2200545481453a52b2a2cd0bf4d`。
   第二接口存在时创建 0x1c 字节 VariantStream，调用该接口 `+0x10`；否则对第一
   接口调用 `+0x14`，传入当前 ObjectStream。仍未核对第二 IID 的同版本声明和
   VariantStream 字段编码，不能将两分支的对象体等同。
4. 析构 `0x1006f380`（`ev_e875577a6b327914b8eb578236d58b2f997de33b345118e4afb82a50f434d31a`）
   遍历表项释放对象，销毁两张表和底层流。不能由此推定失败中途会回滚登记。

## HRESULT 与安全边界

汇编确认：空输出参数返回 E_POINTER；没有底层流返回 E_FAIL；全零 GUID 返回
S_FALSE（`0x100700f1`）；读取 GUID/键仅判断 HRESULT 符号，未比较实际读取量。
表命中调用 QI 后 `XOR EAX,EAX`，覆盖 QI 的 HRESULT；成功持久化分支同样返回
S_OK。创建或持久化失败的所见分支保留相应错误，释放局部引用；没有看到删除
已登记键的调用。这不是“任何失败都能重试”的保证。

AfCore AfStreamRead 已确认只转发 IStream::Read，见
[字符串研究](native-strings.md)，所以不能把上述符号判断称为严格短读检测。
现有 Rust 对象头读取器坚持完整读取且失败游标不变；其异常处理有意严格于原生。
原生重复键、失败后残留、未知对象和资源限制尚不足以形成完整 Rust 上下文。

本轮还检查 `+0x48` 的 `0x100704e0`，证据
`ev_45d7fa426383ebad0071f669b6f26c23868c2afccdf051ee3cff62a1cd144c39`：
它对调用方传入对象读 GUID、键并调持久化，不是通用重置入口；未据此添加 API。

替换对象的保存分支、第二接口/VariantStream 的完整编码、对象体边界、失败恢复、
版本属性来源、循环处理及同版本具体 Map/Layer 类型依然未闭环。
所有原始证据保存于忽略的 `temp/arcgis/systemcore-context-snapshot.json`，不提交 DLL。
