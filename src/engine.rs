//! 엔진 격리 좌석 — alacritty_terminal 을 만지는 유일한 모듈. 미러·직렬화기는 여기가
//! 내놓는 엔진-중립 뷰(스칼라 상태 + [`GridCell`] 행 읽기)만 쓴다. 엔진 교체
//! (예: soksak-sidecar-terminal-wezterm)는 이 파일만 갈아끼우면 되고, 나머지 도메인
//! 로직(복원 직렬화·체크포인트 정책)은 불변이다 — 그것이 엔진-중립 계약의 실체다.
//!
//! 합격시험은 계약이 소유하고, 정답은 선언된 reference state이다 — 이 엔진이 하는 짓이 정답인 것이 아니다.

use std::sync::{Arc, Mutex};

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point, Side};
use alacritty_terminal::selection::{Selection, SelectionType};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::test::TermSize;
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::vte::ansi::{
    Color, CursorShape as AlacrittyCursorShape, NamedColor, Processor,
};
use soksak_kit_sidecar_terminal::mirror::TerminalEngine;
pub use soksak_kit_sidecar_terminal::mirror::{
    TerminalCell as GridCell, TerminalColor as ColorSnap, TerminalCursorAnimation,
    EngineSelectionPoint, EngineWheelInput, SelectionKind, SelectionModifiers, TerminalCursorShape,
    TerminalCursorStyle, TerminalModes as ModeSnap, TerminalRgb, TerminalThemeOverrides,
};

/// 엔진이 유지하는 스크롤백 행 수. 바이트 충실 복원의 바닥 — 전체 의미 이력은
/// command_blocks(app.data)가 소유하고, 이 수치는 화면 재현용 창이다.
pub const MIRROR_SCROLLBACK_LINES: usize = 1000;
pub const CURSOR_BLINK_INTERVAL_MS: u32 = 750;

// ── 이벤트 프록시 — 터미널이 PTY 에 쓰려는 응답을 포획한다 ─────────────────────

#[derive(Clone, Default)]
struct ReplyTap(Arc<Mutex<Vec<String>>>);

impl EventListener for ReplyTap {
    fn send_event(&self, event: Event) {
        if let Event::PtyWrite(text) = event {
            self.0.lock().unwrap_or_else(|e| e.into_inner()).push(text);
        }
    }
}

// ── Engine — 유일한 alacritty 좌석 ───────────────────────────────────────────

/// 바이트를 실제 렌더해 화면 상태를 유지하는 헤드리스 VT 엔진. 미러(복원 로직)와
/// 판정자(픽스처 오라클)가 공유하는 좌석이며, "이 바이트를 먹은 터미널이 PTY 에
/// 무엇을 되쓰려 했는가"(`captured_replies`)의 프로브이기도 하다.
pub struct Engine {
    term: Term<ReplyTap>,
    parser: Processor,
    replies: Arc<Mutex<Vec<String>>>,
    cols: u16,
    rows: u16,
}

impl Engine {
    pub fn new(cols: u16, rows: u16) -> Self {
        let tap = ReplyTap::default();
        let replies = tap.0.clone();
        let config = Config {
            scrolling_history: MIRROR_SCROLLBACK_LINES,
            ..Config::default()
        };
        let term = Term::new(config, &TermSize::new(cols as usize, rows as usize), tap);
        Engine {
            term,
            parser: Processor::new(),
            replies,
            cols,
            rows,
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut self.term, bytes);
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        self.term
            .resize(TermSize::new(cols as usize, rows as usize));
    }

    pub fn wheel_input(&mut self, _input: EngineWheelInput) -> Result<Vec<u8>, String> {
        Err("WHEEL_INPUT_UNIMPLEMENTED".into())
    }

    pub fn cols(&self) -> u16 {
        self.cols
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }

    /// 이 엔진이 PTY 에 되쓰려 한 응답들(DA1/DSR/OSC 질의 답). 재생 가드의 프로브 —
    /// 복원 시퀀스를 먹인 엔진에서 이게 비어 있지 않으면 이중응답이다.
    pub fn captured_replies(&self) -> Vec<String> {
        self.replies
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn alt_active(&self) -> bool {
        self.term.mode().contains(TermMode::ALT_SCREEN)
    }

    /// 커서 위치(화면 기준 0-base row, col).
    pub fn cursor(&self) -> (usize, usize) {
        let p = self.term.grid().cursor.point;
        (p.line.0.max(0) as usize, p.column.0)
    }

    pub fn cursor_style(&self) -> TerminalCursorStyle {
        let style = self.term.cursor_style();
        let shape = match style.shape {
            AlacrittyCursorShape::Block
            | AlacrittyCursorShape::HollowBlock
            | AlacrittyCursorShape::Hidden => TerminalCursorShape::Block,
            AlacrittyCursorShape::Underline => TerminalCursorShape::Underline,
            AlacrittyCursorShape::Beam => TerminalCursorShape::Bar,
        };
        TerminalCursorStyle {
            shape,
            blinking: style.blinking,
        }
    }

    pub fn cursor_animation(&self) -> TerminalCursorAnimation {
        TerminalCursorAnimation {
            interval_ms: CURSOR_BLINK_INTERVAL_MS,
        }
    }

    pub fn theme_overrides(&self) -> TerminalThemeOverrides {
        let colors = self.term.colors();
        let rgb = |value: Option<alacritty_terminal::vte::ansi::Rgb>| {
            value.map(|value| TerminalRgb { r: value.r, g: value.g, b: value.b })
        };
        let mut overrides = TerminalThemeOverrides::default();
        overrides.foreground = rgb(colors[NamedColor::Foreground]);
        overrides.background = rgb(colors[NamedColor::Background]);
        overrides.cursor = rgb(colors[NamedColor::Cursor]);
        for (index, slot) in overrides.ansi.iter_mut().enumerate() {
            *slot = rgb(colors[index]);
        }
        overrides
    }

    /// 현재 스크롤백(화면 위로 밀려난) 행 수.
    pub fn history_size(&self) -> usize {
        self.term.grid().history_size()
    }

    fn selection_point(&self, point: EngineSelectionPoint) -> (Point, Side) {
        let top = -(self.term.grid().history_size() as i32);
        let bottom = self.rows.saturating_sub(1) as i32;
        let line = Line(point.line.clamp(top, bottom));
        let column = Column(usize::from(point.col.min(self.cols.saturating_sub(1))));
        let side = match point.side {
            soksak_kit_sidecar_terminal::mirror::CellSide::Left => Side::Left,
            soksak_kit_sidecar_terminal::mirror::CellSide::Right => Side::Right,
        };
        (Point::new(line, column), side)
    }

    pub fn selection_begin(
        &mut self,
        kind: SelectionKind,
        point: EngineSelectionPoint,
        _modifiers: SelectionModifiers,
    ) -> Result<(), String> {
        let (point, side) = self.selection_point(point);
        if kind == SelectionKind::Extend {
            let selection = self.term.selection.as_mut().ok_or("no selection to extend")?;
            selection.update(point, side);
            return Ok(());
        }
        let kind = match kind {
            SelectionKind::Simple => SelectionType::Simple,
            SelectionKind::Block => SelectionType::Block,
            SelectionKind::Semantic => SelectionType::Semantic,
            SelectionKind::Line => SelectionType::Lines,
            SelectionKind::Extend => unreachable!(),
        };
        self.term.selection = Some(Selection::new(kind, point, side));
        Ok(())
    }

    pub fn selection_update(
        &mut self,
        point: EngineSelectionPoint,
        _modifiers: SelectionModifiers,
    ) -> Result<(), String> {
        let (point, side) = self.selection_point(point);
        self.term.selection.as_mut().ok_or("no active selection")?.update(point, side);
        Ok(())
    }

    pub fn selection_clear(&mut self) {
        self.term.selection = None;
    }

    pub fn selection_text(&self) -> Option<String> {
        self.term.selection.as_ref()?;
        Some(self.term.selection_to_string().unwrap_or_default())
    }

    pub fn selection_range(&self, line: i32) -> Option<(u16, u16)> {
        let range = self.term.selection.as_ref()?.to_range(&self.term)?;
        let line = Line(line);
        if line < range.start.line || line > range.end.line {
            return None;
        }
        let last = self.cols.saturating_sub(1);
        let start = if range.is_block || line == range.start.line {
            u16::try_from(range.start.column.0).ok()?.min(last)
        } else {
            0
        };
        let end = if range.is_block || line == range.end.line {
            u16::try_from(range.end.column.0).ok()?.min(last)
        } else {
            last
        };
        Some((start.min(end), start.max(end)))
    }

    pub fn modes(&self) -> ModeSnap {
        let m = self.term.mode();
        ModeSnap {
            bracketed_paste: m.contains(TermMode::BRACKETED_PASTE),
            app_cursor: m.contains(TermMode::APP_CURSOR),
            app_keypad: m.contains(TermMode::APP_KEYPAD),
            mouse_click: m.contains(TermMode::MOUSE_REPORT_CLICK),
            mouse_drag: m.contains(TermMode::MOUSE_DRAG),
            mouse_motion: m.contains(TermMode::MOUSE_MOTION),
            sgr_mouse: m.contains(TermMode::SGR_MOUSE),
            utf8_mouse: m.contains(TermMode::UTF8_MOUSE),
            focus_in_out: m.contains(TermMode::FOCUS_IN_OUT),
            alternate_scroll: m.contains(TermMode::ALTERNATE_SCROLL),
            show_cursor: m.contains(TermMode::SHOW_CURSOR),
            line_wrap: m.contains(TermMode::LINE_WRAP),
            insert: m.contains(TermMode::INSERT),
        }
    }

    /// 한 행(line index; 음수 = 스크롤백)을 엔진-중립 셀 벡터로 읽는다. 길이는 항상
    /// `cols` — spacer 포함(직렬화기가 skip 판정을 소유한다). 직렬화기·판정자 공용의
    /// 유일한 그리드 창.
    pub fn line_cells(&self, line: i32) -> Vec<GridCell> {
        let grid = self.term.grid();
        // A line outside the grid answers a blank row. The caller's geometry
        // can outrun the mirror's (a pane painted before its resize lands);
        // indexing the grid there aborts the whole render thread.
        if line >= self.rows as i32 || line < -(grid.history_size() as i32) {
            return (0..self.cols as usize)
                .map(|_| GridCell {
                    ch: ' ',
                    fg: ColorSnap::Default,
                    bg: ColorSnap::Default,
                    bold: false,
                    dim: false,
                    italic: false,
                    underline: false,
                    inverse: false,
                    strikeout: false,
                    hidden: false,
                    wide: false,
                    spacer: false,
                    wrapline: false,
                    zerowidth: Vec::new(),
                    link: None,
                })
                .collect();
        }
        let row = &grid[Line(line)];
        (0..self.cols as usize)
            .map(|col| {
                let cell = &row[Column(col)];
                GridCell {
                    ch: cell.c,
                    fg: snap_color(&cell.fg),
                    bg: snap_color(&cell.bg),
                    bold: cell.flags.contains(Flags::BOLD),
                    dim: cell.flags.contains(Flags::DIM),
                    italic: cell.flags.contains(Flags::ITALIC),
                    underline: cell.flags.intersects(Flags::ALL_UNDERLINES),
                    inverse: cell.flags.contains(Flags::INVERSE),
                    strikeout: cell.flags.contains(Flags::STRIKEOUT),
                    hidden: cell.flags.contains(Flags::HIDDEN),
                    wide: cell.flags.contains(Flags::WIDE_CHAR),
                    spacer: cell
                        .flags
                        .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER),
                    wrapline: cell.flags.contains(Flags::WRAPLINE),
                    zerowidth: cell.zerowidth().map(|z| z.to_vec()).unwrap_or_default(),
                    // This engine does not track OSC 8; capabilities.hyperlinks stays false.
                    link: None,
                }
            })
            .collect()
    }
}

fn snap_color(color: &Color) -> ColorSnap {
    match color {
        Color::Named(NamedColor::Foreground) | Color::Named(NamedColor::Background) => {
            ColorSnap::Default
        }
        Color::Named(n) => ColorSnap::Named(*n as u8),
        Color::Indexed(i) => ColorSnap::Indexed(*i),
        Color::Spec(rgb) => ColorSnap::Rgb(rgb.r, rgb.g, rgb.b),
    }
}

impl TerminalEngine for Engine {
    fn new(cols: u16, rows: u16) -> Self {
        Engine::new(cols, rows)
    }
    fn initialize(&mut self) {
        self.feed(b"\x1b[?1007l");
    }
    fn feed(&mut self, bytes: &[u8]) {
        Engine::feed(self, bytes);
    }
    fn resize(&mut self, cols: u16, rows: u16) {
        Engine::resize(self, cols, rows);
    }
    fn cols(&self) -> u16 {
        Engine::cols(self)
    }
    fn rows(&self) -> u16 {
        Engine::rows(self)
    }
    fn cursor(&self) -> (usize, usize) {
        Engine::cursor(self)
    }
    fn cursor_style(&self) -> TerminalCursorStyle {
        Engine::cursor_style(self)
    }
    fn cursor_animation(&self) -> TerminalCursorAnimation {
        Engine::cursor_animation(self)
    }
    fn theme_overrides(&self) -> TerminalThemeOverrides {
        Engine::theme_overrides(self)
    }
    fn alt_active(&self) -> bool {
        Engine::alt_active(self)
    }
    fn history_size(&self) -> usize {
        Engine::history_size(self)
    }
    fn modes(&self) -> ModeSnap {
        Engine::modes(self)
    }
    fn line_cells(&self, line: i32) -> Vec<GridCell> {
        Engine::line_cells(self, line)
    }
    fn selection_begin(
        &mut self, kind: SelectionKind, point: EngineSelectionPoint,
        modifiers: SelectionModifiers,
    ) -> Result<(), String> {
        Engine::selection_begin(self, kind, point, modifiers)
    }
    fn selection_update(
        &mut self, point: EngineSelectionPoint, modifiers: SelectionModifiers,
    ) -> Result<(), String> {
        Engine::selection_update(self, point, modifiers)
    }
    fn selection_clear(&mut self) {
        Engine::selection_clear(self)
    }
    fn selection_text(&self) -> Option<String> {
        Engine::selection_text(self)
    }
    fn selection_range(&self, line: i32) -> Option<(u16, u16)> {
        Engine::selection_range(self, line)
    }
    fn wheel_input(&mut self, input: EngineWheelInput) -> Result<Vec<u8>, String> {
        Engine::wheel_input(self, input)
    }
    fn suppressed_replies(&self) -> u64 {
        self.captured_replies().len() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soksak_kit_sidecar_terminal::mirror::TerminalRgb;

    #[test]
    fn engine_exposes_osc_palette_and_dynamic_color_state() {
        let mut engine = Engine::new(4, 2);
        engine.feed(
            b"\x1b]4;1;#123456\x07\x1b]10;#abcdef\x07\x1b]11;#223344\x07\x1b]12;#654321\x07",
        );
        let colors = TerminalEngine::theme_overrides(&engine);
        assert_eq!(colors.ansi[1], Some(TerminalRgb { r: 0x12, g: 0x34, b: 0x56 }));
        assert_eq!(colors.foreground, Some(TerminalRgb { r: 0xab, g: 0xcd, b: 0xef }));
        assert_eq!(colors.background, Some(TerminalRgb { r: 0x22, g: 0x33, b: 0x44 }));
        assert_eq!(colors.cursor, Some(TerminalRgb { r: 0x65, g: 0x43, b: 0x21 }));

        engine.feed(b"\x1b]104;1\x07\x1b]110\x07\x1b]111\x07\x1b]112\x07");
        let reset = TerminalEngine::theme_overrides(&engine);
        assert_eq!(reset.ansi[1], None);
        assert_eq!((reset.foreground, reset.background, reset.cursor), (None, None, None));
    }
}
