use serde::Serialize;

#[derive(Debug, sqlx::Type, PartialEq, Clone, Serialize)]
#[sqlx(type_name = "request_action_type", rename_all = "snake_case")]
pub enum RequestActionType {
    Create,
    ChangeDate,
    Delete,
    ChangeContent,
    ChangeDevices,
}

impl RequestActionType {
    pub fn label(self) -> &'static str {
        match self {
            RequestActionType::Create => "Create",
            RequestActionType::ChangeDate => "Change Date",
            RequestActionType::Delete => "Delete",
            RequestActionType::ChangeContent => "Change Content",
            RequestActionType::ChangeDevices => "Change Devices",
        }
    }

    pub fn value(self) -> &'static str {
        match self {
            RequestActionType::Create => "create",
            RequestActionType::ChangeDate => "change_date",
            RequestActionType::Delete => "delete",
            RequestActionType::ChangeContent => "change_content",
            RequestActionType::ChangeDevices => "change_devices",
        }
    }
}

pub enum RequestErrorCode {
    EntityNotFound,
    InternalServerError,
}

impl std::fmt::Display for RequestErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestErrorCode::EntityNotFound => write!(f, "ENTITY_NOT_FOUND"),
            RequestErrorCode::InternalServerError => write!(f, "INTERNAL_SERVER_ERROR"),
        }
    }
}

#[derive(Debug)]
pub enum CreateRequestError {
    EntityNotFound(String),
    InternalServerError,
}

impl std::fmt::Display for CreateRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateRequestError::EntityNotFound(message) => write!(f, "{}", message),
            CreateRequestError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}
