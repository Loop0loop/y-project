use super::training::AdvocateStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CourtStatus {
    Empty,
    Running,
    Resolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CourtVerdict {
    Win,
    Loss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CourtResult {
    pub(crate) verdict: CourtVerdict,
    pub(crate) final_momentum: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CourtState {
    pub(super) turn: u16,
    pub(super) status: CourtStatus,
    pub(super) ally_hp: i16,
    pub(super) enemy_hp: i16,
    pub(super) momentum: i16,
    pub(super) log: Vec<String>,
    pub(super) result: Option<CourtResult>,
}

impl Default for CourtState {
    fn default() -> Self {
        Self {
            turn: 0,
            status: CourtStatus::Empty,
            ally_hp: 100,
            enemy_hp: 100,
            momentum: 0,
            log: Vec::new(),
            result: None,
        }
    }
}

pub(super) fn simulate_court(stats: AdvocateStats, seed: u64) -> CourtState {
    let mut state = CourtState {
        status: CourtStatus::Running,
        ..CourtState::default()
    };
    let stat_score = stats.total() as i16;
    let seed_bias = ((seed % 17) as i16) - 8;

    for turn in 1..=3 {
        let pressure = 132 + (turn as i16 * 9);
        let swing = ((stat_score - pressure) / 8 + seed_bias).clamp(-24, 24);
        state.turn = turn;
        state.momentum += swing;
        if swing >= 0 {
            state.enemy_hp = (state.enemy_hp - 12 - swing / 3).max(0);
            state.log.push(format!(
                "turn {turn}: Furina finds a contradiction (+{swing})"
            ));
        } else {
            state.ally_hp = (state.ally_hp - 10 + swing / 3).max(0);
            state.log.push(format!(
                "turn {turn}: prosecutor pressures the timeline ({swing})"
            ));
        }
    }

    let verdict = if state.momentum >= -6 {
        CourtVerdict::Win
    } else {
        CourtVerdict::Loss
    };
    state.status = CourtStatus::Resolved;
    state.result = Some(CourtResult {
        verdict,
        final_momentum: state.momentum,
    });
    state
}
