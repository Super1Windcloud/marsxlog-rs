use mars_xlog_core::compress::{
    decompress_raw_zlib, decompress_zstd_frames, ZlibStreamCompressor, ZstdLevel,
    ZstdStreamCompressor,
};

#[test]
fn zlib_stream_roundtrip() {
    let mut compressor = ZlibStreamCompressor::default();
    let mut out = Vec::new();

    compressor.write_plain(b"hello", &mut out).unwrap();
    compressor.write_plain(b" world", &mut out).unwrap();
    compressor.finish_into(&mut out).unwrap();

    let plain = decompress_raw_zlib(&out).unwrap();
    assert_eq!(plain, b"hello world");
}

#[test]
fn zstd_frames_roundtrip() {
    let mut compressor = ZstdStreamCompressor::new(ZstdLevel::try_from(3).unwrap()).unwrap();
    let mut out = Vec::new();

    compressor.write_plain(b"mars", &mut out).unwrap();
    compressor.write_plain(b" xlog", &mut out).unwrap();
    compressor.finish_into(&mut out).unwrap();

    let plain = decompress_zstd_frames(&out).unwrap();
    assert_eq!(plain, b"mars xlog");
}

#[test]
fn zlib_stream_chunk_without_final_flush_roundtrip() {
    let mut compressor = ZlibStreamCompressor::default();
    let mut out = Vec::new();
    // Only write_plain (Z_SYNC_FLUSH). Do not call finish_into(), which ends the stream.
    // This is the shape of async blocks flushed at the buffer threshold or
    // recovered from the mmap after a crash. flate2 >= 1.1.10 rejects it
    // through its `Read` adapters ("incomplete deflate stream"); the
    // `latest-deps` CI job runs this against the newest flate2.
    compressor.write_plain(b"line-1\n", &mut out).unwrap();
    compressor.write_plain(b"line-2\n", &mut out).unwrap();

    let plain = decompress_raw_zlib(&out).unwrap();
    assert_eq!(plain, b"line-1\nline-2\n");
}
