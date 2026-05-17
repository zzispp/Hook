use storage::provider::{RequestCandidateRecordInput, RequestCandidateRecordPatch, RequestRecordRecordPatch};
use types::model::PatchField;

use crate::llm_proxy::audit::{AttemptRecordInput, total_tokens};

pub(super) fn candidate_patch(input: &AttemptRecordInput<'_>, patch: &mut RequestCandidateRecordPatch) {
    patch.prompt_tokens = input.usage.and_then(|usage| usage.prompt_tokens);
    patch.completion_tokens = input.usage.and_then(|usage| usage.completion_tokens);
    patch.total_tokens = input.usage.and_then(|usage| usage.total_tokens);
    patch.cache_creation_input_tokens = input.usage.and_then(|usage| usage.cache_creation_input_tokens);
    patch.cache_read_input_tokens = input.usage.and_then(|usage| usage.cache_read_input_tokens);
    patch.input_text_tokens = input.usage.and_then(|usage| usage.input_text_tokens);
    patch.input_audio_tokens = input.usage.and_then(|usage| usage.input_audio_tokens);
    patch.input_image_tokens = input.usage.and_then(|usage| usage.input_image_tokens);
    patch.output_text_tokens = input.usage.and_then(|usage| usage.output_text_tokens);
    patch.output_audio_tokens = input.usage.and_then(|usage| usage.output_audio_tokens);
    patch.output_image_tokens = input.usage.and_then(|usage| usage.output_image_tokens);
    patch.reasoning_tokens = input.usage.and_then(|usage| usage.reasoning_tokens);
    patch.cache_creation_5m_input_tokens = input.usage.and_then(|usage| usage.cache_creation_5m_input_tokens);
    patch.cache_creation_1h_input_tokens = input.usage.and_then(|usage| usage.cache_creation_1h_input_tokens);
    patch.usage_source = input.usage.and_then(|usage| usage.usage_source.map(str::to_owned));
    patch.usage_semantic = input.usage.and_then(|usage| usage.usage_semantic.map(str::to_owned));
}

pub(super) fn candidate_input(input: &AttemptRecordInput<'_>, record: &mut RequestCandidateRecordInput) {
    record.prompt_tokens = input.usage.and_then(|usage| usage.prompt_tokens);
    record.completion_tokens = input.usage.and_then(|usage| usage.completion_tokens);
    record.total_tokens = input.usage.and_then(|usage| usage.total_tokens);
    record.cache_creation_input_tokens = input.usage.and_then(|usage| usage.cache_creation_input_tokens);
    record.cache_read_input_tokens = input.usage.and_then(|usage| usage.cache_read_input_tokens);
    record.input_text_tokens = input.usage.and_then(|usage| usage.input_text_tokens);
    record.input_audio_tokens = input.usage.and_then(|usage| usage.input_audio_tokens);
    record.input_image_tokens = input.usage.and_then(|usage| usage.input_image_tokens);
    record.output_text_tokens = input.usage.and_then(|usage| usage.output_text_tokens);
    record.output_audio_tokens = input.usage.and_then(|usage| usage.output_audio_tokens);
    record.output_image_tokens = input.usage.and_then(|usage| usage.output_image_tokens);
    record.reasoning_tokens = input.usage.and_then(|usage| usage.reasoning_tokens);
    record.cache_creation_5m_input_tokens = input.usage.and_then(|usage| usage.cache_creation_5m_input_tokens);
    record.cache_creation_1h_input_tokens = input.usage.and_then(|usage| usage.cache_creation_1h_input_tokens);
    record.usage_source = input.usage.and_then(|usage| usage.usage_source.map(str::to_owned));
    record.usage_semantic = input.usage.and_then(|usage| usage.usage_semantic.map(str::to_owned));
}

pub(super) fn request_patch(input: &AttemptRecordInput<'_>, patch: &mut RequestRecordRecordPatch) {
    patch.prompt_tokens = option_patch(input.usage.and_then(|usage| usage.prompt_tokens));
    patch.completion_tokens = option_patch(input.usage.and_then(|usage| usage.completion_tokens));
    patch.total_tokens = option_patch(total_tokens(input.usage));
    patch.cache_creation_input_tokens = option_patch(input.usage.and_then(|usage| usage.cache_creation_input_tokens));
    patch.cache_read_input_tokens = option_patch(input.usage.and_then(|usage| usage.cache_read_input_tokens));
    patch.input_text_tokens = option_patch(input.usage.and_then(|usage| usage.input_text_tokens));
    patch.input_audio_tokens = option_patch(input.usage.and_then(|usage| usage.input_audio_tokens));
    patch.input_image_tokens = option_patch(input.usage.and_then(|usage| usage.input_image_tokens));
    patch.output_text_tokens = option_patch(input.usage.and_then(|usage| usage.output_text_tokens));
    patch.output_audio_tokens = option_patch(input.usage.and_then(|usage| usage.output_audio_tokens));
    patch.output_image_tokens = option_patch(input.usage.and_then(|usage| usage.output_image_tokens));
    patch.reasoning_tokens = option_patch(input.usage.and_then(|usage| usage.reasoning_tokens));
    patch.cache_creation_5m_input_tokens = option_patch(input.usage.and_then(|usage| usage.cache_creation_5m_input_tokens));
    patch.cache_creation_1h_input_tokens = option_patch(input.usage.and_then(|usage| usage.cache_creation_1h_input_tokens));
    patch.usage_source = option_patch(input.usage.and_then(|usage| usage.usage_source.map(str::to_owned)));
    patch.usage_semantic = option_patch(input.usage.and_then(|usage| usage.usage_semantic.map(str::to_owned)));
}

fn option_patch<T>(value: Option<T>) -> PatchField<T> {
    match value {
        Some(value) => PatchField::Value(value),
        None => PatchField::Null,
    }
}
