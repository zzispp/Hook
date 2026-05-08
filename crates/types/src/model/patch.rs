use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum PatchField<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}

impl<T> PatchField<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }
}

pub fn deserialize_patch_value<'de, T, D>(deserializer: D) -> Result<PatchField<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let value = Option::<T>::deserialize(deserializer)?;
    Ok(match value {
        Some(value) => PatchField::Value(value),
        None => PatchField::Null,
    })
}
