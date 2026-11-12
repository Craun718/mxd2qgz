use mxd2qgz::{model::*, qgis::*, source_path};
use roxmltree::Document;
use std::fs;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

fn crs(definition: &str) -> CoordinateSystem {
    CoordinateSystem {
        definition: definition.into(),
        unknown_fields: vec![],
    }
}

fn extent() -> Extent {
    Extent {
        xmin: 0.,
        ymin: 0.,
        xmax: 10.,
        ymax: 10.,
    }
}

fn layer(name: &str, path: &str, visible: bool) -> LayerNode {
    LayerNode::Layer(Layer {
        name: name.into(),
        visible,
        source: Some(DataSource {
            provider: Some("Shapefile".into()),
            path: Some(path.into()),
            connection: None,
            unknown_fields: vec![],
        }),
        crs: Some(crs("EPSG:4326")),
        extent: Some(extent()),
        unknown_fields: vec![],
    })
}

fn map() -> Map {
    Map {
        name: Some("构造 <&> \"'\n工程".into()),
        frames: vec![DataFrame {
            name: "主框".into(),
            children: vec![
                LayerNode::Group(LayerGroup {
                    name: "组 <&> \"'\t\n".into(),
                    visible: false,
                    children: vec![
                        layer("重复 <&> \"'", "data/points.shp", true),
                        LayerNode::Group(LayerGroup {
                            name: "子组".into(),
                            visible: true,
                            children: vec![layer("重复 <&> \"'", "data/points.shp", false)],
                            unknown_fields: vec![],
                        }),
                    ],
                    unknown_fields: vec![],
                }),
                layer("断链", "missing.shp", true),
            ],
            crs: Some(crs("EPSG:3857")),
            extent: Some(extent()),
            unknown_fields: vec![],
        }],
        unknown_fields: vec![],
    }
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/qgis")
}
fn options() -> WriteOptions {
    WriteOptions::new(fixtures().to_str().unwrap())
}
fn render(map: &Map) -> QgsOutput {
    to_qgs(map, &options(), "/tmp/output").unwrap()
}

#[test]
fn xml_roundtrip_preserves_names_tree_order_visibility_and_distinct_ids() {
    let model = map();
    let rendered = render(&model);
    let document = Document::parse(&rendered.xml).unwrap();
    let root = document.root_element();
    assert_eq!(root.attribute("projectname"), model.name.as_deref());
    let tree = root
        .children()
        .find(|n| n.has_tag_name("layer-tree-group"))
        .unwrap();
    let children: Vec<_> = tree.children().filter(|n| n.is_element()).collect();
    assert_eq!(children[0].attribute("name"), Some("组 <&> \"'\t\n"));
    assert_eq!(children[0].attribute("checked"), Some("Qt::Unchecked"));
    assert_eq!(children[1].attribute("name"), Some("断链"));
    let leaves: Vec<_> = tree
        .descendants()
        .filter(|n| n.has_tag_name("layer-tree-layer"))
        .collect();
    let ids: Vec<_> = leaves.iter().map(|n| n.attribute("id").unwrap()).collect();
    assert_eq!(
        ids,
        [
            "mxd2qgz_frame_0_0_0",
            "mxd2qgz_frame_0_0_1_0",
            "mxd2qgz_frame_0_1"
        ]
    );
    assert_eq!(leaves[0].attribute("checked"), Some("Qt::Checked"));
    assert_eq!(leaves[1].attribute("checked"), Some("Qt::Unchecked"));
    let layer_ids: Vec<_> = document
        .descendants()
        .filter(|n| n.has_tag_name("maplayer"))
        .map(|n| {
            n.children()
                .find(|n| n.has_tag_name("id"))
                .unwrap()
                .text()
                .unwrap()
        })
        .collect();
    assert_eq!(ids, layer_ids);
    let canvas = document
        .descendants()
        .find(|n| n.has_tag_name("DefaultViewExtent"))
        .unwrap();
    assert_eq!(canvas.attribute("xmax"), Some("10"));
    assert_eq!(
        canvas
            .descendants()
            .find(|n| n.has_tag_name("authid"))
            .unwrap()
            .text(),
        Some("EPSG:3857")
    );
    for srs in document
        .descendants()
        .filter(|n| n.has_tag_name("maplayer"))
    {
        assert_eq!(
            srs.descendants()
                .find(|n| n.has_tag_name("authid"))
                .unwrap()
                .text(),
            Some("EPSG:4326")
        );
    }
    assert!(
        rendered
            .diagnostics
            .iter()
            .any(|d| d.code == "missing-source")
    );
    assert!(
        rendered
            .diagnostics
            .iter()
            .any(|d| d.code == "source-order")
    );
    assert!(
        rendered
            .diagnostics
            .iter()
            .all(|d| !d.message.contains(env!("CARGO_MANIFEST_DIR")))
    );
}

#[test]
fn zip_has_one_safe_xml_member_and_deterministic_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first.qgz");
    let second = temp.path().join("second.QGZ");
    let plain = temp.path().join("plain.qgs");
    for path in [&first, &second, &plain] {
        write_project(&map(), path, &options()).unwrap();
    }
    assert_eq!(fs::read(&first).unwrap(), fs::read(&second).unwrap());
    let mut archive = zip::ZipArchive::new(fs::File::open(&first).unwrap()).unwrap();
    assert_eq!(archive.len(), 1);
    let mut member = archive.by_index(0).unwrap();
    assert_eq!(member.name(), "project.qgs");
    assert_eq!(member.enclosed_name().unwrap(), Path::new("project.qgs"));
    let mut xml = String::new();
    member.read_to_string(&mut xml).unwrap();
    Document::parse(&xml).unwrap();
    assert_eq!(xml, fs::read_to_string(&plain).unwrap());
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 3);
}

#[test]
fn stable_ids_do_not_depend_on_names_sources_or_visibility() {
    let first = render(&map());
    let mut changed = map();
    changed.name = Some("新名".into());
    if let LayerNode::Layer(l) = &mut changed.frames[0].children[1] {
        l.name = "另一层".into();
        l.visible = false;
        l.source.as_mut().unwrap().path = Some("different.shp".into());
    }
    let second = render(&changed);
    let ids = |s: &str| {
        Document::parse(s)
            .unwrap()
            .descendants()
            .filter(|n| n.has_tag_name("id"))
            .map(|n| n.text().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(&first.xml), ids(&second.xml));
    assert_eq!(render(&map()).xml, first.xml);
}

#[test]
fn source_paths_are_xml_escaped_and_foreign_paths_are_diagnosed() {
    let mut model = map();
    model.frames[0].children = vec![layer("路径", "C:/synthetic/中文 & \"quoted\".shp", true)];
    let output = render(&model);
    let document = Document::parse(&output.xml).unwrap();
    let datasource = document
        .descendants()
        .find(|n| n.has_tag_name("datasource"))
        .unwrap();
    assert_eq!(
        datasource.text(),
        Some("C:/synthetic/中文 & \"quoted\".shp")
    );
    if !cfg!(windows) {
        assert!(output.diagnostics.iter().any(|d| d.code == "foreign-path"));
    }
}

#[test]
fn concurrent_noclobber_writers_publish_exactly_one_complete_archive() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("race.qgz");
    let barrier = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let destination = &destination;
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    write_project(&map(), destination, &options())
                })
            })
            .collect();
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
        assert!(
            results
                .iter()
                .filter_map(|r| r.as_ref().err())
                .all(|e| e.kind() == ErrorKind::AlreadyExists)
        );
    });
    assert!(zip::ZipArchive::new(fs::File::open(destination).unwrap()).is_ok());
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
}

#[test]
fn path_rebasing_is_lexical_and_platform_independent() {
    for (path, source, output, absolute, relative) in [
        (
            "data/roads.shp",
            "/maps/source",
            "/maps/out",
            "/maps/source/data/roads.shp",
            "../source/data/roads.shp",
        ),
        (
            "../data/中文 & 道路.SHP",
            "/maps/source",
            "/maps",
            "/maps/data/中文 & 道路.SHP",
            "./data/中文 & 道路.SHP",
        ),
        (
            r"data\roads.shp",
            r"C:\maps\source",
            "c:/maps/out",
            "C:/maps/source/data/roads.shp",
            "../source/data/roads.shp",
        ),
        (
            r"C:\data\roads.shp",
            "D:/source",
            "D:/out",
            "C:/data/roads.shp",
            "C:/data/roads.shp",
        ),
        (
            r"\\server\share\roads.shp",
            "C:/source",
            "/out",
            "//server/share/roads.shp",
            "//server/share/roads.shp",
        ),
        (
            "../roads.shp",
            "//server/share/maps",
            "//SERVER/SHARE/out",
            "//server/share/roads.shp",
            "../roads.shp",
        ),
        (
            "/data/roads.shp",
            "C:/source",
            "/out",
            "/data/roads.shp",
            "../data/roads.shp",
        ),
    ] {
        let actual = source_path::resolve(path, source, output).unwrap();
        assert_eq!(actual.absolute, absolute);
        assert_eq!(actual.qgis, relative);
    }
    assert!(
        source_path::resolve("C:/data/a.shp", "C:/source", "/out")
            .unwrap()
            .foreign
            == !cfg!(windows)
    );
}

#[test]
fn ambiguous_uri_and_root_escape_paths_are_rejected() {
    for path in [
        "",
        "C:roads.shp",
        r"\roads.shp",
        "//server",
        "//server//roads.shp",
        "../../../roads.shp",
        "https://example/a.shp",
        "a.shp|layerid=0",
        "a.shp?x=1",
        "a\0.shp",
        "C:/../roads.shp",
    ] {
        assert!(
            source_path::resolve(path, "/maps/source", "/out").is_err(),
            "{path:?}"
        );
    }
    assert!(source_path::resolve("a.shp", "relative", "/out").is_err());
    assert!(source_path::resolve("a.shp", "/source", "relative").is_err());
}

#[test]
fn absent_and_unsupported_crs_are_diagnosed_and_not_replaced_with_epsg() {
    let mut model = map();
    model.frames[0].children.clear();
    model.frames[0].crs = Some(crs("LOCAL_CS[\"未知 & 本地\"]"));
    let rendered = render(&model);
    let document = Document::parse(&rendered.xml).unwrap();
    assert!(!document.descendants().any(|n| n.has_tag_name("authid")));
    assert!(
        !document
            .descendants()
            .any(|n| n.has_tag_name("DefaultViewExtent"))
    );
    assert!(
        document
            .descendants()
            .any(|n| n.attribute("name") == Some("sourceCrs")
                && n.text() == Some("LOCAL_CS[\"未知 & 本地\"]"))
    );
    assert!(
        rendered
            .diagnostics
            .iter()
            .any(|d| d.code == "unsupported-crs")
    );
    assert!(
        rendered
            .diagnostics
            .iter()
            .any(|d| d.code == "extent-without-crs")
    );
    model.frames[0].crs = None;
    assert!(
        render(&model)
            .diagnostics
            .iter()
            .any(|d| d.code == "missing-crs")
    );
}

#[test]
fn unknown_bytes_are_reported_without_exporting_sensitive_content() {
    let mut model = map();
    model.unknown_fields.push(UnknownField {
        stream: "sensitive".into(),
        offset: u64::MAX,
        bytes: b"secret".to_vec(),
    });
    let rendered = render(&model);
    assert!(
        rendered
            .diagnostics
            .iter()
            .any(|d| d.code == "unknown-fields")
    );
    assert!(!rendered.xml.contains("secret"));
}

#[test]
fn invalid_xml_extent_multiple_frames_and_unsupported_sources_leave_no_output() {
    let temp = tempfile::tempdir().unwrap();
    let mut cases = vec![];
    let mut m = map();
    m.name = Some("bad\u{1}".into());
    cases.push(m);
    for extent in [
        Extent {
            xmin: f64::NAN,
            ..extent()
        },
        Extent {
            xmax: f64::INFINITY,
            ..extent()
        },
        Extent {
            xmax: -1.,
            ..extent()
        },
        Extent {
            xmax: 0.,
            ..extent()
        },
        Extent {
            xmin: -f64::MAX,
            xmax: f64::MAX,
            ..extent()
        },
    ] {
        let mut m = map();
        m.frames[0].extent = Some(extent);
        cases.push(m);
    }
    let mut m = map();
    m.frames.push(m.frames[0].clone());
    cases.push(m);
    for mutation in 0..6 {
        let mut m = map();
        if let LayerNode::Layer(l) = &mut m.frames[0].children[1] {
            match mutation {
                0 => l.source = None,
                1 => l.source.as_mut().unwrap().provider = Some("postgres".into()),
                2 => l.source.as_mut().unwrap().connection = Some("password=secret".into()),
                3 => l.source.as_mut().unwrap().path = Some("data.tif".into()),
                4 => l.source.as_mut().unwrap().path = None,
                _ => {
                    l.extent = Some(Extent {
                        ymin: 10.,
                        ..extent()
                    })
                }
            }
        }
        cases.push(m);
    }
    for m in cases {
        assert_eq!(
            write_project(&m, &temp.path().join("bad.qgz"), &options())
                .unwrap_err()
                .kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }
}

#[test]
fn output_errors_and_overwrite_policy_preserve_existing_data_and_clean_temporary_files() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("existing.qgz");
    fs::write(&destination, b"keep me").unwrap();
    assert_eq!(
        write_project(&map(), &destination, &options())
            .unwrap_err()
            .kind(),
        ErrorKind::AlreadyExists
    );
    assert_eq!(fs::read(&destination).unwrap(), b"keep me");
    assert!(write_project(&map(), &temp.path().join("x.zip"), &options()).is_err());
    assert!(write_project(&map(), &temp.path().join("missing/x.qgs"), &options()).is_err());
    let mut overwrite = options();
    overwrite.overwrite = true;
    let mut bad = map();
    bad.name = Some("\0".into());
    assert!(write_project(&bad, &destination, &overwrite).is_err());
    assert_eq!(fs::read(&destination).unwrap(), b"keep me");
    let directory = temp.path().join("directory.qgz");
    fs::create_dir(&directory).unwrap();
    // 写入已完成但发布失败，必须清理同目录临时文件。
    assert!(write_project(&map(), &directory, &overwrite).is_err());
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    write_project(&map(), &destination, &overwrite).unwrap();
    assert!(zip::ZipArchive::new(fs::File::open(destination).unwrap()).is_ok());
}

#[cfg(unix)]
#[test]
fn dangling_symlink_is_not_overwritten_by_default() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("link.qgs");
    std::os::unix::fs::symlink("absent", &path).unwrap();
    assert_eq!(
        write_project(&map(), &path, &options()).unwrap_err().kind(),
        ErrorKind::AlreadyExists
    );
    assert!(fs::symlink_metadata(path).unwrap().is_symlink());
}

#[test]
fn empty_map_has_explicit_invalid_crs_and_valid_xml() {
    let rendered = render(&Map {
        name: None,
        frames: vec![],
        unknown_fields: vec![],
    });
    let doc = Document::parse(&rendered.xml).unwrap();
    assert!(doc.descendants().any(|n| n.has_tag_name("projectCrs")));
    assert!(rendered.diagnostics.iter().any(|d| d.code == "empty-map"));
}

#[test]
fn recursion_limit_returns_error() {
    let mut node = layer("leaf", "a.shp", true);
    for _ in 0..130 {
        node = LayerNode::Group(LayerGroup {
            name: "g".into(),
            visible: true,
            children: vec![node],
            unknown_fields: vec![],
        });
    }
    let mut model = map();
    model.frames[0].children = vec![node];
    assert!(to_qgs(&model, &options(), "/tmp/out").is_err());
}

#[test]
#[ignore = "需要 /usr/bin/python3 的 QGIS API；运行 cargo test --test qgis qgis_api -- --ignored --nocapture"]
fn qgis_api_loads_constructed_qgs_qgz_and_baseline() {
    let temp = tempfile::tempdir().unwrap();
    for ext in ["qgs", "qgz"] {
        write_project(
            &map(),
            &temp.path().join(format!("constructed.{ext}")),
            &options(),
        )
        .unwrap();
    }
    let mut unknown = map();
    unknown.frames[0].children.clear();
    unknown.frames[0].crs = Some(crs("unknown"));
    write_project(&unknown, &temp.path().join("unknown.qgs"), &options()).unwrap();
    write_project(
        &Map {
            name: None,
            frames: vec![],
            unknown_fields: vec![],
        },
        &temp.path().join("empty.qgz"),
        &options(),
    )
    .unwrap();
    let result = std::process::Command::new("/usr/bin/python3")
        .arg(fixtures().join("verify_api.py"))
        .arg(temp.path())
        .env("QT_QPA_PLATFORM", "offscreen")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    println!("{}", String::from_utf8_lossy(&result.stdout));
}
