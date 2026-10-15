//! Built-in core decoding and decompression engine for Tencent Mars xlog format.

pub mod compress;
pub mod crypto;
pub mod decoder;
pub mod protocol;

pub use decoder::{decode_xlog, parse_private_key_hex, DecodeError, DecodeOptions, DecodeSummary};
