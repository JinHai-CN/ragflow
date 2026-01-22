//! Knowledgebase entity definition.
//!
//! This module defines the Knowledgebase entity for the `knowledgebase` table in the database.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Knowledgebase entity
#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize, Validate)]
#[sea_orm(table_name = "knowledgebase")]
pub struct Model {
    /// Primary key (32 characters)
    #[sea_orm(primary_key, column_type = "String(Some(32))")]
    pub id: String,

    /// Avatar image as base64 string (nullable)
    #[sea_orm(column_type = "Text", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,

    /// Tenant ID
    #[sea_orm(column_type = "String(Some(32))", indexed)]
    pub tenant_id: String,

    /// Knowledge base name
    #[sea_orm(column_type = "String(Some(128))", indexed)]
    #[validate(length(min = 1, max = 128))]
    pub name: String,

    /// Language preference: "English" or "Chinese"
    #[sea_orm(column_type = "String(Some(32))", nullable, indexed)]
    #[serde(default = "default_language")]
    pub language: Option<String>,

    /// Description of the knowledge base
    #[sea_orm(column_type = "Text", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default embedding model ID
    #[sea_orm(column_type = "String(Some(128))", indexed)]
    pub embd_id: String,

    /// Permission level: "me" or "team"
    #[sea_orm(column_type = "String(Some(16))", indexed)]
    #[serde(default = "default_permission")]
    pub permission: String,

    /// Creator user ID
    #[sea_orm(column_type = "String(Some(32))", indexed)]
    pub created_by: String,

    /// Number of documents in the knowledge base
    #[sea_orm(indexed)]
    #[serde(default = "default_zero")]
    pub doc_num: i32,

    /// Total number of tokens across all documents
    #[sea_orm(indexed)]
    #[serde(default = "default_zero")]
    pub token_num: i32,

    /// Total number of chunks across all documents
    #[sea_orm(indexed)]
    #[serde(default = "default_zero")]
    pub chunk_num: i32,

    /// Similarity threshold for retrieval
    #[sea_orm(indexed)]
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,

    /// Weight for vector similarity in scoring
    #[sea_orm(indexed)]
    #[serde(default = "default_vector_similarity_weight")]
    pub vector_similarity_weight: f32,

    /// Parser ID (default: "naive")
    #[sea_orm(column_type = "String(Some(32))", indexed)]
    #[serde(default = "default_parser_id")]
    pub parser_id: String,

    /// Pipeline ID (nullable)
    #[sea_orm(column_type = "String(Some(32))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline_id: Option<String>,

    /// Parser configuration as JSON
    #[sea_orm(column_type = "Json")]
    #[serde(default = "default_parser_config")]
    pub parser_config: Json,

    /// PageRank score (default: 0)
    #[serde(default = "default_zero")]
    pub pagerank: i32,

    /// GraphRAG task ID (nullable)
    #[sea_orm(column_type = "String(Some(32))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphrag_task_id: Option<String>,

    /// GraphRAG task finish time (nullable)
    #[sea_orm(column_type = "DateTime", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphrag_task_finish_at: Option<DateTime<Utc>>,

    /// RAPTOR task ID (nullable)
    #[sea_orm(column_type = "String(Some(32))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raptor_task_id: Option<String>,

    /// RAPTOR task finish time (nullable)
    #[sea_orm(column_type = "DateTime", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raptor_task_finish_at: Option<DateTime<Utc>>,

    /// Mindmap task ID (nullable)
    #[sea_orm(column_type = "String(Some(32))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mindmap_task_id: Option<String>,

    /// Mindmap task finish time (nullable)
    #[sea_orm(column_type = "DateTime", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mindmap_task_finish_at: Option<DateTime<Utc>>,

    /// Account status: "1" for valid, "0" for invalid (deleted/disabled)
    #[sea_orm(column_type = "String(Some(1))", nullable, indexed)]
    #[serde(default = "default_status")]
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

/// Default value for permission field
fn default_permission() -> String {
    "me".to_string()
}

/// Default zero value for integer fields
fn default_zero() -> i32 {
    0
}

/// Default similarity threshold
fn default_similarity_threshold() -> f32 {
    0.2
}

/// Default vector similarity weight
fn default_vector_similarity_weight() -> f32 {
    0.3
}

/// Default parser ID
fn default_parser_id() -> String {
    "naive".to_string()
}

/// Default parser configuration
fn default_parser_config() -> Json {
    Json::from(serde_json::json!({
        "pages": [[1, 1000000]],
        "table_context_size": 0,
        "image_context_size": 0
    }))
}

/// Default value for status field
fn default_status() -> Option<String> {
    Some("1".to_string())
}

/// Knowledgebase entity relations
#[derive(Debug, Copy, Clone, EnumIter, DeriveRelation)]
pub enum Relation {
    // Define relationships here when needed
    // e.g., #[sea_orm(has_many = "super::document::Entity")]
}

impl ActiveModelBehavior for ActiveModel {}