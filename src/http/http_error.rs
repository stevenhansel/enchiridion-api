use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpErrorResponse {
    error_code: String,
    messages: Vec<String>,
}

impl HttpErrorResponse {
    pub fn new(error_code: String, messages: Vec<String>) -> HttpErrorResponse {
        HttpErrorResponse {
            error_code,
            messages,
        }
    }
}
