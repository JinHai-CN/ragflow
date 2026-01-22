//! Tenant service for database operations.
//!
//! This module provides service methods for tenant management using SeaORM.

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder, Set,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::entities::tenant::{self, Entity as TenantEntity, Model as TenantModel};

/// Tenant service trait defining tenant management operations
#[async_trait]
pub trait TenantServiceTrait {
    /// Create a new tenant
    async fn create_tenant(
        &self,
        name: Option<String>,
        public_key: Option<String>,
        llm_id: String,
        embd_id: String,
        asr_id: String,
        img2txt_id: String,
        rerank_id: String,
        tts_id: Option<String>,
        parser_ids: String,
    ) -> Result<TenantModel, TenantServiceError>;

    /// Get tenant by ID
    async fn get_tenant_by_id(&self, tenant_id: &str) -> Result<Option<TenantModel>, TenantServiceError>;

    /// Get tenant by name (exact match)
    async fn get_tenant_by_name(&self, name: &str) -> Result<Option<TenantModel>, TenantServiceError>;

    /// Update tenant information
    async fn update_tenant(
        &self,
        tenant_id: &str,
        updates: TenantUpdate,
    ) -> Result<TenantModel, TenantServiceError>;

    /// Delete tenant (soft delete by setting status to "0")
    async fn delete_tenant(&self, tenant_id: &str) -> Result<(), TenantServiceError>;

    /// List all tenants
    async fn list_all_tenants(&self) -> Result<Vec<TenantModel>, TenantServiceError>;

    /// Get tenant information by user ID (tenants where user is owner)
    async fn get_info_by_user_id(&self, user_id: &str) -> Result<Vec<TenantInfo>, TenantServiceError>;

    /// Get joined tenants by user ID (tenants where user has normal role)
    async fn get_joined_tenants_by_user_id(&self, user_id: &str) -> Result<Vec<TenantInfo>, TenantServiceError>;

    /// Decrease tenant credit by specified amount
    async fn decrease_credit(&self, tenant_id: &str, amount: i32) -> Result<(), TenantServiceError>;

    /// Compute user gateway based on tenant ID (hash-based sharding)
    fn user_gateway(&self, tenant_id: &str) -> usize;
}

/// Tenant service implementation
pub struct TenantService {
    db: DatabaseConnection,
}

impl TenantService {
    /// Create a new tenant service instance
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Generate a new UUID for tenant ID
    fn generate_tenant_id() -> String {
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

#[async_trait]
impl TenantServiceTrait for TenantService {
    async fn create_tenant(
        &self,
        name: Option<String>,
        public_key: Option<String>,
        llm_id: String,
        embd_id: String,
        asr_id: String,
        img2txt_id: String,
        rerank_id: String,
        tts_id: Option<String>,
        parser_ids: String,
    ) -> Result<TenantModel, TenantServiceError> {
        // Check if tenant with same name already exists (if name provided)
        if let Some(ref name_val) = name {
            let existing_tenant = self.get_tenant_by_name(name_val).await?;
            if existing_tenant.is_some() {
                return Err(TenantServiceError::TenantAlreadyExists(name_val.clone()));
            }
        }

        let tenant_id = Self::generate_tenant_id();
        let current_timestamp = Self::current_timestamp();
        let current_datetime = Self::timestamp_to_datetime(current_timestamp);

        let tenant = tenant::ActiveModel {
            id: Set(tenant_id),
            name: Set(name),
            public_key: Set(public_key),
            llm_id: Set(llm_id),
            embd_id: Set(embd_id),
            asr_id: Set(asr_id),
            img2txt_id: Set(img2txt_id),
            rerank_id: Set(rerank_id),
            tts_id: Set(tts_id),
            parser_ids: Set(parser_ids),
            credit: Set(512),
            status: Set(Some("1".to_string())),
            create_time: Set(current_timestamp),
            create_date: Set(Some(current_datetime)),
            update_time: Set(current_timestamp),
            update_date: Set(Some(current_datetime)),
        };

        let tenant = tenant.insert(&self.db).await.map_err(TenantServiceError::DbError)?;

        Ok(tenant)
    }

    async fn get_tenant_by_id(&self, tenant_id: &str) -> Result<Option<TenantModel>, TenantServiceError> {
        TenantEntity::find_by_id(tenant_id.to_string())
            .one(&self.db)
            .await
            .map_err(TenantServiceError::DbError)
    }

    async fn get_tenant_by_name(&self, name: &str) -> Result<Option<TenantModel>, TenantServiceError> {
        TenantEntity::find()
            .filter(tenant::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(TenantServiceError::DbError)
    }

    async fn update_tenant(
        &self,
        tenant_id: &str,
        updates: TenantUpdate,
    ) -> Result<TenantModel, TenantServiceError> {
        let tenant = self
            .get_tenant_by_id(tenant_id)
            .await?
            .ok_or_else(|| TenantServiceError::TenantNotFound(tenant_id.to_string()))?;

        let mut active_tenant: tenant::ActiveModel = tenant.into();

        // Update fields if provided
        if let Some(name) = updates.name {
            // Check if name already exists for another tenant
            if let Some(existing) = self.get_tenant_by_name(&name).await? {
                if existing.id != tenant_id {
                    return Err(TenantServiceError::TenantAlreadyExists(name));
                }
            }
            active_tenant.name = Set(Some(name));
        }

        if updates.public_key.is_some() {
            active_tenant.public_key = Set(updates.public_key);
        }

        if let Some(llm_id) = updates.llm_id {
            active_tenant.llm_id = Set(llm_id);
        }

        if let Some(embd_id) = updates.embd_id {
            active_tenant.embd_id = Set(embd_id);
        }

        if let Some(asr_id) = updates.asr_id {
            active_tenant.asr_id = Set(asr_id);
        }

        if let Some(img2txt_id) = updates.img2txt_id {
            active_tenant.img2txt_id = Set(img2txt_id);
        }

        if let Some(rerank_id) = updates.rerank_id {
            active_tenant.rerank_id = Set(rerank_id);
        }

        if updates.tts_id.is_some() {
            active_tenant.tts_id = Set(updates.tts_id);
        }

        if let Some(parser_ids) = updates.parser_ids {
            active_tenant.parser_ids = Set(parser_ids);
        }

        if let Some(credit) = updates.credit {
            active_tenant.credit = Set(credit);
        }

        if updates.status.is_some() {
            active_tenant.status = Set(updates.status);
        }

        // Update timestamps
        let current_timestamp = Self::current_timestamp();
        let current_datetime = Self::timestamp_to_datetime(current_timestamp);
        active_tenant.update_time = Set(current_timestamp);
        active_tenant.update_date = Set(Some(current_datetime));

        let tenant = active_tenant
            .update(&self.db)
            .await
            .map_err(TenantServiceError::DbError)?;

        Ok(tenant)
    }

    async fn delete_tenant(&self, tenant_id: &str) -> Result<(), TenantServiceError> {
        let update = TenantUpdate {
            status: Some("0".to_string()),
            ..Default::default()
        };

        self.update_tenant(tenant_id, update).await?;
        Ok(())
    }

    async fn list_all_tenants(&self) -> Result<Vec<TenantModel>, TenantServiceError> {
        TenantEntity::find()
            .order_by_asc(tenant::Column::Name)
            .all(&self.db)
            .await
            .map_err(TenantServiceError::DbError)
    }

    async fn get_info_by_user_id(&self, user_id: &str) -> Result<Vec<TenantInfo>, TenantServiceError> {
        // This method requires joining with UserTenant table and filtering by owner role.
        // Since we haven't implemented UserTenant entity yet, we'll implement a simplified version.
        // For now, we'll return all tenants where the tenant_id equals user_id (owner assumption).
        // TODO: Implement proper join when UserTenant entity is available.
        let tenants = TenantEntity::find()
            .filter(tenant::Column::Id.eq(user_id))
            .filter(tenant::Column::Status.eq("1"))
            .all(&self.db)
            .await
            .map_err(TenantServiceError::DbError)?;

        let tenant_info = tenants.into_iter().map(|tenant| TenantInfo {
            tenant_id: tenant.id,
            name: tenant.name,
            llm_id: Some(tenant.llm_id),
            embd_id: Some(tenant.embd_id),
            rerank_id: Some(tenant.rerank_id),
            asr_id: Some(tenant.asr_id),
            img2txt_id: Some(tenant.img2txt_id),
            tts_id: tenant.tts_id,
            parser_ids: Some(tenant.parser_ids),
            role: Some("OWNER".to_string()),
        }).collect();

        Ok(tenant_info)
    }

    async fn get_joined_tenants_by_user_id(&self, user_id: &str) -> Result<Vec<TenantInfo>, TenantServiceError> {
        // Similar to get_info_by_user_id but for normal role.
        // TODO: Implement proper join when UserTenant entity is available.
        let tenants = TenantEntity::find()
            .filter(tenant::Column::Id.eq(user_id))
            .filter(tenant::Column::Status.eq("1"))
            .all(&self.db)
            .await
            .map_err(TenantServiceError::DbError)?;

        let tenant_info = tenants.into_iter().map(|tenant| TenantInfo {
            tenant_id: tenant.id,
            name: tenant.name,
            llm_id: Some(tenant.llm_id),
            embd_id: Some(tenant.embd_id),
            rerank_id: Some(tenant.rerank_id),
            asr_id: Some(tenant.asr_id),
            img2txt_id: Some(tenant.img2txt_id),
            tts_id: tenant.tts_id,
            parser_ids: Some(tenant.parser_ids),
            role: Some("NORMAL".to_string()),
        }).collect();

        Ok(tenant_info)
    }

    async fn decrease_credit(&self, tenant_id: &str, amount: i32) -> Result<(), TenantServiceError> {
        let tenant = self
            .get_tenant_by_id(tenant_id)
            .await?
            .ok_or_else(|| TenantServiceError::TenantNotFound(tenant_id.to_string()))?;

        let new_credit = tenant.credit - amount;
        if new_credit < 0 {
            return Err(TenantServiceError::InsufficientCredit(tenant_id.to_string(), tenant.credit));
        }

        let update = TenantUpdate {
            credit: Some(new_credit),
            ..Default::default()
        };

        self.update_tenant(tenant_id, update).await?;
        Ok(())
    }

    fn user_gateway(&self, tenant_id: &str) -> usize {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(tenant_id.as_bytes());
        let result = hasher.finalize();
        // Convert first 8 bytes to u64, then modulo some number (e.g., number of gateways)
        let hash_val = u64::from_be_bytes(result[..8].try_into().unwrap());
        // For now, assume 10 gateways (placeholder)
        (hash_val % 10) as usize
    }
}

/// Data structure for updating tenant information
#[derive(Debug, Clone, Default)]
pub struct TenantUpdate {
    pub name: Option<String>,
    pub public_key: Option<String>,
    pub llm_id: Option<String>,
    pub embd_id: Option<String>,
    pub asr_id: Option<String>,
    pub img2txt_id: Option<String>,
    pub rerank_id: Option<String>,
    pub tts_id: Option<String>,
    pub parser_ids: Option<String>,
    pub credit: Option<i32>,
    pub status: Option<String>,
}

/// Tenant information for user-specific queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    pub tenant_id: String,
    pub name: Option<String>,
    pub llm_id: Option<String>,
    pub embd_id: Option<String>,
    pub rerank_id: Option<String>,
    pub asr_id: Option<String>,
    pub img2txt_id: Option<String>,
    pub tts_id: Option<String>,
    pub parser_ids: Option<String>,
    pub role: Option<String>,
}

/// Tenant service errors
#[derive(Debug, thiserror::Error)]
pub enum TenantServiceError {
    #[error("Database error: {0}")]
    DbError(#[from] DbErr),

    #[error("Tenant not found: {0}")]
    TenantNotFound(String),

    #[error("Tenant already exists: {0}")]
    TenantAlreadyExists(String),

    #[error("Insufficient credit for tenant {0}: current credit {1}")]
    InsufficientCredit(String, i32),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl From<validator::ValidationErrors> for TenantServiceError {
    fn from(err: validator::ValidationErrors) -> Self {
        TenantServiceError::ValidationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, Schema};
    use sea_orm::sea_query::TableCreateStatement;
    use sea_orm::ConnectionTrait;
    use tempfile::tempdir;

    /// Setup an in-memory SQLite database for testing
    async fn setup_test_db() -> anyhow::Result<DatabaseConnection> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite://{}", db_path.display());

        let db = Database::connect(&database_url).await?;

        // Create tenant table
        let schema = Schema::new(db.get_database_backend());
        let stmt: TableCreateStatement = schema.create_table_from_entity(TenantEntity);
        db.execute(db.get_database_backend().build(&stmt)).await?;

        Ok(db)
    }

    #[tokio::test]
    async fn test_create_tenant() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = TenantService::new(db);

        // Create a tenant
        let tenant = service
            .create_tenant(
                Some("Test Tenant".to_string()),
                Some("public_key_123".to_string()),
                "llm_1".to_string(),
                "embd_1".to_string(),
                "asr_1".to_string(),
                "img2txt_1".to_string(),
                "rerank_1".to_string(),
                Some("tts_1".to_string()),
                "parser1,parser2".to_string(),
            )
            .await
            .expect("Failed to create tenant");

        // Verify tenant fields
        assert!(!tenant.id.is_empty());
        assert_eq!(tenant.name, Some("Test Tenant".to_string()));
        assert_eq!(tenant.public_key, Some("public_key_123".to_string()));
        assert_eq!(tenant.llm_id, "llm_1");
        assert_eq!(tenant.embd_id, "embd_1");
        assert_eq!(tenant.asr_id, "asr_1");
        assert_eq!(tenant.img2txt_id, "img2txt_1");
        assert_eq!(tenant.rerank_id, "rerank_1");
        assert_eq!(tenant.tts_id, Some("tts_1".to_string()));
        assert_eq!(tenant.parser_ids, "parser1,parser2");
        assert_eq!(tenant.credit, 512);
        assert_eq!(tenant.status, Some("1".to_string()));
        assert!(tenant.create_time > 0);
        assert!(tenant.update_time > 0);

        // Verify we can retrieve the tenant by ID
        let retrieved = service
            .get_tenant_by_id(&tenant.id)
            .await
            .expect("Failed to retrieve tenant");
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, tenant.id);
        assert_eq!(retrieved.name, tenant.name);
    }

    #[tokio::test]
    async fn test_create_tenant_duplicate_name() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = TenantService::new(db);

        // Create first tenant
        service
            .create_tenant(
                Some("Same Name".to_string()),
                None,
                "llm_1".to_string(),
                "embd_1".to_string(),
                "asr_1".to_string(),
                "img2txt_1".to_string(),
                "rerank_1".to_string(),
                None,
                "parser1".to_string(),
            )
            .await
            .expect("Failed to create first tenant");

        // Try to create second tenant with same name
        let result = service
            .create_tenant(
                Some("Same Name".to_string()),
                None,
                "llm_2".to_string(),
                "embd_2".to_string(),
                "asr_2".to_string(),
                "img2txt_2".to_string(),
                "rerank_2".to_string(),
                None,
                "parser2".to_string(),
            )
            .await;

        assert!(result.is_err());
        match result {
            Err(TenantServiceError::TenantAlreadyExists(name)) => {
                assert_eq!(name, "Same Name");
            }
            _ => panic!("Expected TenantAlreadyExists error"),
        }
    }

    #[tokio::test]
    async fn test_update_tenant() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = TenantService::new(db);

        // Create a tenant
        let tenant = service
            .create_tenant(
                Some("Original Name".to_string()),
                None,
                "llm_1".to_string(),
                "embd_1".to_string(),
                "asr_1".to_string(),
                "img2txt_1".to_string(),
                "rerank_1".to_string(),
                None,
                "parser1".to_string(),
            )
            .await
            .expect("Failed to create tenant");

        let original_update_time = tenant.update_time;

        // Update tenant information
        let updates = TenantUpdate {
            name: Some("Updated Name".to_string()),
            llm_id: Some("llm_2".to_string()),
            credit: Some(1000),
            ..Default::default()
        };

        let updated_tenant = service
            .update_tenant(&tenant.id, updates)
            .await
            .expect("Failed to update tenant");

        // Verify updated fields
        assert_eq!(updated_tenant.name, Some("Updated Name".to_string()));
        assert_eq!(updated_tenant.llm_id, "llm_2");
        assert_eq!(updated_tenant.credit, 1000);

        // Verify update_time was updated
        assert!(updated_tenant.update_time > original_update_time);
    }

    #[tokio::test]
    async fn test_delete_tenant() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = TenantService::new(db);

        // Create a tenant
        let tenant = service
            .create_tenant(
                Some("Delete Test".to_string()),
                None,
                "llm_1".to_string(),
                "embd_1".to_string(),
                "asr_1".to_string(),
                "img2txt_1".to_string(),
                "rerank_1".to_string(),
                None,
                "parser1".to_string(),
            )
            .await
            .expect("Failed to create tenant");

        // Delete tenant (soft delete)
        service
            .delete_tenant(&tenant.id)
            .await
            .expect("Failed to delete tenant");

        // Verify tenant status is "0"
        let deleted_tenant = service
            .get_tenant_by_id(&tenant.id)
            .await
            .expect("Failed to retrieve deleted tenant")
            .expect("Tenant should still exist after soft delete");

        assert_eq!(deleted_tenant.status, Some("0".to_string()));
    }

    #[tokio::test]
    async fn test_decrease_credit() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = TenantService::new(db);

        // Create a tenant with default credit (512)
        let tenant = service
            .create_tenant(
                Some("Credit Test".to_string()),
                None,
                "llm_1".to_string(),
                "embd_1".to_string(),
                "asr_1".to_string(),
                "img2txt_1".to_string(),
                "rerank_1".to_string(),
                None,
                "parser1".to_string(),
            )
            .await
            .expect("Failed to create tenant");

        // Decrease credit by 100
        service
            .decrease_credit(&tenant.id, 100)
            .await
            .expect("Failed to decrease credit");

        let updated_tenant = service
            .get_tenant_by_id(&tenant.id)
            .await
            .expect("Failed to retrieve tenant")
            .unwrap();

        assert_eq!(updated_tenant.credit, 412);

        // Try to decrease more than available credit
        let result = service.decrease_credit(&tenant.id, 500).await;
        assert!(result.is_err());
        match result {
            Err(TenantServiceError::InsufficientCredit(id, credit)) => {
                assert_eq!(id, tenant.id);
                assert_eq!(credit, 412);
            }
            _ => panic!("Expected InsufficientCredit error"),
        }
    }

    #[tokio::test]
    async fn test_list_all_tenants() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = TenantService::new(db);

        // Initially should be empty
        let tenants = service
            .list_all_tenants()
            .await
            .expect("Failed to list tenants");
        assert_eq!(tenants.len(), 0);

        // Create multiple tenants
        let tenant1 = service
            .create_tenant(
                Some("Tenant A".to_string()),
                None,
                "llm_1".to_string(),
                "embd_1".to_string(),
                "asr_1".to_string(),
                "img2txt_1".to_string(),
                "rerank_1".to_string(),
                None,
                "parser1".to_string(),
            )
            .await
            .expect("Failed to create tenant1");

        let tenant2 = service
            .create_tenant(
                Some("Tenant B".to_string()),
                None,
                "llm_2".to_string(),
                "embd_2".to_string(),
                "asr_2".to_string(),
                "img2txt_2".to_string(),
                "rerank_2".to_string(),
                None,
                "parser2".to_string(),
            )
            .await
            .expect("Failed to create tenant2");

        // List all tenants - should be sorted by name
        let tenants = service
            .list_all_tenants()
            .await
            .expect("Failed to list tenants");
        assert_eq!(tenants.len(), 2);

        // Verify sorting by name
        let names: Vec<Option<String>> = tenants.iter().map(|t| t.name.clone()).collect();
        assert_eq!(names, vec![Some("Tenant A".to_string()), Some("Tenant B".to_string())]);
    }

    #[tokio::test]
    async fn test_user_gateway() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = TenantService::new(db);

        // Test that gateway returns a consistent value for same tenant ID
        let tenant_id = "test_tenant_123";
        let gateway1 = service.user_gateway(tenant_id);
        let gateway2 = service.user_gateway(tenant_id);
        assert_eq!(gateway1, gateway2);

        // Different tenant IDs should (likely) produce different gateways
        let other_gateway = service.user_gateway("different_tenant");
        // Note: There's a small chance of collision, but we'll assume it doesn't happen
        assert_ne!(gateway1, other_gateway);
    }
}