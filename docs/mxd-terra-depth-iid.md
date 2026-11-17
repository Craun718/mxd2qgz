# FeatureLayer 版本 ≥17 的请求 IID 复核

2026-11-17。对 ArcMap 10.1.0.3035 介质提取的 x86 FeatureLayer.dll 重算 SHA-256：
`2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`，与
[上次身份记录](mxd-terra-follow-feature.md)一致。地址为静态映像 VA。

复核既有隔离 Ghidra 导出的 `temp/terra-follow-feature/0x6f6e9940.c` 并检查原始 PE：

- `0x6f6ea995..0x6f6ea99b` 将有符号版本与 `0x11` 比较，小于则跳过。
- `0x6f6ea9a0` 将 IID 地址 `0x6f7ada24` 入栈；紧接着取得 `EBX+0x238`
  的输出地址，再于 `0x6f6ea9b1` 调用 LoadObject。
- 映像 `0x6f7ada24` 的原始字节是
  `9b d5 14 17 22 fb d1 11 94 a2 08 00 09 ee be cb`，
  即 `{1714D59B-FB22-11D1-94A2-080009EEBECB}`，与
  [Map 对象取证表](arcmap-10.1-map-object.md)及 [FeatureLayer 后继段㉙](arcmap-10.1-featurelayer-tail.md)
  所记的 IUID 槽一致。此处确认的是请求 IID，流内 coclass 仍由对象头决定。

因此，曾归到此槽的 `{97D5B914-FB22-11D1-94A2-080009EEBECB}` 不属于这条读取路径；
不需要为该错误候选新增解析分支。补充有界字面扫描在本地候选筛出的 6267 个 PE 或类型库
文件中未检出旧候选完整 GUID 或其首四个小端字节。这不是穷尽所有 ArcGIS 安装件，
也不证明该 GUID 在别处无定义。今后只有发现独立的实际调用点时，才需要另行追踪身份。
未运行目标程序，未改读取逻辑。
