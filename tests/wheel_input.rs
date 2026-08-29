use soksak_kit_sidecar_terminal::mirror::{
    EngineWheelInput, EngineWheelRoute, SelectionModifiers,
};
use soksak_sidecar_terminal_alacritty::engine::Engine;

fn wheel(vertical: i32, route: EngineWheelRoute) -> EngineWheelInput {
    EngineWheelInput {
        row: 2,
        col: 1,
        horizontal: 0,
        vertical,
        modifiers: SelectionModifiers::default(),
        route,
    }
}

#[test]
fn sgr_mouse_wheel_encodes_direction_position_and_repetition() {
    let mut engine = Engine::new(120, 40);
    engine.feed(b"\x1b[?1000h\x1b[?1006h");
    assert_eq!(
        engine.wheel_input(wheel(-2, EngineWheelRoute::MouseReport)).unwrap(),
        b"\x1b[<64;2;3M\x1b[<64;2;3M"
    );
    assert_eq!(
        engine.wheel_input(wheel(1, EngineWheelRoute::MouseReport)).unwrap(),
        b"\x1b[<65;2;3M"
    );
}

#[test]
fn alternate_scroll_uses_application_cursor_keys_on_both_axes() {
    let mut engine = Engine::new(80, 24);
    engine.feed(b"\x1b[?1049h\x1b[?1007h");
    let mut input = wheel(-1, EngineWheelRoute::AlternateScroll);
    input.horizontal = 1;
    assert_eq!(engine.wheel_input(input).unwrap(), b"\x1bOA\x1bOC");
}

#[test]
fn legacy_and_utf8_mouse_encodings_keep_modifiers_and_extended_coordinates() {
    let mut engine = Engine::new(240, 120);
    engine.feed(b"\x1b[?1000h");
    let mut legacy = wheel(-1, EngineWheelRoute::MouseReport);
    legacy.modifiers = SelectionModifiers { shift: true, alt: true, control: true, meta: false };
    assert_eq!(engine.wheel_input(legacy).unwrap(), [0x1b, b'[', b'M', 124, 34, 35]);

    engine.feed(b"\x1b[?1005h");
    let mut extended = wheel(-1, EngineWheelRoute::MouseReport);
    extended.col = 100;
    assert_eq!(engine.wheel_input(extended).unwrap(), [0x1b, b'[', b'M', 96, 0xc2, 0x85, 35]);
}
