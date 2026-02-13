use crate::types::*;
use gloo_net::http::Request;
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use web_sys::{FormData, File, WebSocket};
use leptos::*;

pub struct IsoService;

impl IsoService {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_distros() -> Result<Vec<DistroTemplate>, String> {
        let response = Request::get("/api/distros")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch distros: {}", e))?;

        if response.ok() {
            let distros: Vec<DistroTemplate> = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse distros: {}", e))?;
            Ok(distros)
        } else {
            Err(format!("Failed to fetch distros: {}", response.status_text()))
        }
    }

    pub async fn create_iso(config: IsoConfig) -> Result<BuildJob, String> {
        let response = Request::post("/api/iso/create")
            .json(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Failed to create ISO: {}", e))?;

        if response.ok() {
            let job: BuildJob = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(job)
        } else {
            Err(format!("Failed to create ISO: {}", response.status_text()))
        }
    }

    pub async fn get_build_job(id: Uuid) -> Result<BuildJob, String> {
        let response = Request::get(&format!("/api/build/{}", id))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch build job: {}", e))?;

        if response.ok() {
            let job: BuildJob = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(job)
        } else {
            Err(format!("Failed to fetch build job: {}", response.status_text()))
        }
    }

    pub async fn get_gallery() -> Result<Vec<BuildJob>, String> {
        let response = Request::get("/api/gallery")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch gallery: {}", e))?;

        if response.ok() {
            let jobs: Vec<BuildJob> = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(jobs)
        } else {
            Err(format!("Failed to fetch gallery: {}", response.status_text()))
        }
    }

    pub fn connect_websocket(job_id: Uuid, callback: impl Fn(WebSocketMessage) + 'static) -> Result<WebSocket, String> {
        let ws_url = format!("wss://your-worker-domain.workers.dev/ws/{}", job_id);
        let ws = WebSocket::new(&ws_url)
            .map_err(|e| format!("Failed to create WebSocket: {}", e))?;

        let callback = Rc::new(RefCell::new(callback));
        
        // Set up event handlers
        let onmessage_callback = {
            let callback = callback.clone();
            Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
                if let Ok(text) = event.data().as_string() {
                    if let Ok(message) = serde_json::from_str::<WebSocketMessage>(&text) {
                        callback.borrow()(message);
                    }
                }
            }) as Box<dyn Fn(web_sys::MessageEvent)>)
        };

        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget(); // Prevent callback from being dropped

        Ok(ws)
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Helper function to create reactive API calls
pub fn create_async_action<T, R, E>(
    operation: impl Fn(T) -> Pin<Box<dyn Future<Output = Result<R, E>>>> + 'static,
) -> Action<T, Result<R, E>> {
    create_action(move |input: &T| {
        let input = input.clone();
        Box::pin(operation(input))
    })
}
