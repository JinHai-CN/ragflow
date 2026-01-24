//! User service for database operations.
//!
//! This module provides service methods for user management using SeaORM.

use async_trait::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::models::entities::user::{self, Entity as UserEntity, Model as UserModel};

/// User service trait defining user management operations
#[async_trait]
pub trait UserServiceTrait {
    /// Create a new user with hashed password
    async fn create_user(
        &self,
        nickname: String,
        email: String,
        password: Option<String>,
    ) -> Result<UserModel, UserServiceError>;

    /// Get user by ID
    async fn get_user_by_id(&self, user_id: &str) -> Result<Option<UserModel>, UserServiceError>;

    /// Get user by email
    async fn get_user_by_email(&self, email: &str) -> Result<Option<UserModel>, UserServiceError>;

    /// Authenticate user with email and password
    async fn authenticate_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Option<UserModel>, UserServiceError>;

    /// Update user information
    async fn update_user(
        &self,
        user_id: &str,
        updates: UserUpdate,
    ) -> Result<UserModel, UserServiceError>;

    /// Update user password
    async fn update_user_password(
        &self,
        user_id: &str,
        new_password: &str,
    ) -> Result<UserModel, UserServiceError>;

    /// Delete user (soft delete by setting status to "0")
    async fn delete_user(&self, user_id: &str) -> Result<(), UserServiceError>;

    /// Check if user is superuser
    async fn is_superuser(&self, user_id: &str) -> Result<bool, UserServiceError>;

    /// List all users
    async fn list_all_users(&self) -> Result<Vec<UserModel>, UserServiceError>;
}

/// User service implementation
#[derive(Clone)]
pub struct UserService {
    db: DatabaseConnection,
}

impl UserService {
    /// Create a new user service instance
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Generate a new UUID for user ID
    fn generate_user_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Hash a password using bcrypt
    fn hash_password(password: &str) -> Result<String, UserServiceError> {
        hash(password, DEFAULT_COST).map_err(|e| UserServiceError::PasswordHashError(e.to_string()))
    }

    /// Verify a password against a hash
    fn verify_password(password: &str, hash: &str) -> Result<bool, UserServiceError> {
        Ok(crate::utils::check_password_hash(hash, password))
        // verify(password, hash).map_err(|e| UserServiceError::PasswordVerifyError(e.to_string()))
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
impl UserServiceTrait for UserService {
    async fn create_user(
        &self,
        nickname: String,
        email: String,
        password: Option<String>,
    ) -> Result<UserModel, UserServiceError> {
        // Check if user with email already exists
        let existing_user = self.get_user_by_email(&email).await?;
        if existing_user.is_some() {
            return Err(UserServiceError::UserAlreadyExists(email));
        }

        let user_id = Self::generate_user_id();
        let hashed_password = password.map(|p| Self::hash_password(&p)).transpose()?;

        let current_timestamp = Self::current_timestamp();
        let current_datetime = Self::timestamp_to_datetime(current_timestamp);

        let user = user::ActiveModel {
            id: Set(user_id),
            nickname: Set(nickname),
            email: Set(email),
            password: Set(hashed_password),
            create_time: Set(current_timestamp),
            create_date: Set(Some(current_datetime)),
            update_time: Set(current_timestamp),
            update_date: Set(Some(current_datetime)),
            is_authenticated: Set("1".to_string()),
            is_active: Set("1".to_string()),
            is_anonymous: Set("0".to_string()),
            status: Set(Some("1".to_string())),
            is_superuser: Set(false),
            ..Default::default()
        };

        let user = user.insert(&self.db).await.map_err(UserServiceError::DbError)?;

        Ok(user)
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<Option<UserModel>, UserServiceError> {
        UserEntity::find_by_id(user_id.to_string())
            .one(&self.db)
            .await
            .map_err(UserServiceError::DbError)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<UserModel>, UserServiceError> {
        UserEntity::find()
            .filter(user::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(UserServiceError::DbError)
    }

    async fn authenticate_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Option<UserModel>, UserServiceError> {
        let user = self.get_user_by_email(email).await?;

        match user {
            Some(user) => {
                // Check if user is active and valid
                if user.status != Some("1".to_string()) || user.is_active != "1" {
                    return Ok(None);
                }

                // Verify password if user has a password (SSO users may not have one)
                if let Some(hashed_password) = &user.password {
                    let valid = Self::verify_password(password, hashed_password)?;
                    if valid {
                        Ok(Some(user))
                    } else {
                        Ok(None)
                    }
                } else {
                    // User doesn't have a password (SSO user)
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    async fn update_user(
        &self,
        user_id: &str,
        updates: UserUpdate,
    ) -> Result<UserModel, UserServiceError> {
        let user = self
            .get_user_by_id(user_id)
            .await?
            .ok_or_else(|| UserServiceError::UserNotFound(user_id.to_string()))?;

        let mut active_user: user::ActiveModel = user.into();

        // Update fields if provided
        if let Some(nickname) = updates.nickname {
            active_user.nickname = Set(nickname);
        }

        if let Some(email) = updates.email {
            // Check if email already exists for another user
            let existing_user = self.get_user_by_email(&email).await?;
            if existing_user.is_some() && existing_user.unwrap().id != user_id {
                return Err(UserServiceError::UserAlreadyExists(email));
            }
            active_user.email = Set(email);
        }

        // For optional fields, only update if Some is provided
        // Using map to convert Option<String> to Option<Set> then set if Some
        if updates.avatar.is_some() {
            active_user.avatar = Set(updates.avatar);
        }

        if updates.language.is_some() {
            active_user.language = Set(updates.language);
        }

        if updates.color_schema.is_some() {
            active_user.color_schema = Set(updates.color_schema);
        }

        if updates.timezone.is_some() {
            active_user.timezone = Set(updates.timezone);
        }

        if updates.login_channel.is_some() {
            active_user.login_channel = Set(updates.login_channel);
        }

        if updates.status.is_some() {
            active_user.status = Set(updates.status);
        }

        if let Some(is_superuser) = updates.is_superuser {
            active_user.is_superuser = Set(is_superuser);
        }

        if updates.access_token.is_some() {
            active_user.access_token = Set(updates.access_token);
        }

        // Update timestamps
        let current_timestamp = Self::current_timestamp();
        let current_datetime = Self::timestamp_to_datetime(current_timestamp);
        active_user.update_time = Set(current_timestamp);
        active_user.update_date = Set(Some(current_datetime));

        let user = active_user
            .update(&self.db)
            .await
            .map_err(UserServiceError::DbError)?;

        Ok(user)
    }

    async fn update_user_password(
        &self,
        user_id: &str,
        new_password: &str,
    ) -> Result<UserModel, UserServiceError> {
        let user = self
            .get_user_by_id(user_id)
            .await?
            .ok_or_else(|| UserServiceError::UserNotFound(user_id.to_string()))?;

        let mut active_user: user::ActiveModel = user.into();
        let hashed_password = Self::hash_password(new_password)?;

        active_user.password = Set(Some(hashed_password));

        // Update timestamps
        let current_timestamp = Self::current_timestamp();
        let current_datetime = Self::timestamp_to_datetime(current_timestamp);
        active_user.update_time = Set(current_timestamp);
        active_user.update_date = Set(Some(current_datetime));

        let user = active_user
            .update(&self.db)
            .await
            .map_err(UserServiceError::DbError)?;

        Ok(user)
    }

    async fn delete_user(&self, user_id: &str) -> Result<(), UserServiceError> {
        let update = UserUpdate {
            status: Some("0".to_string()),
            ..Default::default()
        };

        self.update_user(user_id, update).await?;
        Ok(())
    }

    async fn is_superuser(&self, user_id: &str) -> Result<bool, UserServiceError> {
        let user = self
            .get_user_by_id(user_id)
            .await?
            .ok_or_else(|| UserServiceError::UserNotFound(user_id.to_string()))?;

        Ok(user.is_superuser)
    }

    async fn list_all_users(&self) -> Result<Vec<UserModel>, UserServiceError> {
        UserEntity::find()
            .order_by_asc(user::Column::Email)
            .all(&self.db)
            .await
            .map_err(UserServiceError::DbError)
    }
}

/// Data structure for updating user information
#[derive(Debug, Clone, Default)]
pub struct UserUpdate {
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub language: Option<String>,
    pub color_schema: Option<String>,
    pub timezone: Option<String>,
    pub login_channel: Option<String>,
    pub status: Option<String>,
    pub is_superuser: Option<bool>,
    pub access_token: Option<String>,
}

/// User service errors
#[derive(Debug, thiserror::Error)]
pub enum UserServiceError {
    #[error("Database error: {0}")]
    DbError(#[from] DbErr),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("User already exists: {0}")]
    UserAlreadyExists(String),

    #[error("Password hash error: {0}")]
    PasswordHashError(String),

    #[error("Password verification error: {0}")]
    PasswordVerifyError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl From<validator::ValidationErrors> for UserServiceError {
    fn from(err: validator::ValidationErrors) -> Self {
        UserServiceError::ValidationError(err.to_string())
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
        // Create a temporary directory for SQLite file (in-memory can also work)
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite://{}", db_path.display());

        // Connect to database
        let db = Database::connect(&database_url).await?;

        // Create user table
        let schema = Schema::new(db.get_database_backend());
        let stmt: TableCreateStatement = schema.create_table_from_entity(UserEntity);
        db.execute(db.get_database_backend().build(&stmt)).await?;

        Ok(db)
    }

    #[tokio::test]
    async fn test_create_user() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = UserService::new(db);

        // Create a user
        let user = service
            .create_user(
                "Test User".to_string(),
                "test@example.com".to_string(),
                Some("password123".to_string()),
            )
            .await
            .expect("Failed to create user");

        // Verify user fields
        assert!(!user.id.is_empty());
        assert_eq!(user.nickname, "Test User");
        assert_eq!(user.email, "test@example.com");
        assert!(user.password.is_some());
        assert_eq!(user.is_authenticated, "1");
        assert_eq!(user.is_active, "1");
        assert_eq!(user.is_anonymous, "0");
        assert_eq!(user.status, Some("1".to_string()));
        assert!(!user.is_superuser);
        assert!(user.create_time > 0);
        assert!(user.update_time > 0);

        // Verify we can retrieve the user by ID
        let retrieved = service
            .get_user_by_id(&user.id)
            .await
            .expect("Failed to retrieve user");
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, user.id);
        assert_eq!(retrieved.email, user.email);

        // Verify we can retrieve the user by email
        let by_email = service
            .get_user_by_email("test@example.com")
            .await
            .expect("Failed to retrieve user by email");
        assert!(by_email.is_some());
        assert_eq!(by_email.unwrap().id, user.id);
    }

    #[tokio::test]
    async fn test_create_user_duplicate_email() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = UserService::new(db);

        // Create first user
        service
            .create_user(
                "User1".to_string(),
                "same@example.com".to_string(),
                Some("pass1".to_string()),
            )
            .await
            .expect("Failed to create first user");

        // Try to create second user with same email
        let result = service
            .create_user(
                "User2".to_string(),
                "same@example.com".to_string(),
                Some("pass2".to_string()),
            )
            .await;

        assert!(result.is_err());
        match result {
            Err(UserServiceError::UserAlreadyExists(email)) => {
                assert_eq!(email, "same@example.com");
            }
            _ => panic!("Expected UserAlreadyExists error"),
        }
    }

    #[tokio::test]
    async fn test_authenticate_user() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = UserService::new(db);

        // Create a user
        let user = service
            .create_user(
                "Auth Test".to_string(),
                "auth@example.com".to_string(),
                Some("securepassword".to_string()),
            )
            .await
            .expect("Failed to create user");

        // Test successful authentication
        let auth_result = service
            .authenticate_user("auth@example.com", "securepassword")
            .await
            .expect("Failed during authentication");
        assert!(auth_result.is_some());
        let auth_user = auth_result.unwrap();
        assert_eq!(auth_user.id, user.id);

        // Test wrong password
        let wrong_pass = service
            .authenticate_user("auth@example.com", "wrongpassword")
            .await
            .expect("Failed during authentication");
        assert!(wrong_pass.is_none());

        // Test non-existent user
        let non_existent = service
            .authenticate_user("nonexistent@example.com", "password")
            .await
            .expect("Failed during authentication");
        assert!(non_existent.is_none());
    }

    #[tokio::test]
    async fn test_update_user() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = UserService::new(db);

        // Create a user
        let user = service
            .create_user(
                "Original Name".to_string(),
                "original@example.com".to_string(),
                Some("password".to_string()),
            )
            .await
            .expect("Failed to create user");

        let original_update_time = user.update_time;

        // Update user information
        let updates = UserUpdate {
            nickname: Some("Updated Name".to_string()),
            email: Some("updated@example.com".to_string()),
            language: Some("English".to_string()),
            color_schema: Some("Dark".to_string()),
            timezone: Some("UTC".to_string()),
            ..Default::default()
        };

        let updated_user = service
            .update_user(&user.id, updates)
            .await
            .expect("Failed to update user");

        // Verify updated fields
        assert_eq!(updated_user.nickname, "Updated Name");
        assert_eq!(updated_user.email, "updated@example.com");
        assert_eq!(updated_user.language, Some("English".to_string()));
        assert_eq!(updated_user.color_schema, Some("Dark".to_string()));
        assert_eq!(updated_user.timezone, Some("UTC".to_string()));

        // Verify update_time was updated
        assert!(updated_user.update_time > original_update_time);
    }

    #[tokio::test]
    async fn test_update_user_password() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = UserService::new(db);

        // Create a user
        let user = service
            .create_user(
                "Password Test".to_string(),
                "password@example.com".to_string(),
                Some("oldpassword".to_string()),
            )
            .await
            .expect("Failed to create user");

        let old_hash = user.password.clone().unwrap();

        // Update password
        let updated_user = service
            .update_user_password(&user.id, "newpassword")
            .await
            .expect("Failed to update password");

        let new_hash = updated_user.password.unwrap();

        // Verify hash changed
        assert_ne!(old_hash, new_hash);

        // Verify new password works
        let auth_result = service
            .authenticate_user("password@example.com", "newpassword")
            .await
            .expect("Failed during authentication");
        assert!(auth_result.is_some());

        // Verify old password doesn't work
        let old_auth = service
            .authenticate_user("password@example.com", "oldpassword")
            .await
            .expect("Failed during authentication");
        assert!(old_auth.is_none());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = UserService::new(db);

        // Create a user
        let user = service
            .create_user(
                "Delete Test".to_string(),
                "delete@example.com".to_string(),
                Some("password".to_string()),
            )
            .await
            .expect("Failed to create user");

        // Delete user (soft delete)
        service
            .delete_user(&user.id)
            .await
            .expect("Failed to delete user");

        // Verify user status is "0"
        let deleted_user = service
            .get_user_by_id(&user.id)
            .await
            .expect("Failed to retrieve deleted user")
            .expect("User should still exist after soft delete");

        assert_eq!(deleted_user.status, Some("0".to_string()));
    }

    #[tokio::test]
    async fn test_is_superuser() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = UserService::new(db);

        // Create a regular user
        let regular_user = service
            .create_user(
                "Regular User".to_string(),
                "regular@example.com".to_string(),
                Some("password".to_string()),
            )
            .await
            .expect("Failed to create regular user");

        // Create a superuser
        let super_user = service
            .create_user(
                "Super User".to_string(),
                "super@example.com".to_string(),
                Some("password".to_string()),
            )
            .await
            .expect("Failed to create super user");

        // Update superuser status
        let updates = UserUpdate {
            is_superuser: Some(true),
            ..Default::default()
        };

        let super_user = service
            .update_user(&super_user.id, updates)
            .await
            .expect("Failed to update superuser status");

        // Test is_superuser method
        let regular_is_super = service
            .is_superuser(&regular_user.id)
            .await
            .expect("Failed to check superuser status");
        assert!(!regular_is_super);

        let super_is_super = service
            .is_superuser(&super_user.id)
            .await
            .expect("Failed to check superuser status");
        assert!(super_is_super);
    }

    #[tokio::test]
    async fn test_list_all_users() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = UserService::new(db);

        // Initially should be empty
        let users = service
            .list_all_users()
            .await
            .expect("Failed to list users");
        assert_eq!(users.len(), 0);

        // Create multiple users
        let user1 = service
            .create_user(
                "User A".to_string(),
                "a@example.com".to_string(),
                Some("pass1".to_string()),
            )
            .await
            .expect("Failed to create user1");

        let user2 = service
            .create_user(
                "User B".to_string(),
                "b@example.com".to_string(),
                Some("pass2".to_string()),
            )
            .await
            .expect("Failed to create user2");

        let user3 = service
            .create_user(
                "User C".to_string(),
                "c@example.com".to_string(),
                Some("pass3".to_string()),
            )
            .await
            .expect("Failed to create user3");

        // List all users - should be sorted by email
        let users = service
            .list_all_users()
            .await
            .expect("Failed to list users");
        assert_eq!(users.len(), 3);

        // Verify sorting by email
        let emails: Vec<String> = users.iter().map(|u| u.email.clone()).collect();
        assert_eq!(emails, vec!["a@example.com", "b@example.com", "c@example.com"]);

        // Verify all users are present
        let user_ids: Vec<String> = users.iter().map(|u| u.id.clone()).collect();
        assert!(user_ids.contains(&user1.id));
        assert!(user_ids.contains(&user2.id));
        assert!(user_ids.contains(&user3.id));
    }

    #[tokio::test]
    async fn test_user_validation() {
        let db = setup_test_db().await.expect("Failed to setup test database");
        let service = UserService::new(db);

        // Test with empty nickname (should fail validation)
        let result = service
            .create_user(
                "".to_string(), // Empty nickname
                "valid@example.com".to_string(),
                Some("password".to_string()),
            )
            .await;

        assert!(result.is_err());

        // Test with invalid email (should fail validation)
        let result = service
            .create_user(
                "Valid Name".to_string(),
                "invalid-email".to_string(), // Invalid email
                Some("password".to_string()),
            )
            .await;

        assert!(result.is_err());
    }
}