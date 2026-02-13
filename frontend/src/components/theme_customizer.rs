use leptos::*;
use crate::types::*;

#[component]
pub fn ThemeCustomizer(
    #[prop(into)] theme: Signal<ThemeConfig>,
    #[prop(into)] on_change: Callback<ThemeConfig>,
) -> impl IntoView {
    let (wallpaper_url, set_wallpaper_url) = create_signal(theme.get().wallpaper.unwrap_or_default());
    let (gtk_theme, set_gtk_theme) = create_signal(theme.get().gtk_theme.unwrap_or_else(|| "Adwaita".to_string()));
    let (icon_theme, set_icon_theme) = create_signal(theme.get().icon_theme.unwrap_or_else(|| "Adwaita".to_string()));
    let (primary_color, set_primary_color) = create_signal(theme.get().colors.primary.clone());
    let (secondary_color, set_secondary_color) = create_signal(theme.get().colors.secondary.clone());
    let (background_color, set_background_color) = create_signal(theme.get().colors.background.clone());
    let (text_color, set_text_color) = create_signal(theme.get().colors.text.clone());

    let gtk_themes = vec![
        "Adwaita", "Yaru", "Materia", "Arc", "Flat-Remix", "Pop", "Orchis"
    ];

    let icon_themes = vec![
        "Adwaita", "Papirus", "Numix", "Moka", "Faenza", "La Capitaine"
    ];

    let update_theme = move || {
        let new_theme = ThemeConfig {
            wallpaper: Some(wallpaper_url.get()),
            gtk_theme: Some(gtk_theme.get()),
            icon_theme: Some(icon_theme.get()),
            colors: ColorScheme {
                primary: primary_color.get(),
                secondary: secondary_color.get(),
                background: background_color.get(),
                text: text_color.get(),
            },
        };
        on_change.call(new_theme);
    };

    view! {
        <div class="w-full">
            <h2 class="text-2xl font-bold text-white mb-6">"Customize Appearance"</h2>
            
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                // Wallpaper Section
                <div class="space-y-4">
                    <h3 class="text-lg font-semibold text-white">"Wallpaper"</h3>
                    <div class="space-y-2">
                        <label class="block text-sm font-medium text-gray-300">"Wallpaper URL"</label>
                        <input
                            type="url"
                            placeholder="https://example.com/wallpaper.jpg"
                            class="w-full px-4 py-2 bg-white/10 border border-white/20 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-purple-500"
                            prop:value=wallpaper_url
                            on:input=move |e| {
                                set_wallpaper_url.set(event_target_value(&e));
                                update_theme();
                            }
                        />
                    </div>
                    
                    <div class="grid grid-cols-3 gap-2">
                        {["https://images.unsplash.com/photo-1506905925346-21bda4d32df4",
                          "https://images.unsplash.com/photo-1501594907352-04cda38ebc29", 
                          "https://images.unsplash.com/photo-1469474968028-56623f02e42e"]
                            .iter()
                            .map(|url| {
                                view! {
                                    <div
                                        class="relative h-20 rounded-lg overflow-hidden cursor-pointer border-2 border-transparent hover:border-purple-500 transition-colors"
                                        style=move || format!("background-image: url('{}'); background-size: cover; background-position: center;", url)
                                        on:click=move |_| {
                                            set_wallpaper_url.set(url.to_string());
                                            update_theme();
                                        }
                                    >
                                        <div class="absolute inset-0 bg-black/20"></div>
                                    </div>
                                }
                            })
                            .collect_view()
                        }
                    </div>
                </div>

                // GTK Theme Section
                <div class="space-y-4">
                    <h3 class="text-lg font-semibold text-white">"GTK Theme"</h3>
                    <div class="space-y-2">
                        <label class="block text-sm font-medium text-gray-300">"Theme"</label>
                        <select
                            class="w-full px-4 py-2 bg-white/10 border border-white/20 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                            on:change=move |e| {
                                set_gtk_theme.set(event_target_value(&e));
                                update_theme();
                            }
                        >
                            <option value="" selected=move || gtk_theme.get().is_empty()>"Select theme"</option>
                            {gtk_themes.iter().map(|theme| {
                                view! {
                                    <option value=*theme selected=move || gtk_theme.get() == *theme>
                                        {theme}
                                    </option>
                                }
                            }).collect_view()}
                        </select>
                    </div>
                </div>

                // Icon Theme Section
                <div class="space-y-4">
                    <h3 class="text-lg font-semibold text-white">"Icon Theme"</h3>
                    <div class="space-y-2">
                        <label class="block text-sm font-medium text-gray-300">"Icons"</label>
                        <select
                            class="w-full px-4 py-2 bg-white/10 border border-white/20 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                            on:change=move |e| {
                                set_icon_theme.set(event_target_value(&e));
                                update_theme();
                            }
                        >
                            <option value="" selected=move || icon_theme.get().is_empty()>"Select icon theme"</option>
                            {icon_themes.iter().map(|theme| {
                                view! {
                                    <option value=*theme selected=move || icon_theme.get() == *theme>
                                        {theme}
                                    </option>
                                }
                            }).collect_view()}
                        </select>
                    </div>
                </div>

                // Color Scheme Section
                <div class="space-y-4">
                    <h3 class="text-lg font-semibold text-white">"Color Scheme"</h3>
                    <div class="grid grid-cols-2 gap-4">
                        <div class="space-y-2">
                            <label class="block text-sm font-medium text-gray-300">"Primary Color"</label>
                            <div class="flex items-center space-x-2">
                                <input
                                    type="color"
                                    class="h-10 w-20 bg-white/10 border border-white/20 rounded cursor-pointer"
                                    prop:value=primary_color
                                    on:input=move |e| {
                                        set_primary_color.set(event_target_value(&e));
                                        update_theme();
                                    }
                                />
                                <input
                                    type="text"
                                    class="flex-1 px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-white text-sm"
                                    prop:value=primary_color
                                    on:input=move |e| {
                                        set_primary_color.set(event_target_value(&e));
                                        update_theme();
                                    }
                                />
                            </div>
                        </div>
                        
                        <div class="space-y-2">
                            <label class="block text-sm font-medium text-gray-300">"Secondary Color"</label>
                            <div class="flex items-center space-x-2">
                                <input
                                    type="color"
                                    class="h-10 w-20 bg-white/10 border border-white/20 rounded cursor-pointer"
                                    prop:value=secondary_color
                                    on:input=move |e| {
                                        set_secondary_color.set(event_target_value(&e));
                                        update_theme();
                                    }
                                />
                                <input
                                    type="text"
                                    class="flex-1 px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-white text-sm"
                                    prop:value=secondary_color
                                    on:input=move |e| {
                                        set_secondary_color.set(event_target_value(&e));
                                        update_theme();
                                    }
                                />
                            </div>
                        </div>
                        
                        <div class="space-y-2">
                            <label class="block text-sm font-medium text-gray-300">"Background Color"</label>
                            <div class="flex items-center space-x-2">
                                <input
                                    type="color"
                                    class="h-10 w-20 bg-white/10 border border-white/20 rounded cursor-pointer"
                                    prop:value=background_color
                                    on:input=move |e| {
                                        set_background_color.set(event_target_value(&e));
                                        update_theme();
                                    }
                                />
                                <input
                                    type="text"
                                    class="flex-1 px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-white text-sm"
                                    prop:value=background_color
                                    on:input=move |e| {
                                        set_background_color.set(event_target_value(&e));
                                        update_theme();
                                    }
                                />
                            </div>
                        </div>
                        
                        <div class="space-y-2">
                            <label class="block text-sm font-medium text-gray-300">"Text Color"</label>
                            <div class="flex items-center space-x-2">
                                <input
                                    type="color"
                                    class="h-10 w-20 bg-white/10 border border-white/20 rounded cursor-pointer"
                                    prop:value=text_color
                                    on:input=move |e| {
                                        set_text_color.set(event_target_value(&e));
                                        update_theme();
                                    }
                                />
                                <input
                                    type="text"
                                    class="flex-1 px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-white text-sm"
                                    prop:value=text_color
                                    on:input=move |e| {
                                        set_text_color.set(event_target_value(&e));
                                        update_theme();
                                    }
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            // Preview Section
            <div class="mt-8 p-6 bg-white/5 backdrop-blur-md rounded-lg border border-white/10">
                <h3 class="text-lg font-semibold text-white mb-4">"Preview"</h3>
                <div 
                    class="p-4 rounded-lg"
                    style=move || format!(
                        "background-color: {}; color: {}; border: 2px solid {};",
                        background_color.get(),
                        text_color.get(),
                        primary_color.get()
                    )
                >
                    <div class="flex items-center space-x-4">
                        <div 
                            class="w-12 h-12 rounded-full"
                            style=move || format!("background-color: {};", primary_color.get())
                        ></div>
                        <div>
                            <h4 style=move || format!("color: {};", text_color.get())>"Sample Window"</h4>
                            <p style=move || format!("color: {};", text_color.get())>"This is how your theme will look"</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
