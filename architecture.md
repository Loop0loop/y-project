# Project-Y Architecture

Project-Y uses a small ports-and-adapters shape that fits Rust modules without adding framework code.

## Module Layout

```text
src/
  main.rs
  app/
    mod.rs
    app_loop.rs
    spa.rs
  domain/
    mod.rs
    session.rs
    training.rs
    court.rs
    dating.rs
  render/
    mod.rs
    svg.rs
  terminal/
    mod.rs
    kitty.rs
    layout.rs
    metrics.rs
```

## Boundaries

- `domain`: pure game rules and state transitions. No terminal IO, SVG, timers, or crossterm.
- `app`: application lifecycle, screen state, key handling, and ViewModel creation.
- `render`: SVG scene selection, token injection, `resvg` rasterization.
- `terminal`: terminal metrics, cell-to-pixel layout, Kitty Graphics Protocol output.
- `main.rs`: CLI dispatch only.

`mod.rs` files declare the local modules and re-export only the small API needed by the parent layer.

## Dependency Direction

```text
main -> app -> domain
main -> render -> app/domain
main -> terminal
app -> render/terminal
render -> terminal
```

`domain` stays at the bottom. If a domain module needs terminal size, SVG tokens, keyboard codes, or wall-clock time, that logic belongs in `app`, `render`, or `terminal` instead.

## Rendering Performance Plan

Current slow path:

```text
ViewModel change -> SVG string replace -> usvg parse -> full RGBA raster -> base64 Kitty transmit
```

Fix order:

1. Cap internal SVG raster size and let Kitty scale to the terminal placement.
2. Cache terminal metrics briefly instead of probing every frame.
3. Throttle visual-only progress updates to the lowest acceptable FPS.
4. Measure `probe_terminal`, SVG build, SVG parse/raster, and Kitty transmit separately.
5. Split static scene background from dynamic text/progress layers if full SVG raster remains slow.
6. Add scene/size caches only after measurement shows repeated work.

Current runtime policy:

- `cargo run` starts the SVG game loop.
- `cargo run dev` starts the same SVG game loop.
- Input/event polling runs at about `60Hz`.
- Internal SVG raster is capped at `960x540`.
- Terminal metrics are refreshed at most every `250ms`.
- Animated SVG rerenders are throttled to `33ms`; screen changes and training/result changes render immediately.
- The SVG loop clears the alternate screen only on entry, not before every Kitty image update.

Skipped for now: a separate `SceneId`. `app::Screen` already selects the current SVG scene; add `SceneId` only when app screens and render assets diverge.
