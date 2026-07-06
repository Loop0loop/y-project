# MVP Lifecycle

## Decision

앱 lifecycle은 도메인 lifecycle과 renderer lifecycle을 분리한다.

도메인 lifecycle은 게임 진행을 관리한다. renderer lifecycle은 terminal capability, resize, layer cache,
placement id, frame pacing을 관리한다.

## Design

### App Lifecycle

```text
Process Start
  -> Config Load
  -> Terminal Probe
  -> Renderer Init
  -> Asset Warmup
  -> GameSession Init
  -> Main Loop
  -> Shutdown
```

### Boot

1. config를 읽는다.
2. terminal raw mode / alternate screen을 켠다.
3. terminal capability를 probe한다.
4. pixel metrics를 측정한다.
5. backend를 선택한다.

Backend 선택:

```text
Kitty graphics available -> KittyBackend
Sixel available          -> SixelBackend
else                     -> TextFallbackBackend
```

### Renderer Init

1. Ratatui virtual layout root를 만든다.
2. 현재 terminal metric으로 pixel scale을 계산한다.
3. static SVG asset을 parse/render/cache한다.
4. dynamic layer buffer를 할당한다.
5. Kitty placement id namespace를 초기화한다.

`shm`은 여기서 기본으로 켜지 않는다. direct/tempfile 경로가 먼저다.

### GameSession Init

초기 상태:

```text
phase = Training
week = 1
stats = baseline
court = empty
relationship = neutral
transcript = empty
```

### Main Loop

```text
read input / resize / tick / llm chunk
  -> update domain or renderer state
  -> produce ViewModel
  -> mark dirty layers
  -> redraw dirty layers
  -> compose frame
  -> present via backend
```

이벤트 종류:

```rust
enum AppEvent {
    Input(InputEvent),
    Resize { cols: u16, rows: u16 },
    Tick,
    LlmChunk(String),
    LlmDone,
    LlmError(String),
    LlmCancelled,
    RendererError(String),
}
```

### Phase Transition

```text
Training
  -> SelectTrainingAction
  -> stats updated
  -> Court

Court
  -> StartCourt
  -> simulation finished immediately
  -> court result stored
  -> DatingContext built
  -> Dating

Dating
  -> LLM stream finished, failed, cancelled, or user exits
  -> Result

Result
  -> Exit or restart
```

전환은 domain command로만 발생한다.

```rust
enum DomainCommand {
    SelectTrainingAction(TrainingActionId),
    StartCourt,
    SubmitDatingInput(String),
    FinishDating(DatingEndReason),
    EndSession,
}
```

`SelectTrainingAction`은 `Training` 상태에서만 허용하고, 성공하면 `Court`로 전이한다.
`StartCourt`는 `Court` 상태에서만 허용하고, MVP에서는 전체 결과와 로그를 즉시 생성한 뒤 `Dating`으로 전이한다.
`SubmitDatingInput`은 `Dating` 상태에서만 허용한다. `FinishDating`은 성공, 실패, 취소, timeout을 모두 처리한다.

### Resize Lifecycle

resize는 게임 상태를 바꾸지 않는다.

```text
Resize event
  -> refresh terminal metrics
  -> recompute Ratatui layout
  -> recompute pixel bounding boxes
  -> recreate size-dependent static layers
  -> mark all layers dirty
  -> present new frame
```

`213 x 60`은 resize 판단에 충분하지 않다. 같은 cell grid여도 pixel metric이 바뀌면 resize로 처리한다.

### Shutdown

1. LLM worker를 중단한다.
2. 3D worker가 있으면 중단한다.
3. Kitty image placements를 삭제한다.
4. alternate screen/raw mode를 복구한다.
5. terminal cursor를 복구한다.

## Delivery

Lifecycle 완료 기준:

- Phase 1: boot/probe 실패 시 terminal을 정상 복구한다.
- Phase 1: resize가 domain state를 변경하지 않는다는 assert를 둔다.
- Phase 2: phase transition이 domain reducer 한 곳에서만 일어난다.
- Phase 2: LLM 실패/취소/timeout이 `Result`로 종료된다.
- Phase 4: 종료 시 Kitty placement와 terminal mode가 정리된다는 로그/검증을 둔다.
- 전체: renderer error가 domain을 오염시키지 않는다.

이번 범위에서 제외:

- crash recovery
- save file migration
- background update service
- multi-session management
