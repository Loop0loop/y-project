# Phase 2: 게임 루프와 LLM 스트리밍

## Decision

그래픽 파이프라인이 안정화된 뒤 MVP 루프를 연결한다.

```text
1 training turn -> 1 automatic court battle -> 1 LLM dialogue session
```

LLM streaming은 token마다 전체 SVG parse/raster를 유발하면 안 된다.

## Design

dirty rendering을 사용한다.

```text
GameState update
  -> 영향받은 UI layer만 dirty 표시
  -> bar, focus glow, log, dialogue text만 다시 그림
  -> static background와 UI chrome layer는 재사용
```

LLM chunk는 렌더링 전에 coalesce한다. 첫 목표는 token마다 렌더링이 아니라 30-80ms 단위의 시각 업데이트다.

Ratatui는 계속 아래 역할을 맡는다.

- focus index
- button hit box
- panel layout
- keyboard navigation
- scroll state

보이는 텍스트는 bundled font를 기준으로 결정론적으로 렌더링해야 한다. 정적 label에는 SVG text를 허용한다.
하지만 live dialogue는 SVG text fidelity가 반드시 필요한 경우가 아니라면 앱의 text layer에서 직접 그린다.

## Delivery

Phase 2 완료 기준:

- 키보드 focus 변경이 선택된 action의 시각 상태를 바꾼다.
- training 결과가 stat에 반영되고 필요한 bar만 업데이트된다.
- court log가 full-screen flicker 없이 append된다.
- LLM chunk가 고정 dialogue viewport 안에서 streaming된다.
- 긴 한국어/영어 텍스트가 layout을 깨지 않고 wrap된다.

이번 Phase에서 제외:

- 실시간 AI 이미지 생성

AI 이미지 생성은 main render loop가 아니라 별도의 asset cache pipeline에서 처리한다.

## 현재 검증 상태

2026-07-06 기준 도메인 루프 통과:

- `--domain-demo`: `Training -> Dating -> Result` 진행 확인
- `--mvp-loop`: 키보드 focus, training 선택, court log replay, fake LLM stream, result 종료 연결
- `--mvp-svg-loop`: SPA `ViewModel`을 full-screen SVG UI로 렌더링하고 Kitty backend에 표시함
- `GameSession`: renderer 없이 MVP 상태를 표현함
- `DomainCommand`: phase 전이를 reducer 한 곳에서 처리함
- training action: `CompleteTrainingAction` 하나로 stats 변경, court simulation, `DatingContext` 생성을 완료함
- court replay: deterministic result와 3줄 log를 화면에서 재생함
- dating finish: completed/failed/cancelled/timeout 모두 `Result`로 종료 가능함
- wrap test: 한국어/영어 문자가 누락되지 않고 width 안에서 줄바꿈됨
- SPA 구조: `spa.rs`가 `SpaApp`, `Screen`, `AppViewModel`을 소유하고 `app_loop.rs`는 terminal IO만 담당함
- SVG 구조: Rust 코드 내 inline SVG를 제거하고 `assets/svg/panel.svg` 템플릿을 사용함
- SVG UI: 좌측 phase rail, 중앙 메인 패널, 우측 side panel, progress bar, footer를 한 SVG에서 반응형 좌표로 구성함
- 최적화: `SvgPresenter`가 `RenderKey`를 비교해 ViewModel 또는 terminal metric이 바뀔 때만 SVG를 다시 래스터화함

남은 Phase 2 후속 작업:

- 실제 LLM API worker 연결
- 긴 대화 로그 scroll buffer
- SVG 텍스트 렌더링의 CJK font 품질 검증
