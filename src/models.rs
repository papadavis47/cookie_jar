use chrono::{DateTime, Local, Utc};

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

    /// Format the creation timestamp in local time with a user-friendly format
    pub fn formatted_created_at(&self) -> String {
        let local_time = self.created_at.with_timezone(&Local);
        local_time.format("%b %d, %Y").to_string()
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

    /// Format the creation timestamp in local time with a user-friendly format
    pub fn formatted_created_at(&self) -> String {
        let local_time = self.created_at.with_timezone(&Local);
        local_time.format("%b %d, %Y at %I:%M %p").to_string()
    }
}
