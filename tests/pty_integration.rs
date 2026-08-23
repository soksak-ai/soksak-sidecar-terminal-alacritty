use std::path::{Path, PathBuf};

#[test]
fn warm_restore_uses_the_installed_pty_sidecar() {
    let pty = std::env::var("SOKSAK_PTY_SIDECAR")
        .map(PathBuf::from)
        .expect("SOKSAK_PTY_SIDECAR must name the installed PTY sidecar");
    assert!(
        pty.is_file(),
        "PTY sidecar is not a file: {}",
        pty.display()
    );
    let service = Path::new(env!("CARGO_BIN_EXE_soksak-sidecar-terminal-alacritty"));
    soksak_kit_sidecar_terminal::integration::assert_warm_restore(
        &pty,
        service,
        "soksak-sidecar-terminal-alacritty",
    );
}
