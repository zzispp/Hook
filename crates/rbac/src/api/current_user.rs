#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentUser {
    pub id: String,
    pub username: String,
    pub role: String,
    pub group_codes: Vec<String>,
    pub system: bool,
}
