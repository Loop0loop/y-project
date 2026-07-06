use super::{court::CourtResult, training::AdvocateStats};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DatingEndReason {
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DatingContext {
    pub(super) court_result: CourtResult,
    pub(super) stats_snapshot: AdvocateStats,
    pub(super) relationship: i16,
    pub(super) case_summary: String,
    pub(super) evidence_summary: String,
    pub(super) injected_summary: String,
}
