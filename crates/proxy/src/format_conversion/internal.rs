use serde_json::Value;

use super::FormatConversionError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InternalRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InternalContentBlock {
    Text(String),
    Thinking {
        text: String,
        signature: Option<String>,
    },
    Image {
        url: Option<String>,
        data: Option<String>,
        media_type: Option<String>,
    },
    File {
        file_id: Option<String>,
        file_url: Option<String>,
        data: Option<String>,
        media_type: Option<String>,
        filename: Option<String>,
    },
    Audio {
        data: String,
        format: Option<String>,
        media_type: Option<String>,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        tool_use_id: String,
        tool_name: Option<String>,
        content: Vec<InternalContentBlock>,
        is_error: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct InternalTool {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InternalToolChoice {
    Auto,
    None,
    Required,
    Tool(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct InternalMessage {
    pub role: InternalRole,
    pub content: Vec<InternalContentBlock>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InternalRequest {
    pub model: String,
    pub messages: Vec<InternalMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
    pub tools: Vec<InternalTool>,
    pub tool_choice: Option<InternalToolChoice>,
    pub parallel_tool_calls: Option<bool>,
    pub top_p: Option<f64>,
    pub stop_sequences: Vec<String>,
    pub response_format: Option<Value>,
    pub reasoning_effort: Option<String>,
    pub thinking_budget_tokens: Option<u32>,
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
    pub openai_pending_done: Option<PendingStreamDone>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingStreamDone {
    pub reason: Option<StopReason>,
    pub usage: Option<InternalUsage>,
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

impl InternalContentBlock {
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub fn plain_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text.as_str()),
            _ => None,
        }
    }
}

impl InternalMessage {
    pub fn text(role: InternalRole, text: impl Into<String>) -> Self {
        Self {
            role,
            content: vec![InternalContentBlock::text(text)],
        }
    }

    pub fn text_content(&self) -> Result<String, FormatConversionError> {
        let mut output = String::new();
        for block in &self.content {
            let Some(text) = block.plain_text() else {
                return Err(FormatConversionError::unsupported_content("internal", "non-text content cannot be flattened"));
            };
            output.push_str(text);
        }
        Ok(output)
    }
}

impl InternalRequest {
    pub fn new(model: String, messages: Vec<InternalMessage>, stream: bool) -> Self {
        Self {
            model,
            messages,
            temperature: None,
            max_tokens: None,
            stream,
            tools: Vec::new(),
            tool_choice: None,
            parallel_tool_calls: None,
            top_p: None,
            stop_sequences: Vec::new(),
            response_format: None,
            reasoning_effort: None,
            thinking_budget_tokens: None,
        }
    }
}
