use crate::domain::AdvocateStats;

use super::scene::SceneKind;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RenderView {
    pub(crate) scene: SceneKind,
    pub(crate) phase_label: String,
    pub(crate) title: String,
    pub(crate) subtitle: String,
    pub(crate) body: String,
    pub(crate) side_title: String,
    pub(crate) side_body: String,
    pub(crate) progress: f32,
    pub(crate) stats: AdvocateStats,
    pub(crate) week: u16,
    pub(crate) focused_action: usize,
    pub(crate) ally_hp: i16,
    pub(crate) enemy_hp: i16,
    pub(crate) momentum: i16,
    pub(crate) ui_opacity: String,
    pub(crate) current_tab: usize,
}
