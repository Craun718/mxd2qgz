# 第八轮：空间参考接口与守卫增量证据

日期：2026-11-17。仅记录本轮新增证据，既有对象体布局及 GCS 守卫的位置、
错误路径见 [上一轮](mxd-terra-integrate-sr.md)。以下均为静态观察，未运行
ArcMap，也未在 ArcMap 或 QGIS 人工检查。

## 证据身份和方法

- Desktop 安装介质声明 10.1.3035（见 README）。本轮重算提取的 PE32 x86
  Geometry.dll SHA-256 为
  `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`；
  VS_FIXEDFILEINFO 与文件资源的 FileVersion/ProductVersion 均为 10.1.0.3035，
  InternalName 为 Geometry、OriginalFilename 为 Geometry.DLL。这是身份声明与
  摘要，不是签名验证。
- 介质的 esriGeometry.olb 既有提取物 SHA-256 为
  `d45fb2105c869cda034fa80101cf2d0b932e08ea6f31eca8d4322fe7f7cd3e8e`，
  已提取的类型库资源 SHA-256 为
  `341646ec9ef602e82aa75072a116bd3468a6b023475b827dbd418481e97d9409`。
  复核既有 Wine `LoadTypeLibEx` 类型名/GUID 转储；这是类型库元数据，不是
  ArcMap 运行观察。
- 仓库 `third_party/ForestRight/vendor/ESRI.ArcGIS.Geometry.dll` 的 SHA-256 为
  `87335492b11917c32e1a08b383cd3912f57c73dd2e80308f4f2193b67ce97603`；
  PE 文件版本和产品版本声明 10.1.0.3035，ProductName 声明由 esriGeometry
  类型库导入。以 .NET `System.Reflection.Metadata` 只读枚举类型、方法签名和
  自定义属性；没有加载或执行该程序集。程序集不是从上述介质重新提取的，
  因此以下结合原生虚表及介质类型库的同 IID 证据，不把文件版本单独作为同源证明。
- 原生反编译采用独立 pyghidra 工程副本、Ghidra `/opt/ghidra`、JDK 25 和
  仓库专用临时目录；未切换或修改共享 Ghidra 桥接工具 会话。汇编另用 objdump 核对。
  临时探针及输出在 `temp/terra-round8-sr/`，仅为可重做的辅助材料。

## ISRPersistStream slot 8 的成员名

互操作程序集的 `ISRPersistStream` 类型携带 GUID
`{E88C7DB0-9E84-11D2-BD00-0000F875BCCE}`，与介质类型库中该接口 IID 及
[原生 QI/虚表绑定](arcmap-10.1-pcs-geometry-slots.md)一致。程序集元数据方法
依次为 `GetClassID`、`IsDirty`、`Load`、`Save`、`GetSizeMax`、
**`GetPrecisionSizeMax`**、`LoadPrecision`、`SavePrecision`；前面还有 IUnknown
的三个槽。因而对应原生虚表 slot 8 的名称为 **GetPrecisionSizeMax**，slot 9/10
分别与 LoadPrecision/SavePrecision 对应。程序集元数据中该方法签名字节
`20 01 01 10 08` 表示实例方法、一个按引用的 int32 参数及 void 返回；这是
.NET 互操作签名，不能直接当作原生 HRESULT 声明。

PCS 表 `0x71a29ccc` 的 slot 8 位于 `0x71a29cec`，GCS 表 `0x71a26818` 的
slot 8 位于 `0x71a26838`，均指向 `0x719bf440`。该函数对第二个栈参数指向
的 4 字节写 `0x38`，EAX 置零并 `ret 8`；它未检查输出指针。函数仅返回固定
值，不据此断言其他版本、其他实现或任意对象的实际流长度。本轮只闭合该成员名，
未将 56 字节解释为含标志或容差的完整对象体长度。

## GCS +0xc4 的接口关系与初始化

介质类型库中 `{3B219F24-D3FF-11D1-BC9B-0000F875BCCE}` 是
**IGeographicCoordinateSystemEdit**；互操作程序集同名类型的 GUID 与之吻合，
其方法为 `DefineEx`、`Define`。结合上一轮 GCS QI 的同 IID 比较与 +0xc4 条件，
现在可将该条件精确表述为：**+0xc4 非零时，针对
IGeographicCoordinateSystemEdit 的 QI 分支拒绝提供该接口，且持久化 Load 在
读流前失败**。这证明条件与编辑接口暴露及 Load 相关，但不证明字段名称或为什么
必须拒绝。

新增构造路径：`0x719c3330` 的 `0x719c3383` 将 EAX 清零，`0x719c354d` 和
`0x719c355d` 将该零值分别写到完整对象 +0xbc 和 +0xc4；`0x719c3610` 的复制
构造路径也将新对象 +0xc4 设零，而复制源的 +0xc8 到新对象 +0xc8。加上上一轮
的默认构造及析构证据，这些路径不能说明所有可能的写入者，也不能排除经别名或
外部代码赋非零。非零状态产生路径、字段类型和业务含义仍未闭合，不能据此给
Rust 流读取器添加一个从流内推断的守卫。

## 域串的新边界

GCS Save `0x719bf34c..0x719bf353` 将持久化接口 this+0x8c 的指针放入 EAX，
随后调用四串写入器 `0x719bad00`；换算为完整对象是 +0xb8。对象构造器
`0x719bab40` 初始化该指向结构的 +4、+8、+0xc、+0x10 四个 BString 为空，
并在 +0x14 保存传入的坐标系对象参数。由此确认四串属于 +0xb8 所指的共享
内部结构，而不是 GCS 完整对象 +0xc4 守卫；仍未找到四个 getter 的可靠身份或
填充来源。该静态关系不证明所有共享或复制时的生命周期，也不确定四串业务名。

## 可勾范围和验证

可独立闭合的是「ISRPersistStream slot 8 成员名」；「GCS 守卫语义」仅部分
推进到编辑接口身份与构造初始化，域串业务名/getter 来源仍开放。现有 TODO 将
这些事项与其他问题合在同一条，该整条不能勾选；本轮按要求未修改 TODO。

本轮只增加研究记录，没有更改 Rust 实现或格式解析，因此不声称新增解析能力。
已核对文件摘要、PE 版本、接口元数据、原生汇编与独立 pyghidra 反编译；相关
方法与条件均未在 ArcMap 运行验证，QGIS 转换结果也未人工验证。
