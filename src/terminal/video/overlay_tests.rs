use super::layout::{OverlayLayout, cap_render_size, fit_portrait_rect};
use super::*;
use crate::terminal::metrics::{TerminalGrid, TerminalPixels};

#[test]
fn opacity_scales_rgba_channels() {
    let mut out = Vec::new();
    apply_opacity(&[50, 20, 10, 128], 0.5, &mut out);
    assert_eq!(out, vec![25, 10, 5, 64]);
}

#[test]
fn caps_overlay_render_size() {
    assert_eq!(cap_render_size(600, 900, 600, 900), (600, 900));
    assert_eq!(cap_render_size(1200, 1800, 600, 900), (600, 900));
}

#[test]
fn overlay_rect_uses_largest_portrait_fit() {
    let rect = fit_portrait_rect(
        TerminalGrid {
            cols: 160,
            rows: 60,
        },
        TerminalPixels {
            width: 1600,
            height: 1200,
        },
        160,
        60,
    );
    assert_eq!(rect.height, 60);
    assert_eq!(rect.width, 80);
    assert_eq!(rect.x, 40);
}

#[test]
fn renders_overlay_at_current_frame_size() {
    let mut overlay = SplashOverlay::new("assets/svg/splash.svg".into());
    overlay
        .render_overlay_if_stale(Path::new("assets/svg/splash.svg"), 60, 40, 0.0)
        .unwrap();
    assert!(overlay.last_state.is_some());
}

#[test]
fn invalidates_overlay_resize_cache() {
    let mut overlay = SplashOverlay::new("assets/svg/splash.svg".into());
    overlay.last_layout = Some(OverlayLayout {
        terminal_cols: 80,
        terminal_rows: 24,
        cell_rect: crate::terminal::layout::CellRect {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        },
        render_width: 60,
        render_height: 90,
    });
    overlay
        .render_overlay_if_stale(Path::new("assets/svg/splash.svg"), 60, 90, 0.0)
        .unwrap();

    overlay.invalidate_layout();

    assert!(overlay.last_layout.is_none());
    assert!(overlay.last_state.is_none());
}

#[test]
fn fade_opacity_does_not_force_svg_reraster() {
    let mut overlay = SplashOverlay::new("assets/svg/splash.svg".into());
    overlay.phase = OverlayPhase::SplashFadeOut {
        start: Instant::now() + std::time::Duration::from_secs(10),
    };
    let frame = overlay.next_overlay_frame();
    overlay
        .render_overlay_if_stale(&frame.2, 60, 40, frame.1)
        .unwrap();
    let first = overlay.last_state.clone();
    let frame = overlay.next_overlay_frame();
    overlay
        .render_overlay_if_stale(&frame.2, 60, 40, frame.1)
        .unwrap();
    assert_eq!(overlay.last_state, first);
}
