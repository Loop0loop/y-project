# Project-Y 작업 기록

## 현재 방향

Project-Y는 터미널을 Kitty 그래픽 레이어 기반의 실시간 SVG 래스터 UI 캔버스로 쓰는 Rust TUI/그래픽 하이브리드 게임이다.

현재 역할 분리는 다음으로 고정했다.

- SVG 디자인: Gemini/외부 디자인 툴이 담당
- Rust 로직: 게임 상태, 씬 전환, SVG 토큰 주입, `resvg` 래스터화, Kitty 전송 담당
- Ratatui식 박스 UI는 지양하고, SVG는 외부 씬 파일로 취급한다

## 완료한 것

### Phase 1: 터미널/Kitty 그래픽 PoC

- `--probe`
  - 터미널 grid/pixel/cell 크기 감지
  - `ioctl(TIOCGWINSZ)` 기반
  - Kitty 환경 감지 확인됨
- `--dump-layout`
  - cell rect를 pixel bbox로 변환하는 계산 검증
- `--watch-metrics`
  - 터미널 리사이즈 감지 루프 구현
- `--kitty-demo`
  - 작은 RGBA 버퍼를 Kitty Graphics Protocol로 표시 확인
- `--svg-demo`
  - SVG를 `resvg`로 RGBA 래스터화 후 Kitty에 출력 확인

### MVP 도메인/SPA 루프

- `src/domain.rs`
  - `GameSession`
  - `GamePhase`
  - `AdvocateStats`
  - `TrainingAction`
  - 자동 재판 결과
  - 데이팅 입력/종료
- `src/spa.rs`
  - `SpaApp`
  - `Screen`
  - `AppViewModel`
  - 키 입력 기반 화면 전환
  - fake LLM 스트리밍
- `src/app_loop.rs`
  - `--mvp-loop`: 텍스트 fallback 루프
  - `--mvp-svg-loop`: SVG/Kitty 기반 루프
  - ViewModel/터미널 크기가 바뀔 때만 SVG 재래스터화

### SVG 처리

- `assets/svg/panel.svg`
  - 초기 MVP SVG 패널
  - 현재는 실험용이며 최종 디자인 소스가 아님
- `assets/svg/splash.svg`
  - 외부 디자인 SVG 샘플 씬
  - `{{WIDTH}}`, `{{HEIGHT}}`, `{{PROGRESS}}`, `{{PROGRESS_BAR_WIDTH}}` 토큰 주입
- `src/svg_panel.rs`
  - `render_svg_panel()`
  - `render_splash()`
  - 공통 `render_svg()` 래스터 함수
  - `--splash-demo` 실행 경로

### 문서

- `docs/phase/README.md`
- `docs/phase/phase-1-terminal-graphics-poc.md`
- `docs/phase/phase-2-game-loop-llm.md`
- `docs/phase/phase-3-offscreen-3d.md`
- `docs/phase/phase-4-optimization.md`
- `docs/phase/mvp-domain-architecture.md`
- `docs/phase/mvp-lifecycle.md`
- `docs/phase/references.md`
- `docs/phase/svg-scene-contract.md`

## 현재 CLI

```bash
cargo run -- --probe
cargo run -- --dump-layout
cargo run -- --kitty-demo
cargo run -- --svg-demo
cargo run -- --splash-demo
cargo run -- --watch-metrics
cargo run -- --domain-demo
cargo run -- --mvp-loop
cargo run -- --mvp-svg-loop
```

## 검증 상태

마지막 테스트 결과:

```bash
cargo test
```

결과:

```text
15 passed
```

## 중요한 판단

- 전체화면 권장.
  - 터미널마다 cell 크기와 pixel 해상도가 다르기 때문에 fullscreen에 가까울수록 SVG 스케일 품질과 레이아웃 안정성이 좋아진다.
- SVG 반응형은 Rust 좌표 재계산보다 `viewBox` 스케일링을 우선한다.
- SVG는 코드 안에 inline하지 않고 `assets/svg/*.svg` 파일로 둔다.
- Gemini가 넘기는 SVG는 토큰 계약만 맞추면 Rust에서 그대로 띄운다.
- `foreignObject`, 외부 URL 이미지, JS, 웹폰트 다운로드는 피한다.
- LLM API는 후순위. 지금은 fake stream으로 lifecycle 검증만 한다.
- 3D/GLB/three.rs/wgpu는 Phase 3 이후. 지금은 씬 전환과 상태 주입이 먼저다.

## 다음 작업

1. Splash lifecycle을 `--mvp-svg-loop` 앞단에 붙인다.
2. `SceneId`를 도입해서 `Splash`, `Training`, `Court`, `Dating`, `Result` SVG를 분리한다.
3. `AppViewModel`을 scene별 토큰 map으로 바꾼다.
4. Gemini가 만든 `training.svg`를 `assets/svg/training.svg`로 넣고 토큰 주입만 연결한다.
5. 이후 `court.svg`, `dating.svg`를 같은 방식으로 추가한다.

## 보류한 것

- SVG 디자인 직접 개선
- 하이브리드 ASCII cell overlay
- 실제 LLM API
- 오프스크린 3D 렌더링
- shm 최적화
- Delta 렌더링
