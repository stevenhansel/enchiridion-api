use validator::ValidationErrors;

pub struct ApiValidationError(ValidationErrors);

pub const API_VALIDATION_ERROR_CODE: &'static str = "API_VALIDATION_ERROR";

impl ApiValidationError {
    pub fn new(e: ValidationErrors) -> ApiValidationError {
        ApiValidationError(e)
    }

    pub fn code(&self) -> String {
        String::from("API_VALIDATION_ERROR")
    }

    pub fn messages(&self) -> Vec<String> {
        let mut messages: Vec<String> = vec![];

        for (_, v) in self.0.field_errors() {
            if v.len() == 0 {
                continue;
            }

            if let Some(msg) = &v[0].message {
                messages.push(msg.to_string());
            }
        }

        messages
    }
}
