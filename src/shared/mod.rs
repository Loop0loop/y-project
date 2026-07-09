pub(crate) mod easing;
pub(crate) mod text;

pub(crate) use easing::ease_out;
pub(crate) use text::escape_xml;

pub(crate) const PORTRAIT_WIDTH: u32 = 600;
pub(crate) const PORTRAIT_HEIGHT: u32 = 900;
pub(crate) const PORTRAIT_ASPECT: f64 = PORTRAIT_WIDTH as f64 / PORTRAIT_HEIGHT as f64;
pub(crate) const SPLASH_PROGRESS_WIDTH: f32 = 500.0;
pub(crate) const LOADING_PROGRESS_WIDTH: f32 = 520.0;

const LOADING_TIPS: [(&str, &str); 4] = [
    (
        "휴식 팁",
        "휴식은 전략입니다. 체력이 떨어지면 변론의 타격감이 줄어요.",
    ),
    (
        "변론 팁",
        "상대의 모순을 발견하면 과감하게 '이의있소!'를 외치세요.",
    ),
    (
        "훈련 팁",
        "주차별 일정을 계획하여 능력치를 골고루 성장시켜야 합니다.",
    ),
    (
        "Fontaine 법률",
        "모든 공판은 물의 신 푸리나 님의 참관 하에 집행됩니다.",
    ),
];

pub(crate) fn loading_tip(seed: u32) -> (&'static str, &'static str) {
    LOADING_TIPS[seed as usize % LOADING_TIPS.len()]
}
