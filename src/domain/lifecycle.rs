use super::{dating::DatingEndReason, phase::GamePhase, training::TrainingActionId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomainCommand {
    CompleteTrainingAction(TrainingActionId),
    SubmitDatingInput(String),
    FinishDating(DatingEndReason),
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

pub(crate) fn ensure_phase(actual: GamePhase, expected: GamePhase) -> Result<(), DomainError> {
    if actual == expected {
        Ok(())
    } else {
        Err(DomainError::InvalidPhase { expected, actual })
    }
}
