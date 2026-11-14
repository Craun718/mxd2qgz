//! Synthetic OLE streams exercise the CLI, not the MXD serialization format.
use std::ffi::OsStr;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "mxd2qgz-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let fixture = Self(root);
        let mut document = cfb::create(fixture.path("地图.mxd")).unwrap();
        let bytes: Vec<_> = "地图😀图层\0"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        document
            .create_stream("/Maps")
            .unwrap()
            .write_all(&bytes)
            .unwrap();
        fixture
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_mxd2qgz"))
            .current_dir(&self.0)
            .args(args.iter().map(OsStr::new))
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn success(output: Output) -> String {
    assert!(output.status.success(), "{:?}", output);
    String::from_utf8(output.stdout).unwrap()
}

fn failure(output: Output) -> String {
    assert_eq!(output.status.code(), Some(1), "{:?}", output);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("error: "), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
    stderr
}

#[test]
fn reads_synthetic_streams_through_documented_commands() {
    let fixture = Fixture::new();
    assert!(success(fixture.run(&["inspect", "地图.mxd"])).contains("/Maps"));
    assert!(success(fixture.run(&["strings", "地图.mxd"])).contains("0x00000000  地图😀图层"));
    assert!(
        success(fixture.run(&["hex", "地图.mxd", "/Maps", "0x0", "2"]))
            .contains("0x00000000  30 57")
    );
    assert_eq!(
        success(fixture.run(&["compare", "地图.mxd", "地图.mxd", "/Maps"])),
        "Stream /Maps: 14 vs 14 bytes\n"
    );
    success(fixture.run(&["extract", "地图.mxd", "/Maps", "maps.bin"]));
    let expected: Vec<_> = "地图😀图层\0"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    assert_eq!(std::fs::read(fixture.path("maps.bin")).unwrap(), expected);
    failure(fixture.run(&["extract", "地图.mxd", "/Maps", "maps.bin"]));
    assert_eq!(std::fs::read(fixture.path("maps.bin")).unwrap(), expected);
}

#[test]
fn rejects_invalid_arguments_missing_streams_and_out_of_bounds_ranges() {
    let fixture = Fixture::new();
    for args in [
        vec![],
        vec!["convert", "地图.mxd"],
        vec!["inspect", "地图.mxd", "extra"],
        vec!["hex", "地图.mxd", "/Maps", "0"],
        vec!["compare", "地图.mxd"],
        vec!["extract", "地图.mxd", "/Maps"],
    ] {
        assert!(failure(fixture.run(&args)).contains("usage:"));
    }
    for (offset, length) in [
        ("-1", "1"),
        ("0x", "1"),
        ("0", "999999999999999999999999999999999999"),
        ("14", "1"),
        ("15", "0"),
    ] {
        failure(fixture.run(&["hex", "地图.mxd", "/Maps", offset, length]));
    }
    assert_eq!(
        success(fixture.run(&["hex", "地图.mxd", "/Maps", "14", "0"])),
        ""
    );
    failure(fixture.run(&["hex", "地图.mxd", "/Missing", "0", "1"]));
    failure(fixture.run(&["extract", "地图.mxd", "/Missing", "missing.bin"]));
    assert!(!fixture.path("missing.bin").exists());
    failure(fixture.run(&["inspect", "missing.mxd"]));
}

#[test]
fn rejects_truncated_ole_containers_without_panicking() {
    let fixture = Fixture::new();
    let original = std::fs::read(fixture.path("地图.mxd")).unwrap();
    for length in [0, 1, 8, 64, 511] {
        std::fs::write(fixture.path("truncated.mxd"), &original[..length]).unwrap();
        failure(fixture.run(&["inspect", "truncated.mxd"]));
        failure(fixture.run(&["strings", "truncated.mxd"]));
    }
}

/// 构造显式 10.1 已确认前缀布局的合成 `/Maps` 字节；与单元测试的构造
/// 依据相同（AfCore String 布局、版本 51、版本 6 PCS 体），集成测试与
/// 库内测试无法互相引用，按同一证据独立构造。
fn maps_stream_10_1() -> Vec<u8> {
    use mxd2qgz::map_object::MAP_CLASS_ID;
    use mxd2qgz::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;

    /// AfCore String::Write 布局：u32 字节长度前缀 + 含终止零的 UTF-16LE。
    fn encoded(text: &str) -> Vec<u8> {
        let payload: Vec<_> = text
            .encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect();
        let mut bytes = (payload.len() as u32).to_le_bytes().to_vec();
        bytes.extend(payload);
        bytes
    }

    let wkt = "PROJCS[\"CGCS2000_3_Degree_GK_Zone_37\",AUTHORITY[\"EPSG\",\"4490\"]]";
    let mut bytes = 1_i32.to_le_bytes().to_vec(); // 数据框数量候选
    bytes.extend(MAP_CLASS_ID);
    bytes.extend(0x2a_u32.to_le_bytes()); // 首对象头键
    bytes.extend(51_u16.to_le_bytes()); // Map 架构版本
    bytes.extend(encoded("新建数据框"));
    bytes.extend(encoded("")); // 第二名称：6 字节空名称模式
    bytes.extend(PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
    bytes.extend(0x1e_u32.to_le_bytes()); // PCS 对象头键
    bytes.extend(6_u16.to_le_bytes()); // PCS 保存版本
    bytes.extend((wkt.len() as u32).to_le_bytes());
    bytes.extend(wkt.as_bytes());
    for _ in 0..4 {
        bytes.extend(0_u32.to_le_bytes()); // 空域字符串
    }
    bytes.push(0x00); // 垂向域标志
    bytes.extend(0x0001_u16.to_le_bytes()); // 精度标志
    for value in [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 0.001, 0.001, 0.001] {
        bytes.extend(value.to_le_bytes()); // 7+3 个 double（语义未确认）
    }
    bytes.extend_from_slice(&[0xaa, 0xbb, 0xcc]); // 已确认前缀之后的未知尾部
    bytes
}

/// 在 Fixture 临时目录创建仅含一个数据流的 OLE 容器（Fixture Drop 时清理）。
fn stream_fixture(root: &std::path::Path, name: &str, stream: &[u8]) -> std::path::PathBuf {
    let path = root.join(name);
    let mut document = cfb::create(&path).unwrap();
    document
        .create_stream("/Maps")
        .unwrap()
        .write_all(stream)
        .unwrap();
    path
}

#[test]
fn parse_reports_confirmed_fields_with_diagnostics_on_synthetic_maps() {
    // 合成流不含 /Version 数据流：parse 应输出候选缺失诊断而不失败。
    let fixture = Fixture::new();
    let path = stream_fixture(&fixture.0, "正常.mxd", &maps_stream_10_1());
    let output = success(fixture.run(&["parse", path.file_name().unwrap().to_str().unwrap()]));
    assert!(output.contains("Frame count candidate: 1"), "{output}");
    assert!(output.contains("新建数据框"), "{output}");
    assert!(output.contains("PROJCS["), "{output}");
    assert!(output.contains("Unknown: stream=/Maps"), "{output}");
    assert!(output.contains("Version candidate:"), "{output}");
    assert!(output.contains("Note: "), "{output}");

    // 异常输入：数量为负时返回明确错误。
    let bad = stream_fixture(&fixture.0, "负数.mxd", &(-1_i32).to_le_bytes().to_vec());
    let stderr = failure(fixture.run(&["parse", bad.file_name().unwrap().to_str().unwrap()]));
    assert!(stderr.contains("error:"), "{stderr}");
}
