//! Standalone GUI binary for Port Sweeper.

fn main() {
    if let Err(e) = psweep::gui::run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
