fn main() {
    if let Err(err) = xlog_decoder::run() {
        eprintln!("\n❌ Error: {err}");
        std::process::exit(2);
    }
}
