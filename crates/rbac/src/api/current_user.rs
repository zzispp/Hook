#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentUser {
    pub id: String,
    pub username: String,
    pub role: String,
    pub group_code: String,
    pub system: bool,
}
