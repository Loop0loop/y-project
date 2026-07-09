use super::phase::GamePhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DomainError {
    InvalidPhase {
        expected: GamePhase,
        actual: GamePhase,
    },
    UnknownTrainingAction,
}

pub(crate) fn ensure_phase(actual: GamePhase, expected: GamePhase) -> Result<(), DomainError> {
    if actual == expected {
        Ok(())
    } else {
        Err(DomainError::InvalidPhase { expected, actual })
    }
}
