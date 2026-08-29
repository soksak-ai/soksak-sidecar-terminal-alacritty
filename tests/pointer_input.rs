use soksak_kit_sidecar_terminal::mirror::{
    EnginePointerInput, PointerButton, PointerPhase, SelectionModifiers,
};
use soksak_sidecar_terminal_alacritty::engine::Engine;

fn pointer(phase: PointerPhase, button: PointerButton) -> EnginePointerInput {
    EnginePointerInput {
        row: 2,
        col: 1,
        phase,
        button,
        click_count: 1,
        modifiers: SelectionModifiers::default(),
    }
}

#[test]
fn sgr_pointer_encodes_press_release_and_drag_motion() {
    let mut engine = Engine::new(120, 40);
    engine.feed(b"\x1b[?1002h\x1b[?1006h");
    assert_eq!(
        engine.pointer_input(pointer(PointerPhase::Down, PointerButton::Left)).unwrap(),
        b"\x1b[<0;2;3M"
    );
    assert_eq!(
        engine.pointer_input(pointer(PointerPhase::Move, PointerButton::Left)).unwrap(),
        b"\x1b[<32;2;3M"
    );
    assert_eq!(
        engine.pointer_input(pointer(PointerPhase::Up, PointerButton::Left)).unwrap(),
        b"\x1b[<0;2;3m"
    );
}

#[test]
fn any_motion_reports_no_button_with_motion_bit() {
    let mut engine = Engine::new(120, 40);
    engine.feed(b"\x1b[?1003h\x1b[?1006h");
    assert_eq!(
        engine.pointer_input(pointer(PointerPhase::Move, PointerButton::None)).unwrap(),
        b"\x1b[<35;2;3M"
    );
}

#[test]
fn legacy_pointer_release_uses_button_three_and_keeps_modifiers() {
    let mut engine = Engine::new(120, 40);
    engine.feed(b"\x1b[?1000h");
    let mut down = pointer(PointerPhase::Down, PointerButton::Right);
    down.modifiers.control = true;
    assert_eq!(engine.pointer_input(down).unwrap(), [0x1b, b'[', b'M', 50, 34, 35]);
    let mut up = pointer(PointerPhase::Up, PointerButton::Right);
    up.modifiers.control = true;
    assert_eq!(engine.pointer_input(up).unwrap(), [0x1b, b'[', b'M', 51, 34, 35]);
}
