use crate::models::*;
use crate::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};
use uuid::Uuid;

pub async fn handle_websocket(
    state: AppState,
    job_id: Uuid,
) -> Result<Response, axum::http::StatusCode> {
    info!("WebSocket connection requested for job {}", job_id);

    let handler = WebSocketUpgrade::from_request(())
        .protocol_callback(|protocols| {
            protocols
                .iter()
                .find(|p| p == "iso-creator")
                .map(String::from)
        })
        .on_upgrade(move |socket| handle_socket(socket, state, job_id));

    Ok(handler.into_response())
}

async fn handle_socket(socket: WebSocket, state: AppState, job_id: Uuid) {
    info!("WebSocket connection established for job {}", job_id);

    let (mut sender, mut receiver) = socket.split();
    let jobs = state.jobs.clone();

    // Send initial job status
    if let Ok(job) = jobs.read().await.get(&job_id).cloned() {
        if let Ok(message) = serde_json::to_string(&WebSocketMessage {
            job_id,
            type_: MessageType::StatusUpdate,
            data: serde_json::json!({
                "status": job.status,
                "progress": job.progress,
            }),
        }) {
            if let Err(e) = sender.send(Message::Text(message)).await {
                error!("Failed to send initial status: {}", e);
                return;
            }
        }
    }

    // Spawn a task to monitor job updates
    let job_id_clone = job_id;
    let jobs_clone = jobs.clone();
    let sender_clone = sender.clone();
    
    tokio::spawn(async move {
        let mut last_status = None;
        let mut last_progress = 0;
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            let job = {
                let jobs = jobs_clone.read().await;
                jobs.get(&job_id_clone).cloned()
            };
            
            if let Some(job) = job {
                // Check if status changed
                if last_status.as_ref() != Some(&job.status) {
                    if let Ok(message) = serde_json::to_string(&WebSocketMessage {
                        job_id: job_id_clone,
                        type_: MessageType::StatusUpdate,
                        data: serde_json::json!({
                            "status": job.status,
                        }),
                    }) {
                        if let Err(e) = sender_clone.send(Message::Text(message)).await {
                            error!("Failed to send status update: {}", e);
                            break;
                        }
                    }
                    last_status = Some(job.status.clone());
                }
                
                // Check if progress changed
                if last_progress != job.progress {
                    if let Ok(message) = serde_json::to_string(&WebSocketMessage {
                        job_id: job_id_clone,
                        type_: MessageType::ProgressUpdate,
                        data: serde_json::json!({
                            "progress": job.progress,
                        }),
                    }) {
                        if let Err(e) = sender_clone.send(Message::Text(message)).await {
                            error!("Failed to send progress update: {}", e);
                            break;
                        }
                    }
                    last_progress = job.progress;
                }
                
                // Send new log messages
                if let Some(last_log) = job.logs.last() {
                    if let Ok(message) = serde_json::to_string(&WebSocketMessage {
                        job_id: job_id_clone,
                        type_: MessageType::LogMessage,
                        data: serde_json::json!({
                            "timestamp": last_log.timestamp,
                            "level": last_log.level,
                            "message": last_log.message,
                        }),
                    }) {
                        if let Err(e) = sender_clone.send(Message::Text(message)).await {
                            error!("Failed to send log message: {}", e);
                            break;
                        }
                    }
                }
                
                // Send completion message
                if matches!(job.status, BuildStatus::Completed) {
                    if let Ok(message) = serde_json::to_string(&WebSocketMessage {
                        job_id: job_id_clone,
                        type_: MessageType::Completed,
                        data: serde_json::json!({
                            "download_url": job.download_url,
                        }),
                    }) {
                        if let Err(e) = sender_clone.send(Message::Text(message)).await {
                            error!("Failed to send completion message: {}", e);
                        }
                    }
                    break;
                }
                
                // Send error message
                if matches!(job.status, BuildStatus::Failed) {
                    if let Ok(message) = serde_json::to_string(&WebSocketMessage {
                        job_id: job_id_clone,
                        type_: MessageType::Error,
                        data: serde_json::json!({
                            "error": "Build failed",
                        }),
                    }) {
                        if let Err(e) = sender_clone.send(Message::Text(message)).await {
                            error!("Failed to send error message: {}", e);
                        }
                    }
                    break;
                }
            } else {
                // Job no longer exists
                warn!("Job {} no longer exists, closing WebSocket", job_id_clone);
                break;
            }
        }
    });

    // Handle incoming messages (ping/pong, etc.)
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Received text message: {}", text);
                // Handle client messages if needed
            }
            Ok(Message::Binary(bin)) => {
                info!("Received binary message: {} bytes", bin.len());
            }
            Ok(Message::Ping(ping)) => {
                if let Err(e) = sender.send(Message::Pong(ping)).await {
                    error!("Failed to send pong: {}", e);
                    break;
                }
            }
            Ok(Message::Pong(_)) => {
                // Handle pong response
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed by client");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    info!("WebSocket connection closed for job {}", job_id);
}
