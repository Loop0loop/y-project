pub(crate) mod app_loop;
pub(crate) mod spa;
#[cfg(test)]
mod spa_tests;
mod svg_presenter;
mod text_view;

pub(crate) use app_loop::{run_mvp_loop, run_mvp_svg_loop};
pub(crate) use spa::{AppViewModel, Screen, SpaApp, FAKE_RESPONSE};
