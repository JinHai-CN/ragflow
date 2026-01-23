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
 * the maximum extent possible under applicable law.
 *
 * THIS SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED.
 */

use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use actix_web::middleware::Compress;
use clap::Parser;
use log::{info, warn};
use ragflow::config::Config;
use ragflow::server::AppState;

// Import API handlers from modules
use ragflow::api::health::health_check;
use ragflow::api::system::ping;
use ragflow::api::misc::{root, api_docs};
use ragflow::api::version::get_version;
use ragflow::api::knowledge_bases::list_knowledge_bases;
use ragflow::api::chat::chat_completions;
use ragflow::api::documents::upload_document;
use ragflow::api::user::{
    login, register, logout, get_profile, update_settings,
    get_login_channels, oauth_login, oauth_callback
};

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

// Display RAGFlow logo and version
fn display_banner() {
    info!(
        r"
        ____   ___    ______ ______ __
       / __ \ /   |  / ____// ____// /____  _      __
      / /_/ // /| | / / __ / /_   / // __ \| | /| / /
     / _, _// ___ |/ /_/ // __/  / // /_/ /| |/ |/ /
    /_/ |_|/_/  |_|\____//_/    /_/ \____/ |__/|__/
    "
    );
    info!("RAGFlow version: {}", env!("CARGO_PKG_VERSION"));
    info!("Starting RAGFlow API Server (Rust implementation with Actix-web)");
}

// Background task for update progress (simplified version)
async fn update_progress_task(stop_signal: std::sync::Arc<std::sync::atomic::AtomicBool>) {
    info!("Starting update_progress background task");

    while !stop_signal.load(std::sync::atomic::Ordering::Relaxed) {
        // TODO: Implement actual progress update logic
        // This is a placeholder that simulates the Python update_progress function
        info!("Update progress task running...");

        // Wait for 6 seconds as in Python code
        tokio::time::sleep(std::time::Duration::from_secs(6)).await;
    }

    info!("Update progress task stopped");
}

// Initialize application with given configuration
async fn init_app_with_config(debug: bool, config: Config) -> anyhow::Result<AppState> {
    // Initialize logging
    if debug {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    config.print_all();

    // Display banner
    display_banner();

    // Show configuration
    info!("Debug mode: {}", debug);
    // Get host and port from configuration for display
    let host = config.services
        .get::<String>("ragflow.host")
        .unwrap_or("0.0.0.0".to_string());
    let port = config.services
        .get::<u16>("ragflow.http_port")
        .unwrap_or(9380);
    info!("Host IP: {} (from config file)", host);
    info!("Port: {} (from config file)", port);

    // Create database connection
    let db = config.create_database_connection().await?;
    info!("Database connection established");

    // Create user service
    let user_service = ragflow::models::services::user::UserService::new(db.clone());

    // Initialize application state
    let state = AppState {
        debug_mode: debug,
        server_start_time: std::time::Instant::now(),
        config,
        db,
        user_service,
    };

    // TODO: Initialize Redis connection
    // TODO: Load plugins

    Ok(state)
}

// Initialize application (for backward compatibility)
async fn init_app(debug: bool) -> anyhow::Result<AppState> {
    let config = Config::from_env()?;
    init_app_with_config(debug, config).await
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args = Cli::parse();

    // Handle --version flag
    if args.version {
        println!("RAGFlow version: {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Load configuration from environment
    let config = Config::from_env()?;
    
    // Determine host and port from configuration file, fallback to command line arguments
    let host = config.services
        .get::<String>("ragflow.host")
        .unwrap_or(args.host);
    let port = config.services
        .get::<u16>("ragflow.http_port")
        .unwrap_or(args.port);

    // Initialize application with the loaded configuration
    let app_state = init_app_with_config(args.debug, config).await?;

    // Handle --init-superuser flag
    if args.init_superuser {
        warn!("--init-superuser flag is not implemented in Rust version yet");
        // TODO: Implement superuser initialization
    }

    // Create stop signal for background tasks
    let stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_signal_for_task = stop_signal.clone();

    // Start background task for update progress
    // let background_task = tokio::spawn(async move {
    //     update_progress_task(stop_signal_for_task).await;
    // });

    // Configure HTTP server using configuration values
    info!(
        "Starting HTTP server on {}:{} (from config file)",
        host, port
    );

    let addr = format!("{}:{}", host, port);

    info!("Server starting on http://{}", addr);

    // Create and run Actix-web server
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(Compress::default())
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .service(root)
            .route("/health", web::get().to(health_check))
            .route("/api/v1/health", web::get().to(health_check))
            .service(get_version)
            .service(api_docs)
            .service(ping)
            .service(login)
            .service(register)
            .service(logout)
            .service(get_profile)
            .service(update_settings)
            .service(get_login_channels)
            .service(oauth_login)
            .service(oauth_callback)
            .service(list_knowledge_bases)
            .service(chat_completions)
            .service(upload_document)
            .default_service(web::route().to(ragflow::api::misc::not_found))
    })
    .bind(&addr)?
    .worker_max_blocking_threads(1024)
    .run();

    // Handle graceful shutdown
    let server_handle = server.handle();
    let stop_signal_clone = stop_signal.clone();
    // let background_task_handle = background_task;

    tokio::select! {
        _ = server => {
            info!("Server stopped");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down gracefully...");
            stop_signal_clone.store(true, std::sync::atomic::Ordering::Relaxed);
            server_handle.stop(true).await;
            // let _ = background_task_handle.await;
            info!("Shutdown complete");
        }
    }

    Ok(())
}