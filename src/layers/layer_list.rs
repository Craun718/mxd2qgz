//! ArcMap 10.1 Map 图层列表（`FUN_794035f0`，架构版本 ≥0x17 分支）的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-map-object.md`「Map::Save 方向、ExtraSafe 前缀与
//! 图层列表样本闭合（2026-11-14 第四轮）」。读侧 Map.dll `FUN_794035f0`
//! （ver≥0x17，样本 ver=51 全走此支）：int32 计数 → 逐条目 int16 标志 →
//! 标志 1 走普通 `LoadObject`（16 字节类标识 + 4 字节键，新对象后跟体，体无
//! 外部长度字段）；标志 2/3 走 AfCore `ExtraSafeReadObject`（`0x68a7be60`）：
//! 先读 8 字节前缀 `[A:int32][B:int32]`，`A==0 && B==0` 为空条目（原生返回 1、
//! 跳过），否则 `LoadObject`，`B!=0` 时原生 Seek 到「前缀后位置 + B」强同步。
//! 标志为其他值时原生返回 `0x80004005`（汇编 `0x794036b2`：`CMP AX,1/2/3`）。
//! 循环后读 int32 活动图层索引（写侧初值 -1，样本 -1 @/Maps:0x16fb4）。
//! 条目分类（新对象/回引/空标记）经 [`crate::storage::object_refs`] 的会话键登记表。
//!
//! 样本逐条目闭合（`samples/测试底图.mxd` /Maps 导出，辅助观察）：0x274 计数 6；
//! 0x278 标志 1 FDOGraphicsLayer 键 31；0x2d22 标志 2 前缀 [20][0] 回引键 32；
//! 0x2d40 标志 2 [5490][0] 新对象键 158；0x42bc [3318][0] 键 218；0x4fbc
//! [62941][0] RasterLayer 键 259；0x145a3 [10761][0] RasterLayer 键 1544；
//! 0x16fb6 活动索引 -1。样本吻合为辅助观察，未在目标软件中验证。
//!
//! 样本核对结论（2026-11-14，`temp/agent-layer-list/` 临时核对程序逐字节验证）：
//!
//! - 完整 /Maps 喂入后缀读取器在条目 1（标志 1 新对象）停止，剩余尾部起点
//!   0x28e 恰为该条目体起点（与证据文档「体 0x28e..0x2d22」一致）；
//! - 从 0x2d22 起合成前导（计数 5）读取条目 2..6：每条目按「体长 = A - 20」
//!   跳过后恰好落在下一条目标志处，活动索引读出 -1，A 推定 6/6 吻合；
//! - 与证据文档条目表的两处出入（以样本字节为准）：条目 4 前缀为 `f6 0c`
//!   = **3318**（文档表格十进制「3324」为笔误，其十六进制一致）；活动索引
//!   实际位于 **0x16fb6**（文档表格与任务摘要记 0x16fb4；0x16fb6 恰为条目 6
//!   按 A 推定的终点，0x16fb4 与之矛盾，`ff ff ff ff` 后紧接 CGL 头 0x16fba）。
//!
//! ## 有界未知体的跳过依据
//!
//! 体前缀经类分派解析后仍有未闭合剩余（或类没有读取器）时，按范围跳过：
//!
//! - `B != 0`：跳到「前缀后位置 + B」。这是 `ExtraSafeReadObject` 原生 Seek 的
//!   确定行为（反编译证据）。
//! - `B == 0` 且 `A >= 20`：按「体长度 = A - 20」跳过。**观察推定**：`A` 的读侧
//!   消费在原生代码中未见，写侧 `ExtraSafeWriteObject`（`0x68a7bd70`）把 8 字节
//!   零占位回填为 `[A][B]`，样本 6/6 条目满足 A = 头 20 字节 + 体总体积、B = 0。
//!   该推定未经原生运行验证，仅用于本子集的有界跳过。
//! - `B == 0` 且 `A < 20`：与上述观察矛盾（写侧 A ≥ 头 20 字节），按数据错误拒绝。
//!
//! ## 体前缀的类分派（已接入）
//!
//! 新对象体按流内类标识分派给已实现读取器解析已确认前缀（见
//! [`parse_body_prefix`]）：FeatureLayer → `feature_layer`、GroupLayer →
//! `group_layer`、CompositeGraphicsLayer → `composite_graphics`、
//! FDOGraphicsLayer → `fdo_graphics_layer`（2026-11-14 第六轮闭合，
//! AnnoLayer.dll `0x6b1955a0`，样本条目 1 的类）。RasterLayer 仅在
//! ExtraSafe 条目界内读取已确认前缀；无读取器的类按有界未知跳过。体前缀解析失败
//! （版本超限、嵌套类不支持等）不使整条列表失败：按有界未知降级并在条目
//! 上保留失败原因；真实截断仍会因有界跳过超出流长度而整体失败。标志 1 的
//! 新对象体解析出前缀后游标停在前缀终点（体剩余无外部长度，列表停止）；
//! 未解析出前缀时停在对象头之后。
//!
//! FDOGraphicsLayer 条目（样本条目 1，标志 1）解析出前缀后，其两个内嵌
//! FeatureLayer 体（关联图层 + 委派注记图层）各自的剩余仍无外部长度，列表
//! 照样停在该条目——越过该条目需要 FeatureLayer 渲染器/字段信息等尾段链
//! 的逐类闭合（开放，见 `docs/arcmap-10.1-map-object.md` 第六轮）。
//!
//! ## 跳过体键未登记对回引判定的影响
//!
//! 按范围跳过的体字节内部含有的对象头键不会进入登记表（原生会经各对象自
//! 身的 Load 登记）。后续条目回引这些键时，本子集按「未见过键 = 新对象」的
//! 原生对偶语义误分类——但标志 2/3 条目的边界来自 A/B 前缀而非体解析，误分
//! 类不影响后续条目对位（样本条目 2 对键 32 的回引即属此情况：A=20、体长
//! 0）；仅诊断标签（新对象/回引）与可提取信息受影响。标志 1 条目无前缀，
//! 本子集在首个标志 1 新对象体处即停止，不受此影响。
//!
//! ## 未解决问题
//!
//! - `A` 前缀的读侧消费未见（仅有写侧回填与样本观察，见上）；
//! - 写侧标志 1（`SaveObject` 路径）的触发状态未逐分支定性（样本条目 1 为标志 1，
//!   其余为标志 2）；
//! - 标志 2 与 3 对读侧无差别（仅写侧按 docVersion 选择，10.1 docVersion=3 时
//!   写 2），本读取器按相同路径处理并原样保留标志值；
//! - 图层列表元素的业务语义（绘制顺序、图层树展开方式）不在本模块范围。

use crate::layers::cad_layer::{
    CAD_ANNOTATION_LAYER_CLASS_ID, CAD_FEATURE_LAYER_CLASS_ID, CAD_LAYER_CLASS_ID,
    CadAnnotationLayer10_1, CadFeatureLayer10_1, CadLayer10_1, read_cad_annotation_layer_10_1,
    read_cad_feature_layer_10_1, read_cad_layer_10_1,
};
use crate::layers::composite_graphics::read_composite_graphics_layer_prefix_10_1;
use crate::layers::coverage_annotation_layer::{
    COVERAGE_ANNOTATION_LAYER_CLASS_ID, CoverageAnnotationLayer10_1,
    read_coverage_annotation_layer_10_1,
};
use crate::layers::fdo_graphics_layer::{
    FDO_GRAPHICS_LAYER_CLASS_ID, FdoGraphicsLayerPrefix10_1, read_fdo_graphics_layer_prefix_10_1,
};
use crate::layers::feature_layer::read_feature_layer_prefix_10_1;
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

// 类标识常量由归属模块定义（证据注释见各模块），此处重导出仅为保持本模块
// 既有引用路径（mxd_maps / 测试）稳定，避免双源。
pub use crate::layers::composite_graphics::COMPOSITE_GRAPHICS_LAYER_CLASS_ID;
pub use crate::layers::feature_layer::FEATURE_LAYER_CLASS_ID;
pub use crate::layers::group_layer::GROUP_LAYER_CLASS_ID;

/// 标志 1：普通 `LoadObject` 条目（无 ExtraSafe 前缀）。
const FLAG_LOAD_OBJECT: u16 = 1;
/// ExtraSafe 标志下界：标志 2（写侧 docVersion<4 或图层指针为空时取 2）。
const FLAG_EXTRA_SAFE_LO: u16 = 2;
/// ExtraSafe 标志上界：标志 3（写侧 docVersion>=4 时取 3；10.1 docVersion=3 样本全为 2）。
const FLAG_EXTRA_SAFE_HI: u16 = 3;
/// 对象头长度：16 字节类标识 + 4 字节键。
const OBJECT_HEADER_BYTES: usize = 20;
/// 单个条目的最小字节数：2 字节标志 + 8 字节全零前缀（ExtraSafe 空条目）。
const MIN_ENTRY_BYTES: usize = 10;

/// RasterLayer 类标识 `{D02371C9-35F7-11D2-B1F2-00C04F8EDEFF}` 的文件字节序。
pub(crate) const RASTER_LAYER_CLASS_ID: [u8; 16] = [
    0xc9, 0x71, 0x23, 0xd0, 0xf7, 0x35, 0xd2, 0x11, 0xb1, 0xf2, 0x00, 0xc0, 0x4f, 0x8e, 0xde, 0xff,
];

use crate::layers::graphics_3d::{
    GLOBE_GRAPHICS_LAYER_CLASS_ID, GRAPHICS_LAYER_3D_CLASS_ID, Graphics3dLayer10_1,
    read_graphics_3d_layer_10_1,
};

use super::cache_tile_reader::{TiledLayer10_1, is_tiled_layer, read_tiled_layer_10_1};
use super::wcs_layer::{WCS_LAYER_CLASS_ID, WcsLayer10_1, read_wcs_layer_body_10_1};

/// 类标识的 32 字符小写十六进制串（诊断用）：实现移至
/// [`crate::storage::class_id_hex`]（消除 objects→layers 依赖倒置），此处保
/// 留原名重导出。
pub use crate::storage::class_id_hex;

/// 新对象体经类分派解析出的已确认前缀。
///
/// 分派依据是流内对象头的类标识（原生不按 GUID 分派、由类表创建，此处按
/// 已实现读取器的类集合分派）；各读取器只覆盖其已确认前缀，前缀之后的体
/// 字段仍为未知（有 A 前缀的条目按范围跳过，见 [`UnknownBodyRange`]）。
#[derive(Debug, Clone, PartialEq)]
pub enum LayerBodyPrefix10_1 {
    /// FeatureLayer 体前缀（名称、第二名称、可见性、数据源 FCN/WorkspaceName
    /// 链、渲染器槽位或 +0x144 GUID 停止）。
    FeatureLayer(Box<crate::layers::feature_layer::FeatureLayerPrefix10_1>),
    /// GroupLayer 体前缀（名称、三个 short、两个 double、子图层循环、可见性
    /// 等版本条件字段，到首个不可闭合字段为止）。
    GroupLayer(Box<crate::layers::group_layer::GroupLayerPrefix10_1>),
    /// 使用内嵌组图层提供子层的地理处理图层。
    ProcessLayer(Box<super::process_layer::ProcessLayer10_1>),
    /// CompositeGraphicsLayer 体前缀（元素组、名称、标量组、子图层循环，到
    /// 首个不可闭合字段为止）。
    CompositeGraphicsLayer(
        Box<crate::layers::composite_graphics::CompositeGraphicsLayerPrefix10_1>,
    ),
    /// FDOGraphicsLayer 体前缀（版本、int32/short 标量、关联要素图层
    /// ILayer 槽、IElementCollection2 槽、委派 FeatureLayer 体，第六轮闭合）。
    FdoGraphicsLayer(Box<FdoGraphicsLayerPrefix10_1>),
    /// Scene 或 Globe 图形层完整对象体。
    Graphics3d(Box<Graphics3dLayer10_1>),
    NaLayer(crate::layers::na_layer::NaLayerPrefix10_1),
    /// 有界 RasterLayer 的已确认前缀。
    RasterLayer(crate::layers::raster_layer::RasterLayerPrefix10_1),
    WcsLayer(Box<WcsLayer10_1>),
    TiledLayer(Box<TiledLayer10_1>),
    CadLayer(Box<CadLayer10_1>),
    CadFeatureLayer(Box<CadFeatureLayer10_1>),
    CadAnnotationLayer(Box<CadAnnotationLayer10_1>),
    CoverageAnnotationLayer(Box<CoverageAnnotationLayer10_1>),
    /// 有界 ImageServerLayer 的 IPersistVariant 体，可能在未知对象前停止。
    ImageServerLayer(Box<crate::layers::image_server_layer_tail::ImageServerLayerBody10_1>),
    /// 有界 BasemapLayer 的结果对象与子图层回引。
    BasemapLayer(crate::layers::basemap_layer::BasemapPrefix10_1),
}

impl LayerBodyPrefix10_1 {
    /// 体前缀内的图层名称（诊断与模型填充用）。
    ///
    /// FDOGraphicsLayer 自身无名称字段：取关联要素图层（ILayer 槽）的名称，
    /// 关联槽为空/回引时取委派子对象（注记要素图层）的名称，两者皆无时为
    /// 空串。
    pub fn layer_name(&self) -> &str {
        match self {
            Self::FeatureLayer(prefix) => &prefix.name,
            Self::GroupLayer(prefix) => &prefix.name,
            Self::ProcessLayer(body) => &body.group.name,
            Self::CompositeGraphicsLayer(prefix) => &prefix.name,
            Self::Graphics3d(body) => &body.name,
            Self::NaLayer(body) => &body.name,
            Self::TiledLayer(body) => body.name(),
            Self::CadLayer(body) => &body.name,
            Self::CadFeatureLayer(body) => body.feature.as_ref().map_or("", |base| &base.name),
            Self::CadAnnotationLayer(body) => {
                body.annotation.as_ref().map_or("", |base| base.name())
            }
            Self::CoverageAnnotationLayer(body) => body.name(),
            Self::RasterLayer(prefix) => &prefix.first_string,
            Self::WcsLayer(body) => body
                .raster
                .body
                .as_ref()
                .map_or("", |raster| &raster.first_string),
            Self::ImageServerLayer(body) => &body.prefix.name,
            Self::BasemapLayer(prefix) => &prefix.name,
            Self::FdoGraphicsLayer(prefix) => match &prefix.associated_layer {
                crate::layers::fdo_graphics_layer::AssociatedLayer10_1::FeatureLayer(layer) => {
                    &layer.name
                }
                _ => prefix
                    .delegated_layer
                    .as_ref()
                    .map(|layer| layer.name.as_str())
                    .unwrap_or(""),
            },
        }
    }
}

/// 图层列表条目的对象部分分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerEntryObject {
    /// ExtraSafe 前缀 `A==0 && B==0`：空条目，无对象头、无对象体（原生返回 1 并跳过）。
    Empty,
    /// 实际读取的对象条目，经 [`crate::storage::object_refs`] 分类（空标记 / 回引 / 新对象）。
    Entry(ObjectEntry10_1),
}

/// 有界未知体的跳过依据。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyBoundBasis {
    /// `B != 0`：跳到「前缀后位置 + B」，即 `ExtraSafeReadObject` 的原生强同步
    /// （反编译确定行为）。
    NativeSeek,
    /// `B == 0`：按「体长度 = A - 20」跳过。观察推定（写侧占位回填 + 样本 6/6
    /// 吻合），未经原生运行验证。
    PrefixAInference,
}

/// 新对象体按范围跳过的有界未知区（偏移相对传入 Reader 的底层字节）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownBodyRange {
    /// 体起点（对象头 20 字节之后）。
    pub offset: usize,
    /// 体长度。
    pub len: usize,
    /// 跳过依据，见 [`BodyBoundBasis`]。
    pub basis: BodyBoundBasis,
}

/// 单个图层列表条目的解析结果与原始字段。
#[derive(Debug, Clone, PartialEq)]
pub struct LayerListEntry {
    /// 条目序号（0 起，即列表内的绘制顺序位置候选）。
    pub index: usize,
    /// 条目起点偏移，相对传入 Reader 的底层字节。
    pub offset: usize,
    /// 2 字节标志原始值（1/2/3）；无标志的旧版条目记为 0。
    pub flag: u16,
    /// ExtraSafe 8 字节前缀 `[A][B]` 原始位模式；标志 1 无前缀，为 `None`。
    pub prefix: Option<(u32, u32)>,
    /// 对象部分分类（空条目 / 空标记 / 回引 / 新对象）。
    pub object: LayerEntryObject,
    /// 新对象条目被有界跳过的未知体范围；其余条目为 `None`。
    ///
    /// 体前缀经 [`Self::parsed`] 解析后，该范围只覆盖前缀终点到条目终点的
    /// 未解析剩余（可能长度为 0）；长度为 0 表示体前缀恰好覆盖到条目边界。
    pub body: Option<UnknownBodyRange>,
    /// 新对象体经类分派解析出的已确认前缀；无对应读取器或解析失败时为
    /// `None`（失败原因见 [`Self::parse_note`]）。
    pub parsed: Option<LayerBodyPrefix10_1>,
    /// 体前缀解析失败的诊断说明（解析成功或未分派时为 `None`）。
    pub parse_note: Option<String>,
}

impl LayerListEntry {
    /// 单行诊断描述：标志、前缀 A/B、分类、类标识十六进制、键、体前缀与有界
    /// 体范围。
    pub fn describe(&self) -> String {
        let mut parts = vec![if self.flag == 0 {
            "旧版无标志条目".to_string()
        } else {
            format!("标志 {}", self.flag)
        }];
        if let Some((a, b)) = self.prefix {
            parts.push(format!("前缀 A={a} B={b}"));
        }
        match &self.object {
            LayerEntryObject::Empty => parts.push("空条目（A=0 且 B=0）".to_string()),
            LayerEntryObject::Entry(ObjectEntry10_1::Null) => {
                parts.push("空标记（16 零类标识）".to_string())
            }
            LayerEntryObject::Entry(ObjectEntry10_1::BackReference { class_id, key }) => {
                parts.push(format!("回引 类={} 键 {key}", class_id_hex(class_id)));
            }
            LayerEntryObject::Entry(ObjectEntry10_1::NewObject { class_id, key }) => {
                parts.push(format!("新对象 类={} 键 {key}", class_id_hex(class_id)));
            }
        }
        match &self.parsed {
            Some(LayerBodyPrefix10_1::TiledLayer(body)) => {
                parts.push(format!("瓦片图层对象体已解析：名称 {:?}", body.name()));
            }
            Some(LayerBodyPrefix10_1::CadLayer(body)) => {
                parts.push(format!(
                    "CadLayer 对象体已解析（版本 {}，名称 {:?}）",
                    body.version, body.name
                ));
            }
            Some(LayerBodyPrefix10_1::WcsLayer(body)) => {
                parts.push(format!("WCSLayer 对象体已解析：版本 {}", body.version));
            }
            Some(LayerBodyPrefix10_1::RasterLayer(prefix)) => {
                parts.push(format!("RasterLayer 体前缀已解析：版本 {}", prefix.version));
                if let Some(continuation) = &prefix.continuation
                    && let Some(crate::storage::object_stream::ObjectHeader::Object {
                        class_id,
                        key,
                    }) = &continuation.first_object_header
                {
                    parts.push(format!(
                        "首个 IName 对象头 类={} 键 {key}",
                        class_id_hex(class_id)
                    ));
                }
                use crate::layers::raster_dataset_name::RasterDatasetNameEntryBody10_1;
                for name in &prefix.names {
                    match &name.body {
                        Some(RasterDatasetNameEntryBody10_1::Dataset(_)) => {
                            parts.push("RasterDatasetName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::Cache(_)) => {
                            parts.push("CacheRasterDatasetName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::WcsCoverage(_)) => {
                            parts.push("WCSCoverageName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::Dataset83 { .. }) => {
                            parts.push("RasterDatasetName83 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::Band(_)) => {
                            parts.push("RasterBandName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::BandCollection(_)) => {
                            parts.push("RasterBandCollectionName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::Catalog(_)) => {
                            parts.push("RasterCatalogName 的委派名称体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::Mosaic(_)) => {
                            parts.push("MosaicDatasetName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::Database(database)) => {
                            use crate::layers::raster_dataset_name::ACCESS_RASTER_DATASET_NAME_CLASS_ID;
                            let name = if database.class_id == ACCESS_RASTER_DATASET_NAME_CLASS_ID {
                                "AccessRasterDatasetName"
                            } else {
                                "SdeRasterDatasetName"
                            };
                            parts.push(format!("{name} 对象体已解析"));
                        }
                        Some(RasterDatasetNameEntryBody10_1::NetCdf(_)) => {
                            parts.push("NetCDFRasterDatasetName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::ImageService(_)) => {
                            parts.push("ISRasterDatasetName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::ImageServerReference(_)) => {
                            parts.push("ImageServerName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::Function(_)) => {
                            parts.push("FunctionRasterDatasetName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::InternetTiledLayer(_)) => {
                            parts.push("InternetTiledLayerName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::AisImageService(_)) => {
                            parts.push("AISImageServiceName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::RasterType(_)) => {
                            parts.push("RasterTypeName 对象体已解析".into());
                        }
                        Some(RasterDatasetNameEntryBody10_1::FileGeodatabase { .. }) => {
                            parts.push("FgdbRasterDatasetName 的委派名称体已解析".into());
                            parts.push("FgdbRasterDatasetName 的 RasterIO 已读到结束记录".into());
                        }
                        None => {}
                    }
                }
                if let Some(stop) = &prefix.names_stop {
                    parts.push(format!("RasterLayer 名称列表未解析完：{stop}"));
                }
                if let Some(renderer) = &prefix.renderer {
                    use crate::layers::raster_renderer::RasterRendererPrefix10_1::{
                        Classify, Colormap, Discrete, LegacyRgb, Rgb, Sai, Simple, Stretch,
                        UniqueValue,
                    };
                    parts.push(match &renderer.body {
                        Some(Colormap { version, stop, .. }) => match stop {
                            Some(stop) => format!("色表渲染器 v{version} 前缀已解析：{stop}"),
                            None => format!("色表渲染器 v{version} 对象体已解析"),
                        },
                        Some(Rgb {
                            version,
                            bands,
                            stop,
                            ..
                        }) => match stop {
                            Some(stop) => format!(
                                "RGB 渲染器 v{version} 前缀已解析，波段索引 {bands:?}：{stop}"
                            ),
                            None => {
                                format!("RGB 渲染器 v{version} 对象体已解析，波段索引 {bands:?}")
                            }
                        },
                        Some(LegacyRgb(body)) => format!(
                            "RGB 渲染器 v{} 对象体已解析，波段索引 {:?}",
                            body.version, body.bands
                        ),
                        Some(Simple { version, .. }) => {
                            format!("SimpleRasterRenderer v{version} 对象体已解析")
                        }
                        Some(Sai { version, .. }) => {
                            format!("SAIRenderer v{version} 对象体已解析")
                        }
                        Some(Discrete(body)) => format!(
                            "离散色渲染器 v{} 对象体已解析，{} 个颜色值",
                            body.version,
                            body.colors.len()
                        ),
                        Some(UniqueValue(body)) => format!(
                            "栅格唯一值渲染器 v{} 对象体已解析，{} 个类别",
                            body.version,
                            body.classes.len()
                        ),
                        Some(Classify(body)) => format!(
                            "栅格分类渲染器 v{} 对象体已解析，{} 个断点",
                            body.version,
                            body.breaks.len()
                        ),
                        Some(Stretch(body)) => format!(
                            "栅格拉伸渲染器 v{} 对象体已解析，波段索引 {}",
                            body.version, body.band
                        ),
                        None => format!("栅格渲染器对象槽 {:?}", renderer.entry),
                    });
                }
                if let Some(stop) = &prefix.renderer_stop {
                    parts.push(format!("栅格渲染器槽未解析：{stop}"));
                }
                if prefix.tail.is_some() {
                    parts.push("RasterLayer 渲染器后的尾段已解析".into());
                }
                if let Some(stop) = &prefix.tail_stop {
                    parts.push(format!("RasterLayer 尾段未解析：{stop}"));
                }
            }
            Some(parsed) => parts.push(format!("体前缀已解析：名称 {:?}", parsed.layer_name())),
            None => {}
        }
        if let Some(body) = &self.body {
            if body.len > 0 {
                let basis = match body.basis {
                    BodyBoundBasis::NativeSeek => "B 原生同步",
                    BodyBoundBasis::PrefixAInference => "A 观察推定",
                };
                parts.push(format!(
                    "体剩余 offset=0x{:x} len={}（{basis}，未解析）",
                    body.offset, body.len
                ));
            }
        }
        if let Some(note) = &self.parse_note {
            parts.push(note.clone());
        }
        parts.join("，")
    }
}

/// 图层列表读取的停止原因（非错误：已消费内容有效，其后保留为未知）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerListStop {
    /// 标志 1 的新对象条目：对象体无外部长度字段且读取器未接入（集成点），
    /// 无法有界跳过。游标停在该条目对象头之后（键已登记，与原生「登记先于
    /// 对象体」一致），体起点即停止点。
    Flag1NewObjectBody {
        /// 条目序号。
        index: usize,
        /// 条目起点（标志处）偏移。
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 旧图层体未完整读取，游标保留在条目起点。
    LegacyObjectBody {
        index: usize,
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
}

/// 图层列表（计数 + 条目 + 活动索引）的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct LayerList10_1 {
    /// 已读取的列表计数之和。负数在读取时已拒绝。
    pub count: i32,
    /// 旧版双列表的各自计数；第一列表中途停止时第二计数未知。
    pub legacy_counts: Option<(i32, Option<i32>)>,
    /// 已消费的条目（按流内顺序）。停止时少于 `count`。
    pub entries: Vec<LayerListEntry>,
    /// 循环完成后读取的活动图层索引原始位模式（样本 -1 = 无活动图层候选）；
    /// 中途停止时为 `None`。语义（0<=idx<count 时指向活动层）为候选观察。
    pub active_index: Option<i32>,
    /// 停止原因；正常完成为 `None`。
    pub stop: Option<LayerListStop>,
    /// 停止点（或完成点）之后剩余的字节数。
    pub remaining_bytes: usize,
}

/// 从已确认的图层列表起点读取 10.1 图层列表，失败时不推进游标、不修改登记表。
///
/// 调用约定：调用者已确认 Map 架构版本 >=0x17（否则原生走 `FUN_794038a0` 旧
/// 双集合分支，本读取器不适用）并把 Reader 定位在 int32 计数处；`table` 是
/// 跨流共享的会话键登记表（[`RefTable10_1`]），本函数在 trial 副本上登记，
/// 仅在成功返回（含停止）时提交——任何错误路径既不推进游标也不改动调用者的
/// 登记表。所有偏移相对传入 Reader 的底层字节（对 /Maps 全流 Reader 即流内偏移）。
///
/// `max_bytes` 是列表整体的资源限额：在任何条目读取之前预检「声明计数 ×
/// 最小条目字节数（10）」不超过限额（防止按异常计数申请解析工作）；条目内
/// 的有界跳过只移动游标、不分配，长度受流长度自然约束。
///
/// 返回 [`LayerList10_1`]；标志 1 新对象体等停止点是成功返回（已闭合部分 +
/// 剩余字节），与错误（截断、非法标志、负计数等，游标与登记表均不变）区分。
/// 这是严格安全子集，不代表完整原生行为，通过也不代表 MXD 格式已验证。
pub fn read_layer_list_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<LayerList10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let list_offset = trial.position();

    // int32 计数：负数原生不进入循环（写侧非负），这里显式拒绝并说明原生语义。
    let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if count < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "图层列表 offset 0x{list_offset:x}: 计数为负数（{count}）；原生读取仅按 \
                 有符号比较跳过循环，负数语义未知，本读取器拒绝"
            ),
        ));
    }
    let count_usize = usize::try_from(count).expect("计数已确认为非负");
    // 限额预检：每个条目至少 2 字节标志 + 8 字节前缀（ExtraSafe 空条目）。
    let min_total = count_usize
        .checked_mul(MIN_ENTRY_BYTES)
        .ok_or_else(|| invalid(list_offset, "计数 × 最小条目字节数溢出"))?;
    if min_total > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "图层列表 offset 0x{list_offset:x}: 声明计数 {count} × 最小条目字节数 \
                 {MIN_ENTRY_BYTES} 超过资源限额 {max_bytes}，在任何条目读取之前拒绝"
            ),
        ));
    }

    let mut entries = Vec::new();
    let mut stop = None;
    for index in 0..count_usize {
        let entry = read_entry(&mut trial, index, &mut trial_table, list_offset, max_bytes)?;
        // 标志 1 的对象体没有外部长度；只有已确认完整消费的类才能继续。
        if let LayerListEntry {
            object: LayerEntryObject::Entry(ObjectEntry10_1::NewObject { class_id, key }),
            flag: FLAG_LOAD_OBJECT,
            offset,
            ..
        } = &entry
        {
            let completed = matches!(
                &entry.parsed,
                Some(LayerBodyPrefix10_1::FdoGraphicsLayer(body))
                    if body.completion().is_some()
            ) || matches!(
                &entry.parsed,
                Some(
                    LayerBodyPrefix10_1::Graphics3d(_)
                        | LayerBodyPrefix10_1::ProcessLayer(_)
                        | LayerBodyPrefix10_1::WcsLayer(_)
                        | LayerBodyPrefix10_1::CadLayer(_)
                        | LayerBodyPrefix10_1::CadFeatureLayer(_)
                        | LayerBodyPrefix10_1::CadAnnotationLayer(_)
                        | LayerBodyPrefix10_1::CoverageAnnotationLayer(_)
                        | LayerBodyPrefix10_1::TiledLayer(_)
                )
            );
            let completed = completed
                || matches!(&entry.parsed, Some(LayerBodyPrefix10_1::GroupLayer(body))
                    if body.stop == crate::layers::group_layer::GroupLayerStop10_1::Complete);
            let completed = completed
                || matches!(&entry.parsed, Some(LayerBodyPrefix10_1::RasterLayer(body))
                    if body.tail.is_some());
            if !completed {
                stop = Some(LayerListStop::Flag1NewObjectBody {
                    index,
                    offset: *offset,
                    class_id: *class_id,
                    key: *key,
                });
                entries.push(entry);
                break;
            }
        }
        entries.push(entry);
    }

    // 循环完成后读取活动图层索引（原生无条件 `AfStreamRead(4)`）。
    let active_index = if stop.is_none() {
        Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
    } else {
        None
    };

    let result = LayerList10_1 {
        count,
        legacy_counts: None,
        entries,
        active_index,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// Map.dll 0x794038a0 的双列表。第一列表在 v18–v22 使用 ExtraSafe；
/// 第二列表始终是普通 ILayer 对象，两者共用对象键表。
pub(crate) fn read_legacy_layer_lists_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    version: u16,
) -> io::Result<LayerList10_1> {
    if version == 17 {
        return Err(invalid(reader.position(), "原生拒绝 Map v17 的旧图层列表"));
    }
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let mut counts = [None; 2];
    let mut count = 0_i32;
    let mut entries = Vec::new();
    let mut stop = None;
    for (list_index, slot) in counts.iter_mut().enumerate() {
        let extra_safe = list_index == 0 && version > 17;
        let min_width = if extra_safe { 8 } else { 16 };
        let offset = trial.position();
        let size = trial.read_u32_le()? as i32;
        let width = usize::try_from(size)
            .ok()
            .and_then(|size| size.checked_mul(min_width))
            .filter(|&width| width <= trial.remaining() && width <= max_bytes);
        if width.is_none() {
            return Err(invalid(offset, "旧图层列表计数为负数或超过范围"));
        }
        count = count
            .checked_add(size)
            .ok_or_else(|| invalid(offset, "旧图层列表累计计数溢出"))?;
        *slot = Some(size);
        for _ in 0..size {
            let index = entries.len();
            let offset = trial.position();
            let mut current = trial.clone();
            let prefix = extra_safe
                .then(|| -> io::Result<_> { Ok((current.read_u32_le()?, current.read_u32_le()?)) })
                .transpose()?;
            let mut refs = staged.clone();
            let mut entry = LayerListEntry {
                index,
                offset,
                flag: 0,
                prefix,
                object: LayerEntryObject::Empty,
                body: None,
                parsed: None,
                parse_note: None,
            };
            if prefix != Some((0, 0)) {
                let mut body = if let Some((a, b)) = prefix {
                    // B 非零时原生从前缀末尾寻址；否则 A 给出对象区间。
                    let size = if b != 0 { b } else { a } as usize;
                    Some(current.read_region(size)?)
                } else {
                    None
                };
                let objects = body.as_mut().unwrap_or(&mut current);
                let object = read_object_entry_10_1(objects, &mut refs)?;
                entry.object = LayerEntryObject::Entry(object);
                if let ObjectEntry10_1::NewObject { class_id, key } = object {
                    // 旧 Map 的第一组也可用 ExtraSafe；v12 栅格靠这个界区分可选尾部。
                    let parsed = match (prefix, class_id) {
                        (Some(_), RASTER_LAYER_CLASS_ID) => {
                            super::raster_layer::read_complete_bounded_raster_body_10_1(
                                objects, max_bytes, &mut refs,
                            )
                            .map(LayerBodyPrefix10_1::RasterLayer)
                        }
                        (Some(_), WCS_LAYER_CLASS_ID) => {
                            read_wcs_layer_body_10_1(objects, max_bytes, &mut refs, true)
                                .map(|body| LayerBodyPrefix10_1::WcsLayer(Box::new(body)))
                        }
                        _ => read_complete_layer_body_10_1(objects, class_id, max_bytes, &mut refs),
                    };
                    match parsed {
                        Ok(parsed) => entry.parsed = Some(parsed),
                        Err(error) => {
                            stop = Some(LayerListStop::LegacyObjectBody {
                                index,
                                offset,
                                class_id,
                                key,
                            });
                            entry.parse_note = Some(error.to_string());
                            entries.push(entry);
                            break;
                        }
                    }
                }
                if let Some(body) = body
                    && body.remaining() != 0
                {
                    return Err(invalid(offset, "旧图层体未完整消费 ExtraSafe 区间"));
                }
            }
            trial = current;
            staged = refs;
            entries.push(entry);
        }
        if stop.is_some() {
            break;
        }
    }
    let active_index = stop
        .is_none()
        .then(|| trial.read_u32_le().map(|raw| raw as i32))
        .transpose()?;
    let result = LayerList10_1 {
        count,
        legacy_counts: Some((counts[0].expect("已读取首个计数"), counts[1])),
        entries,
        active_index,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = staged;
    Ok(result)
}

/// 读取单个条目（标志、可选前缀、对象条目分类、体前缀分派与有界未知体）。
///
/// `max_bytes` 原样转发给体前缀读取器（约束其内部字符串声明长度）；失败时
/// 返回错误（整体回滚由调用者的 trial 游标/登记表保证）。
fn read_entry(
    trial: &mut Reader<'_>,
    index: usize,
    trial_table: &mut RefTable10_1,
    list_offset: usize,
    max_bytes: usize,
) -> io::Result<LayerListEntry> {
    let offset = trial.position();
    let flag = trial.read_u16_le()?;
    let extra_safe = (FLAG_EXTRA_SAFE_LO..=FLAG_EXTRA_SAFE_HI).contains(&flag);
    if flag != FLAG_LOAD_OBJECT && !extra_safe {
        // 原生对 1/2/3 之外的标志返回 0x80004005（汇编 0x794036b2）。
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "图层列表 offset 0x{offset:x}（列表起点 0x{list_offset:x}）: 条目 {index} \
                 标志 {flag} 不在 {{1,2,3}} 内，原生返回 0x80004005，本读取器拒绝"
            ),
        ));
    }

    // 标志 2/3：ExtraSafe 8 字节前缀 [A][B]；A==0 且 B==0 为空条目。
    let mut prefix = None;
    let mut prefix_end = None;
    if extra_safe {
        let a = trial.read_u32_le()?;
        let b = trial.read_u32_le()?;
        prefix_end = Some(trial.position());
        if a == 0 && b == 0 {
            return Ok(LayerListEntry {
                index,
                offset,
                flag,
                prefix: Some((a, b)),
                object: LayerEntryObject::Empty,
                body: None,
                parsed: None,
                parse_note: None,
            });
        }
        prefix = Some((a, b));
    }

    let entry = read_object_entry_10_1(trial, trial_table)?;
    let (mut parsed, mut parse_note) = (None, None);
    let body = match &entry {
        ObjectEntry10_1::NewObject { class_id, .. } => {
            // 有界终点先于体前缀解析计算：A 推定路径以「对象头终点 + (A-20)」
            // 为基准（bounded_end 用调用时的游标位置），解析前缀会把游标推进
            // 到前缀终点，之后再算会越过条目边界。
            let bound = if extra_safe {
                let (a, b) = prefix.expect("ExtraSafe 条目必有前缀");
                let prefix_end = prefix_end.expect("ExtraSafe 条目必有前缀终点");
                Some(bounded_end(trial, prefix_end, a, b, offset)?)
            } else {
                None
            };
            // 类分派：FeatureLayer / GroupLayer / CompositeGraphicsLayer /
            // FDOGraphicsLayer 的新对象体先解析已确认前缀（各读取器事务
            // 性：失败不推进游标、不改登记表），再按条目边界跳过前缀之后
            // 的剩余字节。没有读取器的类整体按未知处理。
            // Basemap 必须在外层已证或既有观察推定的界内读取；子读取器偏移相对
            // 有界切片，只有成功时才提交它完整读取的对象登记。
            let dispatch = if *class_id == RASTER_LAYER_CLASS_ID {
                if let Some((end, _)) = bound {
                    read_bounded_raster_prefix(trial, end, max_bytes, offset, trial_table)
                        .map(|prefix| Some(LayerBodyPrefix10_1::RasterLayer(prefix)))
                } else {
                    read_complete_layer_body_10_1(trial, *class_id, max_bytes, trial_table)
                        .map(Some)
                }
            } else if *class_id == WCS_LAYER_CLASS_ID {
                if let Some((end, _)) = bound {
                    let mut source = trial.clone();
                    let len = end
                        .checked_sub(source.position())
                        .ok_or_else(|| invalid(offset, "WCSLayer 外层边界早于对象体"))?;
                    let mut body = source.read_region(len)?;
                    let parsed = read_wcs_layer_body_10_1(&mut body, max_bytes, trial_table, true);
                    parsed.map(|body| {
                        *trial = source;
                        Some(LayerBodyPrefix10_1::WcsLayer(Box::new(body)))
                    })
                } else {
                    read_complete_layer_body_10_1(trial, *class_id, max_bytes, trial_table)
                        .map(Some)
                }
            } else if (matches!(
                *class_id,
                CAD_LAYER_CLASS_ID
                    | CAD_FEATURE_LAYER_CLASS_ID
                    | CAD_ANNOTATION_LAYER_CLASS_ID
                    | COVERAGE_ANNOTATION_LAYER_CLASS_ID
            ) || is_tiled_layer(*class_id)
                || matches!(
                    *class_id,
                    GRAPHICS_LAYER_3D_CLASS_ID | GLOBE_GRAPHICS_LAYER_CLASS_ID
                ))
                && let Some((end, _)) = bound
            {
                let mut source = trial.clone();
                let len = end
                    .checked_sub(source.position())
                    .ok_or_else(|| invalid(offset, "图层外层边界早于对象体"))?;
                let mut body = source.read_region(len)?;
                let mut refs = trial_table.clone();
                read_complete_layer_body_10_1(&mut body, *class_id, max_bytes, &mut refs).and_then(
                    |parsed| {
                        if body.remaining() != 0 {
                            return Err(invalid(offset, "图层实际消费与 ExtraSafe 边界不一致"));
                        }
                        trial.read_bytes(body.position())?;
                        *trial_table = refs;
                        Ok(Some(parsed))
                    },
                )
            } else if *class_id == crate::layers::image_server_layer::IMAGE_SERVER_LAYER_CLASS_ID
                && let Some((end, _)) = bound
            {
                let mut source = trial.clone();
                let len = end
                    .checked_sub(source.position())
                    .ok_or_else(|| invalid(offset, "ImageServerLayer 外层边界早于对象体"))?;
                let mut body = Reader::new(source.read_bytes(len)?);
                crate::layers::image_server_layer_tail::read_image_server_layer_body_10_1(
                    &mut body,
                    max_bytes,
                    trial_table,
                )
                .and_then(|parsed| {
                    if let Some(stop) = &parsed.stop {
                        parse_note = Some(format!(
                            "ImageServerLayer 在 offset=0x{:x} 停止：{}",
                            trial.position() + stop.offset,
                            stop.reason
                        ));
                    }
                    // 后续未读值只能按宿主给出的范围保留。
                    trial.read_bytes(body.position())?;
                    Ok(Some(LayerBodyPrefix10_1::ImageServerLayer(Box::new(
                        parsed,
                    ))))
                })
            } else if *class_id == crate::layers::basemap_layer::BASEMAP_LAYER_CLASS_ID
                && bound.is_some()
            {
                let (end, _) = bound.expect("已检查外层边界");
                let start = trial.position();
                let len = end
                    .checked_sub(start)
                    .ok_or_else(|| invalid(offset, "Basemap 外层边界早于对象体"))?;
                let mut source = trial.clone();
                let bytes = source.read_bytes(len)?;
                let mut body_reader = Reader::new(bytes);
                let mut body_table = trial_table.clone();
                match crate::layers::basemap_layer::read_basemap_prefix_10_1(
                    &mut body_reader,
                    max_bytes,
                    &mut body_table,
                ) {
                    Ok(prefix)
                        if prefix.stop
                            == crate::layers::basemap_layer::BasemapStop10_1::Complete
                            && body_reader.remaining() != 0 =>
                    {
                        Err(invalid(offset, "Basemap 实际消费与 ExtraSafe 边界不一致"))
                    }
                    Ok(prefix) => {
                        trial.read_bytes(body_reader.position())?;
                        *trial_table = body_table;
                        let reason = match &prefix.stop {
                            crate::layers::basemap_layer::BasemapStop10_1::NewObject {
                                class_id,
                                key,
                                ..
                            } => format!("未闭合新对象 类={} 键 {key}", class_id_hex(class_id)),
                            crate::layers::basemap_layer::BasemapStop10_1::FeatureLayer {
                                key,
                                body_stop_offset,
                                reason,
                            } => format!(
                                "FeatureLayer 键 {key} 体内停于 offset=0x{:x}：{reason:?}",
                                start + body_stop_offset
                            ),
                            crate::layers::basemap_layer::BasemapStop10_1::ShaderElements {
                                count,
                                ..
                            } => format!("未闭合 shader 数组，计数 {count}"),
                            crate::layers::basemap_layer::BasemapStop10_1::Extensions {
                                count,
                                ..
                            } => format!("未闭合扩展集合，计数 {count}"),
                            crate::layers::basemap_layer::BasemapStop10_1::Complete => {
                                "已完整读取".to_string()
                            }
                        };
                        parse_note = Some(format!(
                            "Basemap {:?}：已完整读取图层键 {:?}，offset=0x{:x}：{}",
                            prefix.name,
                            prefix.feature_keys,
                            start + body_reader.position(),
                            reason
                        ));
                        Ok(Some(LayerBodyPrefix10_1::BasemapLayer(prefix)))
                    }
                    Err(error) => Err(error),
                }
            } else if *class_id == super::na_layer::NA_LAYER_CLASS_ID
                && let Some((end, _)) = bound
            {
                let len = end
                    .checked_sub(trial.position())
                    .ok_or_else(|| invalid(offset, "NALayer 外层边界早于对象体"))?;
                let mut region = trial.clone().read_region(len)?;
                super::na_layer::read_na_layer_prefix_10_1(&mut region, max_bytes).and_then(
                    |mut body| {
                        match super::na_layer::read_na_network_name_10_1(
                            &mut region,
                            max_bytes,
                            trial_table,
                        ) {
                            Ok(name) => {
                                body.network_name = Some(name);
                                match super::na_layer::read_na_context_prefix_10_1(
                                    &mut region,
                                    max_bytes,
                                    trial_table,
                                ) {
                                    Ok(context) => body.context = Some(context),
                                    Err(error) => {
                                        parse_note =
                                            Some(format!("NALayer 上下文槽未读取：{error}"))
                                    }
                                }
                            }
                            Err(error) => {
                                parse_note = Some(format!("NALayer 名称槽未读取：{error}"))
                            }
                        }
                        trial.read_bytes(region.position())?;
                        Ok(Some(LayerBodyPrefix10_1::NaLayer(body)))
                    },
                )
            } else if *class_id == GROUP_LAYER_CLASS_ID {
                read_group_layer_list_body(trial, bound.map(|(end, _)| end), max_bytes, trial_table)
            } else if *class_id == super::process_layer::PROCESS_LAYER_CLASS_ID
                && let Some((end, _)) = bound
            {
                let len = end
                    .checked_sub(trial.position())
                    .ok_or_else(|| invalid(offset, "ProcessLayer 外层边界早于对象体"))?;
                let mut region = trial.clone().read_region(len)?;
                let mut refs = trial_table.clone();
                parse_body_prefix(&mut region, *class_id, max_bytes, &mut refs, 0).and_then(
                    |body| {
                        if region.remaining() != 0 {
                            return Err(invalid(offset, "ProcessLayer 未完整消费外层区间"));
                        }
                        trial.read_bytes(region.position())?;
                        *trial_table = refs;
                        Ok(body)
                    },
                )
            } else {
                parse_body_prefix(trial, *class_id, max_bytes, trial_table, 0)
            };
            match dispatch {
                Ok(prefix_result) => parsed = prefix_result,
                Err(error) => {
                    // 体前缀解析失败（截断、版本超限、嵌套类不支持等）：按有
                    // 界未知跳过并保留失败原因。若失败源于真实截断，下方的
                    // 有界跳过会因超出流长度而让整条列表失败（事务回滚）；
                    // 若字节完整而内容不支持，条目边界不受影响。
                    parse_note = Some(format!("体前缀解析失败（{error}），体按未知保留"));
                }
            }
            match bound {
                // 标志 2/3：跳过体前缀终点到条目边界的剩余（可能长度为 0）。
                Some((end, basis)) => Some(skip_to(trial, end, basis, offset)?),
                // 标志 1：无前缀、体无外部长度——不产生体范围；解析出前缀时
                // 游标停在前缀终点，否则停在对象头之后，由调用者停止。
                None => None,
            }
        }
        // 回引与空标记条目没有对象体（样本条目 2：A=20 无体）；B!=0 时仍按
        // 原生 Seek 同步到「前缀后位置 + B」（不产生未知体范围）。
        ObjectEntry10_1::BackReference { .. } | ObjectEntry10_1::Null => {
            if let Some((_, b)) = prefix {
                if b != 0 {
                    let prefix_end = prefix_end.expect("ExtraSafe 条目必有前缀终点");
                    let (end, _) = bounded_end(trial, prefix_end, 0, b, offset)?;
                    skip_to(trial, end, BodyBoundBasis::NativeSeek, offset)?;
                }
            }
            None
        }
    };

    Ok(LayerListEntry {
        index,
        offset,
        flag,
        prefix,
        object: LayerEntryObject::Entry(entry),
        body,
        parsed,
        parse_note,
    })
}

/// 只在当前条目的字节范围内试读，避免首串借用后一个条目或活动索引的字节。
fn read_bounded_raster_prefix(
    trial: &mut Reader<'_>,
    end: usize,
    max_bytes: usize,
    offset: usize,
    refs: &mut RefTable10_1,
) -> io::Result<crate::layers::raster_layer::RasterLayerPrefix10_1> {
    let mut source = trial.clone();
    let len = end
        .checked_sub(source.position())
        .ok_or_else(|| invalid(offset, "RasterLayer 外层边界早于对象体"))?;
    let mut body = Reader::new(source.read_bytes(len)?);
    let prefix = super::raster_layer::read_bounded_raster_body_10_1(&mut body, max_bytes, refs)?;
    trial.read_bytes(body.position())?;
    Ok(prefix)
}

/// 组图层优先完整读取；已有前缀读取器继续提供未知子层的诊断。
fn read_group_layer_list_body(
    reader: &mut Reader<'_>,
    end: Option<usize>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<Option<LayerBodyPrefix10_1>> {
    let mut trial = reader.clone();
    if let Some(end) = end {
        let len = end
            .checked_sub(reader.position())
            .ok_or_else(|| invalid(reader.position(), "GroupLayer 外层边界早于对象体"))?;
        trial = trial.read_region(len)?;
    }
    let start = trial.position();
    let mut refs = table.clone();
    if let Ok(body) =
        read_complete_layer_body_10_1(&mut trial, GROUP_LAYER_CLASS_ID, max_bytes, &mut refs)
        && (end.is_none() || trial.remaining() == 0)
    {
        reader.read_bytes(trial.position() - start)?;
        *table = refs;
        return Ok(Some(body));
    }
    parse_body_prefix(reader, GROUP_LAYER_CLASS_ID, max_bytes, table, 0)
}

/// 按类标识分派新对象体的已确认前缀解析（组图层子图层递归共用，见
/// [`crate::layers::group_layer`]）。
///
/// 只有已实现读取器覆盖的类参与分派；其余类返回 `Ok(None)`（体按未知
/// 处理）。解析失败不作为整条列表的错误（见调用点注释），由调用者按有界
/// 跳过降级。各读取器均为「前缀停止」语义：返回 Ok 时游标停在其已确认
/// 前缀终点或首个不可闭合字段处（容器类读取器把游标回退到该字段起点），
/// 剩余体字节由调用者按条目边界跳过。`depth` 是组图层递归深度（GroupLayer
/// 套 GroupLayer 时经 [`crate::layers::group_layer`] 递增，超过其安全限额后组图层
/// 类不再分派）。
pub(crate) fn parse_body_prefix(
    trial: &mut Reader<'_>,
    class_id: [u8; 16],
    max_bytes: usize,
    trial_table: &mut RefTable10_1,
    depth: u32,
) -> io::Result<Option<LayerBodyPrefix10_1>> {
    if class_id == FEATURE_LAYER_CLASS_ID {
        Ok(Some(LayerBodyPrefix10_1::FeatureLayer(Box::new(
            read_feature_layer_prefix_10_1(trial, max_bytes, trial_table)?,
        ))))
    } else if class_id == GROUP_LAYER_CLASS_ID {
        Ok(Some(LayerBodyPrefix10_1::GroupLayer(Box::new(
            crate::layers::group_layer::read_group_layer_prefix_with_depth_10_1(
                trial,
                max_bytes,
                trial_table,
                depth,
            )?,
        ))))
    } else if class_id == super::process_layer::PROCESS_LAYER_CLASS_ID {
        super::process_layer::read_process_layer_10_1(trial, max_bytes, trial_table)
            .map(|body| Some(LayerBodyPrefix10_1::ProcessLayer(Box::new(body))))
    } else if class_id == COMPOSITE_GRAPHICS_LAYER_CLASS_ID {
        Ok(Some(LayerBodyPrefix10_1::CompositeGraphicsLayer(Box::new(
            read_composite_graphics_layer_prefix_10_1(trial, max_bytes, trial_table)?,
        ))))
    } else if class_id == FDO_GRAPHICS_LAYER_CLASS_ID {
        Ok(Some(LayerBodyPrefix10_1::FdoGraphicsLayer(Box::new(
            read_fdo_graphics_layer_prefix_10_1(trial, max_bytes, trial_table)?,
        ))))
    } else if class_id == WCS_LAYER_CLASS_ID {
        read_wcs_layer_body_10_1(trial, max_bytes, trial_table, false)
            .map(|body| Some(LayerBodyPrefix10_1::WcsLayer(Box::new(body))))
    } else if class_id == CAD_LAYER_CLASS_ID {
        read_cad_layer_10_1(trial, max_bytes, trial_table)
            .map(|body| Some(LayerBodyPrefix10_1::CadLayer(Box::new(body))))
    } else if class_id == CAD_FEATURE_LAYER_CLASS_ID {
        read_cad_feature_layer_10_1(trial, max_bytes, trial_table)
            .map(|body| Some(LayerBodyPrefix10_1::CadFeatureLayer(Box::new(body))))
    } else if class_id == CAD_ANNOTATION_LAYER_CLASS_ID {
        read_cad_annotation_layer_10_1(trial, max_bytes, trial_table)
            .map(|body| Some(LayerBodyPrefix10_1::CadAnnotationLayer(Box::new(body))))
    } else if class_id == COVERAGE_ANNOTATION_LAYER_CLASS_ID {
        read_coverage_annotation_layer_10_1(trial, max_bytes, trial_table)
            .map(|body| Some(LayerBodyPrefix10_1::CoverageAnnotationLayer(Box::new(body))))
    } else if is_tiled_layer(class_id) {
        read_tiled_layer_10_1(class_id, trial, max_bytes, trial_table)
            .map(|body| Some(LayerBodyPrefix10_1::TiledLayer(Box::new(body))))
    } else if matches!(
        class_id,
        GRAPHICS_LAYER_3D_CLASS_ID | GLOBE_GRAPHICS_LAYER_CLASS_ID
    ) {
        Ok(Some(LayerBodyPrefix10_1::Graphics3d(Box::new(
            read_graphics_3d_layer_10_1(&class_id, trial, max_bytes, trial_table)?,
        ))))
    } else {
        Ok(None)
    }
}

/// 消费普通 LoadObject 槽的完整图层体。没有外层长度，部分读取结果不提交。
pub(crate) fn read_complete_layer_body_10_1(
    reader: &mut Reader<'_>,
    class_id: [u8; 16],
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<LayerBodyPrefix10_1> {
    use crate::layers::composite_graphics::CompositeGraphicsLayerStop10_1;
    use crate::layers::feature_layer::FeatureLayerStop10_1;
    use crate::layers::feature_layer_tail::SuccessorStop10_1;

    let unsupported = |reason| {
        io::Error::new(
            ErrorKind::Unsupported,
            format!("图层类 {}：{reason}", class_id_hex(&class_id)),
        )
    };
    let mut trial = reader.clone();
    let mut refs = table.clone();
    let body = refs
        .with_nested_graphics(|refs| match class_id {
            crate::layers::basemap_layer::BASEMAP_LAYER_CLASS_ID => {
                crate::layers::basemap_layer::read_basemap_prefix_10_1(&mut trial, max_bytes, refs)
                    .map(|body| Some(LayerBodyPrefix10_1::BasemapLayer(body)))
            }
            crate::layers::image_server_layer::IMAGE_SERVER_LAYER_CLASS_ID => {
                crate::layers::image_server_layer_tail::read_image_server_layer_body_10_1(
                    &mut trial, max_bytes, refs,
                )
                .map(|body| Some(LayerBodyPrefix10_1::ImageServerLayer(Box::new(body))))
            }
            RASTER_LAYER_CLASS_ID => crate::layers::raster_layer::read_inline_raster_body_10_1(
                &mut trial, max_bytes, refs,
            )
            .map(|body| Some(LayerBodyPrefix10_1::RasterLayer(body))),
            GROUP_LAYER_CLASS_ID => {
                crate::layers::group_layer::read_group_layer_body_10_1(&mut trial, max_bytes, refs)
                    .map(|body| Some(LayerBodyPrefix10_1::GroupLayer(Box::new(body))))
            }
            _ => parse_body_prefix(&mut trial, class_id, max_bytes, refs, 0),
        })?
        .ok_or_else(|| unsupported("当前没有无外层长度的对象体读取器"))?;
    let complete = match &body {
        LayerBodyPrefix10_1::FeatureLayer(body) => matches!(
            body.stop,
            FeatureLayerStop10_1::Successor(SuccessorStop10_1::TailCompleted)
        ),
        LayerBodyPrefix10_1::CompositeGraphicsLayer(body) => {
            body.stop == CompositeGraphicsLayerStop10_1::Completed
        }
        LayerBodyPrefix10_1::FdoGraphicsLayer(body) => body.completion().is_some(),
        LayerBodyPrefix10_1::GroupLayer(body) => {
            body.stop == crate::layers::group_layer::GroupLayerStop10_1::Complete
        }
        LayerBodyPrefix10_1::RasterLayer(body) => body.tail.is_some(),
        LayerBodyPrefix10_1::NaLayer(_) => false,
        LayerBodyPrefix10_1::Graphics3d(_)
        | LayerBodyPrefix10_1::ProcessLayer(_)
        | LayerBodyPrefix10_1::WcsLayer(_)
        | LayerBodyPrefix10_1::CadLayer(_)
        | LayerBodyPrefix10_1::CadFeatureLayer(_)
        | LayerBodyPrefix10_1::CadAnnotationLayer(_)
        | LayerBodyPrefix10_1::CoverageAnnotationLayer(_)
        | LayerBodyPrefix10_1::TiledLayer(_) => true,
        LayerBodyPrefix10_1::BasemapLayer(body) => {
            body.stop == crate::layers::basemap_layer::BasemapStop10_1::Complete
        }
        LayerBodyPrefix10_1::ImageServerLayer(body) => {
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(stop.error_kind, stop.reason.clone()));
            }
            true
        }
    };
    if !complete {
        return Err(unsupported("对象体未完整读取"));
    }
    *reader = trial;
    *table = refs;
    Ok(body)
}

/// 计算条目对象部分的终点（有界跳过目标）与跳过依据。
///
/// `b != 0`：前缀后位置 + b（原生 Seek，确定行为）；`b == 0`：当前位置 + (a - 20)
/// （A 观察推定，样本 6/6 条目 A = 头 20 + 体总体积）。a < 20 与观察矛盾，拒绝。
fn bounded_end(
    trial: &Reader<'_>,
    prefix_end: usize,
    a: u32,
    b: u32,
    entry_offset: usize,
) -> io::Result<(usize, BodyBoundBasis)> {
    if b != 0 {
        let delta = usize::try_from(b).expect("u32 可表示为本平台 usize");
        let end = prefix_end
            .checked_add(delta)
            .ok_or_else(|| invalid(entry_offset, "前缀 B 同步位置溢出"))?;
        Ok((end, BodyBoundBasis::NativeSeek))
    } else {
        let a = usize::try_from(a).expect("u32 可表示为本平台 usize");
        if a < OBJECT_HEADER_BYTES {
            return Err(invalid(
                entry_offset,
                &format!(
                    "前缀 A={a} 小于对象头 {OBJECT_HEADER_BYTES} 字节，与「A = 头 20 + 体 \
                     总体积」的样本观察矛盾，无法有界跳过"
                ),
            ));
        }
        // 观察推定：体长度 = A - 20；当前位置在对象头之后。
        let end = trial
            .position()
            .checked_add(a - OBJECT_HEADER_BYTES)
            .ok_or_else(|| invalid(entry_offset, "前缀 A 推定体长度溢出"))?;
        Ok((end, BodyBoundBasis::PrefixAInference))
    }
}

/// 把游标推进到 `end`，返回被跳过的有界未知体范围。
///
/// `end` 小于当前位置（原生 Seek 允许倒退，本子集拒绝）或超出流长度时报错。
fn skip_to(
    trial: &mut Reader<'_>,
    end: usize,
    basis: BodyBoundBasis,
    entry_offset: usize,
) -> io::Result<UnknownBodyRange> {
    let position = trial.position();
    if end < position {
        return Err(invalid(
            entry_offset,
            "有界跳过终点早于当前位置（原生 Seek 允许倒退，本读取器拒绝）",
        ));
    }
    let len = end - position;
    if len > trial.remaining() {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            format!(
                "图层列表 offset 0x{entry_offset:x}: 有界未知体长度 {len} 超出剩余流 \
                 长度（推定边界与实际流矛盾）"
            ),
        ));
    }
    // 只移动游标，不复制、不解释体内容。
    trial.read_bytes(len)?;
    Ok(UnknownBodyRange {
        offset: position,
        len,
        basis,
    })
}

/// 构造带偏移的 InvalidData 错误。
fn invalid(offset: usize, reason: &str) -> io::Error {
    io::Error::new(
        ErrorKind::InvalidData,
        format!("图层列表 offset 0x{offset:x}: {reason}"),
    )
}
