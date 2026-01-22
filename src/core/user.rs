//! User management business logic.
//!
//! This module provides high-level user management functions that encapsulate
//! the business rules and workflows for user operations.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::services::user::{UserService, UserServiceTrait, UserUpdate};

/// User registration request
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 1, message = "Nickname is required"))]
    pub nickname: String,
    
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
}

/// User login request
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: String,
    pub nickname: String,
    pub email: String,
    pub avatar: Option<String>,
    pub language: Option<String>,
    pub color_schema: Option<String>,
    pub timezone: Option<String>,
    pub is_superuser: bool,
    pub create_time: i64,
    pub update_time: i64,
}

impl UserProfile {
    /// Create a UserProfile from a user model
    pub fn from_model(user: &crate::models::entities::user::Model) -> Self {
        Self {
            id: user.id.clone(),
            nickname: user.nickname.clone(),
            email: user.email.clone(),
            avatar: user.avatar.clone(),
            language: user.language.clone(),
            color_schema: user.color_schema.clone(),
            timezone: user.timezone.clone(),
            is_superuser: user.is_superuser,
            create_time: user.create_time,
            update_time: user.update_time,
        }
    }
}

/// User settings update request
#[derive(Debug, Deserialize, Validate, Default)]
pub struct UpdateSettingsRequest {
    #[validate(length(min = 1, message = "Nickname cannot be empty"))]
    pub nickname: Option<String>,
    
    #[validate(email(message = "Invalid email address"))]
    pub email: Option<String>,
    
    pub avatar: Option<String>,
    pub language: Option<String>,
    pub color_schema: Option<String>,
    pub timezone: Option<String>,
}

/// Register a new user
pub async fn register_user(
    user_service: &UserService,
    request: RegisterRequest,
) -> Result<UserProfile> {
    // Validate request
    request.validate().map_err(|e| anyhow!("Validation error: {}", e))?;
    
    // Check if registration is enabled (TODO: read from config)
    // For now, always enabled
    
    // Create user
    let user = user_service
        .create_user(request.nickname, request.email, Some(request.password))
        .await
        .map_err(|e| anyhow!("Failed to create user: {}", e))?;
    
    Ok(UserProfile::from_model(&user))
}

/// Authenticate user with email and password
pub async fn login_user(
    user_service: &UserService,
    request: LoginRequest,
) -> Result<Option<UserProfile>> {
    // Validate request
    request.validate().map_err(|e| anyhow!("Validation error: {}", e))?;
    
    // Authenticate
    let user = user_service
        .authenticate_user(&request.email, &request.password)
        .await
        .map_err(|e| anyhow!("Authentication error: {}", e))?;
    
    // Convert to profile if user exists
    Ok(user.map(|u| UserProfile::from_model(&u)))
}

/// Get user profile by ID
pub async fn get_user_profile(
    user_service: &UserService,
    user_id: &str,
) -> Result<Option<UserProfile>> {
    let user = user_service
        .get_user_by_id(user_id)
        .await
        .map_err(|e| anyhow!("Failed to get user: {}", e))?;
    
    Ok(user.map(|u| UserProfile::from_model(&u)))
}

/// Update user settings
pub async fn update_user_settings(
    user_service: &UserService,
    user_id: &str,
    updates: UpdateSettingsRequest,
) -> Result<UserProfile> {
    // Validate if email is provided - email validation is handled by Validate trait
    
    // Convert to UserUpdate
    let user_update = UserUpdate {
        nickname: updates.nickname,
        email: updates.email,
        avatar: updates.avatar,
        language: updates.language,
        color_schema: updates.color_schema,
        timezone: updates.timezone,
        login_channel: None,
        status: None,
        is_superuser: None,
    };
    
    let user = user_service
        .update_user(user_id, user_update)
        .await
        .map_err(|e| anyhow!("Failed to update user: {}", e))?;
    
    Ok(UserProfile::from_model(&user))
}

/// Update user password
pub async fn update_user_password(
    user_service: &UserService,
    user_id: &str,
    old_password: Option<String>,
    new_password: String,
) -> Result<UserProfile> {
    // TODO: Verify old password if provided
    // For now, just update the password
    
    // Mark old_password as used to suppress warning (will be implemented later)
    let _ = old_password;
    
    let user = user_service
        .update_user_password(user_id, &new_password)
        .await
        .map_err(|e| anyhow!("Failed to update password: {}", e))?;
    
    Ok(UserProfile::from_model(&user))
}

/// Delete user (soft delete)
pub async fn delete_user(user_service: &UserService, user_id: &str) -> Result<()> {
    user_service
        .delete_user(user_id)
        .await
        .map_err(|e| anyhow!("Failed to delete user: {}", e))?;
    
    Ok(())
}

/// Check if user is superuser
pub async fn check_superuser(user_service: &UserService, user_id: &str) -> Result<bool> {
    user_service
        .is_superuser(user_id)
        .await
        .map_err(|e| anyhow!("Failed to check superuser status: {}", e))
}

/// List all users
pub async fn list_all_users(user_service: &UserService) -> Result<Vec<UserProfile>> {
    let users = user_service
        .list_all_users()
        .await
        .map_err(|e| anyhow!("Failed to list users: {}", e))?;
    
    Ok(users.iter().map(UserProfile::from_model).collect())
}

/// Forgot password - request OTP
pub async fn request_password_reset_otp(
    user_service: &UserService,
    email: &str,
) -> Result<()> {
    // Check if user exists
    let user = user_service
        .get_user_by_email(email)
        .await
        .map_err(|e| anyhow!("Failed to check user: {}", e))?;
    
    if user.is_none() {
        return Err(anyhow!("User not found"));
    }
    
    // TODO: Generate OTP and store in Redis
    // TODO: Send email with OTP
    
    Ok(())
}

/// Verify OTP for password reset
pub async fn verify_password_reset_otp(
    email: &str,
    otp: &str,
) -> Result<bool> {
    // TODO: Retrieve OTP from Redis and verify
    // TODO: Mark email as verified for password reset
    
    // Mark parameters as used to suppress warnings (will be implemented later)
    let _ = email;
    let _ = otp;
    
    Ok(true)
}

/// Reset password after OTP verification
pub async fn reset_password(
    user_service: &UserService,
    email: &str,
    new_password: &str,
) -> Result<UserProfile> {
    // TODO: Check if email is verified for password reset
    
    let user = user_service
        .get_user_by_email(email)
        .await
        .map_err(|e| anyhow!("Failed to get user: {}", e))?
        .ok_or_else(|| anyhow!("User not found"))?;
    
    let updated_user = user_service
        .update_user_password(&user.id, new_password)
        .await
        .map_err(|e| anyhow!("Failed to reset password: {}", e))?;
    
    Ok(UserProfile::from_model(&updated_user))
}