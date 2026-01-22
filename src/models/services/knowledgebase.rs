//! Knowledgebase service for database operations.
//!
//! This module provides service methods for knowledge base management using SeaORM.

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, Set,
};
use sea_orm::entity::prelude::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::Value as JsonValue;

use crate::models::entities::knowledgebase::{self, Entity as KnowledgebaseEntity, Model as KnowledgebaseModel};

/// Knowledgebase service trait defining knowledge base management operations
#[async_trait]
pub trait KnowledgebaseServiceTrait {
    /// Create a new knowledge base
    async fn create_knowledgebase(
        &self,
        name: String,
        tenant_id: String,
        created_by: String,
        embd_id: String,
        parser_id: String,
        avatar: Option<String>,
        language: Option<String>,
        description: Option<String>,
        permission: Option<String>,
        parser_config: Option<JsonValue>,
        pipeline_id: Option<String>,
    ) -> Result<KnowledgebaseModel, KnowledgebaseServiceError>;

    /// Get knowledge base by ID
    async fn get_knowledgebase_by_id(&self, kb_id: &str) -> Result<Option<KnowledgebaseModel>, KnowledgebaseServiceError>;

    /// Get knowledge base by name and tenant ID
    async fn get_knowledgebase_by_name(&self, name: &str, tenant_id: &str) -> Result<Option<KnowledgebaseModel>, KnowledgebaseServiceError>;

    /// List knowledge bases by tenant IDs with pagination and filtering
    async fn get_by_tenant_ids(
        &self,
        joined_tenant_ids: Vec<String>,
        user_id: String,
        page_number: u32,
        items_per_page: u32,
        orderby: String,
        desc: bool,
        keywords: Option<String>,
        parser_id: Option<String>,
    ) -> Result<(Vec<KnowledgebaseModel>, u64), KnowledgebaseServiceError>;

    /// Get all knowledge base IDs for a tenant
    async fn get_kb_ids(&self, tenant_id: &str) -> Result<Vec<String>, KnowledgebaseServiceError>;

    /// Update knowledge base information
    async fn update_knowledgebase(
        &self,
        kb_id: &str,
        updates: KnowledgebaseUpdate,
    ) -> Result<KnowledgebaseModel, KnowledgebaseServiceError>;

    /// Delete knowledge base (soft delete by setting status to "0")
    async fn delete_knowledgebase(&self, kb_id: &str) -> Result<(), KnowledgebaseServiceError>;

    /// Check if a knowledge base is accessible by a user
    async fn accessible(&self, kb_id: &str, user_id: &str) -> Result<bool, KnowledgebaseServiceError>;

    /// Check if a knowledge base can be deleted by a specific user
    async fn accessible4deletion(&self, kb_id: &str, user_id: &str) -> Result<bool, KnowledgebaseServiceError>;

    /// Check if all documents in the knowledge base have completed parsing
    async fn is_parsed_done(&self, kb_id: &str) -> Result<(bool, Option<String>), KnowledgebaseServiceError>;

    /// Update parser configuration for a knowledge base
    async fn update_parser_config(&self, id: &str, config: JsonValue) -> Result<(), KnowledgebaseServiceError>;

    /// Get field mappings for knowledge bases
    async fn get_field_map(&self, ids: Vec<String>) -> Result<JsonValue, KnowledgebaseServiceError>;

    /// Increase document count atomically
    async fn atomic_increase_doc_num_by_id(&self, kb_id: &str) -> Result<u64, KnowledgebaseServiceError>;

    /// Decrease document numbers when documents are deleted
    async fn decrease_document_num_in_delete(&self, kb_id: &str, doc_num_info: DocumentNumInfo) -> Result<u64, KnowledgebaseServiceError>;
}

/// Knowledgebase update structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgebaseUpdate {
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub language: Option<String>,
    pub description: Option<String>,
    pub permission: Option<String>,
    pub parser_config: Option<JsonValue>,
    pub pipeline_id: Option<String>,
    pub status: Option<String>,
}

/// Document number information for deletion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentNumInfo {
    pub doc_num: i32,
    pub chunk_num: i32,
    pub token_num: i32,
}

/// Knowledgebase service implementation
pub struct KnowledgebaseService {
    db: DatabaseConnection,
}

impl KnowledgebaseService {
    /// Create a new knowledge base service instance
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Generate a new UUID for knowledge base ID
    fn generate_kb_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Get current timestamp in milliseconds
    fn current_timestamp() -> i64 {
        Utc::now().timestamp_millis()
    }

    /// Convert timestamp to DateTime
    fn timestamp_to_datetime(timestamp: i64) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(timestamp).unwrap_or_else(Utc::now)
    }
}

/// Knowledgebase service error type
#[derive(Debug, thiserror::Error)]
pub enum KnowledgebaseServiceError {
    #[error("Database error: {0}")]
    DbError(#[from] DbErr),

    #[error("Knowledge base already exists: {0}")]
    KnowledgebaseAlreadyExists(String),

    #[error("Knowledge base not found: {0}")]
    KnowledgebaseNotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[async_trait]
impl KnowledgebaseServiceTrait for KnowledgebaseService {
    async fn create_knowledgebase(
        &self,
        name: String,
        tenant_id: String,
        created_by: String,
        embd_id: String,
        parser_id: String,
        avatar: Option<String>,
        language: Option<String>,
        description: Option<String>,
        permission: Option<String>,
        parser_config: Option<JsonValue>,
        pipeline_id: Option<String>,
    ) -> Result<KnowledgebaseModel, KnowledgebaseServiceError> {
        // Check if knowledge base with same name already exists for this tenant
        let existing_kb = self.get_knowledgebase_by_name(&name, &tenant_id).await?;
        if existing_kb.is_some() {
            return Err(KnowledgebaseServiceError::KnowledgebaseAlreadyExists(name));
        }

        let kb_id = Self::generate_kb_id();
        let current_timestamp = Self::current_timestamp();
        let current_datetime = Self::timestamp_to_datetime(current_timestamp);

        let kb = knowledgebase::ActiveModel {
            id: Set(kb_id),
            avatar: Set(avatar),
            tenant_id: Set(tenant_id),
            name: Set(name),
            language: Set(language),
            description: Set(description),
            embd_id: Set(embd_id),
            permission: Set(permission.unwrap_or_else(|| "me".to_string())),
            created_by: Set(created_by),
            doc_num: Set(0),
            token_num: Set(0),
            chunk_num: Set(0),
            similarity_threshold: Set(0.2),
            vector_similarity_weight: Set(0.3),
            parser_id: Set(parser_id),
            pipeline_id: Set(pipeline_id),
            parser_config: Set(Json::from(parser_config.unwrap_or_else(|| JsonValue::Object(Default::default())))),
            pagerank: Set(0),
            graphrag_task_id: Set(None),
            graphrag_task_finish_at: Set(None),
            raptor_task_id: Set(None),
            raptor_task_finish_at: Set(None),
            mindmap_task_id: Set(None),
            mindmap_task_finish_at: Set(None),
            status: Set(Some("1".to_string())),
            create_time: Set(current_timestamp),
            create_date: Set(Some(current_datetime)),
            update_time: Set(current_timestamp),
            update_date: Set(Some(current_datetime)),
        };

        let kb = kb.insert(&self.db).await.map_err(KnowledgebaseServiceError::DbError)?;

        Ok(kb)
    }

    async fn get_knowledgebase_by_id(&self, kb_id: &str) -> Result<Option<KnowledgebaseModel>, KnowledgebaseServiceError> {
        KnowledgebaseEntity::find_by_id(kb_id.to_string())
            .one(&self.db)
            .await
            .map_err(KnowledgebaseServiceError::DbError)
    }

    async fn get_knowledgebase_by_name(&self, name: &str, tenant_id: &str) -> Result<Option<KnowledgebaseModel>, KnowledgebaseServiceError> {
        KnowledgebaseEntity::find()
            .filter(knowledgebase::Column::Name.eq(name))
            .filter(knowledgebase::Column::TenantId.eq(tenant_id))
            .filter(knowledgebase::Column::Status.eq(Some("1".to_string())))
            .one(&self.db)
            .await
            .map_err(KnowledgebaseServiceError::DbError)
    }

    async fn get_by_tenant_ids(
        &self,
        _joined_tenant_ids: Vec<String>,
        _user_id: String,
        _page_number: u32,
        _items_per_page: u32,
        _orderby: String,
        _desc: bool,
        _keywords: Option<String>,
        _parser_id: Option<String>,
    ) -> Result<(Vec<KnowledgebaseModel>, u64), KnowledgebaseServiceError> {
        // TODO: Implement complex query with joins and filters
        // For now, return empty result
        Ok((Vec::new(), 0))
    }

    async fn get_kb_ids(&self, tenant_id: &str) -> Result<Vec<String>, KnowledgebaseServiceError> {
        let kbs = KnowledgebaseEntity::find()
            .filter(knowledgebase::Column::TenantId.eq(tenant_id))
            .filter(knowledgebase::Column::Status.eq(Some("1".to_string())))
            .all(&self.db)
            .await
            .map_err(KnowledgebaseServiceError::DbError)?;

        Ok(kbs.into_iter().map(|kb| kb.id).collect())
    }

    async fn update_knowledgebase(
        &self,
        _kb_id: &str,
        _updates: KnowledgebaseUpdate,
    ) -> Result<KnowledgebaseModel, KnowledgebaseServiceError> {
        // TODO: Implement update logic
        Err(KnowledgebaseServiceError::InternalError("Not implemented".to_string()))
    }

    async fn delete_knowledgebase(&self, kb_id: &str) -> Result<(), KnowledgebaseServiceError> {
        // Soft delete by setting status to "0"
        let mut kb: knowledgebase::ActiveModel = KnowledgebaseEntity::find_by_id(kb_id.to_string())
            .one(&self.db)
            .await
            .map_err(KnowledgebaseServiceError::DbError)?
            .ok_or_else(|| KnowledgebaseServiceError::KnowledgebaseNotFound(kb_id.to_string()))?
            .into();

        kb.status = Set(Some("0".to_string()));
        kb.update_time = Set(Self::current_timestamp());
        kb.update_date = Set(Some(Self::timestamp_to_datetime(Self::current_timestamp())));

        kb.update(&self.db).await.map_err(KnowledgebaseServiceError::DbError)?;

        Ok(())
    }

    async fn accessible(&self, _kb_id: &str, _user_id: &str) -> Result<bool, KnowledgebaseServiceError> {
        // TODO: Implement access control check
        // For now, return true
        Ok(true)
    }

    async fn accessible4deletion(&self, kb_id: &str, user_id: &str) -> Result<bool, KnowledgebaseServiceError> {
        // Check if user is the creator
        let kb = self.get_knowledgebase_by_id(kb_id).await?;
        match kb {
            Some(kb) => Ok(kb.created_by == user_id),
            None => Ok(false),
        }
    }

    async fn is_parsed_done(&self, _kb_id: &str) -> Result<(bool, Option<String>), KnowledgebaseServiceError> {
        // TODO: Implement document parsing status check
        // For now, return true
        Ok((true, None))
    }

    async fn update_parser_config(&self, _id: &str, _config: JsonValue) -> Result<(), KnowledgebaseServiceError> {
        // TODO: Implement parser config update
        Err(KnowledgebaseServiceError::InternalError("Not implemented".to_string()))
    }

    async fn get_field_map(&self, _ids: Vec<String>) -> Result<JsonValue, KnowledgebaseServiceError> {
        // TODO: Implement field map retrieval
        Ok(JsonValue::Object(Default::default()))
    }

    async fn atomic_increase_doc_num_by_id(&self, _kb_id: &str) -> Result<u64, KnowledgebaseServiceError> {
        // TODO: Implement atomic increment
        Ok(0)
    }

    async fn decrease_document_num_in_delete(&self, _kb_id: &str, _doc_num_info: DocumentNumInfo) -> Result<u64, KnowledgebaseServiceError> {
        // TODO: Implement atomic decrement
        Ok(0)
    }
}