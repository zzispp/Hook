use serde_json::{Map, Value};

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
        kind: InternalToolKind,
    },
    ToolResult {
        tool_use_id: String,
        tool_name: Option<String>,
        tool_kind: InternalToolKind,
        content: Vec<InternalContentBlock>,
        is_error: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InternalToolKind {
    Function,
    Custom,
}

impl Default for InternalToolKind {
    fn default() -> Self {
        Self::Function
    }
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
