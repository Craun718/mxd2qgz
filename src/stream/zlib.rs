use flate2::{Decompress, FlushDecompress, Status};
use std::io::{self, ErrorKind};

/// 校验完整的 zlib 输入及声明输出长度。调用者先检查声明输出长度的资源限额。
pub(crate) fn decompress_exact(input: &[u8], output_len: usize) -> io::Result<Vec<u8>> {
    let capacity = output_len
        .checked_add(1)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "zlib 解压缓冲区长度溢出"))?;
    let mut output = vec![0; capacity];
    let mut decoder = Decompress::new(true);
    let status = decoder
        .decompress(input, &mut output, FlushDecompress::Finish)
        .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
    if status != Status::StreamEnd
        || decoder.total_in() != input.len() as u64
        || decoder.total_out() != output_len as u64
    {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "zlib 解压字节数与声明不匹配",
        ));
    }
    output.truncate(output_len);
    Ok(output)
}
