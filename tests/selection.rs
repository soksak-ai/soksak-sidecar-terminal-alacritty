#[test]
fn adapter_uses_alacrittys_selection_state_and_text() {
    let engine = include_str!("../src/engine.rs");
    for required in ["Selection::new", "selection_to_string", "selection_begin", "selection_range"] {
        assert!(engine.contains(required), "Alacritty selection adapter omits {required}");
    }
}

use soksak_kit_sidecar_terminal::mirror::{
    CellSide, SelectionKind, SelectionModifiers, SelectionPhase, SelectionPoint, SelectionRequest,
};
use soksak_sidecar_terminal_alacritty::Mirror;

fn gesture(id: &str, phase: SelectionPhase, col: u16, side: CellSide) -> SelectionRequest {
    SelectionRequest::Gesture {
        window: "win-a".into(), pane: "tab-a.1".into(), gesture_id: id.into(),
        phase, kind: SelectionKind::Simple,
        point: SelectionPoint { row: 0, col, side },
        modifiers: SelectionModifiers::default(),
    }
}

#[test]
fn simple_drag_uses_alacritty_text_and_row_range() {
    let mut mirror = Mirror::new(20, 2);
    mirror.feed(b"hello world");
    mirror.selection_command(&gesture("sel-1", SelectionPhase::Begin, 0, CellSide::Left), 0)
        .expect("selection begin");
    let selected = mirror
        .selection_command(&gesture("sel-1", SelectionPhase::End, 4, CellSide::Right), 0)
        .expect("selection end");
    assert_eq!(selected.text, "hello");
    assert_eq!(selected.sequence, 2);
    assert_eq!(mirror.selection_range(0), Some((0, 4)));

    let stale = mirror.selection_command(
        &gesture("old", SelectionPhase::Update, 7, CellSide::Right), 0,
    );
    assert!(stale.unwrap_err().starts_with("STALE_GESTURE:"));
}
