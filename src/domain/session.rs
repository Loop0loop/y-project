use super::{
    court::{CourtResult, CourtState, simulate_court},
    dating::{DatingContext, DatingEndReason},
    training::{AdvocateStats, TRAINING_ACTIONS, TrainingActionId, apply_delta},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GamePhase {
    Training,
    Court,
    Dating,
    Result,
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GameSession {
    pub(crate) phase: GamePhase,
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
            DomainCommand::SelectTrainingAction(id) => self.select_training_action(id),
            DomainCommand::StartCourt => self.start_court(),
            DomainCommand::SubmitDatingInput(input) => self.submit_dating_input(input),
            DomainCommand::FinishDating(reason) => self.finish_dating(reason),
            DomainCommand::EndSession => {
                self.phase = GamePhase::Exit;
                Ok(())
            }
        }
    }

    fn select_training_action(&mut self, id: TrainingActionId) -> Result<(), DomainError> {
        ensure_phase(self.phase, GamePhase::Training)?;
        let action = TRAINING_ACTIONS
            .iter()
            .find(|action| action.id == id)
            .ok_or(DomainError::UnknownTrainingAction)?;
        self.stats = apply_delta(self.stats, action.delta);
        self.week += 1;
        self.phase = GamePhase::Court;
        Ok(())
    }

    fn start_court(&mut self) -> Result<(), DomainError> {
        ensure_phase(self.phase, GamePhase::Court)?;
        self.court = simulate_court(self.stats, self.session_id);
        let result = self.court.result.ok_or(DomainError::CourtNotResolved)?;
        self.dating_context = Some(DatingContext {
            court_result: result,
            stats_snapshot: self.stats,
            relationship: self.relationship,
            case_summary: format!("{} defended a receipt contradiction case.", self.defendant),
            evidence_summary: self.evidence_summary.clone(),
            injected_summary: format!(
                "Verdict={:?}; momentum={}; stats_total={}",
                result.verdict,
                result.final_momentum,
                self.stats.total()
            ),
        });
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomainCommand {
    SelectTrainingAction(TrainingActionId),
    StartCourt,
    SubmitDatingInput(String),
    FinishDating(DatingEndReason),
    EndSession,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DomainError {
    InvalidPhase {
        expected: GamePhase,
        actual: GamePhase,
    },
    UnknownTrainingAction,
    CourtNotResolved,
}

fn ensure_phase(actual: GamePhase, expected: GamePhase) -> Result<(), DomainError> {
    if actual == expected {
        Ok(())
    } else {
        Err(DomainError::InvalidPhase { expected, actual })
    }
}

pub(crate) fn print_domain_demo() {
    let _supported_end_reasons = [
        DatingEndReason::Completed,
        DatingEndReason::Failed,
        DatingEndReason::Cancelled,
        DatingEndReason::Timeout,
    ];
    let _supported_exit_command = DomainCommand::EndSession;
    let mut session = GameSession::new(1);
    println!("phase={:?} stats={:?}", session.phase, session.stats);

    let action = TRAINING_ACTIONS[0];
    session
        .apply(DomainCommand::SelectTrainingAction(action.id))
        .expect("training");
    println!(
        "training={} phase={:?} stats={:?}",
        action.label, session.phase, session.stats
    );

    session.apply(DomainCommand::StartCourt).expect("court");
    println!(
        "phase={:?} court_result={:?}",
        session.phase, session.court.result
    );
    for line in &session.court.log {
        println!("{line}");
    }

    session
        .apply(DomainCommand::SubmitDatingInput(
            "오늘 재판은 꽤 괜찮았어.".to_string(),
        ))
        .expect("dating input");
    session
        .apply(DomainCommand::FinishDating(DatingEndReason::Completed))
        .expect("finish dating");
    println!(
        "phase={:?} transcript_len={}",
        session.phase,
        session.transcript.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::CourtStatus;

    #[test]
    fn training_action_updates_stats_and_enters_court() {
        let mut session = GameSession::new(1);
        session
            .apply(DomainCommand::SelectTrainingAction(
                TrainingActionId::LogicDrill,
            ))
            .unwrap();

        assert_eq!(session.phase, GamePhase::Court);
        assert_eq!(session.week, 2);
        assert_eq!(session.stats.logic_speed, 40);
    }

    #[test]
    fn court_generates_result_and_dating_context() {
        let mut session = GameSession::new(1);
        session
            .apply(DomainCommand::SelectTrainingAction(
                TrainingActionId::LogicDrill,
            ))
            .unwrap();
        session.apply(DomainCommand::StartCourt).unwrap();

        assert_eq!(session.phase, GamePhase::Dating);
        assert_eq!(session.court.status, CourtStatus::Resolved);
        assert_eq!(session.court.log.len(), 3);
        assert!(session.dating_context.is_some());
    }

    #[test]
    fn rejects_command_in_wrong_phase() {
        let mut session = GameSession::new(1);
        let error = session.apply(DomainCommand::StartCourt).unwrap_err();

        assert_eq!(
            error,
            DomainError::InvalidPhase {
                expected: GamePhase::Court,
                actual: GamePhase::Training,
            }
        );
    }

    #[test]
    fn dating_can_finish_for_failure_paths() {
        for reason in [
            DatingEndReason::Completed,
            DatingEndReason::Failed,
            DatingEndReason::Cancelled,
            DatingEndReason::Timeout,
        ] {
            let mut session = GameSession::new(1);
            session
                .apply(DomainCommand::SelectTrainingAction(
                    TrainingActionId::LogicDrill,
                ))
                .unwrap();
            session.apply(DomainCommand::StartCourt).unwrap();
            session.apply(DomainCommand::FinishDating(reason)).unwrap();

            assert_eq!(session.phase, GamePhase::Result);
        }
    }

    #[test]
    fn full_mvp_loop_is_deterministic() {
        fn run() -> GameSession {
            let mut session = GameSession::new(42);
            session
                .apply(DomainCommand::SelectTrainingAction(
                    TrainingActionId::SpeechPractice,
                ))
                .unwrap();
            session.apply(DomainCommand::StartCourt).unwrap();
            session
                .apply(DomainCommand::SubmitDatingInput("nice".to_string()))
                .unwrap();
            session
                .apply(DomainCommand::FinishDating(DatingEndReason::Completed))
                .unwrap();
            session
        }

        assert_eq!(run(), run());
    }
}
