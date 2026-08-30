use soksak_kit_sidecar_terminal::mirror::{
    EnginePointerInput, EngineWheelInput, EngineWheelRoute, PointerButton, PointerPhase,
    SelectionModifiers,
};
use soksak_sidecar_terminal_alacritty::engine::Engine;

fn wheel() -> EngineWheelInput {
    EngineWheelInput {
        row: 0,
        col: 0,
        horizontal: 0,
        vertical: -1,
        modifiers: SelectionModifiers::default(),
        route: EngineWheelRoute::MouseReport,
    }
}

fn pointer() -> EnginePointerInput {
    EnginePointerInput {
        row: 0,
        col: 0,
        phase: PointerPhase::Down,
        button: PointerButton::Left,
        click_count: 1,
        modifiers: SelectionModifiers::default(),
    }
}

#[test]
fn unsupported_legacy_tracking_modes_are_not_aliased_or_admitted() {
    let mut engine = Engine::new(80, 24);
    engine.feed(b"\x1b[?9h\x1b[?1001h");

    let modes = engine.modes();
    assert!(!modes.mouse_x10);
    assert!(!modes.mouse_highlight);
    assert!(!modes.mouse_click);
    assert!(!modes.mouse_drag);
    assert!(!modes.mouse_motion);
    assert!(engine.wheel_input(wheel()).is_err());
    assert!(engine.pointer_input(pointer()).is_err());
}

#[test]
fn click_tracking_stays_distinct_from_unsupported_legacy_modes() {
    let mut engine = Engine::new(80, 24);
    engine.feed(b"\x1b[?1000h");

    let modes = engine.modes();
    assert!(!modes.mouse_x10);
    assert!(!modes.mouse_highlight);
    assert!(modes.mouse_click);
    assert!(engine.wheel_input(wheel()).is_ok());
    assert!(engine.pointer_input(pointer()).is_ok());
}
