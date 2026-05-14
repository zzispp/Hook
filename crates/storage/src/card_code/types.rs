#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardCodeTypeRecordInput {
    pub name: String,
    pub balance_type: String,
    pub status: String,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardCodeTypeRecordPatch {
    pub name: String,
    pub balance_type: String,
    pub status: String,
    pub remark: Option<String>,
}
