//! Tenant entity definition.
//!
//! This module defines the Tenant entity for the `tenant` table in the database.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Tenant entity
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, Validate)]
#[sea_orm(table_name = "tenant")]
pub struct Model {
    /// Primary key (32 characters)
    #[sea_orm(primary_key, column_type = "String(Some(32))")]
    pub id: String,

    /// Tenant name (nullable)
    #[sea_orm(column_type = "String(Some(100))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    /// Public key for tenant authentication (nullable)
    #[sea_orm(column_type = "String(Some(255))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,

    /// Default LLM model ID
    #[sea_orm(column_type = "String(Some(128))", indexed)]
    #[validate(length(min = 1, max = 128))]
    pub llm_id: String,

    /// Default embedding model ID
    #[sea_orm(column_type = "String(Some(128))", indexed)]
    #[validate(length(min = 1, max = 128))]
    pub embd_id: String,

    /// Default ASR (Automatic Speech Recognition) model ID
    #[sea_orm(column_type = "String(Some(128))", indexed)]
    #[validate(length(min = 1, max = 128))]
    pub asr_id: String,

    /// Default image-to-text model ID
    #[sea_orm(column_type = "String(Some(128))", indexed)]
    #[validate(length(min = 1, max = 128))]
    pub img2txt_id: String,

    /// Default rerank model ID
    #[sea_orm(column_type = "String(Some(128))", indexed)]
    #[validate(length(min = 1, max = 128))]
    pub rerank_id: String,

    /// Default TTS (Text-to-Speech) model ID (nullable)
    #[sea_orm(column_type = "String(Some(256))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts_id: Option<String>,

    /// Document processor IDs (comma-separated)
    #[sea_orm(column_type = "String(Some(256))", indexed)]
    #[validate(length(min = 1, max = 256))]
    pub parser_ids: String,

    /// Credit balance (default 512)
    #[sea_orm(indexed)]
    #[serde(default = "default_credit")]
    pub credit: i32,

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

/// Default value for credit field
fn default_credit() -> i32 {
    512
}

/// Default value for status field
fn default_status() -> Option<String> {
    Some("1".to_string())
}

/// Tenant entity relations
#[derive(Debug, Copy, Clone, EnumIter, DeriveRelation)]
pub enum Relation {
    // Define relationships here when needed
    // e.g., #[sea_orm(has_many = "super::user_tenant::Entity")]
}

impl ActiveModelBehavior for ActiveModel {}