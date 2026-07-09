# Project-Y Task Log

## 현재 목표

Project-Y는 터미널에서 동작하는 Rust 기반 게임이다. 현재 초점은 splash 진입부와 이후 SPA식 게임 UI 전환이다.

터미널 splash는 `/assets/video/spash-pc.mov`를 RGB/ASCII 셀로 재생하고, `/assets/svg/splash.svg`를 영상 위에 합성한다.

## 완료

### Loading Screen & Screen Flow
- **로딩 화면 구현**: 600x900 비율의 포트레이트 로딩 오버레이 UI (`assets/svg/loading.svg`) 및 리소스 로드 상태 구현.
- **팁 카드 UI**: 팁 헤더(`{{SUBTITLE}}`)와 팁 설명(`{{BODY}}`)을 포함한 다크 모드 카드 제작 및 Fontainian 팁 무작위 선택 기능 연동.
- **프로그레스 바**: 진행률(`{{PROGRESS}}`)과 Glowing Cyan 바 너비(`{{PROGRESS_BAR_WIDTH}}`)를 520px 트랙 내에서 계산 및 연동.
- **전환 애니메이션**: `AnimatePresence` 식의 opacity 페이드 효과 구현.
  - Splash 퇴장 (`opacity: 1.0` -> `0.0`, `0.8`초)
  - Loading 등장 (`opacity: 0.0` -> `1.0`, `0.5`초)
  - Loading 완료 후 Training 진입
- **SVG 템플릿 복구**: `src/render/svg.rs`에서 `include_str!` 기반의 외부 SVG 파일 로딩 및 토큰 연동 구조 복구 완료.

### Splash Video

- `--rgb-splash-demo` 추가
  - `assets/video/spash-pc.mov` 재생
  - RGB half-block terminal rendering
  - 오디오 재생 지원
  - `ffplay` 우선, 실패 시 macOS `afplay` fallback
- `--ascii-splash-demo` 유지
  - 저부하 ASCII splash 확인용
- resize 대응
  - 터미널 크기 변경 시 현재 셀 크기에 맞춰 영상 재렌더
  - 이전 중앙 축소/고정 배율 문제 수정
  - resize 중 전체 화면을 먼저 clear/flush해서 깜빡이던 문제 수정
- 입력 처리
  - `Enter`: 다음 UI로 이동
  - `q`, `Esc`, `Ctrl-C`: 종료
  - `m` 또는 `M`: 오디오 mute

### Splash Overlay

- `assets/svg/splash.svg`를 영상 위에 합성
- SVG overlay는 터미널 프레임 크기 변경 시 다시 래스터화
- `resvg`에 `text`, `system-fonts` feature 활성화
  - 원인: text feature가 꺼져 있어서 SVG의 `<text>`가 렌더되지 않고 다이아/라인만 보였음
- `src/render/svg.rs`와 splash overlay 렌더링 모두 시스템 폰트 로드 적용

### Audio Sync / Playback

- 오디오 시작 직후를 playback 기준 시간으로 잡음
- 렌더가 늦어질 경우 중간 비디오 프레임을 버려 오디오 sync를 따라가도록 변경
- 영상 종료 시 terminal state 복구

### Performance Work

- 시도 후 롤백한 것
  - RGB 색상 양자화: 품질/체감 대비 이득 작음
  - `max_render_cells`로 RGB splash 해상도 제한: 중앙 배율 버그 재발
- 유지한 것
  - splash 24fps
  - resize 알고리즘 `Lanczos3 -> Bilinear`
  - 오디오 기준 프레임 드랍
  - resize clear를 synchronized frame 안에서 처리

### Domain / SPA

- 도메인 모듈화 진행
  - `src/domain/phase.rs`
  - `src/domain/lifecycle.rs`
  - `src/domain/session.rs`
  - domain tests 분리
- SPA 루프 분리
  - `src/app/app_loop.rs`
  - `src/app/svg_presenter.rs`
  - `src/app/spa_tests.rs`
- LOC 300 이하 유지

## 현재 CLI

```bash
cargo run -- --rgb-splash-demo
cargo run -- --ascii-splash-demo
cargo run -- --mvp-svg-loop
cargo run -- --mvp-loop
cargo run -- --probe
cargo run -- --dump-layout
cargo run -- --watch-metrics
cargo run -- --domain-demo
```

## 검증 상태

마지막 확인:

```bash
cargo test
```

결과:

```text
37 passed
```

LOC 상태:

```text
모든 src/*.rs 파일 300 LOC 이하
```

## 현재 판단

- RGB terminal video는 CPU를 많이 쓰는 구조적 한계가 있다.
- 병목은 대략 다음 순서로 의심된다.
  - terminal truecolor escape 출력량
  - terminal app 자체 렌더 비용
  - 매 프레임 resize
  - SVG overlay 합성
  - ffmpeg decode
- Project-Y의 터미널 게임 정체성에는 ASCII/ANSI splash가 더 맞을 가능성이 높다.
- RGB splash는 고품질 데모/옵션으로 두고, 기본값은 ASCII 계열로 가는 방향을 검토한다.

## 다음 작업

1. Splash 기본 모드 결정
   - 후보 A: 기본 ASCII/ANSI splash, RGB는 옵션
   - 후보 B: 짧은 RGB splash 후 빠르게 SPA 진입
2. CPU 계측 추가
   - frame read
   - resize
   - overlay blend
   - terminal render string build
   - stdout write/flush
3. ASCII splash 품질 개선
   - Gascii 스타일 이식 계속
   - RGB 대신 ANSI 256-color 또는 limited-color ASCII 검토
4. Splash lifecycle 정리
   - `--rgb-splash-demo`만이 아니라 기본 실행 경로 앞단에 splash 연결
   - splash 종료 후 `app::run_mvp_svg_loop()` 진입
5. Scene SPA 구조 확정
   - `Splash`
   - `Training`
   - `CourtReplay`
   - `Dating`
   - `Result`
6. SVG scene token contract 정리
   - 각 SVG가 받는 token 목록 문서화
   - Rust 쪽 scene별 token map 구성

## 보류

- RGB dirty-region renderer
- 프레임 문자열 전체 pre-cache
- Kitty/iTerm image protocol 기반 비디오
- 실제 LLM API
- offscreen 3D rendering
- 복잡한 adaptive quality 시스템
