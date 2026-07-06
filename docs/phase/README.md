# Project-Y Phase 문서

이 디렉터리는 Project-Y의 아키텍처와 단계별 구현 계획을 DDD 형태로 기록한다.

- Decision: 지금 선택한 방향
- Design: 선택한 방향의 동작 방식
- Delivery: 각 Phase에서 만들어야 할 결과물과 검증 기준

가장 중요한 제약은 터미널 환경의 가변성이다. `213 x 60` 같은 값은 셀 그리드일 뿐이다.
macOS 창 모드, 전체화면, Kitty, Ghostty, Windows Terminal, 폰트 크기, 줄 간격, 창 패딩에 따라
같은 셀 그리드라도 실제 물리 픽셀 크기는 달라질 수 있다. Project-Y는 셀 크기와 픽셀 크기를
항상 별개의 런타임 값으로 취급해야 한다.

## 현재 결정

- 전체화면 또는 큰 고정 터미널 창을 권장한다.
- 전체화면을 강제하지는 않는다. 창 모드는 letterbox/pillarbox로 처리한다.
- Ratatui는 레이아웃 계산과 입력 상태 관리에만 사용한다.
- 정적 UI 판때기는 캐시된 SVG asset으로 처리한다.
- 동적 UI 상태와 스트리밍 텍스트는 직접 래스터 레이어로 그린다.
- 고품질 기본 백엔드는 Kitty graphics protocol이다.
- Windows Terminal은 Sixel 후보로 보되, live probe와 clear/redraw 전략이 확인되기 전까지 보장하지 않는다.
- Kitty shared memory는 첫 PoC 경로가 아니라 선택적 고속 경로다.
- `three-rs`는 사용하지 않는다. 3D가 MVP에 들어가면 Bevy headless renderer를 쓴다.

## Phase 문서

- [Phase 1: 터미널 메트릭과 Kitty 이미지 PoC](./phase-1-terminal-graphics-poc.md)
- [Phase 2: 게임 루프와 LLM 스트리밍](./phase-2-game-loop-llm.md)
- [Phase 3: 오프스크린 3D 렌더링](./phase-3-offscreen-3d.md)
- [Phase 4: 최적화와 호환성 강화](./phase-4-optimization.md)
- [MVP 도메인 아키텍처](./mvp-domain-architecture.md)
- [MVP Lifecycle](./mvp-lifecycle.md)
- [참조 자료](./references.md)
