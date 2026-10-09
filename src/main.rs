fn main() {
    if let Err(error) = rimp::app::run() {
        eprintln!("rimp: {error}");
        std::process::exit(1);
    }
}
