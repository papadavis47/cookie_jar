use chrono::{DateTime, Utc};

/// Represents a bucket (category) for organizing cookies
#[derive(Debug, Clone)]
pub struct Bucket {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

/// Represents a cookie (achievement/proud moment)
#[derive(Debug, Clone)]
pub struct Cookie {
    #[allow(dead_code)]
    pub id: i64,
    pub bucket_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Bucket {
    pub fn new(id: i64, name: String, created_at: i64) -> Self {
        Self {
            id,
            name,
            created_at: DateTime::from_timestamp(created_at, 0).unwrap_or_default(),
        }
    }
}

impl Cookie {
    pub fn new(id: i64, bucket_id: i64, content: String, created_at: i64) -> Self {
        Self {
            id,
            bucket_id,
            content,
            created_at: DateTime::from_timestamp(created_at, 0).unwrap_or_default(),
        }
    }
}
