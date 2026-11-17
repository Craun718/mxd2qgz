# AnnoLayer 主 QI 的接口与偏移更正

2026-11-17。静态目标为 ArcMap 10.1.0.3035 的 x86 AnnoLayer.dll；重算 SHA-256
`b487f3e0dc0e70dbc4b75f72eb6dcd8d5521eb422dbea6e3b2e201489a2d33a5`，
与[原生身份记录](mxd-iid-research-20261116.md)一致。本轮重新反汇编
`0x6b1a1890..0x6b1a1a45` 并读取各比较目标的 16 字节，核对命中后的实际指针加数。
类型库名称来自既有 LoadTypeLibEx 枚举的精确 GUID 匹配，并复核 CAB 提取 OLB 摘要：
esriCarto `b95e4a043c5776c41cee5601be0400a77a61cb1f4475b8bd2a5c43f4893cc51c`、
esriDisplay `5f64ecc7511ca3a8174bab83a57f550d3703a5c96131262771ad1a2e78ad3660`、
esriGeoDatabase `09b741f2627633d36b67985e175632055fe6c5b8cd1000484acc3007f5917f9d`。

| IID 原始地址 | GUID | 接口名（类型库索引） | 返回成员 |
| --- | --- | --- | --- |
| `0x6b1ac8d8` | DBCA59AC-6771-4408-8F48-C7D53389440C | IAnnotationSublayer（esriCarto 604） | +0x28 |
| `0x6b1aa36c`、`0x6b1aa35c` | 00000109、0000010C（标准 COM 尾） | IPersistStream、IPersist | 均 +0x40 |
| `0x6b1ac8c8` | 34C20004-4D3C-11D0-92D8-00805F7C28B0 | IGeoDataset（esriGeoDatabase 19） | +0x30 |
| `0x6b1abe60` | 34C20002-4D3C-11D0-92D8-00805F7C28B0 | ILayer | +0x2c |
| `0x6b1ac8b8` | 12E361F0-1907-11D3-80C7-0080C79F0371 | IDisplayAdmin（esriDisplay 51） | +0x34 |
| `0x6b1abf00` | EEF01B2D-8FE1-4759-8B93-51EE22DED70A | ILayerDrawingPhase（esriCarto 103） | +0x38 |
| `0x6b1abf10` | 2425B413-1FFC-4407-B9F0-B4AC9ED50BDE | ILayerGeneralProperties（esriCarto 105） | +0x3c |
| `0x6b1abd10` | C9E7A782-3DDC-411F-8A66-908C579F123D | 未定名，既有类型库枚举无命中 | +0x44 |

命中地址和加数逐条对应，不能把接口名单与下一分支的加数相配。
未命中路径仍委派给基类；本轮没有穷尽其接口、实际运行状态或方法副作用。
未改读取规则；未运行 ArcMap 或 QGIS。

补充核查：该末项的 16 字节在若干原生 DLL 和 OLB 的 PE 外层中命中，
但以 pefile 遍历同一介质提取目录的全部 79 份 TYPELIB 资源后没有命中。
故 PE 外层命中不能当作公开类型库定义，也不能凭包含该常量的模块给它命名。
