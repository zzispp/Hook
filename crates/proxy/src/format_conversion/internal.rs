#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InternalRole {
    System,
    User,
    Assistant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InternalMessage {
    pub role: InternalRole,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InternalRequest {
    pub model: String,
    pub messages: Vec<InternalMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    ContentFiltered,
    Unknown,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InternalUsage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InternalResponse {
    pub id: Option<String>,
    pub model: String,
    pub text: String,
    pub finish_reason: Option<StopReason>,
    pub usage: Option<InternalUsage>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InternalStreamEvent {
    Start { id: Option<String>, model: Option<String> },
    TextDelta(String),
    Done { reason: Option<StopReason>, usage: Option<InternalUsage> },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StreamConversionState {
    pub openai_started: bool,
    pub openai_responses_started: bool,
    pub gemini_started: bool,
    pub gemini_previous_text: String,
    pub target_openai_id: String,
    pub target_openai_model: String,
    pub target_openai_responses_id: String,
    pub target_openai_responses_model: String,
    pub target_claude_id: String,
    pub target_claude_model: String,
    pub target_gemini_model: String,
}

impl InternalUsage {
    pub fn with_total(mut self) -> Self {
        if self.total_tokens.is_none() {
            self.total_tokens = sum_tokens(self.prompt_tokens, self.completion_tokens);
        }
        self
    }
}

fn sum_tokens(prompt: Option<u32>, completion: Option<u32>) -> Option<u32> {
    let prompt_value = prompt?;
    let completion_value = completion?;
    Some(prompt_value.saturating_add(completion_value))
}
