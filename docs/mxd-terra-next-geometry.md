# Geometry 与 CRS 续查（2026-11-17）

任务范围为 TODO 当前行 240、249、279–280。本记录不修改 TODO；样本类别条目需要真实样本或运行观察，静态代码和构造测试不能替代。

## 身份和取证

本轮计算 `temp/arcgis101-bin/geometrydll/Geometry.dll.F84366F9_C0B7_4267_A16E_D48D9E6DD261` 的 SHA-256 为 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`，与 [PCS/GCS 记录](arcmap-10.1-pcs-geometry-slots.md)的 ArcGIS 10.1.0.3035 文件一致；该文亦列明 sg.dll 10.1.0.679 摘要和与 Desktop 组件版本关系尚未核实的限制。

使用仓库内隔离目录 `temp/terra-next-rea` 并显式设置 TMPDIR，以 REA CLI 6.9.0 / Ghidra 12.2 对 Geometry.dll `0x719bec60` 尝试新的独立反编译。导入和自动分析完成，但桥接 Unix 套接字绑定返回 `Operation not permitted`，REA 以 `execution_failure` 退出；原始诊断保存在 `temp/terra-next-rea/gcs-load.json`。该尝试没有返回新的函数证据，本记录采用已有的版本绑定静态研究，不将失败当作原生运行观察。

## 逐项状态

| 行 | 状态 | 依据与剩余工作 |
| --- | --- | --- |
| 240 | 阻塞（运行态） | [Geometry 记录](arcmap-10.1-geometry.md)确认记录解码后经对象 `+0xc8` 指针虚表 `+0x1cc` 槽移交，未证明运行时接收对象身份、指针的装配和调用栈。需要 10.1 进程中捕获赋值、实际接收者与调用。 |
| 249 | 推进 | [PCS/GCS 记录](arcmap-10.1-pcs-geometry-slots.md)已给出域串的 UTF-16LE 字节布局、Unknown 四串第二/第三串成员覆盖、slot 8 恒写 0x38、GCS `+0xac` 非零守卫、XY 默认容差 `max(0.001 / 单位因子, 2 / XYscale)` 等静态事实。字符串业务名与填充来源、slot 成员名、守卫含义及完整 pe/sg 数值推导链仍未闭合；不据常量和样本猜测命名。 |
| 279 | 推进（静态及合成） | Polyline/Polygon 的 Load `0x7192db20`、Point `0x7191c1a0`、Multipoint `0x718ff9c0` 的版本门允许 0/1，v2 才读取空间参考；详见 [分支证据](mxd-parse-agent-sample-evidence-edge-branches-20261117.md)。新增独立测试覆盖四类 v0/v1 空记录、尾随字节、原有引用表和每个截断点。仍没有目标版本旧档或原生运行观察，不满足样本佐证条目。 |
| 280 | 阻塞（运行态） | 四类 Load 的版本读取返回 S_FALSE 时有独立快捷路径；普通字节读取器不能表达 COM IStream 的 HRESULT。需受控 IStream 与实际对象观察返回值、内嵌接口调用及状态；当前严格读取器不模拟此路径。 |

## 本轮验证

`tests/terra_next_geometry.rs` 的输入均为合成数据：v0/v1 的四类空记录在成功后恰消费 10 字节并保留后继字节，截断失败时游标及原有引用表不变。`cargo test --test terra_next_geometry` 与完整 `cargo test` 通过。`cargo fmt --check` 最终通过；本测试已单独 rustfmt，其他代理文件未改。未运行 ArcMap 10.1 或 QGIS，未进行人工目视验证。
