pub(crate) mod app_loop;
pub(crate) mod screen;
pub(crate) mod spa;
#[cfg(test)]
mod spa_tests;
mod svg_presenter;
pub(crate) mod view_model;

pub(crate) use app_loop::run_mvp_svg_loop;
pub(crate) use screen::Screen;
pub(crate) use spa::{FAKE_RESPONSE, SpaApp};
