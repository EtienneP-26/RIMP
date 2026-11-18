//! Starts the RIMP application.

/// Runs the app and exits with code 1 on failure.
fn main() {
    if let Err(error) = rimp::app::run() {
        eprintln!("rimp: {error}");
        std::process::exit(1);
    }
}
