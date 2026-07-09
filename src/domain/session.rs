use super::{
    court::{CourtResult, CourtState, simulate_court},
    dating::DatingEndReason,
    lifecycle::{DomainError, ensure_phase},
    phase::GamePhase,
    training::{AdvocateStats, TRAINING_ACTIONS, TrainingActionId, apply_delta},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GameSession {
    phase: GamePhase,
    session_id: u64,
    week: u16,
    stats: AdvocateStats,
    court: CourtState,
    transcript: Vec<String>,
}

impl GameSession {
    pub(crate) fn new(session_id: u64) -> Self {
        Self {
            phase: GamePhase::Training,
            session_id,
            week: 1,
            stats: AdvocateStats::default(),
            court: CourtState::default(),
            transcript: Vec::new(),
        }
    }

    pub(crate) fn stats(&self) -> AdvocateStats {
        self.stats
    }

    pub(crate) fn phase(&self) -> GamePhase {
        self.phase
    }

    pub(crate) fn court_log(&self) -> &[String] {
        &self.court.log
    }

    pub(crate) fn court_result(&self) -> Option<CourtResult> {
        self.court.result
    }

    pub(crate) fn transcript_len(&self) -> usize {
        self.transcript.len()
    }

    pub(crate) fn week(&self) -> u16 {
        self.week
    }

    pub(crate) fn ally_hp(&self) -> i16 {
        self.court.ally_hp
    }

    pub(crate) fn enemy_hp(&self) -> i16 {
        self.court.enemy_hp
    }

    pub(crate) fn momentum(&self) -> i16 {
        self.court.momentum
    }

    pub(crate) fn complete_training_action(
        &mut self,
        id: TrainingActionId,
    ) -> Result<(), DomainError> {
        ensure_phase(self.phase, GamePhase::Training)?;
        let action = TRAINING_ACTIONS
            .iter()
            .find(|action| action.id == id)
            .ok_or(DomainError::UnknownTrainingAction)?;
        let stats = apply_delta(self.stats, action.delta);
        let week = self.week + 1;
        let court = simulate_court(stats, self.session_id);

        self.stats = stats;
        self.week = week;
        self.court = court;
        self.phase = GamePhase::Dating;
        Ok(())
    }

    pub(crate) fn submit_dating_input(&mut self, input: String) -> Result<(), DomainError> {
        ensure_phase(self.phase, GamePhase::Dating)?;
        if !input.trim().is_empty() {
            self.transcript.push(format!("user: {}", input.trim()));
        }
        Ok(())
    }

    pub(crate) fn finish_dating(&mut self, reason: DatingEndReason) -> Result<(), DomainError> {
        ensure_phase(self.phase, GamePhase::Dating)?;
        self.transcript.push(format!("dating ended: {reason:?}"));
        self.phase = GamePhase::Result;
        Ok(())
    }
}

pub(crate) fn print_domain_demo() -> Result<(), String> {
    let _supported_end_reasons = [
        DatingEndReason::Completed,
        DatingEndReason::Failed,
        DatingEndReason::Cancelled,
        DatingEndReason::Timeout,
    ];
    let mut session = GameSession::new(1);
    println!("phase={:?} stats={:?}", session.phase(), session.stats);

    let action = TRAINING_ACTIONS[0];
    session
        .complete_training_action(action.id)
        .map_err(|error| format!("{error:?}"))?;
    println!(
        "training={} phase={:?} stats={:?}",
        action.label,
        session.phase(),
        session.stats
    );

    println!(
        "phase={:?} court_result={:?}",
        session.phase(),
        session.court.result
    );
    for line in &session.court.log {
        println!("{line}");
    }

    session
        .submit_dating_input("오늘 재판은 꽤 괜찮았어.".to_string())
        .map_err(|error| format!("{error:?}"))?;
    session
        .finish_dating(DatingEndReason::Completed)
        .map_err(|error| format!("{error:?}"))?;
    println!(
        "phase={:?} transcript_len={}",
        session.phase(),
        session.transcript.len()
    );
    Ok(())
}
