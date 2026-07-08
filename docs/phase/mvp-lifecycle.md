# MVP Lifecycle

이 문서는 현재 코드 기준의 lifecycle 규칙이다. 목표는 터미널 상태, Kitty/Ghostty 이미지 placement, 앱 화면 상태, 도메인 phase, 외부 프로세스가 각각 한 곳에서 소유되고 한 곳에서 정리되게 만드는 것이다.

## 참고 자료

- Rust `Drop`: https://doc.rust-lang.org/std/ops/trait.Drop.html
  - 값이 scope를 벗어나면 destructor가 실행된다.
  - 직접 resource를 소유하는 타입은 `Drop`에서 정리하는 것이 맞다.
  - `drop()` 안에서는 panic을 피해야 한다. panic 중 double panic이 나면 abort될 수 있다.
- crossterm raw mode: https://docs.rs/crossterm/latest/crossterm/terminal/index.html#raw-mode
  - raw mode에서는 입력이 line-buffered 되지 않고 byte 단위로 들어온다.
  - Ctrl-C 같은 특수 키도 terminal driver가 처리하지 않는다. 앱이 직접 처리해야 한다.
  - 그래서 `q`, `Esc`, `Ctrl-C`를 모두 명시적으로 cleanup exit로 연결한다.
- crossterm alternate screen: https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html
  - alternate screen에 들어가면 `LeaveAlternateScreen`으로 main screen에 돌아와야 한다.
  - 이 프로젝트는 `TerminalSession`이 enter/leave pair를 소유한다.
- Helix terminal lifecycle reference: https://raw.githubusercontent.com/helix-editor/helix/master/helix-term/src/application.rs
  - Helix는 SIGTERM/SIGINT/SIGTSTP에서 terminal을 restore하고, SIGCONT에서 terminal을 다시 claim/redraw한다.
  - 여기서는 signal subsystem까지 들이지 않고, 현재 범위에 맞게 raw-mode key handling과 panic hook만 적용했다.

## Runtime Shape

```text
main.rs
  -> terminal/video RGB splash, optional
  -> app_loop
      -> TerminalSession enter
      -> SpaApp init
      -> SvgPresenter init, optional
      -> event/tick/render loop
      -> TerminalSession Drop restore
```

`main.rs`는 CLI dispatch만 한다. 실제 lifecycle 소유권은 아래 타입들이 가진다.

- `TerminalSession`: terminal mode와 image placement 소유
- `SpaApp`: app screen, transition, input, progress, domain session 소유
- `GameSession`: domain phase와 game rule transition 소유
- `FfmpegVideo` / `AudioPlayback`: child process 소유
- `SplashOverlay` / `SvgPresenter`: render/image identity 제공, 삭제는 `TerminalSession`에 위임

## TerminalSession

파일: `src/terminal/session.rs`

`TerminalSession`은 terminal resource의 단일 owner다.

소유하는 상태:

- `stdout`
- raw mode 여부
- alternate screen 여부
- cursor hidden 여부
- line wrap disabled 여부
- registered `KittyImage` placement 목록

enter 순서:

```text
install panic restore hook
enable raw mode
enter alternate screen
hide cursor
optional clear
optional disable line wrap
```

restore 순서:

```text
delete registered Kitty images, reverse order
reset style and enable line wrap
show cursor
leave alternate screen
disable raw mode
flush stdout
```

`Drop`은 `restore()`만 호출한다. cleanup 중 에러는 무시한다. 종료 경로에서 cleanup 실패를 다시 panic으로 만들면 terminal이 더 망가질 수 있기 때문이다.

panic hook은 panic 메시지가 alternate screen 뒤에 묻히지 않도록 최소 escape 복구를 먼저 수행한다. 이 hook은 `Once`로 한 번만 설치한다.

## Image Placement Lifecycle

파일:

- `src/app/app_loop.rs`
- `src/app/svg_presenter.rs`
- `src/terminal/video.rs`
- `src/terminal/video/overlay.rs`
- `src/render/svg.rs`
- `src/terminal/kitty.rs`

규칙:

```text
image id 생성
  -> TerminalSession.register_image(image)
  -> present_rgba / present_rgba_with_z
  -> TerminalSession Drop에서 delete_image
```

앱 루프, RGB splash video overlay, SVG demo, Kitty demo 모두 이 규칙을 따른다.

금지:

- 정상 종료 경로마다 `delete_image()`를 직접 흩뿌리기
- render object가 image 삭제까지 소유하기
- resize 중 domain state를 변경하기

`overlay_path: None`인 video config는 `SplashOverlay`를 만들지 않는다. 따라서 ASCII-only splash는 image placement를 등록하지 않는다.

## Raw Mode Input Lifecycle

raw mode에서는 Ctrl-C를 terminal driver가 처리하지 않는다. 그래서 직접 처리한다.

현재 exit key:

- app loop: `q`, `Esc`, `Ctrl-C`
- video loop: `q`, `Esc`, `Ctrl-C`
- demo wait: `q`, `Esc`, `Ctrl-C`

start key:

- splash/video start는 `Enter`만 허용한다.
- Space는 start가 아니다. 이 정책은 사용자가 요구한 "무조건 Enter" 조건이다.

## App Lifecycle

파일:

- `src/app/app_loop.rs`
- `src/app/spa.rs`
- `src/app/spa_tests.rs`

`SpaApp`이 app lifecycle의 단일 owner다. 외부 module은 screen, session, transition, input, progress를 직접 바꾸지 못한다.

private state:

- `session`
- `screen`
- `focused_action`
- `shown_court_logs`
- `input`
- `splash_progress`
- `ui_opacity`
- `transition_to`
- `transition_start`
- `transition_phase`
- `loading_progress`
- `loading_start`
- tip text

외부는 read-only getter와 `view_model()`만 사용한다.

초기 생성:

```text
SpaApp::new_with_screen(Screen::Splash | Screen::Loading | Screen::Training)
```

`CourtReplay`, `Dating`, `Result`로 새 session을 시작하는 것은 금지한다. 새 `GameSession`의 domain phase는 항상 `Training`이기 때문이다.

## Screen / Domain Phase Invariant

현재 합법 조합:

```text
Splash      + GamePhase::Training
Loading     + GamePhase::Training
Training    + GamePhase::Training
CourtReplay + GamePhase::Dating
Dating      + GamePhase::Dating
Result      + GamePhase::Result
```

`CourtReplay`는 화면상 court log replay지만 domain simulation은 이미 끝난 뒤다. 그래서 domain phase는 `Dating`이다.

검증:

- `SpaApp::lifecycle_is_valid()`
- `screen_and_domain_phase_move_together` test
- `rejects_non_initial_start_screens` test

## Domain Lifecycle

파일:

- `src/domain/session.rs`
- `src/domain/lifecycle.rs`
- `src/domain/phase.rs`

도메인 전환은 `GameSession::apply(DomainCommand)` 한 곳에서만 발생한다.

```text
Training
  -> CompleteTrainingAction
  -> Dating
  -> FinishDating
  -> Result
```

`GameSession.phase`는 private이다. 외부는 `phase()`로 읽기만 한다.

`Court`는 더 이상 app-visible domain phase가 아니다. court simulation은 `CompleteTrainingAction` 내부에서 즉시 끝나고, app은 생성된 로그를 `CourtReplay` 화면에서 재생한다.

잘못된 phase의 command는 `DomainError::InvalidPhase`로 반환한다. app key handling과 domain demo는 이 에러를 panic하지 않고 `Result`로 위로 올린다.

## Video / Audio Process Lifecycle

파일:

- `src/terminal/video/process.rs`
- `src/terminal/video.rs`

`FfmpegVideo`는 ffmpeg child와 stdout pipe를 소유한다. `Drop`에서 `kill()` 후 `wait()` 한다.

stdout pipe 획득에 실패하면 아직 wrapper가 만들어지기 전이므로, 즉시 child를 kill/wait하고 에러를 반환한다.

`AudioPlayback`도 child를 소유하고 `Drop`에서 kill/wait 한다. resize 후 audio child가 이미 끝났으면 `restart_if_finished()`로 다시 시작한다.

## Resize Lifecycle

resize는 render state만 바꾼다.

```text
resize event
  -> debounce
  -> viewport recompute
  -> frame/output buffer resize
  -> next frame clear
  -> overlay layout recompute
```

금지:

- resize에서 `GameSession` 변경
- resize에서 `SpaApp.screen` 변경
- resize에서 domain command 실행

## Current Verification

현재 lifecycle 검증 명령:

```sh
cargo fmt --check
cargo test
git diff --check
```

현재 테스트는 38개다. 주요 lifecycle 테스트:

- `domain::session_tests::rejects_command_in_wrong_phase`
- `domain::session_tests::full_mvp_loop_is_deterministic`
- `app::spa_tests::screen_and_domain_phase_move_together`
- `app::spa_tests::rejects_non_initial_start_screens`
- `app::app_loop::tests::resize_event_does_not_mutate_app_lifecycle`
- `app::app_loop::tests::modified_enter_is_not_app_start`
- `app::app_loop::tests::ctrl_c_exits_from_raw_mode_loop`
- `terminal::video::tests::frame_deadline_tracks_rendered_frame_index`
- `terminal::video::resize::tests::resize_preserves_solid_color`

## Deliberate Non-Goals

현재 범위에서 하지 않은 것:

- full signal handling subsystem
- async task supervisor
- save/load lifecycle
- CI matrix
- full terminal integration tests
- renderer performance refactor

필요해지는 기준:

- SIGTSTP/SIGCONT까지 지원해야 하면 Helix처럼 terminal claim/restore/redraw path를 추가한다.
- LLM/background worker가 생기면 `close()` 단계에서 "에러가 있어도 모든 cleanup을 시도"하는 supervisor를 추가한다.
- Ghostty/Kitty 실제 터미널 회귀가 잦아지면 PTY 기반 integration test를 둔다.
