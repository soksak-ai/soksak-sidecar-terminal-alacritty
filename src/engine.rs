//! 엔진 격리 좌석 — alacritty_terminal 을 만지는 유일한 모듈. 미러·직렬화기는 여기가
//! 내놓는 엔진-중립 뷰(스칼라 상태 + [`GridCell`] 행 읽기)만 쓴다. 엔진 교체
//! (예: soksak-sidecar-terminal-wezterm)는 이 파일만 갈아끼우면 되고, 나머지 도메인
//! 로직(복원 직렬화·체크포인트 정책)은 불변이다 — 그것이 엔진-중립 계약의 실체다.
//!
//! 합격시험은 계약이 소유하고, 정답은 선언된 골든이다 — 이 엔진이 하는 짓이 정답인 것이 아니다.

use std::sync::{Arc, Mutex};

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::test::TermSize;
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::vte::ansi::{Color, NamedColor, Processor};

/// 엔진이 유지하는 스크롤백 행 수. 바이트 충실 복원의 바닥 — 전체 의미 이력은
/// command_blocks(app.data)가 소유하고, 이 수치는 화면 재현용 창이다.
pub const MIRROR_SCROLLBACK_LINES: usize = 1000;

// ── 엔진-중립 스냅샷 타입 ─────────────────────────────────────────────────────

/// 색 스냅샷 — alacritty 타입을 밖으로 새지 않게 자체 표현으로 고정한다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorSnap {
    Default,
    Named(u8),
    Indexed(u8),
    Rgb(u8, u8, u8),
}

/// 복원 대상 private mode 집합의 스냅샷(rehydrate 가 재현해야 하는 전부).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModeSnap {
    pub bracketed_paste: bool,
    pub app_cursor: bool,
    pub app_keypad: bool,
    pub mouse_click: bool,
    pub mouse_drag: bool,
    pub mouse_motion: bool,
    pub sgr_mouse: bool,
    pub utf8_mouse: bool,
    pub focus_in_out: bool,
    pub alternate_scroll: bool,
    pub show_cursor: bool,
    pub line_wrap: bool,
    pub insert: bool,
}

/// 직렬화기가 읽는 엔진-중립 셀 — 직렬화에 필요한 것을 다 담는다(spacer·wrapline·zerowidth
/// 포함). 이 타입 하나가 직렬화기의 그리드 읽기 단일 창이다 — 엔진 세부(Flags·Color)는 이 파일 밖으로
/// 나가지 않는다.
pub struct GridCell {
    pub ch: char,
    pub fg: ColorSnap,
    pub bg: ColorSnap,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
    pub strikeout: bool,
    pub hidden: bool,
    /// wide 문자 본체(2칸 점유의 첫 칸).
    pub wide: bool,
    /// wide 문자 스페이서(본체 뒤 칸 또는 줄끝 선두 스페이서) — 직렬화기가 건너뛴다.
    pub spacer: bool,
    /// WRAPLINE — 마지막 칸에서만 의미: 이 행이 자연 개행(wrap)으로 이어진다.
    pub wrapline: bool,
    /// 결합 문자(zero-width) 후속.
    pub zerowidth: Vec<char>,
}

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
        let config = Config { scrolling_history: MIRROR_SCROLLBACK_LINES, ..Config::default() };
        let term = Term::new(config, &TermSize::new(cols as usize, rows as usize), tap);
        Engine { term, parser: Processor::new(), replies, cols, rows }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut self.term, bytes);
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        self.term.resize(TermSize::new(cols as usize, rows as usize));
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
        self.replies.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn alt_active(&self) -> bool {
        self.term.mode().contains(TermMode::ALT_SCREEN)
    }

    /// 커서 위치(화면 기준 0-base row, col).
    pub fn cursor(&self) -> (usize, usize) {
        let p = self.term.grid().cursor.point;
        (p.line.0.max(0) as usize, p.column.0)
    }

    /// 현재 스크롤백(화면 위로 밀려난) 행 수.
    pub fn history_size(&self) -> usize {
        self.term.grid().history_size()
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
