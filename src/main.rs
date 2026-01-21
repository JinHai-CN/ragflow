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
use clap::Parser;
use log::{info, warn};
use serde::Serialize;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::oneshot;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use ragflow::config::Config;
use ragflow::routes;

// Command line arguments
#[derive(Parser, Debug)]
#[command(name = "ragflow-server")]
#[command(about = "RAGFlow API Server")]
struct Cli {
    /// Print version and exit
    #[arg(long)]
    version: bool,

    /// Enable debug mode
    #[arg(long)]
    debug: bool,

    /// Initialize superuser (not implemented in Rust yet)
    #[arg(long)]
    init_superuser: bool,

    /// Host IP to bind
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Port to listen on
    #[arg(long, default_value = "9380")]
    port: u16,
}

// Application state
#[derive(Clone)]
#[allow(dead_code)]
struct AppState {
    debug_mode: bool,
    server_start_time: std::time::Instant,
    config: Config,
}

// API Response structures (kept for compatibility, though routes now use serde_json directly)
#[derive(Serialize)]
struct ApiResponse<T> {
    code: u32,
    message: String,
    data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    fn error(message: &str, code: u32) -> Self {
        Self {
            code,
            message: message.to_string(),
            data: None,
        }
    }
}

// Display RAGFlow logo and version
fn display_banner() {
    info!(r"
        ____   ___    ______ ______ __
       / __ \ /   |  / ____// ____// /____  _      __
      / /_/ // /| | / / __ / /_   / // __ \| | /| / /
     / _, _// ___ |/ /_/ // __/  / // /_/ /| |/ |/ /
    /_/ |_|/_/  |_|\____//_/    /_/ \____/ |__/|__/
    ");
    info!("RAGFlow version: {}", env!("CARGO_PKG_VERSION"));
    info!("Starting RAGFlow API Server (Rust implementation with Axum)");
}

// Background task for update progress (simplified version)
async fn update_progress_task(stop_signal: Arc<AtomicBool>) {
    info!("Starting update_progress background task");
    
    while !stop_signal.load(Ordering::Relaxed) {
        // TODO: Implement actual progress update logic
        // This is a placeholder that simulates the Python update_progress function
        info!("Update progress task running...");
        
        // Wait for 6 seconds as in Python code
        tokio::time::sleep(Duration::from_secs(6)).await;
    }
    
    info!("Update progress task stopped");
}

// Initialize application
async fn init_app(debug: bool) -> anyhow::Result<AppState> {
    // Initialize logging
    if debug {
        env::set_var("RUST_LOG", "debug");
    } else {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    
    // Display banner
    display_banner();
    
    // Show configuration
    info!("Debug mode: {}", debug);
    info!("Host IP: 0.0.0.0 (configurable via --host)");
    info!("Port: 9380 (configurable via --port)");
    
    // Load configuration from environment
    let config = Config::from_env()?;
    
    // Initialize application state
    let state = AppState {
        debug_mode: debug,
        server_start_time: std::time::Instant::now(),
        config,
    };
    
    // TODO: Initialize database connection
    // TODO: Initialize Redis connection
    // TODO: Load plugins
    
    Ok(state)
}

// Create the Axum router
fn create_router(state: AppState) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);
    
    // Build the router
    Router::new()
        .route("/", get(routes::root))
        .route("/health", get(routes::health_check))
        .route("/version", get(routes::get_version))
        .route("/apidocs", get(routes::api_docs))
        .route("/api/v1/knowledge-bases", get(routes::list_knowledge_bases))
        .route("/api/v1/chat/completions", post(routes::chat_completions))
        .route("/api/v1/documents", post(routes::upload_document))
        // Fallback for 404
        .fallback(routes::not_found)
        // Add middleware layers
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CatchPanicLayer::new())
        .layer(cors)
        // Add application state
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args = Cli::parse();
    
    // Handle --version flag
    if args.version {
        println!("RAGFlow version: {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    
    // Initialize application
    let app_state = init_app(args.debug).await?;
    
    // Handle --init-superuser flag
    if args.init_superuser {
        warn!("--init-superuser flag is not implemented in Rust version yet");
        // TODO: Implement superuser initialization
    }
    
    // Create stop signal for background tasks
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_for_task = stop_signal.clone();
    
    // Start background task for update progress
    let background_task = tokio::spawn(async move {
        update_progress_task(stop_signal_for_task).await;
    });
    
    // Configure HTTP server
    info!("Starting HTTP server on {}:{}", args.host, args.port);
    
    // Create router
    let router = create_router(app_state);
    
    // Bind and serve
    let addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on http://{}", addr);
    
    // Create shutdown signal channel
    let (tx, rx) = oneshot::channel();
    
    // Spawn task to handle shutdown signals
    let graceful_shutdown = async move {
        let ctrl_c = async {
            signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
        };
        
        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };
        
        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();
        
        tokio::select! {
            _ = ctrl_c => {
                info!("Received Ctrl+C signal, shutting down...");
            },
            _ = terminate => {
                info!("Received SIGTERM signal, shutting down...");
            },
        }
        
        // Send shutdown signal
        let _ = tx.send(());
    };
    
    // Spawn the graceful shutdown handler
    tokio::spawn(graceful_shutdown);
    
    // Serve with graceful shutdown
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            rx.await.ok();
            info!("Initiating graceful shutdown...");
            // Stop background tasks
            stop_signal.store(true, Ordering::Relaxed);
            // Wait for background task to finish
            let _ = background_task.await;
            info!("Server shutdown complete");
        })
        .await?;
    
    Ok(())
}