//! soksak-sidecar-terminal-alacritty — 라이브러리 면.
//!
//! 도메인 로직(복원 미러·직렬화기)과 엔진 좌석을 모듈로 가른다:
//!   [`engine`]  alacritty_terminal 을 만지는 유일한 모듈(엔진-중립 뷰만 노출).
//!   [`mirror`]  엔진-중립 복원 로직 — [`Mirror`]·[`Screen`]·ANSI 직렬화기.
//!
//! 바이너리(서비스 소켓·데몬 피어링·체크포인트 정책)는 이 라이브러리를 링크한다.
//! 복원 픽스처 7종(tests/restore_fixtures.rs)이 엔진-중립 합격 판정이다.

pub mod engine;
pub mod mirror;

pub use mirror::Mirror;

impl soksak_kit_sidecar_terminal::TerminalStateMirror for Mirror {
    fn feed(&mut self, bytes: &[u8]) {
        Mirror::feed(self, bytes);
    }
    fn resize(&mut self, cols: u16, rows: u16) {
        Mirror::resize(self, cols, rows);
    }
    fn rehydrate(&self) -> Vec<u8> {
        Mirror::rehydrate(self)
    }
    fn cold_paint(&self) -> Vec<u8> {
        Mirror::cold_paint(self)
    }
    fn frame_at(&self, offset: usize) -> soksak_kit_sidecar_terminal::mirror::TerminalFrame {
        Mirror::frame_at(self, offset)
    }
    fn history_size(&self) -> usize {
        Mirror::history_size(self)
    }
    fn modes(&self) -> soksak_kit_sidecar_terminal::mirror::TerminalModes {
        Mirror::modes(self)
    }
    fn capabilities(&self) -> soksak_kit_sidecar_terminal::mirror::MirrorCapabilities {
        Mirror::capabilities(self)
    }
    fn alt_active(&self) -> bool {
        Mirror::alt_active(self)
    }
    fn suppressed_replies(&self) -> u64 {
        Mirror::suppressed_replies(self)
    }
}
