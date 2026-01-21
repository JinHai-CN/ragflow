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

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;

/// Root endpoint
pub async fn root() -> impl IntoResponse {
    "RAGFlow API Server"
}

/// API documentation endpoint (placeholder for Swagger)
pub async fn api_docs() -> impl IntoResponse {
    Json(json!({
        "code": 0,
        "message": "success",
        "data": {
            "swagger": "2.0",
            "info": {
                "title": "RAGFlow API",
                "description": "API documentation will be available soon",
                "version": "1.0.0"
            }
        }
    }))
}

/// Handle 404 - Not Found
pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Json(json!({
        "code": 404,
        "message": "Not Found",
        "data": null
    })))
}