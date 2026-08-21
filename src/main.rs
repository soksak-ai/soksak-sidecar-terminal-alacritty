use soksak_kit_sidecar_terminal::runtime::run_service;
use soksak_sidecar_terminal_alacritty::Mirror;

fn main() {
    if let Err(error) = run_service(
        "soksak-sidecar-terminal-alacritty",
        |cols, rows| Box::new(Mirror::new(cols, rows)),
        std::env::args().skip(1),
    ) {
        eprintln!("soksak-sidecar-terminal-alacritty: {error}");
        std::process::exit(1);
    }
}
