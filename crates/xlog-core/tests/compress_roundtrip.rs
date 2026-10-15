use mars_xlog_core::compress::{
    decompress_raw_zlib, decompress_zstd_frames, StreamCompressor, ZlibStreamCompressor,
    ZstdStreamCompressor,
};

#[test]
fn zlib_stream_roundtrip() {
    let mut compressor = ZlibStreamCompressor::default();
    let mut out = Vec::new();

    compressor.compress_chunk(b"hello", &mut out).unwrap();
    compressor.compress_chunk(b" world", &mut out).unwrap();
    compressor.flush(&mut out).unwrap();

    let plain = decompress_raw_zlib(&out).unwrap();
    assert_eq!(plain, b"hello world");
}

#[test]
fn zstd_frames_roundtrip() {
    let mut compressor = ZstdStreamCompressor::new(3).unwrap();
    let mut out = Vec::new();

    compressor.compress_chunk(b"mars", &mut out).unwrap();
    compressor.compress_chunk(b" xlog", &mut out).unwrap();
    compressor.flush(&mut out).unwrap();

    let plain = decompress_zstd_frames(&out).unwrap();
    assert_eq!(plain, b"mars xlog");
}

#[test]
fn zlib_stream_chunk_without_final_flush_roundtrip() {
    let mut compressor = ZlibStreamCompressor::default();
    let mut out = Vec::new();
    // Only compress_chunk with sync flush, never call compressor.flush() to finish stream
    compressor
        .compress_chunk(b"mars xlog sync flushed chunk", &mut out)
        .unwrap();

    let plain = decompress_raw_zlib(&out).unwrap();
    assert_eq!(plain, b"mars xlog sync flushed chunk");
}
