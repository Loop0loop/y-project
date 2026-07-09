#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DatingEndReason {
    Completed,
    Failed,
    Cancelled,
    Timeout,
}
