use super::{FormatConversionError, InternalContentBlock, InternalMessage, InternalRequest, InternalResponse, InternalRole, InternalUsage, StopReason};
use serde_json::{Map, Value};

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

fn sum_tokens(prompt: Option<u32>, completion: Option<u32>) -> Option<u32> {
    let prompt_value = prompt?;
    let completion_value = completion?;
    Some(prompt_value.saturating_add(completion_value))
}
