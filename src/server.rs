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
    http::Method,
    routing::{get, post},
    Router,
};
use log::info;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::oneshot;

use crate::api::routes;
use crate::config::Config;
use sea_orm::DatabaseConnection;

// Application state for Axum
#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub debug_mode: bool,
    pub server_start_time: std::time::Instant,
    pub config: Config,
    // pub db: DatabaseConnection,
}


// pub struct Server {
//     config: Config,
// }
//
// impl Server {
//     /// Create a new server with the given configuration
//     pub fn new(config: Config) -> Self {
//         Self { config }
//     }
//
//     /// Start the HTTP server
//     // pub async fn run(&self) -> anyhow::Result<()> {
//     //     info!("Starting RAGFlow server on {}", self.config.server_addr());
//     //
//     //     // Initialize application state
//     //     // let db = self.config.create_database_connection().await?;
//     //     let app_state = AppState {
//     //         debug_mode: false, // will be set from config
//     //         server_start_time: std::time::Instant::now(),
//     //         config: self.config.clone(),
//     //         // db,
//     //     };
//     //
//     //     // Create router
//     //     let router = self.create_router(app_state);
//     //
//     //     // Bind and serve
//     //     let addr = self.config.server_addr();
//     //     let listener = tokio::net::TcpListener::bind(&addr).await?;
//     //     info!("Server listening on http://{}", addr);
//     //
//     //     // Create shutdown signal channel
//     //     let (tx, rx) = oneshot::channel();
//     //
//     //     // Create stop signal for background tasks (if any)
//     //     let stop_signal = Arc::new(AtomicBool::new(false));
//     //
//     //     // Spawn task to handle shutdown signals
//     //     let graceful_shutdown = async move {
//     //         let ctrl_c = async {
//     //             signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
//     //         };
//     //
//     //         #[cfg(unix)]
//     //         let terminate = async {
//     //             signal::unix::signal(signal::unix::SignalKind::terminate())
//     //                 .expect("failed to install signal handler")
//     //                 .recv()
//     //                 .await;
//     //         };
//     //
//     //         #[cfg(not(unix))]
//     //         let terminate = std::future::pending::<()>();
//     //
//     //         tokio::select! {
//     //             _ = ctrl_c => {
//     //                 info!("Received Ctrl+C signal, shutting down...");
//     //             },
//     //             _ = terminate => {
//     //                 info!("Received SIGTERM signal, shutting down...");
//     //             },
//     //         }
//     //
//     //         // Send shutdown signal
//     //         let _ = tx.send(());
//     //     };
//     //
//     //     // Spawn the graceful shutdown handler
//     //     tokio::spawn(graceful_shutdown);
//     //
//     //     // Serve with graceful shutdown
//     //     axum::serve(listener, router)
//     //         .with_graceful_shutdown(async move {
//     //             rx.await.ok();
//     //             info!("Initiating graceful shutdown...");
//     //             // Stop background tasks if any
//     //             stop_signal.store(true, Ordering::Relaxed);
//     //             info!("Server shutdown complete");
//     //         })
//     //         .await?;
//     //
//     //     Ok(())
//     // }
//
//     /// Get server configuration
//     pub fn config(&self) -> &Config {
//         &self.config
//     }
//
//     // /// Create Axum router
//     // fn create_router(&self, state: AppState) -> Router {
//     //     // CORS configuration
//     //     let cors = CorsLayer::new()
//     //         .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
//     //         .allow_headers(Any)
//     //         .allow_origin(Any);
//     //
//     //     // Build the router
//     //     Router::new()
//     //         .route("/", get(routes::root))
//     //         .route("/health", get(routes::health_check))
//     //         .route("/version", get(routes::get_version))
//     //         .route("/apidocs", get(routes::api_docs))
//     //         .route("/api/v1/knowledge-bases", get(routes::list_knowledge_bases))
//     //         .route("/api/v1/chat/completions", post(routes::chat_completions))
//     //         .route("/api/v1/documents", post(routes::upload_document))
//     //         // Fallback for 404
//     //         .fallback(routes::not_found)
//     //         // Add middleware layers
//     //         .layer(TraceLayer::new_for_http())
//     //         .layer(CompressionLayer::new())
//     //         .layer(CatchPanicLayer::new())
//     //         .layer(cors)
//     //         // Add application state
//     //         .with_state(state)
//     // }
// }