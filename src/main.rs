use docsearcher::cmd::CliApp;

fn main() {
    if let Err(e) = CliApp::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
