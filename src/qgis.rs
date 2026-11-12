//! 独立的构造模型输出层，不读取 MXD，也不推断源格式语义。

use crate::{model::*, source_path};
use std::fmt::Write as _;
use std::io::{self, ErrorKind, Seek, Write};
use std::path::Path;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

pub const QGIS_BASELINE_VERSION: &str = "4.2.3-Belém do Pará";

#[derive(Debug, Clone)]
pub struct WriteOptions {
    /// 源 MXD 所在绝对目录；允许 Windows 盘符、UNC 或 POSIX 语法。
    pub source_directory: String,
    /// 默认禁止覆盖；开启后只在完整临时文件写入成功后替换目标。
    pub overwrite: bool,
}

impl WriteOptions {
    pub fn new(source_directory: impl Into<String>) -> Self {
        Self {
            source_directory: source_directory.into(),
            overwrite: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    /// 模型中的位置，不包含数据源路径或连接内容。
    pub location: String,
    pub message: &'static str,
}

#[derive(Debug)]
pub struct QgsOutput {
    pub xml: String,
    pub diagnostics: Vec<Diagnostic>,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidInput, message.into())
}

/// 同时用于文本和属性，拒绝 XML 1.0 不允许的控制字符；属性空白不被规范化。
fn xml(value: &str) -> io::Result<String> {
    let mut result = String::new();
    for c in value.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            '\n' => result.push_str("&#10;"),
            '\r' => result.push_str("&#13;"),
            '\t' => result.push_str("&#9;"),
            c if ('\u{20}'..='\u{d7ff}').contains(&c)
                || ('\u{e000}'..='\u{fffd}').contains(&c)
                || c >= '\u{10000}' =>
            {
                result.push(c)
            }
            _ => return Err(invalid("字符串含 XML 1.0 不允许的字符")),
        }
    }
    Ok(result)
}

struct Builder<'a> {
    options: &'a WriteOptions,
    output_directory: &'a str,
    diagnostics: Vec<Diagnostic>,
    layers: String,
    layer_count: usize,
}

impl Builder<'_> {
    fn warn(&mut self, code: &'static str, location: &str, message: &'static str) {
        self.diagnostics.push(Diagnostic {
            code,
            location: location.into(),
            message,
        });
    }

    fn unknown(&mut self, fields: &[UnknownField], location: &str) {
        if !fields.is_empty() {
            self.warn(
                "unknown-fields",
                location,
                "模型含未解释字节，未写入 QGIS 工程；原模型须另行保留",
            );
        }
    }

    fn crs(
        &mut self,
        crs: Option<&CoordinateSystem>,
        location: &str,
    ) -> io::Result<(&'static str, bool)> {
        let Some(crs) = crs else {
            self.warn(
                "missing-crs",
                location,
                "未提供坐标系；不指定默认 EPSG，图层可能采用数据提供者的坐标系",
            );
            return Ok(("<spatialrefsys/>", false));
        };
        self.unknown(&crs.unknown_fields, location);
        let definition = crs.definition.trim();
        let supported = if definition.eq_ignore_ascii_case("EPSG:4326") {
            Some(include_str!("../tests/fixtures/qgis/epsg4326.xml"))
        } else if definition.eq_ignore_ascii_case("EPSG:3857") {
            Some(include_str!("../tests/fixtures/qgis/epsg3857.xml"))
        } else {
            None
        };
        // 即使定义未被采用，也验证其能否安全保留为 XML 文本。
        xml(&crs.definition)?;
        if let Some(srs) = supported {
            return Ok((srs, true));
        }
        self.warn(
            "unsupported-crs",
            location,
            "仅支持基准核对过的 EPSG:4326/3857；原定义保留为自定义属性，不猜测坐标系",
        );
        Ok(("<spatialrefsys/>", false))
    }

    fn extent(
        &mut self,
        extent: Option<Extent>,
        known_crs: bool,
        location: &str,
    ) -> io::Result<Option<Extent>> {
        let Some(e) = extent else {
            return Ok(None);
        };
        if ![e.xmin, e.ymin, e.xmax, e.ymax]
            .iter()
            .all(|v| v.is_finite())
            || e.xmin >= e.xmax
            || e.ymin >= e.ymax
            || !(e.xmax - e.xmin).is_finite()
            || !(e.ymax - e.ymin).is_finite()
        {
            return Err(invalid(format!(
                "{location}: 范围必须有限且 xmin<xmax、ymin<ymax"
            )));
        }
        if !known_crs {
            self.warn(
                "extent-without-crs",
                location,
                "范围未应用：没有受支持的所属坐标系；原值保留为自定义属性",
            );
            return Ok(None);
        }
        Ok(Some(e))
    }

    fn nodes(&mut self, nodes: &[LayerNode], parent: &str, depth: usize) -> io::Result<String> {
        if depth > 128 {
            return Err(invalid("图层树嵌套超过 128 层"));
        }
        let mut tree = String::new();
        for (index, node) in nodes.iter().enumerate() {
            let id = format!("{parent}_{index}");
            match node {
                LayerNode::Group(group) => {
                    self.unknown(&group.unknown_fields, &id);
                    write!(
                        tree,
                        "<layer-tree-group name=\"{}\" checked=\"{}\" expanded=\"1\">",
                        xml(&group.name)?,
                        checked(group.visible)
                    )
                    .unwrap();
                    tree.push_str(&self.nodes(&group.children, &id, depth + 1)?);
                    tree.push_str("</layer-tree-group>");
                }
                LayerNode::Layer(layer) => {
                    self.layer_count += 1;
                    if self.layer_count > 100_000 {
                        return Err(invalid("图层数超过 100000"));
                    }
                    let path = self.layer(layer, &id)?;
                    write!(tree, "<layer-tree-layer id=\"{id}\" name=\"{}\" checked=\"{}\" expanded=\"1\" providerKey=\"ogr\" source=\"{}\"/>", xml(&layer.name)?, checked(layer.visible), xml(&path)?).unwrap();
                }
            }
        }
        Ok(tree)
    }

    fn layer(&mut self, layer: &Layer, id: &str) -> io::Result<String> {
        self.unknown(&layer.unknown_fields, id);
        let source = layer
            .source
            .as_ref()
            .ok_or_else(|| invalid(format!("{id}: 缺少数据源")))?;
        self.unknown(&source.unknown_fields, id);
        if source.connection.as_ref().is_some_and(|s| !s.is_empty()) {
            return Err(invalid(format!(
                "{id}: 不支持连接字符串，不能将其当作 Shapefile 路径"
            )));
        }
        if source
            .provider
            .as_ref()
            .is_some_and(|p| !p.eq_ignore_ascii_case("ogr") && !p.eq_ignore_ascii_case("shapefile"))
        {
            return Err(invalid(format!("{id}: 不支持的数据提供者")));
        }
        let path = source
            .path
            .as_deref()
            .ok_or_else(|| invalid(format!("{id}: 缺少路径")))?;
        if !path.to_ascii_lowercase().ends_with(".shp") {
            return Err(invalid(format!("{id}: 仅支持 Shapefile 路径")));
        }
        let resolved =
            source_path::resolve(path, &self.options.source_directory, self.output_directory)?;
        if resolved.foreign {
            self.warn(
                "foreign-path",
                id,
                "保留异平台绝对路径，无法在本机验证或承诺可加载；需在目标平台检查或重定位",
            );
        } else {
            let path = Path::new(&resolved.absolute);
            if !path.is_file() {
                self.warn(
                    "missing-source",
                    id,
                    "Shapefile 主文件不存在，仍保留图层与断链路径",
                );
            } else if !path.with_extension("shx").is_file() || !path.with_extension("dbf").is_file()
            {
                self.warn(
                    "missing-sidecar",
                    id,
                    "未找到同名小写 shx/dbf，数据完整性需由 QGIS 检查",
                );
            }
        }
        self.warn(
            "default-style",
            id,
            "模型没有符号和标注信息，由 QGIS 采用默认样式；未还原源外观",
        );
        let (srs, known) = self.crs(layer.crs.as_ref(), id)?;
        let extent = self.extent(layer.extent, known, id)?;
        write!(self.layers, "<maplayer type=\"vector\" hasScaleBasedVisibilityFlag=\"0\" labelsEnabled=\"0\"><id>{id}</id><datasource>{}</datasource><layername>{}</layername><srs>{srs}</srs><provider encoding=\"\">ogr</provider>", xml(&resolved.qgis)?, xml(&layer.name)?).unwrap();
        if let Some(e) = extent {
            write_extent(&mut self.layers, e);
        }
        write!(
            self.layers,
            "<customproperties><Option type=\"Map\">{}</Option></customproperties></maplayer>",
            original_properties(layer.crs.as_ref(), layer.extent)?
        )
        .unwrap();
        Ok(resolved.qgis)
    }
}

fn checked(visible: bool) -> &'static str {
    if visible {
        "Qt::Checked"
    } else {
        "Qt::Unchecked"
    }
}

fn write_extent(out: &mut String, e: Extent) {
    write!(
        out,
        "<extent><xmin>{}</xmin><ymin>{}</ymin><xmax>{}</xmax><ymax>{}</ymax></extent>",
        e.xmin, e.ymin, e.xmax, e.ymax
    )
    .unwrap();
}

fn original_properties(
    crs: Option<&CoordinateSystem>,
    extent: Option<Extent>,
) -> io::Result<String> {
    let mut out = String::new();
    if let Some(crs) = crs {
        write!(
            out,
            "<Option name=\"mxd2qgz/sourceCrs\" type=\"QString\" value=\"{}\"/>",
            xml(&crs.definition)?
        )
        .unwrap();
    }
    if let Some(e) = extent {
        write!(
            out,
            "<Option name=\"mxd2qgz/sourceExtent\" type=\"QString\" value=\"{},{},{},{}\"/>",
            e.xmin, e.ymin, e.xmax, e.ymax
        )
        .unwrap();
    }
    Ok(out)
}

/// 只生成 XML，便于独立测试；输出目录须使用目标平台的绝对路径语法。
/// 当前明确拒绝多个数据框，不会静默丢弃其他框。
pub fn to_qgs(map: &Map, options: &WriteOptions, output_directory: &str) -> io::Result<QgsOutput> {
    if map.frames.len() > 1 {
        return Err(invalid("暂不支持多个数据框；请由调用方明确选择单框"));
    }
    // 即使空工程也检查目录语义，避免延迟到首个图层才报错。
    source_path::resolve("probe.shp", &options.source_directory, output_directory)?;
    let mut b = Builder {
        options,
        output_directory,
        diagnostics: Vec::new(),
        layers: String::new(),
        layer_count: 0,
    };
    b.unknown(&map.unknown_fields, "map");
    let title = xml(map.name.as_deref().unwrap_or_default())?;
    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<qgis version=\"{QGIS_BASELINE_VERSION}\" projectname=\"{title}\"><homePath path=\"\"/><title>{title}</title>"
    );
    let mut tree = String::new();
    let mut project_properties = String::new();
    if let Some(frame) = map.frames.first() {
        b.unknown(&frame.unknown_fields, "frame_0");
        let (srs, known) = b.crs(frame.crs.as_ref(), "frame_0")?;
        let extent = b.extent(frame.extent, known, "frame_0")?;
        write!(out, "<projectCrs>{srs}</projectCrs>").unwrap();
        if let Some(e) = extent {
            write!(out, "<ProjectViewSettings rotation=\"0\" UseProjectScales=\"0\"><DefaultViewExtent xmin=\"{}\" ymin=\"{}\" xmax=\"{}\" ymax=\"{}\">{srs}</DefaultViewExtent></ProjectViewSettings>", e.xmin,e.ymin,e.xmax,e.ymax).unwrap();
        }
        write!(project_properties, "<properties name=\"mxd2qgz\"><properties name=\"frameName\" type=\"QString\">{}</properties>", xml(&frame.name)?).unwrap();
        if let Some(crs) = &frame.crs {
            write!(
                project_properties,
                "<properties name=\"sourceCrs\" type=\"QString\">{}</properties>",
                xml(&crs.definition)?
            )
            .unwrap();
        }
        if let Some(e) = frame.extent {
            write!(
                project_properties,
                "<properties name=\"sourceExtent\" type=\"QString\">{},{},{},{}</properties>",
                e.xmin, e.ymin, e.xmax, e.ymax
            )
            .unwrap();
        } else {
            b.warn(
                "missing-extent",
                "frame_0",
                "未提供主画布范围，交由 QGIS 决定初始视图",
            );
        }
        project_properties.push_str("</properties>");
        // QGIS readLayerXml 只接受长度大于 10 的 ID，短 ID 会被随机替换。
        tree = b.nodes(&frame.children, "mxd2qgz_frame_0", 0)?;
        b.warn(
            "source-order",
            "frame_0",
            "按模型顺序映射图层树，自上而下；模型未确认此顺序等于源渲染顺序",
        );
    } else {
        out.push_str("<projectCrs><spatialrefsys/></projectCrs>");
        b.warn(
            "empty-map",
            "map",
            "模型没有数据框，输出空工程且不指定坐标系",
        );
    }
    // QGIS 4.2 仍以此兼容属性决定是否读取 projectCrs，仅写 projectCrs 不够。
    write!(out, "<layer-tree-group name=\"\" checked=\"Qt::Checked\" expanded=\"1\">{tree}<custom-order enabled=\"0\"/></layer-tree-group><projectlayers>{}</projectlayers><properties name=\"properties\"><properties name=\"Paths\"><properties name=\"Absolute\" type=\"bool\">false</properties></properties><properties name=\"SpatialRefSys\"><properties name=\"ProjectionsEnabled\" type=\"int\">1</properties></properties>{project_properties}</properties></qgis>\n", b.layers).unwrap();
    Ok(QgsOutput {
        xml: out,
        diagnostics: b.diagnostics,
    })
}

fn archive<W: Write + Seek>(sink: W, xml: &str) -> io::Result<()> {
    let mut zip = ZipWriter::new(sink);
    zip.start_file(
        "project.qgs",
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
    )
    .map_err(io::Error::other)?;
    zip.write_all(xml.as_bytes())?;
    zip.finish().map_err(io::Error::other)?;
    Ok(())
}

/// 完成序列化后在目标同目录写临时文件，成功后发布；失败自动删除临时文件。
/// 返回的诊断是输出合同的一部分，调用方必须展示或保存。
pub fn write_project(
    map: &Map,
    destination: &Path,
    options: &WriteOptions,
) -> io::Result<Vec<Diagnostic>> {
    let extension = destination
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension != "qgs" && extension != "qgz" {
        return Err(invalid("输出扩展名必须为 .qgs 或 .qgz"));
    }
    let destination = std::path::absolute(destination)?;
    let parent = destination
        .parent()
        .ok_or_else(|| invalid("输出文件缺少父目录"))?;
    let directory = parent
        .to_str()
        .ok_or_else(|| invalid("输出目录不是 UTF-8"))?;
    if !options.overwrite && destination.symlink_metadata().is_ok() {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            "输出已存在，默认禁止覆盖",
        ));
    }
    let rendered = to_qgs(map, options, directory)?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".mxd2qgz-")
        .tempfile_in(parent)?;
    if extension == "qgz" {
        archive(&mut temporary, &rendered.xml)?;
    } else {
        temporary.write_all(rendered.xml.as_bytes())?;
    }
    temporary.as_file_mut().sync_all()?;
    let result = if options.overwrite {
        temporary.persist(&destination)
    } else {
        temporary.persist_noclobber(&destination)
    };
    result.map_err(|error| error.error)?;
    Ok(rendered.diagnostics)
}
