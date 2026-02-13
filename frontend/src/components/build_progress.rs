use leptos::*;
use crate::types::*;
use crate::services::IsoService;

#[component]
pub fn BuildProgress(
    #[prop(into)] job_id: Signal<uuid::Uuid>,
) -> impl IntoView {
    let (job, set_job) = create_signal(None::<BuildJob>);
    let (logs, set_logs) = create_signal(Vec::<BuildLog>::new());
    let (is_connected, set_is_connected) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);

    // Poll for job status
    create_effect(move |_| {
        let id = job_id.get();
        let set_job = set_job.clone();
        
        spawn_local(async move {
            match IsoService::new().get_build_job(id).await {
                Ok(j) => {
                    set_job.set(Some(j.clone()));
                    set_logs.set(j.logs);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
        });
    });

    // WebSocket connection for real-time updates
    create_effect(move |_| {
        let id = job_id.get();
        let set_job = set_job.clone();
        let set_logs = set_logs.clone();
        let set_is_connected = set_is_connected.clone();

        spawn_local(async move {
            match IsoService::new().connect_websocket(id, move |message| {
                match message.type_ {
                    MessageType::StatusUpdate => {
                        // Update job status
                    }
                    MessageType::ProgressUpdate => {
                        // Update progress
                    }
                    MessageType::LogMessage => {
                        // Add new log
                    }
                    MessageType::Completed => {
                        // Mark as completed
                    }
                    MessageType::Error => {
                        // Show error
                    }
                }
            }) {
                Ok(_ws) => {
                    set_is_connected.set(true);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_is_connected.set(false);
                }
            }
        });
    });

    view! {
        <div class="w-full">
            <Show when=move || error.get().is_some()>
                <div class="bg-red-500/20 border border-red-500/50 rounded-lg p-4 mb-6">
                    <div class="flex items-center">
                        <svg class="w-5 h-5 text-red-400 mr-2" fill="currentColor" viewBox="0 0 20 20">
                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
                        </svg>
                        <span class="text-red-300">{error.get().unwrap_or_default()}</span>
                    </div>
                </div>
            </Show>

            <Show when=move || job.get().is_some()>
                {move || {
                    job.get().map(|j| {
                        let status = j.status.clone();
                        let progress = j.progress;
                        
                        view! {
                            <div class="space-y-6">
                                // Status Header
                                <div class="flex items-center justify-between">
                                    <div>
                                        <h2 class="text-2xl font-bold text-white">"Building Your ISO"</h2>
                                        <p class="text-gray-300 mt-1">
                                            {&j.config.name}
                                        </p>
                                    </div>
                                    <div class="flex items-center space-x-2">
                                        <div class="w-3 h-3 rounded-full" 
                                             class:bg-green-500=move || matches!(status, BuildStatus::Completed)
                                             class:bg-yellow-500=move || matches!(status, BuildStatus::Building | BuildStatus::Packaging | BuildStatus::Uploading)
                                             class:bg-blue-500=move || matches!(status, BuildStatus::Queued)
                                             class:bg-red-500=move || matches!(status, BuildStatus::Failed)>
                                        </div>
                                        <span class="text-white">
                                            {match status {
                                                BuildStatus::Queued => "Queued",
                                                BuildStatus::Building => "Building",
                                                BuildStatus::Packaging => "Packaging",
                                                BuildStatus::Uploading => "Uploading",
                                                BuildStatus::Completed => "Completed",
                                                BuildStatus::Failed => "Failed",
                                            }}
                                        </span>
                                    </div>
                                </div>

                                // Progress Bar
                                <div class="space-y-2">
                                    <div class="flex justify-between text-sm">
                                        <span class="text-gray-300">"Progress"</span>
                                        <span class="text-white">{progress}%"</span>
                                    </div>
                                    <div class="w-full bg-white/10 rounded-full h-3">
                                        <div 
                                            class="bg-gradient-to-r from-purple-500 to-pink-500 h-3 rounded-full transition-all duration-500"
                                            style=move || format!("width: {}%", progress)
                                        ></div>
                                    </div>
                                </div>

                                // Current Step
                                <div class="bg-white/5 backdrop-blur-md rounded-lg p-4 border border-white/10">
                                    <h3 class="text-lg font-semibold text-white mb-3">"Current Step"</h3>
                                    <div class="space-y-2">
                                        {[
                                            ("Preparing environment", 10),
                                            ("Installing base system", 25),
                                            ("Configuring packages", 50),
                                            ("Applying customizations", 75),
                                            ("Creating ISO image", 90),
                                            ("Uploading to cloud", 100),
                                        ].iter().map(|(step, step_progress)| {
                                            let is_active = progress >= *step_progress;
                                            view! {
                                                <div class="flex items-center space-x-3">
                                                    <div class="w-4 h-4 rounded-full border-2 border-purple-500"
                                                         class:bg-purple-500=is_active>
                                                    </div>
                                                    <span class="text-gray-300" class:text-white=is_active>
                                                        {step}
                                                    </span>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                // Build Logs
                                <div class="bg-black/30 backdrop-blur-md rounded-lg p-4 border border-white/10">
                                    <div class="flex items-center justify-between mb-3">
                                        <h3 class="text-lg font-semibold text-white">"Build Logs"</h3>
                                        <div class="flex items-center space-x-2">
                                            <div class="w-2 h-2 rounded-full" 
                                                 class:bg-green-500=move || is_connected.get()
                                                 class:bg-red-500=move || !is_connected.get()>
                                            </div>
                                            <span class="text-sm text-gray-300">
                                                {move || if is_connected.get() { "Live" } else { "Disconnected" }}
                                            </span>
                                        </div>
                                    </div>
                                    <div class="space-y-1 max-h-64 overflow-y-auto font-mono text-sm">
                                        <For
                                            each=move || logs.get()
                                            key=|log| log.timestamp.to_rfc3339()
                                            children=move |log| {
                                                view! {
                                                    <div class="flex items-start space-x-2">
                                                        <span class="text-gray-500 text-xs">
                                                            {log.timestamp.format("%H:%M:%S")}
                                                        </span>
                                                        <span class=move || match log.level {
                                                            LogLevel::Info => "text-blue-400",
                                                            LogLevel::Warning => "text-yellow-400", 
                                                            LogLevel::Error => "text-red-400",
                                                            LogLevel::Debug => "text-gray-400",
                                                        }>
                                                            {log.message}
                                                        </span>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                </div>

                                // Download Section (when completed)
                                <Show when=move || matches!(status, BuildStatus::Completed)>
                                    <div class="bg-green-500/20 border border-green-500/50 rounded-lg p-6">
                                        <div class="text-center">
                                            <svg class="w-16 h-16 text-green-400 mx-auto mb-4" fill="currentColor" viewBox="0 0 20 20">
                                                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
                                            </svg>
                                            <h3 class="text-xl font-semibold text-white mb-2">"ISO Ready!"</h3>
                                            <p class="text-gray-300 mb-4">"Your custom Linux ISO is ready for download"</p>
                                            <a 
                                                href=j.download_url.unwrap_or_default()
                                                target="_blank"
                                                class="inline-flex items-center px-6 py-3 bg-gradient-to-r from-purple-500 to-pink-500 text-white font-semibold rounded-lg hover:from-purple-600 hover:to-pink-600 transition-colors"
                                            >
                                                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                                                </svg>
                                                "Download ISO"
                                            </a>
                                        </div>
                                    </div>
                                </Show>
                            </div>
                        }
                    })
                }}
            </Show>
        </div>
    }
}
