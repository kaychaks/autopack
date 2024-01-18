use autopack::cli::Cli;
use dialoguer::console::Term;
use std::process;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::try_init().expect("tracing sub init failed");
    Term::stdout().set_title("auto-pack");
    match Cli::new().run().await {
        Ok(_) => process::exit(0),
        Err(e) => {
            e.handle();
            process::exit(1)
        }
    }
}
