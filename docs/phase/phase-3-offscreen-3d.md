# Phase 3: 오프스크린 3D

## Decision

3D 캐릭터 경로는 Bevy headless rendering으로 구현한다. `three-rs`는 사용하지 않는다.

`three-rs`는 abandoned 상태이고 오래된 graphics stack에 묶여 있다. Bevy는 glTF 로딩, scene spawn,
animation, camera, light, render target, GPU readback 같은 지루하지만 어려운 부분을 이미 제공한다.

## Design

3D renderer는 RGBA 프레임을 생산하는 sidecar module 또는 sidecar process로 둔다.

```text
Bevy headless app
  -> GLB / glTF load
  -> scene, camera, light spawn
  -> Image / texture에 render
  -> texture를 CPU buffer로 copy
  -> { width, height, stride, rgba }를 compositor로 전달
```

compositor는 이 RGBA 프레임을 최종 캔버스에 직접 blit한다. 매 3D 프레임을 SVG `<image>`로 감싸지 않는다.
나중에 벤치마크가 충분히 싸다고 증명되기 전까지는 SVG를 거치지 않는 쪽이 기본이다.

## Delivery

Phase 2 완료 기준:

- cube 또는 단순 GLB가 OS window 없이 offscreen으로 렌더링된다.
- RGBA 프레임이 터미널 UI 캔버스에 합성된다.
- resize 후 target viewport가 바뀌어도 이미지가 깨지지 않는다.
- GPU readback 비용을 측정한다.

이번 Phase에서 제외:

- physics
- custom shader
- facial animation
- 완전한 캐릭터 pipeline

정적 GLB 경로가 안정화된 뒤에만 추가한다.
