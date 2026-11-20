//! ArcMap 10.1 TrackingWorkspace.dll 的 AMSDatasetName，证据见
//! `docs/mxd-terra-round9-bookmark.md`。

use crate::objects::workspace_name::{
    WORKSPACE_NAME_CLASS_ID, WorkspaceNamePrefix10_1, read_workspace_name_payload_10_1,
};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::native_string::read_atl_bstr_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const AMS_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0x67, 0xab, 0x77, 0xa6, 0xb8, 0x2f, 0xd5, 0x11, 0xb7, 0xe2, 0x00, 0x01, 0x02, 0x65, 0xad, 0xc5,
];

#[derive(Debug, Clone, PartialEq)]
pub struct AmsDatasetName10_1 {
    /// 按原生读写顺序保留四个 BSTR 的载荷及空指针区别。
    pub strings: [Option<Vec<u8>>; 4],
    pub scalars: [i32; 3],
    pub workspace_name: WorkspaceNamePrefix10_1<()>,
    pub trailing_short: i16,
}

pub(crate) fn read_ams_dataset_name_10_1(
    r: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<AmsDatasetName10_1> {
    let mut trial = r.clone();
    let mut staged = refs.clone();
    let strings = [
        read_atl_bstr_10_1(&mut trial, max_bytes)?,
        read_atl_bstr_10_1(&mut trial, max_bytes)?,
        read_atl_bstr_10_1(&mut trial, max_bytes)?,
        read_atl_bstr_10_1(&mut trial, max_bytes)?,
    ];
    let scalars = [
        trial.read_u32_le()? as i32,
        trial.read_u32_le()? as i32,
        trial.read_u32_le()? as i32,
    ];
    let offset = trial.position();
    let vt = trial.read_u16_le()?;
    if vt != 13 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "AMSDatasetName offset 0x{offset:x}: IWorkspaceName 需要 VT_UNKNOWN，实际为 {vt}"
            ),
        ));
    }
    // ReadClassStm 只读取类标识。WorkspaceName::Load 内部的对象槽仍沿用原流。
    let class = trial.read_bytes(16)?;
    if class == [0; 16] {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("AMSDatasetName offset 0x{offset:x}: IWorkspaceName 为空"),
        ));
    }
    if class != WORKSPACE_NAME_CLASS_ID {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("AMSDatasetName offset 0x{offset:x}: 未支持的 IWorkspaceName 类 {class:02x?}"),
        ));
    }
    let workspace_name = read_workspace_name_payload_10_1(&mut trial, max_bytes, &mut staged, ())?;
    if let Some(stop) = &workspace_name.stop {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("AMSDatasetName offset 0x{offset:x}: WorkspaceName 未读完（{stop:?}）"),
        ));
    }

    let trailing_short = trial.read_u16_le()? as i16;
    *r = trial;
    *refs = staged;
    Ok(AmsDatasetName10_1 {
        strings,
        scalars,
        workspace_name,
        trailing_short,
    })
}
