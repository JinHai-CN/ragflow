//! File entity definition.
//!
//! This module defines the File entity for the `file` table in the database.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// File entity
#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize, Validate)]
#[sea_orm(table_name = "file")]
pub struct Model {
    /// Primary key (32 characters)
    #[sea_orm(primary_key, column_type = "String(Some(32))")]
    pub id: String,

    /// Parent folder ID
    #[sea_orm(column_type = "String(Some(32))", indexed)]
    pub parent_id: String,

    /// Tenant ID
    #[sea_orm(column_type = "String(Some(32))", indexed)]
    pub tenant_id: String,

    /// Creator user ID
    #[sea_orm(column_type = "String(Some(32))", indexed)]
    pub created_by: String,

    /// File or folder name
    #[sea_orm(column_type = "String(Some(255))", indexed)]
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Storage location (nullable)
    #[sea_orm(column_type = "String(Some(255))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// File size in bytes
    #[sea_orm(indexed)]
    #[serde(default = "default_zero")]
    pub size: i32,

    /// File extension type
    #[sea_orm(column_type = "String(Some(32))", indexed)]
    #[validate(length(min = 1, max = 32))]
    pub type_: String,

    /// Source type (e.g., "knowledgebase", "local")
    #[sea_orm(column_type = "String(Some(128))", indexed)]
    #[serde(default = "default_source_type")]
    pub source_type: String,

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

/// Default zero value for integer fields
fn default_zero() -> i32 {
    0
}

/// Default value for source type field
fn default_source_type() -> String {
    String::new()
}

/// File entity relations
#[derive(Debug, Copy, Clone, EnumIter, DeriveRelation)]
pub enum Relation {
    // Define relationships here when needed
    // e.g., #[sea_orm(has_many = "super::file2document::Entity")]
}

impl ActiveModelBehavior for ActiveModel {}