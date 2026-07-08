use super::{
    court::{CourtResult, CourtState, simulate_court},
    dating::{DatingContext, DatingEndReason},
    lifecycle::{DomainCommand, DomainError, ensure_phase},
    phase::GamePhase,
    training::{AdvocateStats, TRAINING_ACTIONS, TrainingActionId, apply_delta},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GameSession {
    phase: GamePhase,
    session_id: u64,
    week: u16,
    defendant: String,
    evidence_summary: String,
    stats: AdvocateStats,
    court: CourtState,
    relationship: i16,
    transcript: Vec<String>,
    dating_context: Option<DatingContext>,
}

impl GameSession {
    pub(crate) fn new(session_id: u64) -> Self {
        Self {
            phase: GamePhase::Training,
            session_id,
            week: 1,
            defendant: "Furina".to_string(),
            evidence_summary: "receipt timeline and contradictory cafe testimony".to_string(),
            stats: AdvocateStats::default(),
            court: CourtState::default(),
            relationship: 0,
            transcript: Vec::new(),
            dating_context: None,
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

    pub(crate) fn apply(&mut self, command: DomainCommand) -> Result<(), DomainError> {
        match command {
            DomainCommand::CompleteTrainingAction(id) => self.complete_training_action(id),
            DomainCommand::SubmitDatingInput(input) => self.submit_dating_input(input),
            DomainCommand::FinishDating(reason) => self.finish_dating(reason),
        }
    }

    fn complete_training_action(&mut self, id: TrainingActionId) -> Result<(), DomainError> {
        ensure_phase(self.phase, GamePhase::Training)?;
        let action = TRAINING_ACTIONS
            .iter()
            .find(|action| action.id == id)
            .ok_or(DomainError::UnknownTrainingAction)?;
        let stats = apply_delta(self.stats, action.delta);
        let week = self.week + 1;
        let court = simulate_court(stats, self.session_id);
        let result = court.result.ok_or(DomainError::CourtNotResolved)?;
        let dating_context = DatingContext {
            court_result: result,
            stats_snapshot: stats,
            relationship: self.relationship,
            case_summary: format!("{} defended a receipt contradiction case.", self.defendant),
            evidence_summary: self.evidence_summary.clone(),
            injected_summary: format!(
                "Verdict={:?}; momentum={}; stats_total={}",
                result.verdict,
                result.final_momentum,
                stats.total()
            ),
        };

        self.stats = stats;
        self.week = week;
        self.court = court;
        self.dating_context = Some(dating_context);
        self.phase = GamePhase::Dating;
        Ok(())
    }

    fn submit_dating_input(&mut self, input: String) -> Result<(), DomainError> {
        ensure_phase(self.phase, GamePhase::Dating)?;
        if !input.trim().is_empty() {
            self.transcript.push(format!("user: {}", input.trim()));
        }
        Ok(())
    }

    fn finish_dating(&mut self, reason: DatingEndReason) -> Result<(), DomainError> {
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
        .apply(DomainCommand::CompleteTrainingAction(action.id))
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
        .apply(DomainCommand::SubmitDatingInput(
            "오늘 재판은 꽤 괜찮았어.".to_string(),
        ))
        .map_err(|error| format!("{error:?}"))?;
    session
        .apply(DomainCommand::FinishDating(DatingEndReason::Completed))
        .map_err(|error| format!("{error:?}"))?;
    println!(
        "phase={:?} transcript_len={}",
        session.phase(),
        session.transcript.len()
    );
    Ok(())
}
