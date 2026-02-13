use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

mod iso_builder;
mod models;
mod websocket;

use iso_builder::IsoBuilder;
use models::*;

#[derive(Clone)]
pub struct AppState {
    jobs: Arc<RwLock<HashMap<Uuid, BuildJob>>>,
    iso_builder: IsoBuilder,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "iso_creator_backend=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting ISO Creator Backend");

    let state = AppState {
        jobs: Arc::new(RwLock::new(HashMap::new())),
        iso_builder: IsoBuilder::new(),
    };

    let app = Router::new()
        // API routes
        .route("/api/distros", get(get_distros))
        .route("/api/iso/create", post(create_iso))
        .route("/api/build/:id", get(get_build_job))
        .route("/api/gallery", get(get_gallery))
        .route("/ws/:id", get(websocket_handler))
        // Serve static files
        .nest_service("/static", ServeDir::new("static"))
        // Fallback to frontend
        .fallback_service(ServeDir::new("frontend/dist"))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("Server listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn get_distros() -> Json<Vec<DistroTemplate>> {
    let distros = vec![
        DistroTemplate {
            id: "ubuntu-22.04".to_string(),
            name: "Ubuntu 22.04 LTS".to_string(),
            description: "The world's most popular Linux distribution with long-term support".to_string(),
            icon: "🟠".to_string(),
            category: DistroCategory::Ubuntu,
            default_packages: vec![
                "ubuntu-desktop-minimal".to_string(),
                "gnome-shell".to_string(),
                "firefox".to_string(),
                "libreoffice".to_string(),
            ],
            desktop_environment: "GNOME".to_string(),
            base_image: "ubuntu:22.04".to_string(),
        },
        DistroTemplate {
            id: "debian-12".to_string(),
            name: "Debian 12".to_string(),
            description: "The universal operating system known for stability and security".to_string(),
            icon: "❤️".to_string(),
            category: DistroCategory::Debian,
            default_packages: vec![
                "gnome".to_string(),
                "firefox-esr".to_string(),
                "libreoffice".to_string(),
            ],
            desktop_environment: "GNOME".to_string(),
            base_image: "debian:12".to_string(),
        },
        DistroTemplate {
            id: "arch-linux".to_string(),
            name: "Arch Linux".to_string(),
            description: "A lightweight and flexible Linux distribution that keeps it simple".to_string(),
            icon: "🔷".to_string(),
            category: DistroCategory::Arch,
            default_packages: vec![
                "gnome".to_string(),
                "firefox".to_string(),
                "libreoffice-fresh".to_string(),
            ],
            desktop_environment: "GNOME".to_string(),
            base_image: "archlinux:latest".to_string(),
        },
        DistroTemplate {
            id: "fedora-39".to_string(),
            name: "Fedora 39".to_string(),
            description: "Leading-edge platform for developers, artists, and sysadmins".to_string(),
            icon: "🔵".to_string(),
            category: DistroCategory::Fedora,
            default_packages: vec![
                "@gnome-desktop".to_string(),
                "firefox".to_string(),
                "libreoffice".to_string(),
            ],
            desktop_environment: "GNOME".to_string(),
            base_image: "fedora:39".to_string(),
        },
    ];

    Json(distros)
}

async fn create_iso(
    State(state): State<AppState>,
    Json(config): Json<IsoConfig>,
) -> Result<Json<BuildJob>, (StatusCode, String)> {
    let job_id = Uuid::new_v4();
    let job = BuildJob {
        id: job_id,
        config: config.clone(),
        status: BuildStatus::Queued,
        progress: 0,
        started_at: chrono::Utc::now(),
        completed_at: None,
        download_url: None,
        logs: vec![BuildLog {
            timestamp: chrono::Utc::now(),
            level: LogLevel::Info,
            message: "Build job created and queued".to_string(),
        }],
    };

    // Store job
    {
        let mut jobs = state.jobs.write().await;
        jobs.insert(job_id, job.clone());
    }

    // Start build process
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = state_clone.iso_builder.build_iso(job_id, config).await {
            error!("Build failed for job {}: {}", job_id, e);
            
            // Update job status to failed
            let mut jobs = state_clone.jobs.write().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = BuildStatus::Failed;
                job.logs.push(BuildLog {
                    timestamp: chrono::Utc::now(),
                    level: LogLevel::Error,
                    message: format!("Build failed: {}", e),
                });
            }
        }
    });

    Ok(Json(job))
}

async fn get_build_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<BuildJob>, (StatusCode, String)> {
    let jobs = state.jobs.read().await;
    
    match jobs.get(&job_id) {
        Some(job) => Ok(Json(job.clone())),
        None => Err((StatusCode::NOT_FOUND, "Build job not found".to_string())),
    }
}

async fn get_gallery(State(state): State<AppState>) -> Json<Vec<BuildJob>> {
    let jobs = state.jobs.read().await;
    let completed_jobs = jobs
        .values()
        .filter(|job| matches!(job.status, BuildStatus::Completed))
        .cloned()
        .collect();
    
    Json(completed_jobs)
}

async fn websocket_handler(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    // Check if job exists
    {
        let jobs = state.jobs.read().await;
        if !jobs.contains_key(&job_id) {
            return Err(StatusCode::NOT_FOUND);
        }
    }

    // Upgrade to WebSocket
    websocket::handle_websocket(state, job_id).await
}
