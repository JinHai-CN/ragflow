//! Document entity definition.
//!
//! This module defines the Document entity for the `document` table in the database.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Document entity
#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize, Validate)]
#[sea_orm(table_name = "document")]
pub struct Model {
    /// Primary key (32 characters)
    #[sea_orm(primary_key, column_type = "String(StringLen::N(32))")]
    pub id: String,

    /// Thumbnail as base64 string (nullable)
    #[sea_orm(column_type = "Text", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,

    /// Knowledge base ID
    #[sea_orm(column_type = "String(StringLen::N(256))", indexed)]
    pub kb_id: String,

    /// Parser ID (default: "naive")
    #[sea_orm(column_type = "String(StringLen::N(32))", indexed)]
    #[serde(default = "default_parser_id")]
    pub parser_id: String,

    /// Pipeline ID (nullable)
    #[sea_orm(column_type = "String(StringLen::N(32))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline_id: Option<String>,

    /// Parser configuration as JSON
    #[sea_orm(column_type = "Json")]
    #[serde(default = "default_parser_config")]
    pub parser_config: Json,

    /// Source type: "local" or other
    #[sea_orm(column_type = "String(StringLen::N(128))", indexed)]
    #[serde(default = "default_source_type")]
    pub source_type: String,

    /// File extension type
    #[sea_orm(column_type = "String(StringLen::N(32))", indexed)]
    #[validate(length(min = 1, max = 32))]
    pub type_: String,

    /// Creator user ID
    #[sea_orm(column_type = "String(StringLen::N(32))", indexed)]
    pub created_by: String,

    /// File name (nullable)
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Storage location (nullable)
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// File size in bytes
    #[sea_orm(indexed)]
    #[serde(default = "default_zero")]
    pub size: i32,

    /// Total number of tokens
    #[sea_orm(indexed)]
    #[serde(default = "default_zero")]
    pub token_num: i32,

    /// Total number of chunks
    #[sea_orm(indexed)]
    #[serde(default = "default_zero")]
    pub chunk_num: i32,

    /// Processing progress (0.0 to 1.0)
    #[sea_orm(indexed)]
    #[serde(default = "default_progress")]
    pub progress: f32,

    /// Progress message
    #[sea_orm(column_type = "Text", nullable)]
    #[serde(default = "default_empty_string")]
    pub progress_msg: String,

    /// Processing start time (nullable)
    #[sea_orm(column_type = "DateTime", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_begin_at: Option<DateTime<Utc>>,

    /// Processing duration in seconds
    #[serde(default = "default_zero_f32")]
    pub process_duration: f32,

    /// Metadata fields as JSON (nullable)
    #[sea_orm(column_type = "Json", nullable)]
    #[serde(default = "default_empty_json")]
    pub meta_fields: Option<Json>,

    /// Real file extension suffix
    #[sea_orm(column_type = "String(StringLen::N(32))", indexed)]
    #[validate(length(min = 1, max = 32))]
    pub suffix: String,

    /// Run status: "0" (not started), "1" (running), "2" (canceled)
    #[sea_orm(column_type = "String(StringLen::N(1))", nullable, indexed)]
    #[serde(default = "default_run_status")]
    pub run: Option<String>,

    /// Document status: "0" (wasted), "1" (valid)
    #[sea_orm(column_type = "String(StringLen::N(1))", nullable, indexed)]
    #[serde(default = "default_doc_status")]
    pub status: Option<String>,

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

/// Default value for parser ID field
fn default_parser_id() -> String {
    "naive".to_string()
}

/// Default value for parser configuration
fn default_parser_config() -> Json {
    Json::from(serde_json::json!({
        "pages": [[1, 1000000]],
        "table_context_size": 0,
        "image_context_size": 0
    }))
}

/// Default value for source type field
fn default_source_type() -> String {
    "local".to_string()
}

/// Default zero value for integer fields
fn default_zero() -> i32 {
    0
}

/// Default zero value for float fields
fn default_zero_f32() -> f32 {
    0.0
}

/// Default progress value
fn default_progress() -> f32 {
    0.0
}

/// Default empty string
fn default_empty_string() -> String {
    String::new()
}

/// Default empty JSON object
fn default_empty_json() -> Option<Json> {
    Some(Json::from(serde_json::json!({})))
}

/// Default run status
fn default_run_status() -> Option<String> {
    Some("0".to_string())
}

/// Default document status
fn default_doc_status() -> Option<String> {
    Some("1".to_string())
}

/// Document entity relations
#[derive(Debug, Copy, Clone, EnumIter, DeriveRelation)]
pub enum Relation {
    // Define relationships here when needed
    // e.g., #[sea_orm(belongs_to = "super::knowledgebase::Entity")]
}

impl ActiveModelBehavior for ActiveModel {}