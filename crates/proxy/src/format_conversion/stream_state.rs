#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StreamConversionState {
    pub openai_started: bool,
    pub openai_pending_done: Option<super::internal::PendingStreamDone>,
    pub openai_text_started: bool,
    pub openai_text_stopped: bool,
    pub openai_thinking_started: bool,
    pub openai_thinking_stopped: bool,
    pub openai_text_block_index: Option<u32>,
    pub openai_thinking_block_index: Option<u32>,
    pub openai_next_block_index: u32,
    pub openai_tool_blocks: Vec<OpenAiToolStreamItem>,
    pub openai_responses_started: bool,
    pub openai_responses_text_started: bool,
    pub openai_responses_text_stopped: bool,
    pub openai_responses_text_block_index: Option<u32>,
    pub openai_responses_next_source_block_index: u32,
    pub openai_responses_source_tools: Vec<OpenAiResponsesSourceToolStreamItem>,
    pub gemini_started: bool,
    pub gemini_previous_text: String,
    pub gemini_text_started: bool,
    pub gemini_text_stopped: bool,
    pub gemini_thinking_started: bool,
    pub gemini_thinking_stopped: bool,
    pub gemini_text_block_index: Option<u32>,
    pub gemini_thinking_block_index: Option<u32>,
    pub gemini_next_block_index: u32,
    pub target_openai_id: String,
    pub target_openai_model: String,
    pub target_openai_responses_id: String,
    pub target_openai_responses_model: String,
    pub target_openai_responses_sequence: u32,
    pub target_openai_responses_text: String,
    pub target_openai_responses_thinking_text: String,
    pub target_openai_responses_thinking_signature: Option<String>,
    pub target_openai_responses_message_started: bool,
    pub target_openai_responses_text_started: bool,
    pub target_openai_responses_reasoning_started: bool,
    pub target_openai_responses_next_output_index: u32,
    pub target_openai_responses_message_output_index: Option<u32>,
    pub target_openai_responses_reasoning_output_index: Option<u32>,
    pub target_openai_responses_tools: Vec<OpenAiResponsesToolStreamItem>,
    pub target_claude_id: String,
    pub target_claude_model: String,
    pub target_gemini_model: String,
    pub target_gemini_tools: Vec<GeminiToolStreamItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenAiResponsesSourceToolStreamItem {
    pub item_id: String,
    pub call_id: String,
    pub block_index: u32,
    pub name: String,
    pub arguments: String,
    pub stopped: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenAiToolStreamItem {
    pub tool_index: u32,
    pub block_index: u32,
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenAiResponsesToolStreamItem {
    pub block_index: u32,
    pub output_index: u32,
    pub call_id: String,
    pub item_id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GeminiToolStreamItem {
    pub block_index: u32,
    pub id: String,
    pub name: String,
    pub arguments: String,
}
