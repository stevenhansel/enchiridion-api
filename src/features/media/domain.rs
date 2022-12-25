use serde::{Deserialize, Serialize};

#[derive(Debug, sqlx::Type, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "media_type", rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Video,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MediaType::Image => write!(f, "image"),
            MediaType::Video => write!(f, "video"),
        }
    }
}

impl std::str::FromStr for MediaType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "image" => Ok(MediaType::Image),
            "video" => Ok(MediaType::Video),
            _ => Err(())
        }
    }
}
