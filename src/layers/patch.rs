//! ArcMap 10.1 LinePatch / AreaPatch（线/面图例补丁）对象体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-legend-family.md`（2026-11-16 本轮研究记录）。
//! 二进制证据为 Layer.dll 10.1.0.3035（SHA-256 `fe52bd99…9efed8`，与既有
//! 基线一致）的静态反编译：两类的 IPersistStream 实现**共享同一对 Load/Save
//! 函数**（基类 Patch 的实现；LinePatch 位移 0x48 的 IPS 虚表
//! `0x7866e87c` 与 AreaPatch 位移 0x48 的 `0x7866e7a0` 函数指针完全相同：
//! Load thunk `0x78662610` → 实体 `0x78666b70`、Save thunk `0x78662730` →
//! 实体 `0x78666c40`），类的区别只在对象头 16 字节 CLSID 与 GetClassID
//! 写出（LinePatch `0x7865c390` 写出 `{2066267F-…}`、AreaPatch `0x7865c640`
//! 写出 `{2066267E-…}`，与 esri.clsid 声明模块 Layer.dll 三方互证；RTTI
//! `.?AVLinePatch@@`/`.?AVAreaPatch@@`）。仓库参考样本
//! `samples/测试底图.mxd` 两类 CLSID 0 命中（本轮全流扫描），全部布局结论
//! **无样本字节佐证**，单元测试全部使用合成输入。
//!
//! ## 字段序（Patch 共享体 Load `0x78666b70` / Save `0x78666c40`，对称）
//!
//! 1. int16 版本：汇编 `3 < (short)v` 有符号拒绝 `0x80004005`（负值位模式
//!    原生通过，本读取器一并拒绝，较原生更严）。Save 侧
//!    `AfStreamGetSaveVersion(docVer, 3, DAT_7866d954)`，版本表
//!    `[2,2,3]` → 10.1（docVer=5）恒写 3；
//! 2. `LoadObject`（请求 IID `{1B07790D-38C5-11D0-92D2-00805F7C28B0}` =
//!    IGeometry，类型库 esriGeometry）→ 对象 −8：补丁形状对象；LinePatch
//!    预期 Polyline、AreaPatch 预期 Polygon（类身份由流内 16 字节对象头
//!    决定；请求 IID 只约束接口）。本读取器对新对象按四类闭合几何
//!    （Polyline/Polygon/Point/Multipoint）分派整体消费，其余类停止；
//! 3. String（→ 对象 −0x10，补丁名称候选）；
//! 4. 版本 ≥2：int16（→ 对象 −0xc，语义未定，原样保留）；
//! 5. 版本 <3：原生调 `FUN_786664b0` 做装载后整理（不读流）——10.1 写侧
//!    恒写版本 3，该分支在 10.1 产物中不可达，读取器不模拟。
//!
//! 调用约定与事务性：调用者已用
//! [`crate::storage::object_stream::read_header_10_1`] 读取对象头、比对
//! [`LINE_PATCH_CLASS_ID`] 或 [`AREA_PATCH_CLASS_ID`] 并取得键，从对象头
//! 之后的位置传入 Reader；本模块不重复读取对象头。几何槽经
//! [`crate::storage::object_refs::read_object_entry_10_1`] 分类：空标记与
//! 回引消费后继续；新对象为闭合几何类时整体消费，其余类为停止点（游标
//! 回退到槽位起点、该键不登记，其后字段不可达）。任何失败不推进调用者
//! 游标、不修改登记表。`max_bytes` 约束名称字符串，`usize::MAX` 表示不
//! 额外设限。
//!
//! 未解决问题（详见研究记录）：名称 String 与 int16 的业务语义未定；
//! 版本 0/1/2 短体形态无字节佐证（Save 恒写 3）；补丁几何类分布未观察
//! （样本 0 实例）；全部结论为静态反编译观察，未经目标软件运行验证，
//! 不得声称已验证。

use crate::objects::geometry::{GeometryBody10_1, read_geometry_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// LinePatch（线补丁）类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{2066267F-E3B8-11D2-B868-00600802E603}`。证据：
/// esri.clsid.ecfg 声明（模块 Layer.dll，类名 LinePatch）；Layer.dll
/// GetClassID `0x7865c390` 写出同一 CLSID（`DAT_7866f240/7866f248`）。
/// 仓库参考样本 /Maps 流 0 命中。
pub const LINE_PATCH_CLASS_ID: [u8; 16] = [
    0x7f, 0x26, 0x66, 0x20, 0xb8, 0xe3, 0xd2, 0x11, 0xb8, 0x68, 0x00, 0x60, 0x08, 0x02, 0xe6, 0x03,
];

/// AreaPatch（面补丁）类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{2066267E-E3B8-11D2-B868-00600802E603}`。证据：
/// esri.clsid.ecfg 声明（模块 Layer.dll，类名 AreaPatch）；Layer.dll
/// GetClassID `0x7865c640` 写出同一 CLSID（`DAT_7866f230/7866f238`）。
/// 仓库参考样本 /Maps 流 0 命中。
pub const AREA_PATCH_CLASS_ID: [u8; 16] = [
    0x7e, 0x26, 0x66, 0x20, 0xb8, 0xe3, 0xd2, 0x11, 0xb8, 0x68, 0x00, 0x60, 0x08, 0x02, 0xe6, 0x03,
];

/// Patch 共享体 Load 接受的最大版本：汇编有符号 `3 < v` 拒绝；Save 版本表
/// `[2,2,3]` 在 10.1 恒写 3。
const MAX_PATCH_VERSION: i16 = 3;

/// 补丁几何槽（请求 IID `{1B07790D-…}` IGeometry）的分类结果。
///
/// 新对象为四类闭合几何（Polyline/Polygon/Point/Multipoint）时整体消费
/// 其体；其余类为停止点（体未知且无外部长度，游标回退到槽位起点）。
#[derive(Debug, Clone, PartialEq)]
pub enum PatchGeometrySlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新对象且类标识为闭合几何类：已读取其完整对象体。
    Geometry(GeometryBody10_1),
    /// 新对象但类标识非闭合几何类：停止点，游标回退到槽位起点、该键不
    /// 登记。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// Patch 体读取的停止原因（体内在首个不可闭合处停止，非错误）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchStop10_1 {
    /// 几何槽出现新对象且类体未闭合；游标已回退到该槽位起点。
    GeometryUnsupported { class_id: [u8; 16], key: u32 },
}

/// LinePatch/AreaPatch 共享对象体（Layer.dll Patch::Load `0x78666b70`）的
/// 已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct PatchBody10_1 {
    /// 对象头键（由调用者在分类新对象时按「登记先于持久化」登记；本
    /// 读取器不重复登记，仅登记嵌套槽位的新键）。
    pub key: u32,
    /// 体首 int16 版本的无符号位模式。原生有符号比较 >3 拒绝（负值原生
    /// 通过，读取器一并拒绝较原生更严）；Save 版本表 `[2,2,3]` 在 10.1
    /// 恒写 3。
    pub version: u16,
    /// 第 2 步几何槽（请求 IID IGeometry；→ 对象 −8）。
    pub geometry: PatchGeometrySlot10_1,
    /// 第 3 步补丁名称字符串（→ 对象 −0x10）。
    pub name: String,
    /// 版本 ≥2 的 int16 原样位模式（→ 对象 −0xc，语义未定）。
    pub extra_short: Option<u16>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<PatchStop10_1>,
    /// 停止点之后剩余的字节数（停止是成功返回）。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 10.1 LinePatch 或 AreaPatch 对象体（两类共享
/// 同一字段序），失败时不推进游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`LINE_PATCH_CLASS_ID`] 或
/// [`AREA_PATCH_CLASS_ID`] 并取得键 `key`。字段序与证据见模块文档。
/// `max_bytes` 约束名称字符串声明长度（`usize::MAX` 不设限）。几何槽
/// 停止为成功返回（游标回退到槽位起点）；截断、版本超限为错误，游标
/// 与登记表不动。
pub fn read_patch_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<PatchBody10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Patch 对象体 offset 0x{start:x}: {reason}"),
        )
    };

    // 1. 版本 short：原生有符号 `3 < v` 拒绝（负值位模式原生通过）。负值
    //    一并拒绝，较原生更严。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_PATCH_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出 Patch::Load 接受范围（有符号 >3 \
             拒绝 0x80004005；Save 版本表 DAT_7866d954 = [2,2,3]，10.1 恒写 3；负值位模式\
             原生通过，此处一并拒绝，较原生更严）"
        )));
    }

    // 2. 几何槽（LoadObject 请求 IID IGeometry）。空标记/回引消费后继续；
    //    新对象为闭合几何类时整体消费，其余类停止并回退到槽位起点。
    let geometry_offset = trial.position();
    let mut staged_table = table.clone();
    let geometry;
    let mut stop = None;
    {
        let snapshot = staged_table.clone();
        match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => geometry = PatchGeometrySlot10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key } => {
                geometry = PatchGeometrySlot10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                match read_geometry_body_10_1(&class_id, &mut trial, max_bytes, &mut staged_table) {
                    Ok(body) => geometry = PatchGeometrySlot10_1::Geometry(body),
                    Err(err) if err.kind() == ErrorKind::Unsupported => {
                        // 非闭合几何类：停止并回退（该键不保留）。
                        stop = Some(PatchStop10_1::GeometryUnsupported { class_id, key });
                        trial = reposition_to(reader, geometry_offset);
                        staged_table = snapshot;
                        geometry = PatchGeometrySlot10_1::Unsupported { class_id, key };
                    }
                    Err(err) => return Err(err),
                }
            }
        }
    }

    // 3. 补丁名称字符串（停止时不可达）。
    let mut name = String::new();
    let mut extra_short = None;
    if stop.is_none() {
        name = read_string_10_1(&mut trial, max_bytes)?;
        // 4. 版本 ≥2：int16 原样。
        if version >= 2 {
            extra_short = Some(trial.read_u16_le()?);
        }
        // 5. 版本 <3：原生装载后整理（不读流），读取器无对应动作。
    }

    let result = PatchBody10_1 {
        key,
        version,
        geometry,
        name,
        extra_short,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 把游标回退到入口窗口内的绝对偏移（Reader 无随机定位原语，用入口克隆
/// 前向读取实现；目标必在已读范围内）。
fn reposition_to<'a>(origin: &Reader<'a>, target: usize) -> Reader<'a> {
    let mut positioned = origin.clone();
    positioned
        .read_bytes(target - origin.position())
        .expect("回退目标在停止前的已读位置，必在窗口内");
    positioned
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 最小闭合 Polyline 体（版本 2 + 44+4+16 字节记录 + 16 字节空标记），
    /// 与 tests/geometry.rs 的样本形态构造同构。
    fn polyline_geometry_bytes() -> Vec<u8> {
        let record = {
            let mut bytes = 3_u32.to_le_bytes().to_vec(); // kind
            for value in [0.0f64, 0.0, 1.0, 1.0] {
                bytes.extend(value.to_le_bytes()); // envelope
            }
            bytes.extend(1_u32.to_le_bytes()); // 路径数
            bytes.extend(2_u32.to_le_bytes()); // 点数（原生共享解析器至少两点）
            bytes.extend(0_u32.to_le_bytes()); // 路径起点
            bytes.extend(0.5f64.to_le_bytes());
            bytes.extend(0.5f64.to_le_bytes()); // 点 1
            bytes.extend(0.25f64.to_le_bytes());
            bytes.extend(0.75f64.to_le_bytes()); // 点 2
            bytes
        };
        let mut bytes = 2_u16.to_le_bytes().to_vec();
        bytes.extend((record.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&record);
        bytes.extend_from_slice(&[0_u8; 16]); // 空间参考空标记
        bytes
    }

    /// 头（16 字节 CLSID + 键 9）+ 体：版本、几何槽、名称、int16。
    fn sample_body(geometry_class: &[u8; 16], version: u16, with_tail: bool) -> Vec<u8> {
        let mut v = Vec::new();
        // 对象头（16 字节类标识占位 + 键）；读取器从第 20 字节起读体。
        v.extend_from_slice(&[0xaau8; 16]);
        v.extend_from_slice(&9u32.to_le_bytes());
        v.extend_from_slice(&version.to_le_bytes());
        v.extend_from_slice(geometry_class);
        v.extend_from_slice(&9u32.to_le_bytes());
        v.extend_from_slice(&polyline_geometry_bytes());
        let name = "\u{8865}\u{4e01}\0".encode_utf16().collect::<Vec<_>>();
        v.extend_from_slice(&((name.len() * 2) as u32).to_le_bytes());
        for unit in name {
            v.extend_from_slice(&unit.to_le_bytes());
        }
        if with_tail {
            v.extend_from_slice(&7u16.to_le_bytes());
        }
        v
    }

    fn read_from(data: &[u8], _class_id: &[u8; 16]) -> io::Result<PatchBody10_1> {
        let mut table = RefTable10_1::default();
        let mut reader = Reader::new(data);
        reader.read_bytes(20).unwrap();
        read_patch_body_10_1(&mut reader, 4096, &mut table, 5)
    }

    #[test]
    fn reads_full_body_with_polyline_geometry() {
        let data = sample_body(&crate::objects::geometry::POLYLINE_CLASS_ID, 3, true);
        let body = read_from(&data, &LINE_PATCH_CLASS_ID).expect("线补丁体");
        assert_eq!(body.version, 3);
        assert_eq!(body.key, 5);
        assert!(matches!(
            body.geometry,
            PatchGeometrySlot10_1::Geometry(GeometryBody10_1::Polyline(_))
        ));
        assert_eq!(body.name, "\u{8865}\u{4e01}");
        assert_eq!(body.extra_short, Some(7));
        assert_eq!(body.stop, None);
        assert_eq!(body.remaining_bytes, 0);
    }

    #[test]
    fn area_patch_accepts_same_layout() {
        // AreaPatch 共享同一字段序（Patch::Load）；几何槽流内类标识同为
        // Polyline 也按闭合类消费（类身份由流内对象头决定）。
        let data = sample_body(&crate::objects::geometry::POLYLINE_CLASS_ID, 3, true);
        let body = read_from(&data, &AREA_PATCH_CLASS_ID).expect("面补丁体");
        assert!(matches!(
            body.geometry,
            PatchGeometrySlot10_1::Geometry(GeometryBody10_1::Polyline(_))
        ));
    }

    #[test]
    fn rejects_version_above_3_and_negative() {
        let data = sample_body(&crate::objects::geometry::POLYLINE_CLASS_ID, 3, true);
        // 体版本在 20 字节对象头之后（偏移 20）。
        let mut over = data.clone();
        over[20] = 4;
        over[21] = 0;
        assert!(read_from(&over, &AREA_PATCH_CLASS_ID).is_err());
        let mut negative = data.clone();
        negative[20] = 0xff;
        negative[21] = 0xff; // -1 位模式
        assert!(read_from(&negative, &AREA_PATCH_CLASS_ID).is_err());
    }

    #[test]
    fn unsupported_geometry_class_stops_at_slot_start() {
        let mut data = sample_body(&crate::objects::geometry::POLYLINE_CLASS_ID, 3, true);
        // 把几何槽类标识（偏移 22..38）换成未知类（全 0x55）。
        for byte in &mut data[22..38] {
            *byte = 0x55;
        }
        let body = read_from(&data, &AREA_PATCH_CLASS_ID).expect("停止是成功返回");
        match body.geometry {
            PatchGeometrySlot10_1::Unsupported { class_id, key } => {
                assert_eq!(key, 9);
                assert!(class_id.iter().all(|b| *b == 0x55));
            }
            other => panic!("应为 Unsupported，得到 {other:?}"),
        }
        assert_eq!(body.name, "");
        assert_eq!(body.extra_short, None);
        assert!(matches!(
            body.stop,
            Some(PatchStop10_1::GeometryUnsupported { .. })
        ));
    }

    #[test]
    fn truncation_leaves_reader_unmoved() {
        let data = sample_body(&crate::objects::geometry::POLYLINE_CLASS_ID, 3, true);
        for cut in 1..data.len() {
            let mut table = RefTable10_1::default();
            let mut reader = Reader::new(&data);
            reader.read_bytes(20).unwrap();
            let pos = reader.position();
            let result = read_patch_body_10_1(&mut reader, 4096, &mut table, 5);
            if result.is_err() {
                assert_eq!(reader.position(), pos, "截断 {cut} 处失败必须不推进游标");
            }
        }
    }

    #[test]
    fn short_versions_read_only_own_fields() {
        // 版本 1：无 int16 尾字段；版本 2：有。
        let without_tail = sample_body(&crate::objects::geometry::POLYLINE_CLASS_ID, 1, false);
        let body = read_from(&without_tail, &AREA_PATCH_CLASS_ID).expect("版本 1 体");
        assert_eq!(body.extra_short, None);
        assert_eq!(body.stop, None);
        assert_eq!(body.remaining_bytes, 0);

        let with_tail = sample_body(&crate::objects::geometry::POLYLINE_CLASS_ID, 2, true);
        let body = read_from(&with_tail, &AREA_PATCH_CLASS_ID).expect("版本 2 体");
        assert_eq!(body.extra_short, Some(7));
    }
}
