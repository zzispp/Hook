use serde_json::{Map, Value};

use super::FormatConversionError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InternalRole {
    System,
    Developer,
    User,
    Assistant,
    Tool,
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum InternalContentBlock {
    Text {
        text: String,
        cache_control: Option<Value>,
    },
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
    pub extra: Map<String, Value>,
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
    pub top_k: Option<u32>,
    pub stop_sequences: Vec<String>,
    pub response_format: Option<Value>,
    pub reasoning_effort: Option<String>,
    pub thinking_budget_tokens: Option<u32>,
    pub n: Option<u32>,
    pub presence_penalty: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub seed: Option<u32>,
    pub logprobs: Option<bool>,
    pub top_logprobs: Option<u32>,
    pub extra: Map<String, Value>,
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
    pub cache_read_tokens: Option<u32>,
    pub cache_creation_tokens: Option<u32>,
    pub reasoning_tokens: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InternalResponse {
    pub id: Option<String>,
    pub model: String,
    pub text: String,
    pub content: Vec<InternalContentBlock>,
    pub finish_reason: Option<StopReason>,
    pub usage: Option<InternalUsage>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InternalStreamEvent {
    Start {
        id: Option<String>,
        model: Option<String>,
    },
    ContentBlockStart {
        index: u32,
        block: InternalContentBlock,
    },
    TextDelta(String),
    ThinkingDelta {
        text: String,
        signature: Option<String>,
    },
    ToolCallDelta {
        index: u32,
        id: Option<String>,
        name: Option<String>,
        arguments_delta: String,
    },
    ContentBlockStop {
        index: u32,
    },
    Usage(InternalUsage),
    Error(InternalError),
    Done {
        reason: Option<StopReason>,
        usage: Option<InternalUsage>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InternalError {
    pub message: String,
    pub error_type: Option<String>,
    pub code: Option<String>,
    pub param: Option<String>,
    pub status: Option<u16>,
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

impl InternalResponse {
    pub fn new(
        id: Option<String>,
        model: String,
        content: Vec<InternalContentBlock>,
        finish_reason: Option<StopReason>,
        usage: Option<InternalUsage>,
    ) -> Result<Self, FormatConversionError> {
        let text = text_from_blocks(&content)?;
        Ok(Self {
            id,
            model,
            text,
            content,
            finish_reason,
            usage,
        })
    }
}

fn sum_tokens(prompt: Option<u32>, completion: Option<u32>) -> Option<u32> {
    let prompt_value = prompt?;
    let completion_value = completion?;
    Some(prompt_value.saturating_add(completion_value))
}

impl InternalContentBlock {
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text {
            text: value.into(),
            cache_control: None,
        }
    }

    pub fn text_with_cache_control(value: impl Into<String>, cache_control: Value) -> Self {
        Self::Text {
            text: value.into(),
            cache_control: Some(cache_control),
        }
    }

    pub fn plain_text(&self) -> Option<&str> {
        match self {
            Self::Text { text, .. } => Some(text.as_str()),
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
            top_k: None,
            stop_sequences: Vec::new(),
            response_format: None,
            reasoning_effort: None,
            thinking_budget_tokens: None,
            n: None,
            presence_penalty: None,
            frequency_penalty: None,
            seed: None,
            logprobs: None,
            top_logprobs: None,
            extra: Map::new(),
        }
    }
}

pub fn text_from_blocks(blocks: &[InternalContentBlock]) -> Result<String, FormatConversionError> {
    let mut output = String::new();
    for block in blocks {
        match block {
            InternalContentBlock::Text { text, .. } => output.push_str(text),
            InternalContentBlock::Thinking { .. } | InternalContentBlock::ToolUse { .. } => {}
            _ => {
                return Err(FormatConversionError::unsupported_content(
                    "internal",
                    "non-text response content cannot be flattened",
                ));
            }
        }
    }
    Ok(output)
}
