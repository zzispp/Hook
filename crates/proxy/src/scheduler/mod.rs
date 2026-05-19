mod builder;
mod error;
mod failover;
mod recorder;
mod types;

pub use builder::CandidateBuilder;
pub use error::SchedulerError;
pub use failover::{AttemptOutcome, FailoverExecutor, UpstreamAttempt};
pub use recorder::{CandidateAuditInput, CandidateAuditRecord, CandidateAuditRecorder};
pub use types::{
    AffinityCandidate, Candidate, EndpointSnapshot, KeySnapshot, ModelAccessPolicy, ModelBindingSnapshot, ProviderSnapshot, SchedulerInput, SchedulingMode,
};
