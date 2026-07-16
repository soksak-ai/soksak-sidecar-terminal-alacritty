//! soksak-sidecar-terminal-alacritty — 라이브러리 면.
//!
//! 도메인 로직(복원 미러·직렬화기)과 엔진 좌석을 모듈로 가른다:
//!   [`engine`]  alacritty_terminal 을 만지는 유일한 모듈(엔진-중립 뷰만 노출).
//!   [`mirror`]  엔진-중립 복원 로직 — [`Mirror`]·[`Screen`]·ANSI 직렬화기.
//!
//! 바이너리(서비스 소켓·데몬 피어링·체크포인트 정책)는 이 라이브러리를 링크한다.
//! 복원 픽스처 7종(tests/restore_fixtures.rs)이 엔진-중립 합격 판정이다.

pub mod checkpoint;
pub mod daemon;
pub mod engine;
pub mod mirror;
pub mod proto;
pub mod service;

pub use mirror::Mirror;
