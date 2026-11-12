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
