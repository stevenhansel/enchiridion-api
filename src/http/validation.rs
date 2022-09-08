use validator::ValidationErrors;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

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

pub fn validate_date_format(
    date_string: &str,
    format: &'static str,
) -> Result<chrono::DateTime<chrono::Utc>, &'static str> {
    let naive_date = match NaiveDate::parse_from_str(date_string, format) {
        Ok(date) => date,
        Err(_) => return Err("Invalid date format"),
    };
    let naive_time = NaiveTime::from_hms(0, 0, 0);
    let naive_date_time = NaiveDateTime::new(naive_date, naive_time);

    Ok(chrono::Utc.from_utc_datetime(&naive_date_time))
}

