# Phase 1: 터미널 그래픽 PoC

## Decision

첫 PoC는 너무 크게 묶지 않는다. `1a -> 1b -> 1c -> 1d`로 쪼개서 실패 지점을 분리한다.

3D, LLM, shared memory부터 시작하지 않는다.

```text
1a. terminal metrics report
1b. Rect -> pixel bbox pure conversion
1c. single Kitty image placement
1d. resize + fallback behavior
```

## Design

### 터미널 크기 모델

Project-Y는 두 좌표계를 모두 저장해야 한다.

```text
TerminalGrid   = { cols, rows }
TerminalPixels = { width_px, height_px }
CellPixels     = {
  width_px:  width_px / cols,
  height_px: height_px / rows
}
```

중요한 예시:

```text
macOS 창 모드의 213 x 60 cells != 고정 픽셀 캔버스
전체화면의 213 x 60 cells != 같은 픽셀 캔버스
폰트 13의 213 x 60 cells != 폰트 15의 213 x 60 cells
```

전체화면을 권장하는 이유는 우발적 resize를 줄이고 큰 캔버스를 안정적으로 확보하기 위해서다.
그래도 고정 해상도를 가정하면 안 된다.

구현 주의점:

- `TIOCGWINSZ`의 `ws_xpixel/ws_ypixel`은 `0`일 수 있다.
- fallback query인 `CSI 14 t`, `CSI 16 t` 응답은 `height;width` 순서다.
- cell size는 fractional value로 보관하고, 최종 image size/position에서만 rounding한다.
- rounding 정책은 `floor origin`, `round size`, `clamp to terminal pixels`로 시작한다.
- resize 직후 값이 stale일 수 있으므로 최소 1회 재측정 또는 debounce를 둔다.
- 터미널 padding과 text area 기준 차이는 probe log에 남긴다.

### Capability Ladder

런타임에서 아래 순서로 probe한다.

1. Kitty graphics protocol 지원 여부
2. Kitty direct transfer dummy image load
3. Kitty tempfile transfer dummy image load
4. 명시적 probe 이후 Kitty shared memory
5. Sixel live probe
6. 미지원 터미널용 Unicode/text fallback

`$TERM`만 믿지 않는다.

Kitty probe는 protocol 지원과 transfer mode 지원을 분리한다. graphics query 뒤 primary DA를 보내고,
응답/timeout 순서로 지원 여부를 판단한다. direct, tempfile, shared memory는 각각 작은 dummy image로
별도 확인한다.

### 렌더링 경로

```text
crossterm resize/input
  -> terminal metric refresh
  -> Ratatui layout split
  -> cell Rect -> pixel Rect
  -> SVG panel rendered by resvg
  -> Kitty image placement replace
```

Kitty placement 교체는 `placement id`만으로 충분하지 않다. 같은 `image id + placement id` 쌍을 재사용해야
기존 placement를 교체할 수 있다. Phase 1부터 image id 재사용, image number 응답 처리, placement 삭제 정책을
검증한다.

### Subphase

#### 1a. Metrics Report

CLI:

```text
project-y --probe
```

출력:

```text
grid=213x60
pixels=3024x1964
cell=14.197x32.733
source=ioctl|csi14t|csi16t|unknown
backend_candidates=kitty-direct,sixel,text
```

#### 1b. Rect 변환 테스트

고정 fixture로 pure function을 검증한다.

```text
grid=213x60 pixels=3024x1964 rect=Rect{x:10,y:5,width:80,height:20}
-> bbox={x:141,y:163,w:1136,h:655}
```

#### 1c. Kitty 단일 이미지

1장의 테스트 PNG/RGBA를 같은 `image id + placement id`로 10회 교체한다.

CLI:

```text
project-y --kitty-demo
```

검증:

- image id가 매번 증가하지 않는다.
- placement가 누적되지 않는다.
- 종료 시 placement delete를 보낸다.

#### 1d. Resize / Fallback

resize 이벤트에서 metric과 bbox를 다시 출력한다. Kitty가 없으면 text fallback으로 probe 결과만 보여준다.
Sixel은 live probe가 통과한 경우에만 fallback 후보로 표시한다.

CLI:

```text
project-y --watch-metrics
```

검증:

- 창 크기를 바꾸면 `changed`가 표시된다.
- 같은 크기를 유지하면 `stable`이 표시된다.
- resize는 domain state를 바꾸지 않는다.

### SVG 생성 방식

Phase 1의 SVG는 이미지나 UI 전체를 벡터화하지 않는다. 목표는 “픽셀 bbox에 맞춘 사선 패널 1개”다.

처음 구현:

```text
PanelSpec { width_px, height_px, accent, title }
  -> SVG string
  -> usvg parse
  -> resvg render
  -> RGBA/PNG
  -> KittyBackend present
```

CLI:

```text
project-y --svg-demo
```

예시 템플릿:

```xml
<svg width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="panel" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#101722"/>
      <stop offset="100%" stop-color="#162436"/>
    </linearGradient>
  </defs>
  <polygon points="24,0 {width},0 {right},{height} 0,{height}" fill="url(#panel)"/>
  <rect x="32" y="56" width="{bar_width}" height="10" fill="#00d5ff"/>
</svg>
```

MVP 이후 최종 구조:

```text
정적 SVG chrome
  -> resize/theme 변경 시에만 parse/render/cache

동적 bar/focus/dialogue
  -> tiny-skia/text layer로 직접 redraw

3D/AI image
  -> raster layer로 직접 blit
```

즉 SVG는 “판때기와 장식” 담당이다. 매 프레임 전체 UI를 SVG 문자열로 재생성하지 않는다.

## Delivery

Phase 1 완료 기준:

- `--probe`가 감지한 `cols x rows`와 `width_px x height_px`를 출력한다.
- 픽셀 크기를 알 수 없으면 경고한다.
- `--dump-layout` 또는 fixture test로 Rect -> pixel bbox 변환을 검증한다.
- Kitty에서 같은 `image id + placement id` 쌍으로 placement 교체가 된다.
- resize 전후 grid, pixel, cell, Ratatui Rect, pixel bbox, 전송 파라미터를 로그로 남긴다.
- 창 모드와 전체화면이 서로 다른 pixel metric으로 감지된다.
- 미지원 터미널은 명확한 fallback 메시지를 보여준다.

이번 Phase에서 제외:

- Bevy
- LLM
- persistent shared memory
- Sixel 최적화

## 현재 검증 상태

2026-07-06 기준 PoC 통과:

- `--probe`: Kitty에서 `ioctl` 기반 `grid/pixels/cell` 감지 확인
- `--dump-layout`: `213x60`, `3024x1964` fixture가 `bbox=x:141,y:163,w:1136,h:655`로 변환됨
- `--kitty-demo`: 작은 RGBA 이미지가 Kitty에 표시되고 같은 `image id + placement id`로 교체됨
- `--watch-metrics`: resize 중 metric 변화가 동적으로 감지됨
- `--svg-demo`: SVG 패널이 `resvg`로 RGBA 래스터화되어 Kitty에 표시됨
- SVG template: Rust 코드 내 inline SVG 대신 `assets/svg/panel.svg`를 사용함

남은 Phase 1 후속 작업:

- 실제 Kitty graphics query 응답 읽기
- `CSI 14t/16t` pixel metric fallback
- Sixel live probe
- shared memory ring buffer 실험
