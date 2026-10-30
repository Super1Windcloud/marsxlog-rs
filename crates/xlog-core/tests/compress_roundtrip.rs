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

#[test]
fn zlib_stream_decodes_payload_much_larger_than_input() {
    // Log text often compresses 5-10x. The decoder used to stop once the input
    // was consumed even though the decompressor still held output that did
    // not fit the first output buffer, truncating the block mid-line.
    let line =
        b"[I][2026-10-30 +8.0 12:00:00.000][1234, 5678][tag][file.rs:1, func][same message]\n";
    let plain: Vec<u8> = line.iter().copied().cycle().take(64 * 1024).collect();

    for finish in [false, true] {
        let mut compressor = ZlibStreamCompressor::default();
        let mut out = Vec::new();
        compressor.write_plain(&plain, &mut out).unwrap();
        if finish {
            compressor.finish_into(&mut out).unwrap();
        }
        assert!(out.len() * 10 < plain.len(), "fixture must compress well");

        let decoded = decompress_raw_zlib(&out).unwrap();
        assert_eq!(decoded.len(), plain.len(), "finish={finish}");
        assert_eq!(decoded, plain, "finish={finish}");
    }
}
