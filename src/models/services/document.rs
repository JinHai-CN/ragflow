//! Document service for database operations.
//!
//! This module provides service methods for document management using SeaORM.

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

use crate::models::entities::document::{self, Entity as DocumentEntity, Model as DocumentModel};

/// Document service trait defining document management operations
#[async_trait]
pub trait DocumentServiceTrait {
    /// Create a new document
    async fn create_document(
        &self,
        kb_id: String,
        parser_id: String,
        created_by: String,
        name: Option<String>,
        location: Option<String>,
        size: i32,
        type_: String,
        source_type: Option<String>,
        parser_config: Option<JsonValue>,
        pipeline_id: Option<String>,
        suffix: String,
        thumbnail: Option<String>,
        meta_fields: Option<JsonValue>,
    ) -> Result<DocumentModel, DocumentServiceError>;

    /// Get document by ID
    async fn get_document_by_id(&self, doc_id: &str) -> Result<Option<DocumentModel>, DocumentServiceError>;

    /// Get document by name and KB ID
    async fn get_document_by_name(&self, name: &str, kb_id: &str) -> Result<Option<DocumentModel>, DocumentServiceError>;

    /// Get documents by KB ID with pagination and filtering
    async fn get_by_kb_id(
        &self,
        kb_id: &str,
        page_number: u32,
        items_per_page: u32,
        orderby: String,
        desc: bool,
        keywords: Option<String>,
        run_status: Option<String>,
        types: Option<Vec<String>>,
        suffix: Option<Vec<String>>,
        doc_ids: Option<Vec<String>>,
        return_empty_metadata: bool,
    ) -> Result<(Vec<DocumentModel>, u64), DocumentServiceError>;

    /// Get all document IDs for a knowledge base
    async fn get_doc_ids_by_kb_id(&self, kb_id: &str) -> Result<Vec<String>, DocumentServiceError>;

    /// Update document information
    async fn update_document(
        &self,
        doc_id: &str,
        updates: DocumentUpdate,
    ) -> Result<DocumentModel, DocumentServiceError>;

    /// Delete document
    async fn delete_document(&self, doc_id: &str) -> Result<(), DocumentServiceError>;

    /// Check if a document is accessible by a user
    async fn accessible(&self, doc_id: &str, user_id: &str) -> Result<bool, DocumentServiceError>;

    /// Check if a document can be deleted by a specific user
    async fn accessible4deletion(&self, doc_id: &str, user_id: &str) -> Result<bool, DocumentServiceError>;

    /// Get embedding model ID for a document
    async fn get_embd_id(&self, doc_id: &str) -> Result<Option<String>, DocumentServiceError>;

    /// Get chunking configuration for a document
    async fn get_chunking_config(&self, doc_id: &str) -> Result<Option<JsonValue>, DocumentServiceError>;

    /// Increment chunk numbers atomically
    async fn increment_chunk_num(
        &self,
        doc_id: &str,
        kb_id: &str,
        token_num: i32,
        chunk_num: i32,
        duration: f32,
    ) -> Result<u64, DocumentServiceError>;

    /// Decrement chunk numbers atomically
    async fn decrement_chunk_num(
        &self,
        doc_id: &str,
        kb_id: &str,
        token_num: i32,
        chunk_num: i32,
        duration: f32,
    ) -> Result<u64, DocumentServiceError>;

    /// Clear chunk numbers when document is deleted
    async fn clear_chunk_num(&self, doc_id: &str) -> Result<u64, DocumentServiceError>;

    /// Clear chunk numbers when document is re-run
    async fn clear_chunk_num_when_rerun(&self, doc_id: &str) -> Result<u64, DocumentServiceError>;

    /// Update parser configuration for a document
    async fn update_parser_config(&self, id: &str, config: JsonValue) -> Result<(), DocumentServiceError>;

    /// Get tenant ID for a document
    async fn get_tenant_id(&self, doc_id: &str) -> Result<Option<String>, DocumentServiceError>;

    /// Get knowledge base ID for a document
    async fn get_knowledgebase_id(&self, doc_id: &str) -> Result<Option<String>, DocumentServiceError>;

    /// Begin parsing a document
    async fn begin2parse(&self, doc_id: &str, keep_progress: bool) -> Result<(), DocumentServiceError>;

    /// Update metadata fields for a document
    async fn update_meta_fields(&self, doc_id: &str, meta_fields: JsonValue) -> Result<u64, DocumentServiceError>;

    /// Get metadata summary for knowledge bases
    async fn get_metadata_summary(&self, kb_id: &str) -> Result<JsonValue, DocumentServiceError>;

    /// Batch update metadata for documents
    async fn batch_update_metadata(
        &self,
        kb_id: &str,
        doc_ids: Vec<String>,
        updates: Option<Vec<MetadataUpdate>>,
        deletes: Option<Vec<MetadataDelete>>,
    ) -> Result<u64, DocumentServiceError>;

    /// Get newly uploaded documents
    async fn get_newly_uploaded(&self) -> Result<Vec<DocumentModel>, DocumentServiceError>;

    /// Get unfinished documents
    async fn get_unfinished_docs(&self) -> Result<Vec<DocumentModel>, DocumentServiceError>;

    /// Update progress for documents
    async fn update_progress(&self) -> Result<(), DocumentServiceError>;

    /// Update progress immediately for specific documents
    async fn update_progress_immediately(&self, docs: Vec<JsonValue>) -> Result<(), DocumentServiceError>;

    /// Get document count by knowledge base ID
    async fn get_doc_count_by_kb_id(&self, kb_id: &str) -> Result<u64, DocumentServiceError>;

    /// Get knowledge base basic information
    async fn knowledgebase_basic_info(&self, kb_id: &str) -> Result<JsonValue, DocumentServiceError>;
}

/// Document update structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUpdate {
    pub name: Option<String>,
    pub location: Option<String>,
    pub size: Option<i32>,
    pub type_: Option<String>,
    pub source_type: Option<String>,
    pub parser_config: Option<JsonValue>,
    pub pipeline_id: Option<String>,
    pub suffix: Option<String>,
    pub thumbnail: Option<String>,
    pub meta_fields: Option<JsonValue>,
    pub progress: Option<f32>,
    pub progress_msg: Option<String>,
    pub process_begin_at: Option<DateTime<Utc>>,
    pub process_duration: Option<f32>,
    pub run: Option<String>,
    pub status: Option<String>,
}

/// Metadata update structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataUpdate {
    pub key: String,
    pub value: JsonValue,
    pub match_: Option<JsonValue>,
}

/// Metadata delete structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataDelete {
    pub key: String,
    pub value: Option<JsonValue>,
}

/// Document service implementation
pub struct DocumentService {
    db: DatabaseConnection,
}

impl DocumentService {
    /// Create a new document service instance
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Generate a new UUID for document ID
    fn generate_doc_id() -> String {
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

/// Document service error type
#[derive(Debug, thiserror::Error)]
pub enum DocumentServiceError {
    #[error("Database error: {0}")]
    DbError(#[from] DbErr),

    #[error("Document already exists: {0}")]
    DocumentAlreadyExists(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[async_trait]
impl DocumentServiceTrait for DocumentService {
    async fn create_document(
        &self,
        kb_id: String,
        parser_id: String,
        created_by: String,
        name: Option<String>,
        location: Option<String>,
        size: i32,
        type_: String,
        source_type: Option<String>,
        parser_config: Option<JsonValue>,
        pipeline_id: Option<String>,
        suffix: String,
        thumbnail: Option<String>,
        meta_fields: Option<JsonValue>,
    ) -> Result<DocumentModel, DocumentServiceError> {
        // Check if document with same name already exists for this KB
        if let Some(name_str) = &name {
            let existing_doc = self.get_document_by_name(name_str, &kb_id).await?;
            if existing_doc.is_some() {
                return Err(DocumentServiceError::DocumentAlreadyExists(name_str.clone()));
            }
        }

        let doc_id = Self::generate_doc_id();
        let current_timestamp = Self::current_timestamp();
        let current_datetime = Self::timestamp_to_datetime(current_timestamp);

        let doc = document::ActiveModel {
            id: Set(doc_id),
            thumbnail: Set(thumbnail),
            kb_id: Set(kb_id),
            parser_id: Set(parser_id),
            pipeline_id: Set(pipeline_id),
            parser_config: Set(Json::from(parser_config.unwrap_or_else(|| JsonValue::Object(Default::default())))),
            source_type: Set(source_type.unwrap_or_else(|| "local".to_string())),
            type_: Set(type_),
            created_by: Set(created_by),
            name: Set(name),
            location: Set(location),
            size: Set(size),
            token_num: Set(0),
            chunk_num: Set(0),
            progress: Set(0.0),
            progress_msg: Set(String::new()),
            process_begin_at: Set(None),
            process_duration: Set(0.0),
            meta_fields: Set(meta_fields.map(Json::from)),
            suffix: Set(suffix),
            run: Set(Some("0".to_string())),
            status: Set(Some("1".to_string())),
            create_time: Set(current_timestamp),
            create_date: Set(Some(current_datetime)),
            update_time: Set(current_timestamp),
            update_date: Set(Some(current_datetime)),
        };

        let doc = doc.insert(&self.db).await.map_err(DocumentServiceError::DbError)?;

        Ok(doc)
    }

    async fn get_document_by_id(&self, doc_id: &str) -> Result<Option<DocumentModel>, DocumentServiceError> {
        DocumentEntity::find_by_id(doc_id.to_string())
            .one(&self.db)
            .await
            .map_err(DocumentServiceError::DbError)
    }

    async fn get_document_by_name(&self, name: &str, kb_id: &str) -> Result<Option<DocumentModel>, DocumentServiceError> {
        DocumentEntity::find()
            .filter(document::Column::Name.eq(name))
            .filter(document::Column::KbId.eq(kb_id))
            .filter(document::Column::Status.eq(Some("1".to_string())))
            .one(&self.db)
            .await
            .map_err(DocumentServiceError::DbError)
    }

    async fn get_by_kb_id(
        &self,
        _kb_id: &str,
        _page_number: u32,
        _items_per_page: u32,
        _orderby: String,
        _desc: bool,
        _keywords: Option<String>,
        _run_status: Option<String>,
        _types: Option<Vec<String>>,
        _suffix: Option<Vec<String>>,
        _doc_ids: Option<Vec<String>>,
        _return_empty_metadata: bool,
    ) -> Result<(Vec<DocumentModel>, u64), DocumentServiceError> {
        // TODO: Implement complex query with joins and filters
        // For now, return empty result
        Ok((Vec::new(), 0))
    }

    async fn get_doc_ids_by_kb_id(&self, kb_id: &str) -> Result<Vec<String>, DocumentServiceError> {
        let docs = DocumentEntity::find()
            .filter(document::Column::KbId.eq(kb_id))
            .filter(document::Column::Status.eq(Some("1".to_string())))
            .all(&self.db)
            .await
            .map_err(DocumentServiceError::DbError)?;

        Ok(docs.into_iter().map(|doc| doc.id).collect())
    }

    async fn update_document(
        &self,
        _doc_id: &str,
        _updates: DocumentUpdate,
    ) -> Result<DocumentModel, DocumentServiceError> {
        // TODO: Implement update logic
        Err(DocumentServiceError::InternalError("Not implemented".to_string()))
    }

    async fn delete_document(&self, doc_id: &str) -> Result<(), DocumentServiceError> {
        // Soft delete by setting status to "0"
        let mut doc: document::ActiveModel = DocumentEntity::find_by_id(doc_id.to_string())
            .one(&self.db)
            .await
            .map_err(DocumentServiceError::DbError)?
            .ok_or_else(|| DocumentServiceError::DocumentNotFound(doc_id.to_string()))?
            .into();

        doc.status = Set(Some("0".to_string()));
        doc.update_time = Set(Self::current_timestamp());
        doc.update_date = Set(Some(Self::timestamp_to_datetime(Self::current_timestamp())));

        doc.update(&self.db).await.map_err(DocumentServiceError::DbError)?;

        Ok(())
    }

    async fn accessible(&self, _doc_id: &str, _user_id: &str) -> Result<bool, DocumentServiceError> {
        // TODO: Implement access control check
        // For now, return true
        Ok(true)
    }

    async fn accessible4deletion(&self, _doc_id: &str, _user_id: &str) -> Result<bool, DocumentServiceError> {
        // TODO: Implement access control check for deletion
        // For now, return true
        Ok(true)
    }

    async fn get_embd_id(&self, _doc_id: &str) -> Result<Option<String>, DocumentServiceError> {
        // TODO: Implement embedding ID retrieval
        Ok(None)
    }

    async fn get_chunking_config(&self, _doc_id: &str) -> Result<Option<JsonValue>, DocumentServiceError> {
        // TODO: Implement chunking config retrieval
        Ok(None)
    }

    async fn increment_chunk_num(
        &self,
        _doc_id: &str,
        _kb_id: &str,
        _token_num: i32,
        _chunk_num: i32,
        _duration: f32,
    ) -> Result<u64, DocumentServiceError> {
        // TODO: Implement atomic increment
        Ok(0)
    }

    async fn decrement_chunk_num(
        &self,
        _doc_id: &str,
        _kb_id: &str,
        _token_num: i32,
        _chunk_num: i32,
        _duration: f32,
    ) -> Result<u64, DocumentServiceError> {
        // TODO: Implement atomic decrement
        Ok(0)
    }

    async fn clear_chunk_num(&self, _doc_id: &str) -> Result<u64, DocumentServiceError> {
        // TODO: Implement chunk number clearing
        Ok(0)
    }

    async fn clear_chunk_num_when_rerun(&self, _doc_id: &str) -> Result<u64, DocumentServiceError> {
        // TODO: Implement chunk number clearing for re-run
        Ok(0)
    }

    async fn update_parser_config(&self, _id: &str, _config: JsonValue) -> Result<(), DocumentServiceError> {
        // TODO: Implement parser config update
        Err(DocumentServiceError::InternalError("Not implemented".to_string()))
    }

    async fn get_tenant_id(&self, _doc_id: &str) -> Result<Option<String>, DocumentServiceError> {
        // TODO: Implement tenant ID retrieval
        Ok(None)
    }

    async fn get_knowledgebase_id(&self, _doc_id: &str) -> Result<Option<String>, DocumentServiceError> {
        // TODO: Implement knowledge base ID retrieval
        Ok(None)
    }

    async fn begin2parse(&self, _doc_id: &str, _keep_progress: bool) -> Result<(), DocumentServiceError> {
        // TODO: Implement begin parsing logic
        Err(DocumentServiceError::InternalError("Not implemented".to_string()))
    }

    async fn update_meta_fields(&self, _doc_id: &str, _meta_fields: JsonValue) -> Result<u64, DocumentServiceError> {
        // TODO: Implement metadata fields update
        Ok(0)
    }

    async fn get_metadata_summary(&self, _kb_id: &str) -> Result<JsonValue, DocumentServiceError> {
        // TODO: Implement metadata summary retrieval
        Ok(JsonValue::Object(Default::default()))
    }

    async fn batch_update_metadata(
        &self,
        _kb_id: &str,
        _doc_ids: Vec<String>,
        _updates: Option<Vec<MetadataUpdate>>,
        _deletes: Option<Vec<MetadataDelete>>,
    ) -> Result<u64, DocumentServiceError> {
        // TODO: Implement batch metadata update
        Ok(0)
    }

    async fn get_newly_uploaded(&self) -> Result<Vec<DocumentModel>, DocumentServiceError> {
        // TODO: Implement newly uploaded documents retrieval
        Ok(Vec::new())
    }

    async fn get_unfinished_docs(&self) -> Result<Vec<DocumentModel>, DocumentServiceError> {
        // TODO: Implement unfinished documents retrieval
        Ok(Vec::new())
    }

    async fn update_progress(&self) -> Result<(), DocumentServiceError> {
        // TODO: Implement progress update
        Err(DocumentServiceError::InternalError("Not implemented".to_string()))
    }

    async fn update_progress_immediately(&self, _docs: Vec<JsonValue>) -> Result<(), DocumentServiceError> {
        // TODO: Implement immediate progress update
        Err(DocumentServiceError::InternalError("Not implemented".to_string()))
    }

    async fn get_doc_count_by_kb_id(&self, _kb_id: &str) -> Result<u64, DocumentServiceError> {
        // TODO: Implement document count retrieval
        Ok(0)
    }

    async fn knowledgebase_basic_info(&self, _kb_id: &str) -> Result<JsonValue, DocumentServiceError> {
        // TODO: Implement knowledge base basic info retrieval
        Ok(JsonValue::Object(Default::default()))
    }
}