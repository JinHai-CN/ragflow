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

use serde::{Deserialize, Serialize};
use std::env;

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
}