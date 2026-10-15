use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::SecretKey;
use mars_xlog_core::decoder::{decode_xlog, DecodeOptions};
use rand_core::OsRng;
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(
    name = "mars-xlog",
    version,
    about = "Decode Tencent Mars xlog files, log directories, and Feedback diagnostic ZIP packages."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Input .xlog file, directory, or feedback .zip archive (flag).
    #[arg(short = 'i', long = "input", value_name = "INPUT_FLAG", global = true)]
    flag_input: Option<PathBuf>,

    /// Output file or target directory (flag).
    #[arg(short = 'o', long = "output", value_name = "OUTPUT_FLAG", global = true)]
    flag_output: Option<PathBuf>,

    /// Input .xlog file, directory, or feedback .zip archive.
    #[arg(value_name = "INPUT", global = true)]
    input: Option<PathBuf>,

    /// Output file or target directory (optional).
    #[arg(value_name = "OUTPUT", global = true)]
    output: Option<PathBuf>,

    /// 64-hex-character private key for encrypted xlog blocks.
    #[arg(short = 'k', short_alias = 'p', long, alias = "private-key", env = "MARS_XLOG_KEY", global = true)]
    key: Option<String>,

    /// Clean (delete) original .xlog files after successful decoding.
    #[arg(long, global = true)]
    clean: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Decode one xlog file, directory, or feedback zip package.
    Decode(DecodeArgs),
    /// Generate a secp256k1 key pair compatible with Mars xlog encryption.
    GenKey,
}

#[derive(Debug, Args)]
struct DecodeArgs {
    /// Input .xlog file, directory, or feedback .zip archive (flag).
    #[arg(short = 'i', long = "input", value_name = "INPUT_FLAG")]
    input: Option<PathBuf>,

    /// Output file or target directory (flag).
    #[arg(short = 'o', long = "output", value_name = "OUTPUT_FLAG")]
    output: Option<PathBuf>,

    /// Positional input argument.
    #[arg(value_name = "INPUT")]
    pos_input: Option<PathBuf>,

    /// Positional output argument.
    #[arg(value_name = "OUTPUT")]
    pos_output: Option<PathBuf>,

    /// 64-hex-character private key for encrypted xlog blocks.
    #[arg(short = 'k', short_alias = 'p', long, alias = "private-key")]
    key: Option<String>,

    /// Clean (delete) original .xlog files after successful decoding.
    #[arg(long)]
    clean: bool,
}

#[derive(Debug, Error)]
enum CliError {
    #[error("input path does not exist: {0}")]
    InputNotFound(String),
    #[error("input path is required. Run with --help for usage.")]
    MissingInput,
    #[error("I/O error at {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("ZIP error: {0}")]
    Zip(String),
    #[error(transparent)]
    Decode(#[from] mars_xlog_core::decoder::DecodeError),
}

fn main() {
    if let Err(err) = run() {
        eprintln!("\n❌ Error: {err}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Decode(args)) => {
            let input = args
                .input
                .or(args.pos_input)
                .or(cli.flag_input)
                .or(cli.input)
                .ok_or(CliError::MissingInput)?;
            let output = args.output.or(args.pos_output).or(cli.flag_output).or(cli.output);
            let key = args.key.or(cli.key);
            let clean = args.clean || cli.clean;
            process_input(input, output, key, clean)
        }
        Some(Command::GenKey) => {
            print_key_pair();
            Ok(())
        }
        None => {
            let input = cli.flag_input.or(cli.input).ok_or(CliError::MissingInput)?;
            let output = cli.flag_output.or(cli.output);
            process_input(input, output, cli.key, cli.clean)
        }
    }
}

fn process_input(
    input: PathBuf,
    output: Option<PathBuf>,
    key: Option<String>,
    clean: bool,
) -> Result<(), CliError> {
    if !input.exists() {
        return Err(CliError::InputNotFound(input.display().to_string()));
    }

    if is_zip_file(&input) {
        process_zip(&input, output.as_deref(), key.as_deref(), clean)
    } else if input.is_dir() {
        process_directory(&input, output.as_deref(), key.as_deref(), clean)
    } else {
        process_single_file(&input, output.as_deref(), key.as_deref(), clean)
    }
}

fn is_zip_file(path: &Path) -> bool {
    if path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("zip")) {
        return true;
    }
    // Also check magic PK\x03\x04
    if let Ok(mut file) = fs::File::open(path) {
        use std::io::Read;
        let mut magic = [0u8; 4];
        if file.read_exact(&mut magic).is_ok() && &magic == b"PK\x03\x04" {
            return true;
        }
    }
    false
}

fn process_zip(
    zip_path: &Path,
    output_dir: Option<&Path>,
    key: Option<&str>,
    clean: bool,
) -> Result<(), CliError> {
    let target_dir = match output_dir {
        Some(dir) => dir.to_path_buf(),
        None => {
            let stem = zip_path.file_stem().and_then(|s| s.to_str()).unwrap_or("feedback");
            let parent = zip_path.parent().unwrap_or_else(|| Path::new("."));
            parent.join(format!("{}_decoded", stem))
        }
    };

    println!("\n📦 Extracting ZIP: {}", zip_path.display());
    println!("📂 Output directory: {}\n", target_dir.display());

    fs::create_dir_all(&target_dir).map_err(|e| CliError::Io {
        path: target_dir.display().to_string(),
        source: e,
    })?;

    let file = fs::File::open(zip_path).map_err(|e| CliError::Io {
        path: zip_path.display().to_string(),
        source: e,
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| CliError::Zip(e.to_string()))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| CliError::Zip(e.to_string()))?;
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath).map_err(|e| CliError::Io {
                path: outpath.display().to_string(),
                source: e,
            })?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| CliError::Io {
                        path: p.display().to_string(),
                        source: e,
                    })?;
                }
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| CliError::Io {
                path: outpath.display().to_string(),
                source: e,
            })?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| CliError::Io {
                path: outpath.display().to_string(),
                source: e,
            })?;
        }
    }

    let mut xlog_files = Vec::new();
    for entry in walkdir::WalkDir::new(&target_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && entry.path().extension().map_or(false, |ext| ext.eq_ignore_ascii_case("xlog"))
        {
            xlog_files.push(entry.into_path());
        }
    }
    xlog_files.sort();

    println!("Found {} .xlog file(s) to decode:", xlog_files.len());
    let mut success_count = 0;
    for xlog_file in &xlog_files {
        let log_file = xlog_file.with_extension("log");
        let rel_path = xlog_file.strip_prefix(&target_dir).unwrap_or(xlog_file);
        print!("• {}: ", rel_path.display());
        match decode_single_xlog(xlog_file, &log_file, key) {
            Ok(bytes) => {
                println!(
                    "[OK] -> {} ({} bytes)",
                    log_file.file_name().unwrap_or_default().to_string_lossy(),
                    bytes
                );
                success_count += 1;
                if clean {
                    let _ = fs::remove_file(xlog_file);
                }
            }
            Err(e) => {
                println!("[FAIL] {}", e);
            }
        }
    }

    let summary_file = target_dir.join("export_summary.txt");
    if summary_file.is_file() {
        if let Ok(content) = fs::read_to_string(&summary_file) {
            println!("\n{}", "=".repeat(70));
            println!("📝 Diagnostic Package Summary (export_summary.txt):");
            println!("{}", "=".repeat(70));
            println!("{}", content.trim());
            println!("{}", "=".repeat(70));
        }
    }

    println!("\n✨ Completed: {}/{} logs decoded successfully.", success_count, xlog_files.len());
    println!("📁 Decoded logs available at: {}\n", target_dir.display());
    Ok(())
}

fn process_directory(
    dir_path: &Path,
    output_dir: Option<&Path>,
    key: Option<&str>,
    clean: bool,
) -> Result<(), CliError> {
    let mut xlog_files = Vec::new();
    for entry in walkdir::WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && entry.path().extension().map_or(false, |ext| ext.eq_ignore_ascii_case("xlog"))
        {
            xlog_files.push(entry.into_path());
        }
    }
    xlog_files.sort();

    if xlog_files.is_empty() {
        println!("No .xlog files found in directory: {}", dir_path.display());
        return Ok(());
    }

    println!("\n📂 Processing directory: {}", dir_path.display());
    println!("Found {} .xlog file(s) to decode:\n", xlog_files.len());

    let mut success_count = 0;
    for xlog_file in &xlog_files {
        let rel_path = xlog_file.strip_prefix(dir_path).unwrap_or(xlog_file);
        let log_file = match output_dir {
            Some(out_dir) if out_dir != dir_path => {
                let target = out_dir.join(rel_path).with_extension("log");
                if let Some(p) = target.parent() {
                    let _ = fs::create_dir_all(p);
                }
                target
            }
            _ => xlog_file.with_extension("log"),
        };

        print!("• {}: ", rel_path.display());
        match decode_single_xlog(xlog_file, &log_file, key) {
            Ok(bytes) => {
                println!(
                    "[OK] -> {} ({} bytes)",
                    log_file.file_name().unwrap_or_default().to_string_lossy(),
                    bytes
                );
                success_count += 1;
                if clean {
                    let _ = fs::remove_file(xlog_file);
                }
            }
            Err(e) => {
                println!("[FAIL] {}", e);
            }
        }
    }

    println!("\n✨ Completed: {}/{} logs decoded successfully.\n", success_count, xlog_files.len());
    Ok(())
}

fn process_single_file(
    input: &Path,
    output: Option<&Path>,
    key: Option<&str>,
    clean: bool,
) -> Result<(), CliError> {
    let output_path = match output {
        Some(out) => {
            if out.is_dir() {
                out.join(input.file_name().unwrap_or_default()).with_extension("log")
            } else {
                out.to_path_buf()
            }
        }
        None => input.with_extension("log"),
    };

    print!("• {}: ", input.display());
    let bytes = decode_single_xlog(input, &output_path, key)?;
    println!(
        "[OK] -> {} ({} bytes)",
        output_path.display(),
        bytes
    );
    if clean {
        let _ = fs::remove_file(input);
    }
    Ok(())
}

fn decode_single_xlog(
    input_file: &Path,
    output_file: &Path,
    key: Option<&str>,
) -> Result<usize, CliError> {
    if let Some(p) = output_file.parent() {
        if !p.exists() {
            let _ = fs::create_dir_all(p);
        }
    }

    let metadata = fs::metadata(input_file).map_err(|e| CliError::Io {
        path: input_file.display().to_string(),
        source: e,
    })?;

    if metadata.len() == 0 {
        fs::write(output_file, b"").map_err(|e| CliError::Io {
            path: output_file.display().to_string(),
            source: e,
        })?;
        return Ok(0);
    }

    let bytes = fs::read(input_file).map_err(|e| CliError::Io {
        path: input_file.display().to_string(),
        source: e,
    })?;

    let mut opts = DecodeOptions::new();
    if let Some(k) = key {
        opts = opts.private_key_hex(k).map_err(CliError::Decode)?;
    }

    let mut decoded = Vec::with_capacity(bytes.len());
    let summary = decode_xlog(&bytes, &mut decoded, &opts).map_err(CliError::Decode)?;
    fs::write(output_file, &decoded).map_err(|e| CliError::Io {
        path: output_file.display().to_string(),
        source: e,
    })?;

    Ok(summary.output_bytes)
}

fn print_key_pair() {
    let secret = SecretKey::random(&mut OsRng);
    let private_key = secret.to_bytes();
    let point = secret.public_key().to_encoded_point(false);
    let bytes = point.as_bytes();

    println!("============================================================");
    println!("🔑 secp256k1 Key Pair for Mars Xlog Encryption");
    println!("============================================================");
    println!("Private Key (64 hex chars - KEEP SECRET FOR DECODER):");
    println!("{}", hex::encode(private_key));
    println!("\nPublic Key (128 hex chars - CONFIGURE IN ANDROID APP):");
    println!("{}", hex::encode(&bytes[1..]));
    println!("============================================================");
}

#[cfg(test)]
mod tests {
    use super::*;
    use mars_xlog_core::compress::{StreamCompressor, ZlibStreamCompressor};
    use mars_xlog_core::protocol::{
        select_magic, AppendMode, CompressionKind, LogHeader,
    };
    use std::io::Write;
    use tempfile::tempdir;

    fn make_test_xlog_bytes(content: &[u8]) -> Vec<u8> {
        let mut zlib = ZlibStreamCompressor::new(6);
        let mut zlib_payload = Vec::new();
        zlib.compress_chunk(content, &mut zlib_payload).unwrap();
        zlib.flush(&mut zlib_payload).unwrap();

        let header = LogHeader {
            magic: select_magic(CompressionKind::Zlib, AppendMode::Async, false),
            seq: 1,
            begin_hour: 1,
            end_hour: 1,
            len: zlib_payload.len() as u32,
            client_pubkey: [0; 64],
        };

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&header.encode());
        bytes.extend_from_slice(&zlib_payload);
        bytes.push(0);
        bytes
    }

    #[test]
    fn test_single_file_decode() {
        let dir = tempdir().unwrap();
        let xlog_path = dir.path().join("test.xlog");
        let log_path = dir.path().join("test.log");

        let log_text = b"Hello Mars Xlog CLI test line\n";
        fs::write(&xlog_path, make_test_xlog_bytes(log_text)).unwrap();

        process_single_file(&xlog_path, Some(&log_path), None, false).unwrap();

        let decoded = fs::read(&log_path).unwrap();
        assert_eq!(decoded, log_text);
    }

    #[test]
    fn test_directory_decode() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir_all(&sub).unwrap();

        let xlog1 = dir.path().join("root.xlog");
        let xlog2 = sub.join("child.xlog");

        fs::write(&xlog1, make_test_xlog_bytes(b"root log\n")).unwrap();
        fs::write(&xlog2, make_test_xlog_bytes(b"child log\n")).unwrap();

        process_directory(dir.path(), None, None, false).unwrap();

        assert_eq!(fs::read(dir.path().join("root.log")).unwrap(), b"root log\n");
        assert_eq!(fs::read(sub.join("child.log")).unwrap(), b"child log\n");
    }

    #[test]
    fn test_zip_decode() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("feedback.zip");

        // Create zip with export_summary.txt and a .xlog file
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(file);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip_writer.start_file("export_summary.txt", options).unwrap();
        zip_writer.write_all(b"User ID: 3920\nApp Version: 3.31.0\n").unwrap();

        zip_writer.start_file("main.xlog", options).unwrap();
        zip_writer.write_all(&make_test_xlog_bytes(b"main process crash log\n")).unwrap();

        zip_writer.finish().unwrap();

        // Process zip
        let out_dir = dir.path().join("feedback_decoded");
        process_zip(&zip_path, Some(&out_dir), None, false).unwrap();

        assert!(out_dir.join("export_summary.txt").is_file());
        assert_eq!(
            fs::read(out_dir.join("main.log")).unwrap(),
            b"main process crash log\n"
        );
    }
}
