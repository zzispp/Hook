use sea_orm::Set;

use super::{PasswordResetTokenRecord, password_reset_tokens, password_reset_tokens::ActiveModel as PasswordResetTokenActiveModel};

pub(super) fn password_reset_token_record(record: password_reset_tokens::Model) -> PasswordResetTokenRecord {
    PasswordResetTokenRecord {
        id: record.id,
        user_id: record.user_id,
        token_hash: record.token_hash,
        expires_at: record.expires_at,
        consumed_at: record.consumed_at,
        created_at: record.created_at,
    }
}

pub(super) fn password_reset_token_active_model(record: PasswordResetTokenRecord) -> PasswordResetTokenActiveModel {
    PasswordResetTokenActiveModel {
        id: Set(record.id),
        user_id: Set(record.user_id),
        token_hash: Set(record.token_hash),
        expires_at: Set(record.expires_at),
        consumed_at: Set(record.consumed_at),
        created_at: Set(record.created_at),
    }
}
