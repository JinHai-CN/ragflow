//! File2Document entity definition.
//!
//! This module defines the File2Document entity for the `file2document` table in the database.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// File2Document entity
#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize, Validate)]
#[sea_orm(table_name = "file2document")]
pub struct Model {
    /// Primary key (32 characters)
    #[sea_orm(primary_key, column_type = "String(StringLen::N(32))")]
    pub id: String,

    /// File ID (nullable)
    #[sea_orm(column_type = "String(StringLen::N(32))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,

    /// Document ID (nullable)
    #[sea_orm(column_type = "String(StringLen::N(32))", nullable, indexed)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,

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

/// File2Document entity relations
#[derive(Debug, Copy, Clone, EnumIter, DeriveRelation)]
pub enum Relation {
    // Define relationships here when needed
    // e.g., #[sea_orm(belongs_to = "super::file::Entity")]
    // e.g., #[sea_orm(belongs_to = "super::document::Entity")]
}

impl ActiveModelBehavior for ActiveModel {}