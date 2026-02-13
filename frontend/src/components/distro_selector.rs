use leptos::*;
use crate::types::*;

#[component]
pub fn DistroSelector(
    #[prop(into)] on_select: Callback<DistroTemplate>,
) -> impl IntoView {
    let (selected_distro, set_selected_distro) = create_signal(None::<DistroTemplate>);
    let (distros, set_distros) = create_signal(Vec::<DistroTemplate>::new());
    let (loading, set_loading) = create_signal(true);

    // Load distros on mount
    create_effect(move |_| {
        let set_distros = set_distros.clone();
        let set_loading = set_loading.clone();
        
        spawn_local(async move {
            match crate::services::IsoService::new().get_distros().await {
                Ok(d) => {
                    set_distros.set(d);
                    set_loading.set(false);
                }
                Err(e) => {
                    log::error!("Failed to load distros: {}", e);
                    set_loading.set(false);
                }
            }
        });
    });

    let on_distro_click = move |distro: DistroTemplate| {
        set_selected_distro.set(Some(distro.clone()));
        on_select.call(distro);
    };

    view! {
        <div class="w-full">
            <h2 class="text-2xl font-bold text-white mb-6">"Choose Your Base Distribution"</h2>
            
            <Show when=move || loading.get()>
                <div class="flex justify-center py-12">
                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-500"></div>
                </div>
            </Show>

            <Show when=move || !loading.get()>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    <For
                        each=move || distros.get()
                        key=|distro| distro.id.clone()
                        children=move |distro| {
                            let is_selected = move || {
                                selected_distro.get()
                                    .as_ref()
                                    .map(|d| d.id == distro.id)
                                    .unwrap_or(false)
                            };
                            
                            view! {
                                <div
                                    class="p-6 bg-white/10 backdrop-blur-md rounded-lg border border-white/20 cursor-pointer transition-all hover:bg-white/20 hover:scale-105"
                                    class:ring-2=move || is_selected()
                                    class:ring-purple-500=move || is_selected()
                                    on:click=move |_| on_distro_click(distro.clone())
                                >
                                    <div class="flex items-center mb-4">
                                        <div class="w-12 h-12 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg mr-4 flex items-center justify-center">
                                            <span class="text-white font-bold text-lg">
                                                {&distro.icon}
                                            </span>
                                        </div>
                                        <div>
                                            <h3 class="text-lg font-semibold text-white">{&distro.name}</h3>
                                            <span class="text-sm text-gray-300">
                                                {match distro.category {
                                                    DistroCategory::Ubuntu => "Ubuntu-based",
                                                    DistroCategory::Debian => "Debian-based", 
                                                    DistroCategory::Arch => "Arch-based",
                                                    DistroCategory::Fedora => "Fedora-based",
                                                    DistroCategory::Custom => "Custom",
                                                }}
                                            </span>
                                        </div>
                                    </div>
                                    <p class="text-gray-300 text-sm mb-4">{&distro.description}</p>
                                    <div class="flex flex-wrap gap-2">
                                        <span class="px-2 py-1 bg-purple-500/20 text-purple-300 rounded text-xs">
                                            {&distro.desktop_environment}
                                        </span>
                                        <span class="px-2 py-1 bg-blue-500/20 text-blue-300 rounded text-xs">
                                            {distro.default_packages.len()} " packages"
                                        </span>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}
