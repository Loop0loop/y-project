# Phase 4: 최적화와 호환성

## Decision

최적화는 완성된 루프가 생긴 뒤에 한다. 첫 구현은 추측성 최적화보다 측정 가능한 병목을 우선한다.

## Design

먼저 측정할 항목:

- SVG parse time
- SVG render time
- dynamic layer draw time
- Bevy GPU readback time
- terminal transfer time
- terminal display latency
- cached layer memory

가능성이 높은 최적화:

- parsed `usvg::Tree` cache
- rendered static `Pixmap` layer cache
- dirty layer만 redraw
- LLM text update coalescing
- 3D viewport FPS와 UI FPS 분리
- 안전한 ring buffer가 있는 경우에만 Kitty shared memory 사용
- Windows Terminal Sixel fallback의 해상도/FPS 낮추기

## Delivery

Phase 4 완료 기준:

- Kitty high-fidelity mode가 probe matrix를 통과한다.
- Ghostty high-fidelity mode가 별도 probe matrix를 통과한다.
- Windows Terminal Sixel fallback은 live probe, redraw, clear 전략이 확인된 경우에만 지원으로 표시한다.
- capability report 화면이 있다.
- fullscreen recommendation 화면이 있다.
- 시간이 지나도 placement id나 image id가 누수되지 않는다.
- memory와 frame-time budget이 아래 형식으로 문서화되어 있다.

```text
backend=kitty-direct|kitty-tempfile|kitty-shm|sixel
terminal=kitty|ghostty|windows-terminal|unknown
grid=...
pixels=...
avg_frame_ms=...
p95_frame_ms=...
present_ms=...
rss_mb=...
image_count_after_5m=...
placement_count_after_5m=...
```
