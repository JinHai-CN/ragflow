/*
 * Copyright (c) 2026 Infiniflow, Inc. All rights reserved.
 *
 * PROPRIETARY AND CONFIDENTIAL
 *
 * This software is the proprietary property of Infiniflow, Inc. and is
 * protected by copyright and other intellectual property laws.
 *
 * RESTRICTIONS:
 * - You may NOT redistribute, sell, lease, or sublicense this software.
 * - You may NOT use this software to provide commercial hosting services
 *   (SaaS/PaaS) without explicit written permission.
 * - You may NOT reverse-engineer, decompile, or disassemble this software.
 * - You may NOT remove or alter this copyright notice.
 *
 * VIOLATION:
 * Any unauthorized use, reproduction, or distribution of this software
 * may result in severe civil and criminal penalties, and will be prosecuted
 * to the maximum extent possible under applicable law.
 *
 * THIS SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED.
 */

use sea_orm::{Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use std::sync::Arc;
use std::sync::OnceLock;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server host IP
    pub host: String,
    
    /// Server port
    pub port: u16,
    
    /// Debug mode
    pub debug: bool,
    
    /// Database connection string
    pub database_url: String,
    
    /// Redis connection string
    pub redis_url: String,
    
    /// Secret key for JWT tokens
    pub secret_key: String,
    
    /// Maximum content length for uploads
    pub max_content_length: usize,
}

impl Config {
    /// Create a new configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        let host = env::var("HOST_IP").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("HOST_PORT")
            .unwrap_or_else(|_| "9380".to_string())
            .parse::<u16>()
            .unwrap_or(9380);
        let debug = env::var("DEBUG").unwrap_or_else(|_| "false".to_string()) == "true";
        
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://ragflow:ragflow@localhost:3306/ragflow".to_string());
        
        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let secret_key = env::var("SECRET_KEY")
            .unwrap_or_else(|_| "your-secret-key-change-this".to_string());
        
        let max_content_length = env::var("MAX_CONTENT_LENGTH")
            .unwrap_or_else(|_| "1073741824".to_string()) // 1GB default
            .parse::<usize>()
            .unwrap_or(1073741824);
        
        Ok(Self {
            host,
            port,
            debug,
            database_url,
            redis_url,
            secret_key,
            max_content_length,
        })
    }
    
    /// Get the server address (host:port)
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
    
    /// Create a database connection pool
    pub async fn create_database_connection(&self) -> anyhow::Result<DatabaseConnection> {
        Database::connect(&self.database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))
    }
    
    /// Get a shared database connection (wrapped in Arc for easy sharing)
    pub async fn get_shared_database_connection(&self) -> anyhow::Result<Arc<DatabaseConnection>> {
        let conn = self.create_database_connection().await?;
        Ok(Arc::new(conn))
    }
}

/// Service configuration loaded from YAML file
/// 
/// This is a singleton that loads configuration from `conf/service_conf.yaml`.
/// It provides type-safe access to configuration values using dot-separated paths.
/// 
/// # Examples
/// 
/// ```no_run
/// use ragflow::ServiceConfig;
/// 
/// // Initialize the singleton (usually done once at application startup)
/// ServiceConfig::init().expect("Failed to load configuration");
/// 
/// // Get the singleton instance
/// let config = ServiceConfig::instance().expect("Configuration not initialized");
/// 
/// // Get specific configuration values
/// let host = config.get::<String>("ragflow.host").expect("ragflow.host not found");
/// let port = config.get::<u16>("ragflow.http_port").expect("ragflow.http_port not found");
/// 
/// // Get with default value if not found
/// let timeout = config.get_or("task_executor.timeout", 30);
/// 
/// // Check if a configuration exists
/// if config.has("mysql.host") {
///     println!("MySQL host is configured");
/// }
/// 
/// // Print all configuration values
/// config.print_all();
/// ```
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    value: serde_yaml::Value,
}

impl ServiceConfig {
    /// Initialize the singleton from the YAML file at `conf/service_conf.yaml`
    pub fn init() -> anyhow::Result<()> {
        let path = Path::new("conf/service_conf.yaml");
        let config = Self::from_yaml_file(path)?;
        SERVICE_CONFIG
            .set(config)
            .map_err(|_| anyhow::anyhow!("ServiceConfig already initialized"))
    }

    /// Initialize with a custom file path
    pub fn init_with_path<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
        let config = Self::from_yaml_file(path.as_ref())?;
        SERVICE_CONFIG
            .set(config)
            .map_err(|_| anyhow::anyhow!("ServiceConfig already initialized"))
    }

    /// Get the singleton instance (must be initialized first)
    /// 
    /// # Errors
    /// 
    /// Returns an error if the singleton hasn't been initialized yet.
    /// Call `init()` or `init_with_path()` before calling this method.
    pub fn instance() -> anyhow::Result<&'static Self> {
        SERVICE_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("ServiceConfig not initialized, call init() first"))
    }
    
    /// Check if the singleton has been initialized
    pub fn is_initialized() -> bool {
        SERVICE_CONFIG.get().is_some()
    }

    /// Load configuration from a YAML file
    fn from_yaml_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let value: serde_yaml::Value = serde_yaml::from_str(&content)?;
        Ok(Self { value })
    }

    /// Get a configuration value by dot-separated path (e.g., "ragflow.host")
    /// Returns `Some(T)` if the path exists and can be deserialized, `None` otherwise
    pub fn get<T>(&self, path: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut current = &self.value;
        for part in path.split('.') {
            current = current.get(part)?;
        }
        serde_yaml::from_value(current.clone()).ok()
    }

    /// Get a configuration value with default if not found
    pub fn get_or<T>(&self, path: &str, default: T) -> T
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        self.get(path).unwrap_or(default)
    }

    /// Check if a configuration path exists
    pub fn has(&self, path: &str) -> bool {
        let mut current = &self.value;
        for part in path.split('.') {
            match current.get(part) {
                Some(next) => current = next,
                None => return false,
            }
        }
        true
    }

    /// Print all configuration values in a human-readable format
    pub fn print_all(&self) {
        println!("=== Service Configuration ===");
        self.print_value(&self.value, 0);
        println!("=============================");
    }

    /// Recursively print YAML value with indentation
    fn print_value(&self, value: &serde_yaml::Value, indent: usize) {
        match value {
            serde_yaml::Value::Mapping(map) => {
                for (key, val) in map {
                    print!("{:indent$}", "", indent = indent * 2);
                    print!("{}: ", key.as_str().unwrap_or("?"));
                    match val {
                        serde_yaml::Value::Mapping(_) => {
                            println!();
                            self.print_value(val, indent + 1);
                        }
                        serde_yaml::Value::Sequence(seq) => {
                            println!();
                            for (i, item) in seq.iter().enumerate() {
                                print!("{:indent$}", "", indent = (indent + 1) * 2);
                                print!("[{}]: ", i);
                                match item {
                                    serde_yaml::Value::Mapping(_) => {
                                        println!();
                                        self.print_value(item, indent + 2);
                                    }
                                    _ => {
                                        self.print_primitive_value(item);
                                    }
                                }
                            }
                        }
                        _ => {
                            self.print_primitive_value(val);
                        }
                    }
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                for (i, item) in seq.iter().enumerate() {
                    print!("{:indent$}", "", indent = indent * 2);
                    print!("[{}]: ", i);
                    match item {
                        serde_yaml::Value::Mapping(_) => {
                            println!();
                            self.print_value(item, indent + 1);
                        }
                        _ => {
                            self.print_primitive_value(item);
                        }
                    }
                }
            }
            _ => {
                print!("{:indent$}", "", indent = indent * 2);
                self.print_primitive_value(value);
            }
        }
    }
    
    /// Print a primitive YAML value (non-mapping, non-sequence)
    fn print_primitive_value(&self, value: &serde_yaml::Value) {
        match value {
            serde_yaml::Value::String(s) => println!("{}", s),
            serde_yaml::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    println!("{}", i);
                } else if let Some(f) = n.as_f64() {
                    println!("{}", f);
                } else {
                    println!("{}", n);
                }
            }
            serde_yaml::Value::Bool(b) => println!("{}", b),
            serde_yaml::Value::Null => println!("null"),
            _ => println!("{:?}", value), // For any other types
        }
    }

    /// Get the raw YAML value for advanced operations
    pub fn raw_value(&self) -> &serde_yaml::Value {
        &self.value
    }
}

/// Global singleton instance of ServiceConfig
static SERVICE_CONFIG: OnceLock<ServiceConfig> = OnceLock::new();

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_service_config_load_from_yaml() {
        let config = ServiceConfig::from_yaml_file("conf/service_conf.yaml");
        assert!(config.is_ok(), "Failed to load YAML config: {:?}", config);
    }
    
    #[test]
    fn test_service_config_get_values() {
        let config = ServiceConfig::from_yaml_file("conf/service_conf.yaml").unwrap();
        
        // 测试获取字符串值
        let host = config.get::<String>("ragflow.host");
        assert_eq!(host, Some("0.0.0.0".to_string()));
        
        // 测试获取数值
        let port = config.get::<u16>("ragflow.http_port");
        assert_eq!(port, Some(9380));
        
        // 测试获取嵌套值
        let mysql_host = config.get::<String>("mysql.host");
        assert_eq!(mysql_host, Some("localhost".to_string()));
        
        // 测试默认值
        let nonexistent = config.get::<String>("nonexistent.key");
        assert_eq!(nonexistent, None);
        
        let with_default = config.get_or("nonexistent.key", "default".to_string());
        assert_eq!(with_default, "default");
    }
    
    #[test]
    fn test_service_config_has_method() {
        let config = ServiceConfig::from_yaml_file("conf/service_conf.yaml").unwrap();
        
        assert!(config.has("ragflow.host"));
        assert!(config.has("mysql.host"));
        assert!(!config.has("nonexistent.key"));
    }
    
    #[test]
    fn test_service_config_print_all_does_not_panic() {
        let config = ServiceConfig::from_yaml_file("conf/service_conf.yaml").unwrap();
        config.print_all(); // 不应该panic
    }
}