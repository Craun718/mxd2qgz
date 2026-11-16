//! OLE 标准字体与标准图片对象（StdFont `{0BE35203-8F91-11CE-9DE3-00AA004BB851}`
//! / StdPicture `{0BE35204-8F91-11CE-9DE3-00AA004BB851}`）的 IPersistStream
//! 持久化体严格读取子集。
//!
//! 这两个实现类由 OLE 系统组件（oleaut32.dll）提供，不在 ArcGIS DLL 内，是
//! IFontDisp/IPicture 槽（TextSymbol、CharacterMarkerSymbol、Picture 系符号
//! 等处）最常见的流内实现类。证据来源分级（详见
//! `docs/arcmap-10.1-ole-stdfont.md`）：
//!
//! - **A 一手样本字节**：仓库参考样本 /Maps 流内 37 处 StdFont 新对象按本模
//!   块字段序逐字节全部吻合（37/37，走查脚本 `temp/agent-olestd/
//!   stdfont_walk_all.py`；其中 TextSymbol 字体槽 21 处、CharacterMarker
//!   系符号字体槽 16 处）。StdPicture 样本 0 实例，仅合成测试。
//! - **B 原生反编译（介质分发件）**：10.1 安装介质 `Desktop/Setup1.cab`
//!   条目 `Global_System_OLEAUT32_f3.8C0C59A0_7DC8_11D2_B95D_006097C4DE24`
//!   = Microsoft **OLEAUT32.DLL 2.40.4275**（OLE 2.40，PE x86，版本资源经
//!   pefile 读取；ImageBase `0x65340000`）：
//!   - `OleFontImpl::IPersistStream::Load` **`0x65399520`** / `Save`
//!     **`0x65399626`**（IPersistStream 虚表 `0x65379d48`，接口位于对象基
//!     址 +0xc；构造器 `0x653981bd`，`OleCreateFontIndirect` `0x653983c6`
//!     的 FONTDESC 字段打包式复核标志位语义；`DllGetClassObject` 分派
//!     `0x653a565d` 以常量 `{0BE35203}`@`0x65346680` 认类）；
//!   - `OlePictureImpl::IPersistStream::Load` thunk `0x653a0764` → 读头循
//!     环 **`0x653a0191`**；`Save` thunk `0x6539f34e` → `0x653a1183`（头
//!     `0x653a1071` + 按 PICTYPE 分派 1=BITMAP `0x653a0ba8`、4=ENHMETAFILE
//!     `0x653a0cab` 等）；IPersistStream 虚表 `0x6537a168`，接口位于对象基
//!     址 +0x8；构造器 `0x6539d4bc`（对象 0x9c 字节）。
//!   - 版本适用性：2.40.4275 是介质为旧系统分发的 OLE 2.40 组件（1999-2000
//!     年代）；ArcMap 10.1 运行时实际调用操作系统自带的 oleaut32（XP 为
//!     5.1.x 系列），本轮未提取 5.1 本体。「OLE 字体/图片持久化格式长期稳
//!     定」是假设，由 A 级 37 处样本字节支撑（样本由目标软件在含 OS oleaut32
//!     的系统上写出）。
//! - **C 公开文档**：MS-OAUT（learn.microsoft.com，OLE Automation Protocol，
//!   rev 22.0 PDF 可下载）公开描述字体/图片序列化；本轮未逐节核对其小节
//!   编号与字段表，不作为独立证据源（见文档「证据分级」）。
//!
//! 其他线索：同 cab 条目 `Global_System_OLEAUT32_f2.*` 经版本资源复核实为
//! ASYCFILT.DLL（条目名有误导性）；`Global_System_OLEPRO32_f0.*`
//! （5.0.4275）为 Win9x 时代图片对象载体，本轮未反编译（oleaut32 已含同等
//! 实现）。
//!
//! ## StdFont 体字段序（对象头 20 字节之后；确定的）
//!
//! | # | 类型 | 字段 | 证据 | 样本观测 |
//! | --- | --- | --- | --- | --- |
//! | 0 | BYTE | 版本：Save 恒写 1；Load 无符号 `>1` 拒绝（`E_FAIL`
//!   0x80004005）；`==0` 合法且跳过字段 1 | Load `0x65399520` `CMP AL,1`
//!   `JA` 拒绝；`TEST AL,AL` 条件读 | 37/37 = 1 |
//! | 1 | SHORT | charset（仅版本 ≥1；IFont `get_Charset`） | Load
//!   `Read(this+0x48,2)`，`this` = 对象基址+0xc、+0x48 = 基址+0x54（charset
//!   槽，`OleCreateFontIndirect` 从 FONTDESC.sCharset 写入同槽） | 134
//!   （GB2312_CHARSET，宋体）/ 0（ANSI_CHARSET，ESRI Default Marker） |
//! | 2 | BYTE | 标志位包：bit1=Italic、bit2=Underline、bit3=Strikethrough
//!   （bit0 与高位保留） | `OleCreateFontIndirect` `((fUnderline&1
//!   \|(fStrikethrough&1)<<1)<<1\|fItalic&1)<<1\|旧值&0xf1` 写基址+0x58；
//!   Save `MOV DL,[this+0x4c]`（= 基址+0x58） | 0（无斜体/下划线/删除线） |
//! | 3 | SHORT | weight（IFont `get_Weight`） | Save `MOV CX,[this+0x4a]`
//!   （= 基址+0x56，weight 槽） | 400（FW_NORMAL）/ 700（FW_BOLD） |
//! | 4 | INT32 | 字号 CY 的低 32 位（点数 ×10000；CY 高 32 位由 Load 恒置
//!   0，流内不含） | Save `MOV EDX,[this+0x38]`（= 基址+0x44 = CY.lo 槽）；
//!   Load `MOV [ESI+0x3c],0`（清 CY.hi） | 120000=12.0pt、82500=8.25pt、
//!   54000=5.4pt 等 |
//! | 5 | BYTE | 名字 ANSI 字节数：Load 无符号 `>0x20` 拒绝（E_FAIL） |
//!   Load `CMP AL,0x20` `JA` 拒绝 | 4（宋体）/ 19（ESRI Default Marker） |
//! | 6 | BYTE[n] | 名字 ANSI 字节，无结尾终止符；Load 后
//!   MultiByteToWideChar 转入 0x20 wchar 内联缓冲 | Load
//!   `MultiByteToWideChar(0,0,buf,n+1,this+0x4e,0x20)`（= 基址+0x5a 名字槽） | GBK
//!   「宋体」/ ASCII「ESRI Default Marker」 |
//!
//! 体长 = 11 + n 字节。Save（`0x65399626`）先以单次 `Write(11)` 写字段
//! 0-5（version+charset+flags+weight+size+名字节数），再以 `Write(n)` 写
//! 名字；Load 对称（1 + [2] + 8 字节块 [flags|weight|size|名字节数] + n）。
//! 体不含任何对象条目/键（无嵌套 LoadObject），注册表不参与。
//!
//! ## StdPicture 体字段序（对象头 20 字节之后；Load 路径确定的）
//!
//! | # | 类型 | 字段 | 证据 |
//! | --- | --- | --- | --- |
//! | 0 | WORD | 魔数 0x746c（LE `'lt'`）：Save 头恒写；Load `!=0x746c` 走
//!   旧格式分支（无头直接按图像魔数嗅探）——本读取器拒绝非 0x746c 形态
//!   （MXD 写侧恒为 Save 形态） | Load `0x653a0191` 循环读 2 字节比对
//!   `0x746c`；SaveHeader `0x653a1071` 写入 |
//! | 1 | WORD | 第二字（Save 写 0；语义未定，Load 不校验） | SaveHeader 第二
//!   个 `Write(4)` 的高半；wine 参考实现同样置 0（非本轮证据） |
//! | 2 | DWORD | 内嵌图像数据字节数；`==0` = 空图（Load 置 PICTYPE 0） |
//!   Load 读 4 字节后零值早退 |
//! | 3 | BYTE[n] | 图像数据。Load 按首 WORD 魔数分派解码器：`0x4D42`（'BM'）
//!   BITMAP、`0xFFD8` JPEG、`0x4749`/`0x6967` GIF、`0xCDD7`、`0x0001`、
//!   `0x0000`（空）；未知魔数拒绝 `0x800a01e1`。本读取器原样保留字节、
//!   不解释格式（解码超出 MXD 解析范围） | Load 内嵌分支魔数 switch |
//!
//! 样本 StdPicture 0 实例：本模块仅按反编译实现读取器，配合成测试；样本对
//! 位无从进行，未在目标软件验证。
//!
//! 调用约定与事务性：起点为对象头之后的第一字节（调用者已用
//! [`crate::storage::object_stream::read_header_10_1`] 读取并分类对象头、键
//! 已登记）。两个读取器均为叶子读取器（体不含嵌套对象条目）：失败不推进
//! 游标（trial 提交式）。`max_bytes` 约束名字字节数与图像数据声明长度。这
//! 是严格安全子集：不代表完整原生 Load 行为（如版本 0 的具体字段默认值、
//! 图像魔数校验等），通过不代表 MXD 格式已验证；全部结论未运行目标软件。

use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// OLE StdFont 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{0BE35203-8F91-11CE-9DE3-00AA004BB851}`。证据：oleaut32
/// 2.40.4275 数据常量 `0x65346680`（DllGetClassObject 分派 `0x653a565d`
/// 比对）与 esri.clsid 表互斥（OLE 系统类不在 Esri 类表内）；样本 /Maps 流
/// 37 处对象头逐字节相同。
pub const STDFONT_CLASS_ID: [u8; 16] = [
    0x03, 0x52, 0xe3, 0x0b, 0x91, 0x8f, 0xce, 0x11, 0x9d, 0xe3, 0x00, 0xaa, 0x00, 0x4b, 0xb8, 0x51,
];

/// OLE StdPicture 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{0BE35204-8F91-11CE-9DE3-00AA004BB851}`。证据：oleaut32
/// 2.40.4275 数据常量 `0x65346d68`（DllGetClassObject 分派与错误对象路径引
/// 用）。样本 /Maps 流 0 实例。
pub const STDPICTURE_CLASS_ID: [u8; 16] = [
    0x04, 0x0b, 0xe3, 0x0b, 0x91, 0x8f, 0xce, 0x11, 0x9d, 0xe3, 0x00, 0xaa, 0x00, 0x4b, 0xb8, 0x51,
];

/// StdPicture 持久化头的魔数字（LE `'lt'`）。
const STDPICTURE_MAGIC: u16 = 0x746c;

/// 原生 Load 接受的最大名字字节数（无符号 `>0x20` 拒绝 E_FAIL）。
const MAX_FONT_NAME_BYTES: u8 = 0x20;

/// 10.1 流内 StdFont 对象体（oleaut32 2.40.4275 `OleFontImpl::IPersistStream`
/// 持久化格式；样本 37/37 逐字节互证）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdFontBody10_1 {
    /// 版本字节（字段 0；Save 恒 1；Load 无符号 `>1` 拒绝；`==0` 合法且无
    /// charset 字段）。
    pub version: u8,
    /// charset（字段 1；仅版本 ≥1 存在；IFont `get_Charset`，如 0=ANSI、
    /// 134=GB2312）。原样位模式。
    pub charset: Option<u16>,
    /// 标志位包（字段 2）：bit1=Italic、bit2=Underline、bit3=Strikethrough，
    /// bit0 与高位保留（原样位模式；样本 37/37 = 0）。
    pub flags: u8,
    /// weight（字段 3；IFont `get_Weight`；样本 400/700）。
    pub weight: u16,
    /// 字号 CY 的低 32 位（字段 4；点数 ×10000，如 120000=12.0pt；CY 高 32
    /// 位不在流内，Load 恒置 0）。
    pub size_cy_lo: i32,
    /// 名字 ANSI 字节（字段 6；无结尾终止符；编码为写侧系统 ANSI 代码页——
    /// 样本观测 GBK「宋体」与 ASCII「ESRI Default Marker」，本读取器不解
    /// 码、原样保留）。
    pub name_bytes: Vec<u8>,
}

/// 10.1 流内 StdPicture 对象体（oleaut32 2.40.4275 `OlePictureImpl::
/// IPersistStream` 持久化格式；样本 0 实例，仅反编译证据 + 合成测试）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdPictureBody10_1 {
    /// 头第二字（Save 写 0，Load 不校验并继续；保留原始位模式）。
    /// 10.1 可用静态证据不能确定该字的业务语义。
    pub word2: u16,
    /// 内嵌图像数据声明字节数（`==0` = 空图；Load 据此早退）。
    pub data_size: u32,
    /// 图像数据原样字节（Load 按首 WORD 魔数分派解码器；本读取器不解释格
    /// 式，未知魔数也保留——解码超出 MXD 解析范围）。
    pub image_data: Vec<u8>,
}

/// 从对象头之后的位置读取 StdFont 对象体。
///
/// `max_bytes` 约束名字 ANSI 字节数（字段 5 声明长度）。失败不推进游标。
/// 版本超限、名字节数超限为 [`ErrorKind::InvalidData`]；截断为
/// [`ErrorKind::UnexpectedEof`]。
pub fn read_std_font_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<StdFontBody10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("StdFont 对象体 offset 0x{start:x}: {reason}"),
        )
    };

    // 0. 版本字节：原生无符号 >1 拒绝（E_FAIL 0x80004005）；本读取器同限。
    let version = trial.read_bytes(1)?[0];
    if version > 1 {
        return Err(invalid(format!(
            "版本 {version} 超出原生接受范围（Load 0x65399520 无符号比较 >1 \
             拒绝 E_FAIL）"
        )));
    }
    // 1. charset（仅版本 ≥1；版本 0 时原生跳过该字段）。
    let charset = if version >= 1 {
        Some(trial.read_u16_le()?)
    } else {
        None
    };
    // 2. 标志位包；3. weight；4. 字号 CY 低 32 位。
    let flags = trial.read_bytes(1)?[0];
    let weight = trial.read_u16_le()?;
    let size_cy_lo = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    // 5. 名字 ANSI 字节数：原生无符号 >0x20 拒绝（E_FAIL）。
    let name_len = trial.read_bytes(1)?[0];
    if name_len > MAX_FONT_NAME_BYTES {
        return Err(invalid(format!(
            "名字字节数 {name_len} 超出原生接受范围（Load 无符号比较 >0x20 \
             拒绝 E_FAIL）"
        )));
    }
    if usize::from(name_len) > max_bytes {
        return Err(invalid(format!(
            "名字字节数 {name_len} 超出调用者限额 {max_bytes}"
        )));
    }
    // 6. 名字 ANSI 字节（无终止符；原样保留不解码）。
    let name_bytes = trial.read_bytes(usize::from(name_len))?.to_vec();

    *reader = trial;
    Ok(StdFontBody10_1 {
        version,
        charset,
        flags,
        weight,
        size_cy_lo,
        name_bytes,
    })
}

/// 从对象头之后的位置读取 StdPicture 对象体。
///
/// `max_bytes` 约束图像数据声明长度。非 `0x746c` 魔数（旧格式无头形态）
/// 为 [`ErrorKind::InvalidData`]（MXD 写侧恒为 Save 头形态）；截断为
/// [`ErrorKind::UnexpectedEof`]。
pub fn read_std_picture_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<StdPictureBody10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("StdPicture 对象体 offset 0x{start:x}: {reason}"),
        )
    };

    // 0. 魔数字：非 0x746c = 旧格式无头形态，本读取器拒绝。
    let magic = trial.read_u16_le()?;
    if magic != STDPICTURE_MAGIC {
        return Err(invalid(format!(
            "魔数 0x{magic:04x} 非 0x746c（Load 0x653a0191 对该值走旧格式内\
             嵌嗅探分支；MXD 写侧恒为 Save 头形态，本读取器不接受）"
        )));
    }
    // 1. 第二字（Save 写 0；Load 不校验，原样保留）。
    let word2 = trial.read_u16_le()?;
    // 2. 图像数据声明字节数（==0 = 空图）。
    let data_size = trial.read_u32_le()?;
    if u64::from(data_size) > max_bytes as u64 {
        return Err(invalid(format!(
            "图像数据声明 {data_size} 字节超出调用者限额 {max_bytes}"
        )));
    }
    // 3. 图像数据原样字节（Load 按首 WORD 魔数分派解码器，本读取器不解码）。
    let image_data = trial.read_bytes(data_size as usize)?.to_vec();

    *reader = trial;
    Ok(StdPictureBody10_1 {
        word2,
        data_size,
        image_data,
    })
}
