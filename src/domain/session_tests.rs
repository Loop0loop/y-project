use super::*;
use crate::domain::lifecycle::DomainError;
use crate::domain::phase::GamePhase;
#[test]
fn training_action_updates_stats_and_enters_court() {
    let mut session = GameSession::new(1);
    session
        .complete_training_action(TrainingActionId::LogicDrill)
        .unwrap();

    assert_eq!(session.phase(), GamePhase::Dating);
    assert_eq!(session.week(), 2);
    assert_eq!(session.stats().logic_speed, 40);
}

#[test]
fn court_generates_result() {
    let mut session = GameSession::new(1);
    session
        .complete_training_action(TrainingActionId::LogicDrill)
        .unwrap();

    assert_eq!(session.phase(), GamePhase::Dating);
    assert!(session.court_result().is_some());
    assert_eq!(session.court_log().len(), 3);
}

#[test]
fn rejects_command_in_wrong_phase() {
    let mut session = GameSession::new(1);
    let before = session.clone();
    let error = session
        .finish_dating(DatingEndReason::Completed)
        .unwrap_err();

    assert_eq!(
        error,
        DomainError::InvalidPhase {
            expected: GamePhase::Dating,
            actual: GamePhase::Training,
        }
    );
    assert_eq!(session, before);
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
            .complete_training_action(TrainingActionId::LogicDrill)
            .unwrap();
        session.finish_dating(reason).unwrap();

        assert_eq!(session.phase(), GamePhase::Result);
    }
}

#[test]
fn full_mvp_loop_is_deterministic() {
    fn run() -> GameSession {
        let mut session = GameSession::new(42);
        session
            .complete_training_action(TrainingActionId::SpeechPractice)
            .unwrap();
        session.submit_dating_input("nice".to_string()).unwrap();
        session.finish_dating(DatingEndReason::Completed).unwrap();
        session
    }

    assert_eq!(run(), run());
}
