use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationResult<T> {
    pub total_pages: i32,
    pub count: i32,
    pub has_next: bool,
    pub contents: Vec<T>,
}
