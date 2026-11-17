//! 基于介质观察构造的合成版本头测试，不依赖原始模板文件。

use mxd2qgz::version::mxd_version::{MxdVersionBranch, candidate_save_index, identify_version};

#[test]
fn observed_93_pair_maps_only_exactly() {
    // 模板观察到的 9.3/16 头部形态，计数 53 为合成输入。
    let bytes = [
        8, 0, 0, 0, b'9', 0, b'.', 0, b'3', 0, 0, 0, 16, 0, 53, 0, 0, 0,
    ];
    let report = identify_version(&bytes).unwrap();
    assert_eq!(report.branch, MxdVersionBranch::ArcMap10_1KnownCandidate);
    assert_eq!(report.candidate_save_index, Some(3));
    assert_eq!(report.remaining_bytes, 0);
    for short in [10, 13, 14, 18, 19] {
        assert_eq!(candidate_save_index("9.3", short), None);
    }
    for text in ["9.2", "10.0", "9.3 ", "任意文本"] {
        assert_eq!(candidate_save_index(text, 16), None);
    }
    assert_eq!(candidate_save_index("8.1", 10), None);
}
