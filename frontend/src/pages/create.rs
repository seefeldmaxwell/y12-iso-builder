use leptos::*;
use crate::types::*;
use crate::components::*;

#[component]
pub fn CreatePage() -> impl IntoView {
    let (selected_distro, set_selected_distro) = create_signal(None::<DistroTemplate>);
    let (theme, set_theme) = create_signal(ThemeConfig {
        wallpaper: None,
        gtk_theme: Some("Adwaita".to_string()),
        icon_theme: Some("Adwaita".to_string()),
        colors: ColorScheme {
            primary: "#8b5cf6".to_string(),
            secondary: "#ec4899".to_string(),
            background: "#1f2937".to_string(),
            text: "#ffffff".to_string(),
        },
    });
    let (packages, set_packages) = create_signal(Vec::<String>::new());
    let (custom_scripts, set_custom_scripts) = create_signal(Vec::<String>::new());
    let (current_step, set_current_step) = create_signal(1);
    let (is_building, set_is_building) = create_signal(false);
    let (build_job, set_build_job) = create_signal(None::<BuildJob>);

    let on_distro_select = move |distro: DistroTemplate| {
        set_selected_distro.set(Some(distro.clone()));
        set_current_step.set(2);
    };

    let on_theme_change = move |new_theme: ThemeConfig| {
        set_theme.set(new_theme);
    };

    let start_build = move || {
        if let Some(distro) = selected_distro.get() {
            set_is_building.set(true);
            
            let config = IsoConfig {
                id: uuid::Uuid::new_v4(),
                name: format!("{}-Custom", distro.name),
                distro,
                packages: packages.get(),
                custom_scripts: custom_scripts.get(),
                desktop_environment: Some("GNOME".to_string()),
                theme: theme.get(),
                created_at: chrono::Utc::now(),
            };

            spawn_local(async move {
                match crate::services::IsoService::new().create_iso(config).await {
                    Ok(job) => {
                        set_build_job.set(Some(job));
                        leptos_router::use_navigate()(&format!("/build/{}", job.id), Default::default());
                    }
                    Err(e) => {
                        log::error!("Failed to start build: {}", e);
                        set_is_building.set(false);
                    }
                }
            });
        }
    };

    view! {
        <div class="min-h-screen px-4 py-8">
            <div class="max-w-6xl mx-auto">
                <div class="mb-8">
                    <h1 class="text-4xl font-bold text-white mb-4">"Create Your Custom Linux"</h1>
                    <div class="flex items-center space-x-4">
                        {[
                            (1, "Choose Distribution", current_step.get() >= 1),
                            (2, "Customize Theme", current_step.get() >= 2),
                            (3, "Select Packages", current_step.get() >= 3),
                            (4, "Build ISO", current_step.get() >= 4),
                        ].iter().map(|(step, title, is_active)| {
                            view! {
                                <div class="flex items-center">
                                    <div class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold"
                                         class:bg-purple-500=is_active
                                         class:bg-white/20=!is_active
                                         class:text-white=is_active
                                         class:text-gray-400=!is_active>
                                        {step}
                                    </div>
                                    <span class="ml-2 text-sm font-medium"
                                          class:text-white=is_active
                                          class:text-gray-400=!is_active>
                                        {title}
                                    </span>
                                    {if *step < 4 {
                                        view! {
                                            <div class="w-8 h-0.5 bg-white/20 mx-4"></div>
                                        }
                                    } else {
                                        view! { <div></div> }
                                    }}
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // Step 1: Distribution Selection
                <Show when=move || current_step.get() == 1>
                    <DistroSelector on_select=on_distro_select/>
                </Show>

                // Step 2: Theme Customization
                <Show when=move || current_step.get() == 2>
                    <div class="space-y-6">
                        <ThemeCustomizer theme=theme on_change=on_theme_change/>
                        
                        <div class="flex justify-between">
                            <button
                                class="px-6 py-3 bg-white/10 backdrop-blur-md text-white font-semibold rounded-lg border border-white/20 hover:bg-white/20 transition-colors"
                                on:click=move |_| set_current_step.set(1)
                            >
                                "Back"
                            </button>
                            <button
                                class="px-6 py-3 bg-gradient-to-r from-purple-500 to-pink-500 text-white font-semibold rounded-lg hover:from-purple-600 hover:to-pink-600 transition-colors"
                                on:click=move |_| set_current_step.set(3)
                            >
                                "Next"
                            </button>
                        </div>
                    </div>
                </Show>

                // Step 3: Package Selection
                <Show when=move || current_step.get() == 3>
                    <div class="space-y-6">
                        <PackageSelector packages=packages set_packages=set_packages/>
                        
                        <div class="flex justify-between">
                            <button
                                class="px-6 py-3 bg-white/10 backdrop-blur-md text-white font-semibold rounded-lg border border-white/20 hover:bg-white/20 transition-colors"
                                on:click=move |_| set_current_step.set(2)
                            >
                                "Back"
                            </button>
                            <button
                                class="px-6 py-3 bg-gradient-to-r from-purple-500 to-pink-500 text-white font-semibold rounded-lg hover:from-purple-600 hover:to-pink-600 transition-colors"
                                on:click=move |_| set_current_step.set(4)
                            >
                                "Next"
                            </button>
                        </div>
                    </div>
                </Show>

                // Step 4: Build Configuration
                <Show when=move || current_step.get() == 4>
                    <div class="space-y-6">
                        <BuildSummary 
                            distro=selected_distro
                            theme=theme
                            packages=packages
                        />
                        
                        <div class="flex justify-between">
                            <button
                                class="px-6 py-3 bg-white/10 backdrop-blur-md text-white font-semibold rounded-lg border border-white/20 hover:bg-white/20 transition-colors"
                                on:click=move |_| set_current_step.set(3)
                            >
                                "Back"
                            </button>
                            <button
                                class="px-8 py-4 bg-gradient-to-r from-purple-500 to-pink-500 text-white font-semibold rounded-lg hover:from-purple-600 hover:to-pink-600 transition-colors disabled:opacity-50"
                                disabled=is_building
                                on:click=start_build
                            >
                                {move || if is_building.get() { 
                                    view! { <span class="flex items-center"><div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div> "Building..."</span> }
                                } else { 
                                    view! { "Build ISO" }
                                }}
                            </button>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn PackageSelector(
    packages: Signal<Vec<String>>,
    set_packages: WriteSignal<Vec<String>>,
) -> impl IntoView {
    let (search_term, set_search_term) = create_signal(String::new());
    let (selected_category, set_selected_category) = create_signal("all".to_string());

    let available_packages = vec![
        ("Development", vec!["git", "vim", "vscode", "docker", "nodejs", "python", "rust", "go"]),
        ("Multimedia", vec!["vlc", "gimp", "inkscape", "audacity", "kdenlive", "obs-studio"]),
        ("Office", vec!["libreoffice", "thunderbird", "evince", "filezilla"]),
        ("Internet", vec!["firefox", "chromium", "telegram-desktop", "discord", "slack"]),
        ("System", vec!["htop", "neofetch", "tree", "wget", "curl", "unzip"]),
    ];

    let filtered_packages = move || {
        let search = search_term.get().to_lowercase();
        let category = selected_category.get();
        
        available_packages.iter()
            .filter(|(cat, _)| category == "all" || cat == &category)
            .flat_map(|(_, pkgs)| pkgs.iter())
            .filter(|pkg| pkg.contains(&search))
            .cloned()
            .collect::<Vec<_>>()
    };

    let toggle_package = move |pkg: String| {
        let current = packages.get();
        if current.contains(&pkg) {
            set_packages.set(current.into_iter().filter(|p| p != &pkg).collect());
        } else {
            set_packages.set([current, vec![pkg]].concat());
        }
    };

    view! {
        <div class="w-full">
            <h2 class="text-2xl font-bold text-white mb-6">"Select Packages"</h2>
            
            // Search and Filter
            <div class="flex flex-col md:flex-row gap-4 mb-6">
                <div class="flex-1">
                    <input
                        type="text"
                        placeholder="Search packages..."
                        class="w-full px-4 py-2 bg-white/10 border border-white/20 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-purple-500"
                        prop:value=search_term
                        on:input=move |e| set_search_term.set(event_target_value(&e))
                    />
                </div>
                <select
                    class="px-4 py-2 bg-white/10 border border-white/20 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                    on:change=move |e| set_selected_category.set(event_target_value(&e))
                >
                    <option value="all">"All Categories"</option>
                    {available_packages.iter().map(|(cat, _)| {
                        view! {
                            <option value=*cat selected=move || selected_category.get() == *cat>
                                {cat}
                            </option>
                        }
                    }).collect_view()}
                </select>
            </div>

            // Package Grid
            <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3 mb-6">
                <For
                    each=filtered_packages
                    key=|pkg| pkg.clone()
                    children=move |pkg| {
                        let is_selected = move || packages.get().contains(&pkg);
                        view! {
                            <div
                                class="p-3 bg-white/10 backdrop-blur-md rounded-lg border border-white/20 cursor-pointer transition-all hover:bg-white/20"
                                class:ring-2=is_selected
                                class:ring-purple-500=is_selected
                                on:click=move |_| toggle_package(pkg.clone())
                            >
                                <div class="flex items-center justify-between">
                                    <span class="text-white text-sm">{pkg}</span>
                                    <div class="w-4 h-4 rounded border-2 border-purple-500"
                                         class:bg-purple-500=is_selected>
                                    </div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>

            // Selected Packages Summary
            <div class="bg-white/5 backdrop-blur-md rounded-lg p-4 border border-white/10">
                <h3 class="text-lg font-semibold text-white mb-2">
                    "Selected Packages (" {packages.get().len()} ")"
                </h3>
                <div class="flex flex-wrap gap-2">
                    <For
                        each=move || packages.get()
                        key=|pkg| pkg.clone()
                        children=move |pkg| {
                            view! {
                                <span class="px-3 py-1 bg-purple-500/20 text-purple-300 rounded-full text-sm flex items-center">
                                    {pkg}
                                    <button
                                        class="ml-2 text-purple-400 hover:text-purple-200"
                                        on:click=move |_| toggle_package(pkg.clone())
                                    >
                                        "×"
                                    </button>
                                </span>
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn BuildSummary(
    distro: Signal<Option<DistroTemplate>>,
    theme: Signal<ThemeConfig>,
    packages: Signal<Vec<String>>,
) -> impl IntoView {
    view! {
        <div class="w-full">
            <h2 class="text-2xl font-bold text-white mb-6">"Build Summary"</h2>
            
            <div class="grid md:grid-cols-2 gap-6">
                // Configuration Summary
                <div class="bg-white/5 backdrop-blur-md rounded-lg p-6 border border-white/10">
                    <h3 class="text-lg font-semibold text-white mb-4">"Configuration"</h3>
                    <div class="space-y-3">
                        <div class="flex justify-between">
                            <span class="text-gray-300">"Base Distribution:"</span>
                            <span class="text-white">
                                {move || distro.get().map(|d| d.name).unwrap_or_default()}
                            </span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-gray-300">"Desktop Environment:"</span>
                            <span class="text-white">"GNOME"</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-gray-300">"GTK Theme:"</span>
                            <span class="text-white">
                                {move || theme.get().gtk_theme.unwrap_or_default()}
                            </span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-gray-300">"Icon Theme:"</span>
                            <span class="text-white">
                                {move || theme.get().icon_theme.unwrap_or_default()}
                            </span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-gray-300">"Packages:"</span>
                            <span class="text-white">{packages.get().len()}</span>
                        </div>
                    </div>
                </div>

                // Color Scheme Preview
                <div class="bg-white/5 backdrop-blur-md rounded-lg p-6 border border-white/10">
                    <h3 class="text-lg font-semibold text-white mb-4">"Color Scheme"</h3>
                    <div class="grid grid-cols-2 gap-4">
                        <div class="space-y-2">
                            <div class="flex items-center space-x-2">
                                <div 
                                    class="w-6 h-6 rounded"
                                    style=move || format!("background-color: {};", theme.get().colors.primary)
                                ></div>
                                <span class="text-gray-300 text-sm">"Primary"</span>
                            </div>
                            <div class="flex items-center space-x-2">
                                <div 
                                    class="w-6 h-6 rounded"
                                    style=move || format!("background-color: {};", theme.get().colors.secondary)
                                ></div>
                                <span class="text-gray-300 text-sm">"Secondary"</span>
                            </div>
                        </div>
                        <div class="space-y-2">
                            <div class="flex items-center space-x-2">
                                <div 
                                    class="w-6 h-6 rounded"
                                    style=move || format!("background-color: {};", theme.get().colors.background)
                                ></div>
                                <span class="text-gray-300 text-sm">"Background"</span>
                            </div>
                            <div class="flex items-center space-x-2">
                                <div 
                                    class="w-6 h-6 rounded"
                                    style=move || format!("background-color: {};", theme.get().colors.text)
                                ></div>
                                <span class="text-gray-300 text-sm">"Text"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            // Package List
            <div class="mt-6 bg-white/5 backdrop-blur-md rounded-lg p-6 border border-white/10">
                <h3 class="text-lg font-semibold text-white mb-4">"Selected Packages"</h3>
                <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
                    <For
                        each=move || packages.get()
                        key=|pkg| pkg.clone()
                        children=move |pkg| {
                            view! {
                                <span class="px-3 py-1 bg-purple-500/20 text-purple-300 rounded-full text-sm">
                                    {pkg}
                                </span>
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
