//! User entity definition.
//!
//! This module defines the User entity for the `user` table in the database.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// User entity
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, Validate)]
#[sea_orm(table_name = "user")]
pub struct Model {
    /// Primary key (32 characters)
    #[sea_orm(primary_key, column_type = "String(StringLen::N(32))")]
    pub id: String,

    /// Access token for API authentication (nullable)
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,

    /// Nickname (display name)
    #[sea_orm(column_type = "String(StringLen::N(100))", indexed)]
    #[validate(length(min = 1, max = 100))]
    pub nickname: String,

    /// Hashed password (nullable for SSO users)
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Email address (unique)
    #[sea_orm(column_type = "String(StringLen::N(255))", indexed)]
    #[validate(email)]
    pub email: String,

    /// Avatar image as base64 string (nullable)
    #[sea_orm(column_type = "Text", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,

    /// Language preference: "English" or "Chinese"
    #[sea_orm(column_type = "String(StringLen::N(32))", nullable, indexed)]
    #[serde(default = "default_language")]
    pub language: Option<String>,

    /// Color scheme: "Bright" or "Dark"
    #[sea_orm(column_type = "String(StringLen::N(32))", nullable, indexed)]
    #[serde(default = "default_color_schema")]
    pub color_schema: Option<String>,

    /// Timezone string (e.g., "UTC+8\tAsia/Shanghai")
    #[sea_orm(column_type = "String(StringLen::N(64))", nullable, indexed)]
    #[serde(default = "default_timezone")]
    pub timezone: Option<String>,

    /// Last login timestamp
    #[sea_orm(column_type = "DateTime", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_time: Option<DateTime<Utc>>,

    /// Authentication status: "1" for authenticated, "0" otherwise
    #[sea_orm(column_type = "String(StringLen::N(1))", indexed)]
    #[serde(default = "default_is_authenticated")]
    pub is_authenticated: String,

    /// Active status: "1" for active, "0" for inactive
    #[sea_orm(column_type = "String(StringLen::N(1))", indexed)]
    #[serde(default = "default_is_active")]
    pub is_active: String,

    /// Anonymous status: "1" for anonymous, "0" for known user
    #[sea_orm(column_type = "String(StringLen::N(1))", indexed)]
    #[serde(default = "default_is_anonymous")]
    pub is_anonymous: String,

    /// Login channel (e.g., "google", "github", "local")
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_channel: Option<String>,

    /// Account status: "1" for valid, "0" for invalid (deleted/disabled)
    #[sea_orm(column_type = "String(StringLen::N(1))", nullable, indexed)]
    #[serde(default = "default_status")]
    pub status: Option<String>,

    /// Superuser flag
    #[sea_orm(indexed)]
    #[serde(default = "default_is_superuser")]
    pub is_superuser: bool,

    /// Creation timestamp (milliseconds since epoch)
    #[sea_orm(indexed)]
    pub create_time: i64,

    /// Creation date (as DateTime)
    #[sea_orm(column_type = "DateTime", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_date: Option<DateTime<Utc>>,

    /// Update timestamp (milliseconds since epoch)
    #[sea_orm(indexed)]
    pub update_time: i64,

    /// Update date (as DateTime)
    #[sea_orm(column_type = "DateTime", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_date: Option<DateTime<Utc>>,
}

/// Default value for language field
fn default_language() -> Option<String> {
    // Check environment variable LANG for Chinese locale
    let lang = std::env::var("LANG").unwrap_or_default();
    if lang.contains("zh_CN") {
        Some("Chinese".to_string())
    } else {
        Some("English".to_string())
    }
}

/// Default value for color_schema field
fn default_color_schema() -> Option<String> {
    Some("Bright".to_string())
}

/// Default value for timezone field
fn default_timezone() -> Option<String> {
    Some("UTC+8\tAsia/Shanghai".to_string())
}

/// Default value for is_authenticated field
fn default_is_authenticated() -> String {
    "1".to_string()
}

/// Default value for is_active field
fn default_is_active() -> String {
    "1".to_string()
}

/// Default value for is_anonymous field
fn default_is_anonymous() -> String {
    "0".to_string()
}

/// Default value for status field
fn default_status() -> Option<String> {
    Some("1".to_string())
}

/// Default value for is_superuser field
fn default_is_superuser() -> bool {
    false
}

/// User entity relations
#[derive(Debug, Copy, Clone, EnumIter, DeriveRelation)]
pub enum Relation {
    // Define relationships here when needed
    // e.g., #[sea_orm(has_many = "super::document::Entity")]
}

impl ActiveModelBehavior for ActiveModel {}