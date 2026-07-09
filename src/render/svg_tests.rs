use super::*;

fn test_view(scene: SceneKind) -> RenderView {
    RenderView {
        scene,
        phase_label: "PHASE".to_string(),
        title: "TITLE".to_string(),
        subtitle: "SUB".to_string(),
        body: "contradiction body".to_string(),
        side_title: "SIDE".to_string(),
        side_body: "SIDE BODY".to_string(),
        progress: 68.0,
        stats: crate::domain::AdvocateStats::default(),
        week: 1,
        focused_action: 0,
        ally_hp: 100,
        enemy_hp: 100,
        momentum: 0,
        ui_opacity: "1.00".to_string(),
        current_tab: 2,
    }
}

#[test]
fn builds_svg_with_requested_size() {
    let mut view = test_view(SceneKind::Home);
    view.body = "A&B".to_string();
    let svg = build_view_svg(320, 140, &view);
    assert!(svg.contains(r#"width="320""#));
    assert!(svg.contains(r#"height="140""#));
    assert!(svg.contains("A&amp;B"));
    assert!(!svg.contains("{{"));
}

#[test]
fn renders_svg_panel_to_rgba_buffer() {
    let rgba = render_view_rgba(64, 32, &test_view(SceneKind::Training)).expect("render svg");
    assert_eq!(rgba.len(), 64 * 32 * 4);
    assert!(rgba.iter().any(|channel| *channel != 0));
}

#[test]
fn renders_splash_to_rgba_buffer() {
    let rgba = render_splash(160, 90, 68.0).expect("render splash");
    assert_eq!(rgba.len(), 160 * 90 * 4);
    assert!(rgba.iter().any(|channel| *channel != 0));
}

#[test]
fn splash_progress_bar_keeps_fractional_width() {
    let mut view = test_view(SceneKind::Loading);
    view.progress = 12.5;
    let svg = build_view_svg(160, 90, &view);
    assert!(svg.contains(r#"width="65.00" height="14""#));
}

#[test]
fn all_scene_templates_resolve_tokens() {
    for scene in [
        SceneKind::Splash,
        SceneKind::Loading,
        SceneKind::Home,
        SceneKind::Training,
        SceneKind::Court,
        SceneKind::Dating,
        SceneKind::Result,
    ] {
        let svg = build_view_svg(320, 480, &test_view(scene));
        assert!(!svg.contains("{{"), "{scene:?} has unresolved token");
    }
}
