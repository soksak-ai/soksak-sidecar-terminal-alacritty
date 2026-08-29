#[test]
fn adapter_uses_alacrittys_selection_state_and_text() {
    let engine = include_str!("../src/engine.rs");
    for required in ["Selection::new", "selection_to_string", "selection_begin", "selection_range"] {
        assert!(engine.contains(required), "Alacritty selection adapter omits {required}");
    }
}
