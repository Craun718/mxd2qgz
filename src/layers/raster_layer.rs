//! ArcMap 10.1 RasterLayer 对象体的已确认前缀。
//!
//! 证据来自 RasterLayer.dll 10.1 安装介质静态映像。文件 SHA-256 为
//! `9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6`，PE32
//! x86，ImageBase `0x7c710000`。主 QI `0x7c73aaa0` 将 IPersistStream IID
//! （`0x7c76778c`）映射到对象偏移 `+0x9c`；该偏移与 RTTI COL
//! `0x7c774088` 中的 `offset=0x9c` 一致。对应 IPersistStream 虚表
//! `0x7c76c2b0` 的 Load 槽 `+0x14` 指向 `0x7c73bd40`。
//!
//! Load `0x7c73bd40` 首先读有符号 int32 版本（`JG` 拒绝大于 17），随后
//! 读取第一条 AfCore String 到 persist-interface this+0x88。独立续段可再读
//! 第二串、定长短值、版本相关数量与首个 IName 对象头；对象体仍未闭合。

use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// RasterLayer 10.1 Load 的已确认前缀。
#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerPrefix10_1 {
    /// 有符号 int32 版本；原生 `JG` 只拒绝大于 17 的值，负值按原生门通过。
    pub version: i32,
    /// RasterLayer::Load 的首条 String，即 ILayer.Name（完整对象 +0x124）。
    pub first_string: String,
    /// 仅在外层已知边界内成功读取续段时填入。
    pub continuation: Option<RasterLayerContinuation10_1>,
    /// 首个 IName 为已支持的新 RasterDatasetName 时保留对象体的读取结果。
    pub dataset_name: Option<Box<crate::layers::raster_dataset_name::RasterDatasetNameBody10_1>>,
    /// 仅在 FileGDB 委派名称体完成后读取的 RasterIO 记录。
    pub fgdb_raster_io:
        Option<Box<crate::layers::fgdb_raster_dataset_name::FgdbRasterIoPrefix10_1>>,
    /// FileGDB 栅格 IName 的委派名称体。
    pub fgdb_dataset_name:
        Option<Box<crate::layers::fgdb_raster_dataset_name::FgdbRasterDatasetNamePrefix10_1>>,
}

/// RasterLayer 第二条 String、三个定长短值及 `0x7c73b380` 循环首对象头。
///
/// 对象体仍未解析；`first_object_header` 对应原生首个 `LoadObject(IName)`
/// 的空标记或 GUID 与键，不代表该对象已加载。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterLayerContinuation10_1 {
    /// Load 中的第二条 String，原生先读入栈上临时 String；业务含义未定。
    pub second_string: String,
    /// 按原生顺序紧邻的三个 short 原值；第一个是 ILayer.Visible。
    pub fixed_shorts: [u16; 3],
    /// 版本大于 8 时 `0x7c73b380` 读取的有符号数量；其余版本没有该流字段。
    pub object_count: Option<i32>,
    /// `0x7c73b380` 循环首个 `LoadObject` 的对象头；对象体及后续循环均未读取。
    pub first_object_header: Option<ObjectHeader>,
}

/// 从 RasterLayer 对象头之后读取 10.1 已确认前缀。
///
/// 成功后游标停在第二条 String 之前。`max_bytes` 限制首条 String 的声明
/// 长度；任何错误均不推进调用者游标。
pub fn read_raster_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<RasterLayerPrefix10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let version = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if version > 17 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "RasterLayer 对象体 offset 0x{start:x}: 版本 {version} 超出 Load 接受范围（有符号比较 >17 拒绝）"
            ),
        ));
    }
    let first_string = read_string_10_1(&mut trial, max_bytes)?;
    *reader = trial;
    Ok(RasterLayerPrefix10_1 {
        version,
        first_string,
        continuation: None,
        dataset_name: None,
        fgdb_raster_io: None,
        fgdb_dataset_name: None,
    })
}

/// 从首条 String 之后继续读 RasterLayer 10.1 的有界前缀。
///
/// 调用者须先成功调用 [`read_raster_layer_prefix_10_1`]，并传入其版本。
/// 成功后最多越过第二条 String、三个 short、版本相关数量和
/// `0x7c73b380` 循环首对象头；该对象体从返回游标处开始。主 Load 在该辅助函数
/// 返回后还会读取其他字段和另一个对象，本函数不推进到那些字段。任何错误都不
/// 推进输入。
pub fn read_raster_layer_continuation_10_1(
    reader: &mut Reader<'_>,
    version: i32,
    max_string_bytes: usize,
) -> io::Result<RasterLayerContinuation10_1> {
    let mut trial = reader.clone();
    if version > 17 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("RasterLayer 续段版本 {version} 超出 Load 接受范围（>17 拒绝）"),
        ));
    }
    let second_string = read_string_10_1(&mut trial, max_string_bytes)?;
    let fixed_shorts = [
        trial.read_u16_le()?,
        trial.read_u16_le()?,
        trial.read_u16_le()?,
    ];

    // RasterLayer::Load 0x7c73bd40 调 0x7c73b380；后者仅在版本 >8 时
    // 从流中覆盖初始数量 1。负数量按原生循环条件不进入首对象读取。
    let object_count = if version > 8 {
        Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
    } else {
        None
    };
    let has_first_object = match object_count {
        Some(count) => count > 0,
        None => true,
    };
    let first_object_header = if has_first_object {
        Some(read_header_10_1(&mut trial)?)
    } else {
        None
    };

    *reader = trial;
    Ok(RasterLayerContinuation10_1 {
        second_string,
        fixed_shorts,
        object_count,
        first_object_header,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded_string(text: &str) -> Vec<u8> {
        let payload: Vec<_> = text
            .encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect();
        let mut bytes = (payload.len() as u32).to_le_bytes().to_vec();
        bytes.extend(payload);
        bytes
    }

    #[test]
    fn reads_confirmed_raster_layer_prefix_and_stops_before_next_field() {
        let mut bytes = 17_i32.to_le_bytes().to_vec();
        bytes.extend(encoded_string("Raster title"));
        let prefix_len = bytes.len();
        bytes.extend(encoded_string("unparsed second string"));

        let mut reader = Reader::new(&bytes);
        let parsed = read_raster_layer_prefix_10_1(&mut reader, 0x1000).unwrap();

        assert_eq!(parsed.version, 17);
        assert_eq!(parsed.first_string, "Raster title");
        assert_eq!(reader.position(), prefix_len);
    }

    #[test]
    fn accepts_negative_version_bits_as_native_signed_gate_does() {
        let mut bytes = (-1_i32).to_le_bytes().to_vec();
        bytes.extend(encoded_string("legacy"));
        let mut reader = Reader::new(&bytes);

        let parsed = read_raster_layer_prefix_10_1(&mut reader, 0x1000).unwrap();

        assert_eq!(parsed.version, -1);
        assert_eq!(parsed.first_string, "legacy");
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn rejects_version_above_native_limit_without_advancing() {
        let bytes = 18_i32.to_le_bytes();
        let mut reader = Reader::new(&bytes);

        let error = read_raster_layer_prefix_10_1(&mut reader, 0x1000).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn every_truncated_prefix_rolls_back() {
        let mut bytes = 4_i32.to_le_bytes().to_vec();
        bytes.extend(encoded_string("landsat"));

        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(
                read_raster_layer_prefix_10_1(&mut reader, 0x1000).is_err(),
                "prefix length {end} must be rejected"
            );
            assert_eq!(reader.position(), 0, "prefix length {end} must roll back");
        }
    }

    #[test]
    fn oversized_first_string_is_rejected_without_advancing() {
        let mut bytes = 1_i32.to_le_bytes().to_vec();
        bytes.extend(encoded_string("too long"));
        let mut reader = Reader::new(&bytes);

        let error = read_raster_layer_prefix_10_1(&mut reader, 4).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn reads_second_string_fixed_shorts_count_and_only_first_object_header() {
        let mut bytes = encoded_string("unbound second string");
        bytes.extend([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        bytes.extend(1_i32.to_le_bytes());
        let header_start = bytes.len();
        bytes.extend([0x7a; 16]);
        bytes.extend(0x1234_u32.to_le_bytes());
        bytes.extend([0xaa, 0xbb]);

        let mut reader = Reader::new(&bytes);
        let parsed = read_raster_layer_continuation_10_1(&mut reader, 17, 0x1000).unwrap();

        assert_eq!(parsed.second_string, "unbound second string");
        assert_eq!(parsed.fixed_shorts, [0x2211, 0x4433, 0x6655]);
        assert_eq!(parsed.object_count, Some(1));
        assert_eq!(
            parsed.first_object_header,
            Some(ObjectHeader::Object {
                class_id: [0x7a; 16],
                key: 0x1234,
            })
        );
        assert_eq!(reader.position(), header_start + 20);
        assert_eq!(reader.read_bytes(2).unwrap(), &[0xaa, 0xbb]);
    }

    #[test]
    fn pre_v9_continuation_has_no_stream_count_before_first_object() {
        let mut bytes = encoded_string("second");
        bytes.extend([0; 6]);
        bytes.extend([0x5b; 16]);
        bytes.extend(9_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);

        let parsed = read_raster_layer_continuation_10_1(&mut reader, 8, 0x1000).unwrap();

        assert_eq!(parsed.object_count, None);
        assert_eq!(
            parsed.first_object_header,
            Some(ObjectHeader::Object {
                class_id: [0x5b; 16],
                key: 9,
            })
        );
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn zero_count_stops_before_any_nested_object() {
        let mut bytes = encoded_string("second");
        bytes.extend([0; 6]);
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend([0x5b; 20]);
        let mut reader = Reader::new(&bytes);

        let parsed = read_raster_layer_continuation_10_1(&mut reader, 17, 0x1000).unwrap();

        assert_eq!(parsed.object_count, Some(0));
        assert_eq!(parsed.first_object_header, None);
        assert_eq!(reader.position(), bytes.len() - 20);
    }

    #[test]
    fn continuation_rejects_version_above_native_limit_without_advancing() {
        let bytes = encoded_string("second");
        let mut reader = Reader::new(&bytes);

        let error = read_raster_layer_continuation_10_1(&mut reader, 18, 0x1000).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn truncated_continuation_rolls_back_at_every_boundary() {
        let mut bytes = encoded_string("second");
        bytes.extend([0; 6]);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0x5b; 20]);

        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_raster_layer_continuation_10_1(&mut reader, 17, 0x1000).is_err());
            assert_eq!(reader.position(), 0, "截断位置 {end} 应回滚");
        }
    }
}
