//! `/Version` 识别候选的合成边界测试和仓库参考样本的辅助观察对照。
//!
//! 全部结果都是候选归类，不构成 ArcMap 或 QGIS 中的软件验证。
use mxd2qgz::mxd_version::{MAX_TEXT_BYTES, MxdVersionBranch, identify_version};
use mxd2qgz::version_stream::write_arcmap_header_10_1;
use std::ffi::OsStr;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 按 String::Write 普通输出布局编码文本（长度前缀 + 终止零 UTF-16LE）。
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

/// 合成完整 /Version 字节；short 大于 11 时按规则附带 4 字节计数。
fn header_bytes(text: &str, short: i16, count: Option<i32>) -> Vec<u8> {
    let mut bytes = encoded(text);
    bytes.extend(short.to_le_bytes());
    if let Some(value) = count {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

#[test]
fn six_save_indices_recognize_the_static_table_candidates() {
    let shorts = [10_i16, 13, 14, 16, 18, 19];
    for (index, short) in shorts.iter().enumerate() {
        let count = (index != 0).then_some(123_i32);
        let bytes = write_arcmap_header_10_1(index as i32, "10.1", count).unwrap();
        let report = identify_version(&bytes).unwrap();
        assert_eq!(report.branch, MxdVersionBranch::ArcMap10_1KnownCandidate);
        assert_eq!(report.text, "10.1");
        assert_eq!(report.compatibility_short, *short);
        assert_eq!(report.object_count, count);
        assert_eq!(report.remaining_bytes, 0);
        assert!(report.note.contains("未经运行时确认"));
        assert!(report.note.contains("未在目标软件中验证"));
    }
    // 原生对超过表末尾的索引截到末项 19；这不表明它是已知版本之外的编号。
    for index in [6, 255, i32::MAX] {
        let bytes = write_arcmap_header_10_1(index, "10.1", Some(7)).unwrap();
        let report = identify_version(&bytes).unwrap();
        assert_eq!(report.branch, MxdVersionBranch::ArcMap10_1KnownCandidate);
        assert_eq!(report.compatibility_short, 19);
        assert_eq!(report.object_count, Some(7));
    }
}

#[test]
fn count_boundary_shorts_11_and_12_stay_unrecognized_candidates() {
    // short 11：无计数字段；short 12：条件计数出现。两者都不在静态表内。
    for (short, count) in [(11_i16, None), (12_i16, Some(-1))] {
        let bytes = header_bytes("版本", short, count);
        let report = identify_version(&bytes).unwrap();
        assert_eq!(report.branch, MxdVersionBranch::ArcMap10_1Unrecognized);
        assert_eq!(report.compatibility_short, short);
        assert_eq!(report.object_count, count);
        assert_eq!(report.remaining_bytes, 0);
        assert!(report.note.contains("未识别文本/short"));
    }
}

#[test]
fn unknown_text_with_known_short_reports_every_field() {
    let bytes = write_arcmap_header_10_1(5, "未知", Some(5)).unwrap();
    let report = identify_version(&bytes).unwrap();
    assert_eq!(report.branch, MxdVersionBranch::ArcMap10_1Unrecognized);
    assert_eq!(report.text, "未知");
    assert_eq!(report.compatibility_short, 19);
    assert_eq!(report.object_count, Some(5));
    assert_eq!(report.remaining_bytes, 0);
    assert!(report.note.contains("原生接受任意文本"));
}

#[test]
fn in_range_unknown_shorts_differ_from_known_boundaries() {
    // 0x12(18) 与 0x13(19) 是静态表已知值，与同样合法的未知位模式区分开。
    for short in [0x12_i16, 0x13] {
        let count = (short > 11).then_some(9);
        let report = identify_version(&header_bytes("10.1", short, count)).unwrap();
        assert_eq!(report.branch, MxdVersionBranch::ArcMap10_1KnownCandidate);
    }
    // 合法但不在静态表内的位模式（含负数与边界内侧值）仍完整报告字段。
    for (short, count) in [
        (0_i16, None),
        (11_i16, None),
        (15_i16, Some(0)),
        (17_i16, Some(0)),
        (-1_i16, None),
    ] {
        let bytes = header_bytes("10.1", short, count);
        let report = identify_version(&bytes).unwrap();
        assert_eq!(report.branch, MxdVersionBranch::ArcMap10_1Unrecognized);
        assert_eq!(report.compatibility_short, short);
        assert_eq!(report.object_count, count);
    }
}

#[test]
fn shorts_above_0x13_fall_back_to_the_10_8_layout_candidate() {
    // 10.1 严格读取拒绝大于 0x13 的 short；10.8 布局保留全部位模式。
    // 该分支不读取对象计数，报告恒为 None；计数 4 字节留在未知尾部。
    for short in [0x14_i16, 0x7fff] {
        let bytes = header_bytes("10.1", short, Some(7));
        let report = identify_version(&bytes).unwrap();
        assert_eq!(report.branch, MxdVersionBranch::Layout10_8Fallback);
        assert_eq!(report.text, "10.1");
        assert_eq!(report.compatibility_short, short);
        assert_eq!(report.object_count, None);
        assert_eq!(report.remaining_bytes, 4);
        assert!(report.note.contains("仅 10.8 布局候选"));
    }
}

#[test]
fn truncation_prefixes_split_between_errors_and_10_8_fallback() {
    let full = write_arcmap_header_10_1(5, "10.1", Some(7)).unwrap();
    // 文本加 short 的完整前缀长度；更短的前缀在两个读取器中都失败。
    let complete_prefix = encoded("10.1").len() + 2;
    for length in 0..full.len() {
        let report = identify_version(&full[..length]);
        if length < complete_prefix {
            let error = report.unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof, "{length}");
        } else {
            let report = report.unwrap();
            assert_eq!(report.branch, MxdVersionBranch::Layout10_8Fallback);
            assert_eq!(report.remaining_bytes, length - complete_prefix);
        }
    }
    assert_eq!(
        identify_version(&full).unwrap().branch,
        MxdVersionBranch::ArcMap10_1KnownCandidate
    );
}

#[test]
fn empty_garbage_and_odd_text_fail_with_explicit_reasons() {
    let inputs: Vec<Vec<u8>> = vec![
        // 空流。
        Vec::new(),
        // 长度前缀为奇数。
        {
            let mut bytes = vec![3, 0, 0, 0];
            bytes.extend([0x41, 0x00]);
            bytes
        },
        // 声明长度内的内容缺少终止零。
        {
            let mut bytes = vec![0x40, 0, 0, 0];
            bytes.extend([0x41; 0x40]);
            bytes
        },
        // 文本完整但没有 short。
        encoded("10.1"),
        // 与任何合法布局都不匹配的垃圾字节。
        vec![0xff; 32],
    ];
    for bytes in &inputs {
        let error = identify_version(bytes).unwrap_err();
        assert!(
            error.to_string().contains("10.1 规则") && error.to_string().contains("10.8 布局"),
            "{error}"
        );
    }
    assert_eq!(
        identify_version(&inputs[0]).unwrap_err().kind(),
        std::io::ErrorKind::UnexpectedEof
    );
    assert_eq!(
        identify_version(&inputs[1]).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
}

#[test]
fn text_limit_keeps_resource_bound_explicit() {
    // 超出 MAX_TEXT_BYTES 的声明长度在两个读取器中都被拒绝。
    let mut bytes = ((MAX_TEXT_BYTES + 2) as u32).to_le_bytes().to_vec();
    bytes.extend([0x41; MAX_TEXT_BYTES + 2]);
    let error = identify_version(&bytes).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("资源限额"));
}

struct Fixture(PathBuf);

impl Fixture {
    fn with_version(version_bytes: Option<&[u8]>) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "mxd2qgz-version-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let mut document = cfb::create(root.join("地图.mxd")).unwrap();
        if let Some(bytes) = version_bytes {
            document
                .create_stream("/Version")
                .unwrap()
                .write_all(bytes)
                .unwrap();
        }
        Self(root)
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
fn version_command_reports_synthetic_stream_fields() {
    let bytes = write_arcmap_header_10_1(5, "10.1", Some(42)).unwrap();
    let fixture = Fixture::with_version(Some(&bytes));
    let stdout = success(fixture.run(&["version", "地图.mxd"]));
    assert!(stdout.contains("Stream /Version: 20 bytes"), "{stdout}");
    assert!(
        stdout.contains("Branch: ArcMap 10.1 头部规则匹配候选"),
        "{stdout}"
    );
    assert!(stdout.contains("Text: \"10.1\""), "{stdout}");
    assert!(stdout.contains("Compatibility short: 19"), "{stdout}");
    assert!(stdout.contains("Object count: 42"), "{stdout}");
    assert!(stdout.contains("Remaining bytes: 0"), "{stdout}");
    assert!(
        stdout.contains("Note: 文本与 short 都落在 ArcMap 10.1 静态候选集合"),
        "{stdout}"
    );
    assert!(stdout.contains("未在目标软件中验证"), "{stdout}");
}

#[test]
fn version_command_reports_missing_stream_and_usage() {
    let fixture = Fixture::with_version(None);
    let stderr = failure(fixture.run(&["version", "地图.mxd"]));
    assert!(stderr.contains("/Version"), "{stderr}");
    for args in [vec!["version"], vec!["version", "地图.mxd", "extra"]] {
        assert!(
            failure(fixture.run(&args)).contains("usage: mxd2qgz version"),
            "{stderr}"
        );
    }
}

#[test]
fn version_command_rejects_bad_files_without_panicking() {
    let bytes = write_arcmap_header_10_1(5, "10.1", Some(42)).unwrap();
    let fixture = Fixture::with_version(Some(&bytes));
    let original = std::fs::read(fixture.path("地图.mxd")).unwrap();
    for length in [0_usize, 8, 64] {
        std::fs::write(
            fixture.path("坏.mxd"),
            &original[..length.min(original.len())],
        )
        .unwrap();
        let stderr = failure(fixture.run(&["version", "坏.mxd"]));
        assert!(stderr.contains("error: "), "{stderr}");
    }
    // 容器合法但 /Version 内容在两种布局下都无法解析。
    let broken = Fixture::with_version(Some(&[0xff; 32]));
    std::fs::write(broken.path("坏.mxd"), b"not an ole file at all").unwrap();
    failure(broken.run(&["version", "坏.mxd"]));
    failure(broken.run(&["version", "地图.mxd"]));
}

#[test]
fn sample_basemap_matches_previous_auxiliary_observation() {
    // 这是仓库参考样本 samples/测试底图.mxd 的辅助观察回归对照：
    // 文本 "10.1"、short 19、计数 2267、无剩余字节。
    // 样本观察不是目标软件执行验证，不能据此声称识别结果已验证。
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    document
        .open_stream("/Version")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let report = identify_version(&bytes).unwrap();
    assert_eq!(report.branch, MxdVersionBranch::ArcMap10_1KnownCandidate);
    assert_eq!(report.text, "10.1");
    assert_eq!(report.compatibility_short, 19);
    assert_eq!(report.object_count, Some(2267));
    assert_eq!(report.remaining_bytes, 0);
}
