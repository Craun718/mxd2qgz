# 第三轮：Map 照明属性槽

10.1.0.3035 的 DisplayCore.dll SHA-256 为 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`。RTTI `IlluminationProps` TypeDescriptor `0x6ec3dc40` → COL `0x6ec20660` → IPersistStream 虚表 `0x6ec13f54`。虚表 +0x0c 经 thunk 到 GetClassID `0x6ebaa9c0`，从 `0x6ec13f84` 复制的 CLSID 为 `{1C352F40-298E-11D3-9F4F-00C04F6BC619}`，不同于 Map 请求的接口 IID `{1C352F3F-…}`。

Ghidra 桥接工具 6.9.0 与 Ghidra 12.2、JDK 25 定点复核了虚表 +0x14 的 Load `0x6ebab040`（Evidence `ev_54082760f2f7f086789a06c46207769c63a716813eb73914d357ec2e792809b3`）及 +0x18 的 Save `0x6ebab0b0`（Evidence `ev_9692e9e43926f85f649febfdf5c1f8b9057c81ae1056db7d401f79c870603a35`）。伪代码在本地 `temp/mxd-round3/`。结合 PE 导入和汇编，序列为 short 版本、24 字节 `_WKSPointZ`、8 字节 double；Load 有符号比较 >1 拒绝，读取器只接受 0/1。按 10.1 Map Load `0x793ff4d0` 与 Save `0x79401850`，照明槽跟在坐标变换集合之后。新读取逻辑只在变换计数为零并完整识别照明槽时继续，未知类和截断不消费该槽、不登记新键。

参考文件从 Map 根连续到 `/Maps:0x24fb2` 读取键 2178 的真实照明体，体首 `0x24fc6`、版本 1，随后 4 个 double，结束位置 `0x24fe8`。WKSEnvelope 仍位于 `0x24f64`；剩余字节保留。从根入口的参考样本回归与逐字节截断合成测试均通过。未解读照明最后一个 double 的业务语义，非空坐标变换和之后的 Map 字段仍开放；因此原 ID 113 暂不勾选。未运行 ArcMap 或 QGIS。
