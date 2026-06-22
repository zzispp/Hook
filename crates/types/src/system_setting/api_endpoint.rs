use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiEndpoint {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: String,
}
