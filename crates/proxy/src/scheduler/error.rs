use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SchedulerError {
    #[error("billing group is inactive: {0}")]
    InactiveGroup(String),
    #[error("model {model} is not allowed by token")]
    TokenModelDenied { model: String },
    #[error("model {model} is not allowed by group {group_code}")]
    GroupModelDenied { group_code: String, model: String },
    #[error("model {model} is not allowed by user")]
    UserModelDenied { model: String },
    #[error("该分组下暂无 {model} 模型可用")]
    NoModelCandidate { model: String },
    #[error("no candidates available")]
    NoCandidates,
    #[error("all candidates failed")]
    AllCandidatesFailed,
}
