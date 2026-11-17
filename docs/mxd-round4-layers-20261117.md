# 第四轮：图层与标注

本表按本轮开始时 TODO 的实际行号裁决；“部分”包含已有静态证据但缺少该行要求的真实字节、属性绑定或穷尽性证据。没有改动 TODO。所有结论限定于现有证据，不把构造输入当成实际工程，也未运行 ArcMap 或 QGIS。

## 身份和本轮复核

本轮核对 `temp/agent-element-coll/work/Layer.dll`：PE x86，版本资源 FileVersion/ProductVersion `10.1.0.3035`，OriginalFilename `Layer.DLL`，SHA-256 `fe52bd9988b66f9d6f906b014e5684f28fc20e9b97b53c90909fa3d44e9efed8`，与 [既有记录](arcmap-10.1-featurelayer-tail.md) 的摘要前缀、版本一致。独立 REA 6.9.0/Ghidra 对 `0x7865b520` 的静态反编译证据 `ev_9eba4ed3e68ecea5cb2c473fa31fb20ce8305839be80d0eaf15aaa4b7cc97544`：有符号版本大于 4 拒绝；读取 short，版本大于 1 再读 short，随后 String 和对象槽；版本大于 2 读 short 和 int32，版本大于 3 再读两个 short。该观察支持现有读取器的版本门，不能证明旧版真实字节或这些字段业务语义。完整证据输出在忽略目录 `temp/mxd-round4/layers/fieldinfo-load-final.json`，含本机路径，分享前应检查脱敏。

首次 REA 服务的 Ghidra 使用 `/tmp`，因空间耗尽未完成；随后将独立进程的 TMPDIR 指向本组目录。独立进程在沙箱中因 Unix 套接字拒绝而失败，取得执行授权后成功；失败结果不作为原生证据。

已检查仓库参考样本与两套介质的现有提取、索引和第三轮对象级记录。10.1 介质的 66 个模板可用；此前对两套模板的解析仍存在未知对象停止点，不能由未到达的字段推断不存在。10.8 介质的 SouthernUSA 已由完整宿主链定位到 11 个 FieldInfo v3，文件保存版本为 9.3；这不证明 v0–2。参见 [图层续研](mxd-layer-label-cgl-20261117.md) 和 [介质清单](mxd-install-media-samples-20261117.md)。本机有 Wine，但未发现已安装可用的 ArcMap 10.1 或已连接 Windows 目标；未以 Wine 的存在推断目标可运行。

## 原行裁决

| 原行 | 状态 | 证据及剩余工作 |
| --- | --- | --- |
| 130 | 部分 | GroupLayer 10.1 Load/Save 的分支已有静态记录；尚无从对象流边界定位的各版本真实体，见图层续研。 |
| 157 | 部分 | VBScript 体已实现；JScript/Python 的类型库登记不证明其持久化体，仍需 GetClassID、虚表与 Load/Save 或有界不支持证据。 |
| 158 | 部分 | 请求 IID 已定为 IGraphicsContainer，参考槽为空；缺非空实现类及其对象体。 |
| 159 | 部分 | 既有符号族和 TextSymbol 已分派；尚未穷尽其他 ISymbol 实现。 |
| 216 | 部分 | 既有 10.1 Load 观察到读取值被默认函数返回值覆盖，默认函数实现与用途仍未定；见 [第三轮显示研究](mxd-round3-display-spatial-20261117.md)。 |
| 226 | 部分 | FDO 旧加载器字段序已有静态记录，接口槽与业务属性未逐项绑定。 |
| 227 | 部分 | ALPC 已有结构与公开接口列表，标量及字符串未逐项绑定访问器。 |
| 228 | 部分 | LELP 读取顺序及部分类型库方法已知，具体持久化成员与访问器仍需逐项关联。 |
| 229 | 部分 | ETM 读取结构、Count/EditTemplate 接口已知，尾字节及其余标量语义尚未闭合。 |
| 230 | 部分 | 第三轮 REA 已证明 Load 与 IFeatureIDSet::Add 操作同一集合；没有证据把所有集合整数等同于数据源 OID。本轮加强数量边界检查，不改变字段语义。 |
| 253 | 部分 | FCN 非零 TopologyName 选择子的静态实现及读取器已有，真实非零对象未从宿主链定位。 |
| 254 | 部分 | FDO 旧体版本分支已有静态规则和读取器，缺真实旧体。 |
| 259 | 部分 | 本轮重新核验 FieldInfo Load 的版本门，新增 v0–2 正常及截断合成测试；真实 v3 有介质字节，v0–2 未获真实字节。 |
| 276 | 部分 | UID/DEP 有原生 Load 证据，BOLP/HLEP 仍缺 GetClassID、持久化虚表与 Load 的唯一关联；参见 [第三轮标注记录](mxd-round3-labels-featurelayer-20261117.md)。 |
| 279 | 部分 | 后继段版本 0x1c..0x1f 的静态门及读取器已有，缺可定位的旧版本对象体。 |
| 280 | 部分 | TimeData 四段有静态规则和合成测试，参考样本为四个空槽；缺真实非空链。 |
| 294 | 部分 | FeatureLayer `+0x1f4` 是 IEnvelope，已知传向数据源接口；未定正式属性及生命周期，见 [空间研究](mxd-parse-agent-map-layer-spatial-20261117.md)。 |
| 295 | 部分 | 已知保存空指针及样本空槽行为；构造初始化、加载后生命周期和运行时副作用未闭合，见同一空间研究。 |

## 代码、测试和接线

`src/layers/feature_id_set.rs` 在遍历前检查非负计数乘四的溢出与剩余长度；负计数仍按原生静态门跳过循环，错误保留游标与引用表。`tests/round4_layers.rs` 测试非空、负数、极大计数、所有截断及 FieldInfo v0–2 合成分支。无需修改公共 mod/lib 或共享接线。

`cargo test --test round4_layers` 两项通过；`cargo test` 通过（部分既有测试按各自条件忽略）。`cargo fmt --check` 在共享工作区因其他组的 `src/maps/map_object.rs` 格式差异失败，本组文件已使用 rustfmt 格式化，未修改该共享文件。没有 ArcMap 10.1 或 QGIS 人工打开和视觉验收；这些测试不证明目标软件互操作结果。
