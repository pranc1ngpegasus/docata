fn main() {
    if let Err(err) = docata::run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
