# ArcMap 10.1 LatLonFormat v4 末字段续查（2026-11-17）

## 身份与范围

重新读取 `temp/arcgis101-bin/SystemCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF`：SHA-256 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`；PE32 x86，资源 FileVersion/ProductVersion `10.1.0.3035`、InternalName `SystemCore`、OriginalFilename `SystemCore.dll`。版本为文件自述，哈希用于确认字节，均非签名认证。以既有 Ghidra 桥接工具 6.9.0/Ghidra 12.1.4 导出 `temp/pseudocode/SystemCore/functions/0x7ffc91b0.c`、`0x7ffc9880.c` 为定位线索，本轮对同一 PE 用 objdump 新读虚表、指令与常量，未切换共享 Ghidra 桥接工具 目标；伪代码不是源码。下述地址为静态映像 VA。

## 最后 short 的地址链和分支

令 O 为 LatLon 构造基址，P 为持久化接口指针，F 为格式化函数参数。

1. 构造 `0x7ffc91d5` 将首参数放入 esi；`0x7ffc91db` 写 O+14 的虚基表指针 `0x80070270`。该表 `0x80070274` 的值是 0x6c；`0x7ffc9274–927e` 在 O+14+0x6c = O+80 安装持久化虚表 `0x80070250`，`0x7ffc927f–9288` 将 P-4 写为 0。Load 虚表槽 `0x80070264` 指向调整器 `0x7ffca480`，按既有记录以 P 减 `[P-4]` 进入 `0x7ffca1e0`。因此本构造布局的 P=O+80。
2. Load `0x7ffca286–296` 比较版本与 3，版本大于 3 时将 P-8 和长度 2 传给流读取调用，地址即 O+78；`0x7ffca29b–2a8` 检查失败。Save `0x7ffca37b–395` 同样向流写 P-8 的 2 字节。构造函数将 O+78 初始化为 0；旧版本没有该流字段，不应将内存默认值当作流中字节。
3. `0x7ffc9235` 在 O+18 安装虚表 `0x8007011c`，其 +0x0c 槽 `0x80070128` 存 `0x7ffc9880`。该函数 `0x7ffc98ba` 将首栈参数 F 放入 esi；它对 `[esi+0x3c]` 的读取对应 O+54，和已对位的 ShowDirections 相符，因此 F=O+18，`[esi+0x60]` 即 O+78。这个虚表及同一函数中已知字段提供两条独立对位证据。
4. `0x7ffc9a85–8e` 对该 short 与 0 比较：非零走 `0x7ffc9a94–acf`，取秒的 double 除以 `0x8007e968` 处的 60.0，加整数分，再调用数字格式化函数 `0x7ffcdee0`；零则跳到 `0x7ffc9b98–bba`，按独立整数分和分隔字符串构造分量。`0x7ffc9c82–8d` 对同一成员再次比较：非零直接清除追加独立秒的标志，零仍受 ShowZeroSeconds 等条件控制。`0x7ffc98bd–98e6` 还表明非零时不启用 ShowDirections 对应的该路径方向标记。可据此定性为**小数分输出模式控制值**：零走独立分、秒路径，非零走分加秒/60 并不追加独立秒的路径。这里是静态分支，不保证所有输入的具体输出、区域设置、舍入或运行时状态。
5. 管理接口的进一步独立地址对位：QI `0x7ffc9565–957f` 对 IID `0x800702c0` 返回 O+4c；构造 `0x7ffc9266` 安装虚表 `0x80070220`，其 +0x0c 槽为 `0x7ffc9850`、+0x10 槽为 `0x7ffd7990`。前者 `0x7ffc9863–986a` 读取接口指针+0x2c，后者 `0x7ffd7993–799a` 写入接口指针+0x2c，均为 O+78。尚未取得此管理接口的可信属性名和槽序元数据，不将其猜称为某个正式属性。

## 九类完成边界

结合 [核心七字段访问器](mxd-terra-follow-display.md) 与 [派生字段对位](mxd-terra-integrate-numeric.md)，九类已确认持久化布局中的非版本标量均有静态定性：Numeric 七字段；Percentage 的 AdjustPercentage；Fraction 的 FractionOption/FractionFactor；Rate 的 RateFactor；LatLon 的 ShowDirections、IsLatitude、ShowZeroMinutes、ShowZeroSeconds 及本轮末字段的小数分模式；Angle 的 AngleInDegrees/DisplayDegrees；Scientific 的 DecimalPlaces。Currency 和 Custom 的已确认体无非版本标量；RateString、FormatString 是字符串，已在派生记录单独绑定。内嵌 Numeric 核心按自身字段计，不推断不同版本的运行状态。

本轮仅在 `number_format.rs` 补注解，保留 `short_08: Option<i16>` 原始值及旧版本 None；没有引入新限制或将所有非零值归一化。格式输出静态语义已闭合到上述范围，末字段正式属性名仍未知；短体真实字节、目标软件运行、内嵌核心实际状态、Angle 失败路径、ArcMap 及 QGIS 人工验收均未验证。未修改 TODO。

## 验证

`rustfmt --check --edition 2024 src/objects/number_format.rs`、`cargo test --test number_format`（19 项）和 `git diff --check -- src/objects/number_format.rs` 通过。`cargo fmt --check` 因本轮范围外的 `tests/terra_depth_media.rs` 导入格式不通过；`cargo test` 在既有/并行修改的 `tests/composite_graphics.rs::complete_sample_context_reads_cgl_sublayers_without_skipping` 失败，偏移实际 151620、预期 151572，其余后续测试未运行。Cargo 全局缓存自动清理因权限不足警告，未妨碍数字格式测试。合成测试不证明原生运行结果。
