//! ArcMap 10.1 `Variant::Read` 对象分支（VT_DISPATCH 9 / VT_UNKNOWN 13）的
//! 显式子集读取器：类 GUID 为零的「空对象」编码。
//!
//! 证据来源：`docs/arcmap-10.1-variant-propertyset-slots.md`（本轮 D 轮研究
//! 记录）；目标实现为 **AfCore.dll 10.1.0.3035**（`temp/arcgis101-bin/` 既有
//! 基线，SHA-256 `031858c4…cc47c17`；SystemCore.dll 经 IAT 按名导入同一导出
//! `?Read@Variant@@QAEJPAUIStream@@_N@Z`，故本分支对 PropertySet 值同样适
//! 用，见该文档「值序列化器身份」节）。
//!
//! ## 原生编码（证据：确定的）
//!
//! `Variant::Read`（`0x68a7c6a0`）的 9/13 分支（`0x68a7ca4d`）调用 AfCore 导
//! 出 `LoadObject`（`0x68a7b6d0`），实参为（出参 this+8、IID 常量
//! `0x68c6bc2c` = IID_IUnknown、流、空 IUnknown）。LoadObject 入口先对流查
//! 询 ESRI 流上下文接口（QI IID `0x68c6dcb8`，`0x68a7c4b0`）；命中则走带键
//! ObjectStream 路径（上下文对象虚表 +0x44）。未命中（MXD 内普通 OLE 存储
//! 流，即本仓库读取模型假设的情形）走普通路径核心 `0x68a7b490`：
//!
//! 1. 读 16 字节类 GUID（`cbRead < 0x10` 返回 `0x80004005`）；
//! 2. 与全零 GUID 常量 `0x68c6bc4c` 比较（`0x68a775f0`，逐 4×dword 相等返
//!    回 1）——**相等（零 GUID）→ 返回 S_OK 且出参保持 NULL**；**不等 →
//!    `AfCreateInstance(&读到的类 GUID, 空, 0x17, IID_IUnknown, 出参)` 创建
//!    实例**（注意：该极性与
//!    `docs/arcmap-10.1-variant-stream.md` 的表述相反，属旧记录笔误，本轮
//!    汇编复核修正）；
//! 3. 非零 GUID 的实例创建成功后，尾部包装 `0x68a7b530` 将流与对象关联
//!    （对象体由该对象自己的持久化方法继续消费）——**对象体边界由类决定，
//!    静态无通用边界**，本读取器不支持。
//!
//! 写侧对称：`Variant::Write`（`0x68a7cb50`）9/13 分支调 `SaveObject(obj,
//! stream, 0)` → `StrmUtil::SimpleSaveObject`（`0x68a7af70`）在对象为空时
//! 直接写 16 字节全零 GUID。故流内编码为 **`[VARTYPE:2][16×00]`**（对象指
//! 针为空时），且无论读路径走普通路径还是带键上下文路径，零 GUID 的消费量
//! 都是恰 16 字节、不产生键——该子集与流上下文无关，可安全闭合。
//!
//! ## 适用范围
//!
//! 仅接受 VARTYPE 9（VT_DISPATCH）或 13（VT_UNKNOWN）且类 GUID 为全零；
//! 非零类 GUID 显式 `Unsupported`（需要 COM 实例化与类相关体边界，不在本
//! 仓库范围）。本函数独立于 [`crate::objects::property_set::read_variant_10_1`]
//! （不改现有读取器），建议接入点为其 `9 | 13` 分支（整合轮落位）。全部字
//! 节在 trial 游标上读取，失败不推进调用者游标。单元测试只覆盖可构造输
//! 入，不代表原生运行已验证。

use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// 对象分支已闭合子集的读取结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantObjectBranch10_1 {
    /// 类 GUID 为全零：原生 `LoadObject` 返回 S_OK 且对象指针为空（无对象）。
    Null,
}

/// 读取一个 `Variant::Read` 对象分支（VARTYPE 9/13）值，失败时不推进游标。
///
/// 仅闭合「类 GUID 全零 → 空对象」子集：流内为 2 字节 VARTYPE + 16 字节
/// 全零类 GUID，共 18 字节。VARTYPE 非 9/13 为调用方分派错误（InvalidData）；
/// 类 GUID 非零为未闭合的 COM 实例化分支（Unsupported，错误信息携带 16 字
/// 节原样字节序供诊断）。
pub fn read_variant_object_branch_10_1(
    reader: &mut Reader<'_>,
) -> io::Result<VariantObjectBranch10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Variant 对象分支 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let value_type = trial.read_u16_le()?;
    if value_type != 9 && value_type != 13 {
        return Err(invalid(format!(
            "VARTYPE {value_type} 不属于对象分支（仅 9/13 走 LoadObject）"
        )));
    }
    let class_id = trial.read_bytes(16)?;
    if class_id != [0u8; 16] {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!(
                "Variant 对象分支 offset 0x{start:x}: 类 GUID {:02x?} 非全零，走 \
                 AfCreateInstance 实例化且对象体边界由类决定，本读取器不支持",
                class_id
            ),
        ));
    }
    *reader = trial;
    Ok(VariantObjectBranch10_1::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    fn reader(bytes: &[u8]) -> Reader<'_> {
        Reader::new(bytes)
    }

    #[test]
    fn 空对象_vt9_十六字节零_guid() {
        let mut bytes = vec![9, 0];
        bytes.extend_from_slice(&[0u8; 16]);
        bytes.extend_from_slice(&[0xAB, 0xCD]);
        let mut r = reader(&bytes);
        let value = read_variant_object_branch_10_1(&mut r).expect("零 GUID 空对象应成功");
        assert_eq!(value, VariantObjectBranch10_1::Null);
        assert_eq!(r.position(), 18);
        assert_eq!(r.remaining(), 2);
    }

    #[test]
    fn 空对象_vt13_同构() {
        let mut bytes = vec![13, 0];
        bytes.extend_from_slice(&[0u8; 16]);
        let mut r = reader(&bytes);
        assert_eq!(
            read_variant_object_branch_10_1(&mut r).expect("VT_UNKNOWN 空对象应成功"),
            VariantObjectBranch10_1::Null
        );
        assert_eq!(r.position(), 18);
    }

    #[test]
    fn 非零类_guid_显式不支持且不推进游标() {
        // 与 PROPERTY_SET_CLASS_ID 同构的非零类 GUID（诊断用，非真实编码）。
        let mut bytes = vec![9, 0];
        bytes.extend_from_slice(&[
            0x11, 0x5a, 0x8e, 0x58, 0x9b, 0xd0, 0xd1, 0x11, 0xaa, 0x7c, 0x00, 0xc0, 0x4f, 0xa3,
            0x3a, 0x15,
        ]);
        let mut r = reader(&bytes);
        let before = r.position();
        let error = read_variant_object_branch_10_1(&mut r).expect_err("非零 GUID 应拒绝");
        assert_eq!(error.kind(), ErrorKind::Unsupported);
        assert!(
            error.to_string().contains("11, 5a, 8e, 58"),
            "错误应携带类 GUID：{error}"
        );
        assert_eq!(r.position(), before, "失败不得推进游标");
    }

    #[test]
    fn vartype_非对象分支_调用方分派错误() {
        for vt in [0u16, 1, 8, 0x13, 0x2009] {
            let mut bytes = vt.to_le_bytes().to_vec();
            bytes.extend_from_slice(&[0u8; 16]);
            let mut r = reader(&bytes);
            let before = r.position();
            let error = read_variant_object_branch_10_1(&mut r).expect_err("非 9/13 应拒绝");
            assert_eq!(error.kind(), ErrorKind::InvalidData, "VARTYPE {vt}");
            assert_eq!(r.position(), before);
        }
    }

    #[test]
    fn 截断各前缀_失败且不推进游标() {
        // 仅 vt 无 GUID；vt + 15 字节 GUID（短读 1 字节）。
        let mut truncated = vec![9, 0];
        truncated.extend_from_slice(&[0u8; 15]);
        let mut r = reader(&truncated);
        let before = r.position();
        let error = read_variant_object_branch_10_1(&mut r).expect_err("截断应失败");
        assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
        assert_eq!(r.position(), before);

        let mut r = reader(&[9]);
        let before = r.position();
        let error = read_variant_object_branch_10_1(&mut r).expect_err("仅 1 字节应失败");
        assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
        assert_eq!(r.position(), before);
    }

    #[test]
    fn 全零但_末尾保留_失败后游标不变() {
        // 16 字节零但 VARTYPE 载荷为其他值：仅验证失败事务性。
        let bytes = [13, 0, 1, 2, 3, 4];
        let mut r = reader(&bytes);
        let before = r.position();
        let error = read_variant_object_branch_10_1(&mut r).expect_err("截断应失败");
        assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
        assert_eq!(r.position(), before);
    }
}
