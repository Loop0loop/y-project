# 참조 자료

## Terminal Graphics

- Kitty Graphics Protocol  
  https://sw.kovidgoyal.net/kitty/graphics-protocol/

  참고한 내용: graphics query, transfer mode, direct/file/shared-memory 경로,
  RGBA/PNG format, placement id, z-index, placement replacement behavior.

- Ghostty features  
  https://ghostty.org/docs/features

  참고한 내용: Kitty graphics compatibility와 platform expectation.

- Windows Terminal v1.22 release  
  https://github.com/microsoft/terminal/releases/tag/v1.22.10352.0

  참고한 내용: Windows Terminal compatibility 확인 후보. Sixel 지원 여부는 release note만 믿지 않고
  live probe로 확인해야 한다.

- Windows Terminal releases  
  https://github.com/microsoft/terminal/releases

  참고한 내용: Sixel 관련 지속 수정과 compatibility risk.

- XTerm control sequences  
  https://invisible-island.net/xterm/ctlseqs/ctlseqs.html

  참고한 내용: `CSI 14 t`, `CSI 16 t` 같은 terminal pixel/cell size fallback query.

## Ratatui와 Terminal Image Layout

- Ratatui `Layout`  
  https://docs.rs/ratatui/latest/ratatui/layout/struct.Layout.html

  참고한 내용: cell 기반 virtual layout rectangle 계산.

- ratatui-image  
  https://docs.rs/ratatui-image/latest/ratatui_image/

  참고한 내용: protocol detection, font-size/pixel-cell mapping, off-thread image resize/encoding.

- crossterm events  
  https://docs.rs/crossterm/latest/crossterm/event/enum.Event.html

  참고한 내용: key, mouse, paste, focus, resize event.

## SVG와 Rasterization

- resvg  
  https://docs.rs/resvg/latest/resvg/

  참고한 내용: SVG tree를 pixmap으로 render.

- usvg  
  https://docs.rs/usvg/latest/usvg/

  참고한 내용: SVG parsing, CSS resolution, image href handling, text caveat.

- tiny-skia  
  https://docs.rs/tiny-skia/latest/tiny_skia/

  참고한 내용: dynamic UI primitive 직접 raster drawing과 cached layer composition.

## 3D Rendering

- Bevy glTF support  
  https://docs.rs/bevy/latest/bevy/gltf/index.html

  참고한 내용: GLB/glTF scene loading과 asset label.

- Bevy headless renderer example  
  https://raw.githubusercontent.com/bevyengine/bevy/main/examples/app/headless_renderer.rs

  참고한 내용: window 없는 render target, texture copy, CPU readback.

- Bevy render-to-texture example  
  https://raw.githubusercontent.com/bevyengine/bevy/main/examples/3d/render_to_texture.rs

  참고한 내용: camera output을 texture/image로 rendering.

- wgpu `CommandEncoder`  
  https://docs.rs/wgpu/latest/wgpu/struct.CommandEncoder.html

  참고한 내용: offscreen readback 뒤의 texture-to-buffer copy primitive.

- three-rs repository  
  https://github.com/three-rs/three

  참고한 내용: 사용하지 않기로 한 결정. repo가 abandoned 상태임을 명시한다.
