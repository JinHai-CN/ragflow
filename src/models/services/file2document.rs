//! File2Document service for database operations.
//!
//! This module provides service methods for File2Document relationship management using SeaORM.

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, Set,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::entities::file2document::{self, Entity as File2DocumentEntity, Model as File2DocumentModel};

/// File2Document service trait defining relationship management operations
#[async_trait]
pub trait File2DocumentServiceTrait {
    /// Create a new File2Document relationship
    async fn create_file2document(
        &self,
        file_id: Option<String>,
        document_id: Option<String>,
    ) -> Result<File2DocumentModel, File2DocumentServiceError>;

    /// Get File2Document relationships by file ID
    async fn get_by_file_id(&self, file_id: &str) -> Result<Vec<File2DocumentModel>, File2DocumentServiceError>;

    /// Get File2Document relationships by document ID
    async fn get_by_document_id(&self, document_id: &str) -> Result<Vec<File2DocumentModel>, File2DocumentServiceError>;

    /// Get File2Document relationships by document IDs
    async fn get_by_document_ids(&self, document_ids: Vec<String>) -> Result<Vec<File2DocumentModel>, File2DocumentServiceError>;

    /// Delete File2Document relationships by file ID
    async fn delete_by_file_id(&self, file_id: &str) -> Result<u64, File2DocumentServiceError>;

    /// Delete File2Document relationships by document ID
    async fn delete_by_document_id(&self, document_id: &str) -> Result<u64, File2DocumentServiceError>;

    /// Delete File2Document relationships by document IDs or file IDs
    async fn delete_by_document_ids_or_file_ids(
        &self,
        document_ids: Option<Vec<String>>,
        file_ids: Option<Vec<String>>,
    ) -> Result<u64, File2DocumentServiceError>;

    /// Update File2Document relationship by file ID
    async fn update_by_file_id(
        &self,
        file_id: &str,
        obj: File2DocumentUpdate,
    ) -> Result<File2DocumentModel, File2DocumentServiceError>;

    /// Get storage address for a document or file
    async fn get_storage_address(
        &self,
        doc_id: Option<&str>,
        file_id: Option<&str>,
    ) -> Result<(String, Option<String>), File2DocumentServiceError>;
}

/// File2Document update structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File2DocumentUpdate {
    pub file_id: Option<String>,
    pub document_id: Option<String>,
}

/// File2Document service implementation
pub struct File2DocumentService {
    db: DatabaseConnection,
}

impl File2DocumentService {
    /// Create a new File2Document service instance
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Generate a new UUID for File2Document ID
    fn generate_file2document_id() -> String {
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

/// File2Document service error type
#[derive(Debug, thiserror::Error)]
pub enum File2DocumentServiceError {
    #[error("Database error: {0}")]
    DbError(#[from] DbErr),

    #[error("File2Document relationship already exists")]
    File2DocumentAlreadyExists,

    #[error("File2Document relationship not found")]
    File2DocumentNotFound,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[async_trait]
impl File2DocumentServiceTrait for File2DocumentService {
    async fn create_file2document(
        &self,
        file_id: Option<String>,
        document_id: Option<String>,
    ) -> Result<File2DocumentModel, File2DocumentServiceError> {
        // Check if relationship already exists
        if let Some(ref doc_id) = document_id {
            let existing = self.get_by_document_id(doc_id).await?;
            if !existing.is_empty() {
                return Err(File2DocumentServiceError::File2DocumentAlreadyExists);
            }
        }

        let id = Self::generate_file2document_id();
        let current_timestamp = Self::current_timestamp();
        let current_datetime = Self::timestamp_to_datetime(current_timestamp);

        let file2doc = file2document::ActiveModel {
            id: Set(id),
            file_id: Set(file_id),
            document_id: Set(document_id),
            create_time: Set(current_timestamp),
            create_date: Set(Some(current_datetime)),
            update_time: Set(current_timestamp),
            update_date: Set(Some(current_datetime)),
        };

        let file2doc = file2doc.insert(&self.db).await.map_err(File2DocumentServiceError::DbError)?;

        Ok(file2doc)
    }

    async fn get_by_file_id(&self, file_id: &str) -> Result<Vec<File2DocumentModel>, File2DocumentServiceError> {
        File2DocumentEntity::find()
            .filter(file2document::Column::FileId.eq(file_id))
            .all(&self.db)
            .await
            .map_err(File2DocumentServiceError::DbError)
    }

    async fn get_by_document_id(&self, document_id: &str) -> Result<Vec<File2DocumentModel>, File2DocumentServiceError> {
        File2DocumentEntity::find()
            .filter(file2document::Column::DocumentId.eq(document_id))
            .all(&self.db)
            .await
            .map_err(File2DocumentServiceError::DbError)
    }

    async fn get_by_document_ids(&self, document_ids: Vec<String>) -> Result<Vec<File2DocumentModel>, File2DocumentServiceError> {
        File2DocumentEntity::find()
            .filter(file2document::Column::DocumentId.is_in(document_ids))
            .all(&self.db)
            .await
            .map_err(File2DocumentServiceError::DbError)
    }

    async fn delete_by_file_id(&self, file_id: &str) -> Result<u64, File2DocumentServiceError> {
        let result = File2DocumentEntity::delete_many()
            .filter(file2document::Column::FileId.eq(file_id))
            .exec(&self.db)
            .await
            .map_err(File2DocumentServiceError::DbError)?;

        Ok(result.rows_affected)
    }

    async fn delete_by_document_id(&self, document_id: &str) -> Result<u64, File2DocumentServiceError> {
        let result = File2DocumentEntity::delete_many()
            .filter(file2document::Column::DocumentId.eq(document_id))
            .exec(&self.db)
            .await
            .map_err(File2DocumentServiceError::DbError)?;

        Ok(result.rows_affected)
    }

    async fn delete_by_document_ids_or_file_ids(
        &self,
        _document_ids: Option<Vec<String>>,
        _file_ids: Option<Vec<String>>,
    ) -> Result<u64, File2DocumentServiceError> {
        // TODO: Implement complex delete logic
        // For now, return empty result
        Ok(0)
    }

    async fn update_by_file_id(
        &self,
        _file_id: &str,
        _obj: File2DocumentUpdate,
    ) -> Result<File2DocumentModel, File2DocumentServiceError> {
        // TODO: Implement update logic
        Err(File2DocumentServiceError::InternalError("Not implemented".to_string()))
    }

    async fn get_storage_address(
        &self,
        _doc_id: Option<&str>,
        _file_id: Option<&str>,
    ) -> Result<(String, Option<String>), File2DocumentServiceError> {
        // TODO: Implement storage address retrieval logic
        // For now, return empty result
        Err(File2DocumentServiceError::InternalError("Not implemented".to_string()))
    }
}