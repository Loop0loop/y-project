# MVP 도메인 아키텍처

## Decision

MVP 도메인은 작게 유지한다.

```text
1 training turn -> 1 automatic court battle -> 1 LLM dialogue session
```

렌더링, 터미널 capability, SVG, Kitty, Bevy는 도메인이 아니다. 도메인은 게임 규칙과 상태 전이만 가진다.
UI와 renderer는 도메인 상태를 읽어서 표현할 뿐이다.

## Design

### Core Domain

```text
GameSession
  - session_id
  - phase
  - week
  - defendant
  - stats
  - evidence
  - court
  - relationship
  - transcript
```

### Phase

```rust
enum GamePhase {
    Training,
    Dating,
    Result,
}
```

`GamePhase`는 화면 이름이 아니라 게임 진행 상태다. Boot, Splash, 화면 전환 애니메이션은 renderer/app
lifecycle 상태로 분리한다.

### Stats

```rust
struct AdvocateStats {
    logic_speed: u16,
    mental_stamina: u16,
    speech_power: u16,
    guts: u16,
    intellect: u16,
}
```

범위는 MVP에서 `0..=100`으로 고정한다. 성장식과 보정식은 별도 config로 빼지 않는다.

### Training

```rust
struct TrainingAction {
    id: TrainingActionId,
    label: String,
    stat_delta: AdvocateStatsDelta,
}
```

MVP에서는 한 턴에 action 하나만 선택한다. 비용, 실패, 대성공, risk는 나중에 추가한다.

### Court

```rust
struct CourtState {
    turn: u16,
    ally_hp: i16,
    enemy_hp: i16,
    momentum: i16,
    log: Vec<CourtLogEntry>,
    result: Option<CourtResult>,
}

```

자동 재판은 deterministic seed를 가진다. 같은 `GameSession`과 같은 seed는 같은 결과를 내야 한다.
MVP에서는 `complete_training_action`이 stat 성장과 court simulation을 한 번에 끝낸다.
화면 연출은 생성된 로그를 tick마다 한 줄씩 공개하는 renderer/view concern으로 처리한다.

### Dating / LLM Context

아직 실제 LLM worker가 없으므로 별도 context struct를 유지하지 않는다. LLM을 붙일 때 court result와 stats snapshot에서 필요한 prompt context를 만든다.

### Render Model

도메인 상태를 그대로 renderer에 넘기지 않는다. 중간에 `ViewModel`을 둔다.

```text
GameSession
  -> ViewModel
  -> RenderCommand
  -> TerminalBackend
```

`ViewModel`은 화면 표현에 필요한 값만 가진다.

```rust
struct TrainingViewModel {
    stats: AdvocateStats,
    actions: Vec<ActionView>,
    focused_action: usize,
    week: u16,
}
```

## Delivery

MVP 도메인 완료 기준:

- `GameSession` 하나로 전체 MVP 진행을 표현할 수 있다.
- `GamePhase` 전이는 단방향으로 검증 가능하다.
- training action 하나가 stats를 변경한다.
- court simulation이 stats snapshot으로 결과를 만든다.
- dating phase가 입력을 transcript에 기록하고 result로 종료된다.
- renderer 없이 도메인 테스트가 가능하다.

이번 범위에서 제외:

- 저장/로드
- 여러 주차 training
- branching story
- 복잡한 evidence matching
- LLM memory persistence
