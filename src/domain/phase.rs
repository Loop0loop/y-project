#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GamePhase {
    Training,
    Court,
    Dating,
    Result,
    Exit,
}
