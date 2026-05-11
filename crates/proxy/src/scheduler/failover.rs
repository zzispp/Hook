use super::{Candidate, SchedulerError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttemptOutcome {
    Success,
    RetryableFailure(String),
    FatalFailure(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpstreamAttempt {
    pub candidate: Candidate,
    pub outcome: AttemptOutcome,
}

pub struct FailoverExecutor;

impl FailoverExecutor {
    pub fn execute<F>(candidates: &[Candidate], mut attempt: F) -> Result<Vec<UpstreamAttempt>, SchedulerError>
    where
        F: FnMut(&Candidate) -> AttemptOutcome,
    {
        if candidates.is_empty() {
            return Err(SchedulerError::NoCandidates);
        }
        let mut attempts = Vec::new();
        for candidate in candidates {
            let outcome = attempt(candidate);
            attempts.push(UpstreamAttempt {
                candidate: candidate.clone(),
                outcome: outcome.clone(),
            });
            if outcome == AttemptOutcome::Success {
                return Ok(attempts);
            }
            if matches!(outcome, AttemptOutcome::FatalFailure(_)) {
                return Err(SchedulerError::AllCandidatesFailed);
            }
        }
        Err(SchedulerError::AllCandidatesFailed)
    }
}
