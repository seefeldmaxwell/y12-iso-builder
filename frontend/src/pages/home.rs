use leptos::*;
use leptos_router::A;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="min-h-screen flex flex-col">
            // Hero Section
            <section class="flex-1 flex items-center justify-center px-4 py-20">
                <div class="max-w-4xl mx-auto text-center">
                    <div class="mb-8">
                        <div class="inline-flex items-center px-4 py-2 bg-purple-500/20 rounded-full text-purple-300 text-sm font-medium mb-6">
                            <svg class="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                <path d="M10 12a2 2 0 100-4 2 2 0 000 4z"/>
                                <path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd"/>
                            </svg>
                            "Powered by Cloudflare Containers"
                        </div>
                        
                        <h1 class="text-5xl md:text-7xl font-bold text-white mb-6 bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
                            "Build Custom Linux"
                            <br/>
                            "Distributions"
                        </h1>
                        
                        <p class="text-xl text-gray-300 mb-8 max-w-2xl mx-auto">
                            "Create personalized Linux ISOs with custom packages, themes, and configurations. "
                            "No local installation required - everything runs in the cloud."
                        </p>
                    </div>

                    <div class="flex flex-col sm:flex-row gap-4 justify-center mb-12">
                        <A 
                            href="/create"
                            class="inline-flex items-center px-8 py-4 bg-gradient-to-r from-purple-500 to-pink-500 text-white font-semibold rounded-lg hover:from-purple-600 hover:to-pink-600 transition-all transform hover:scale-105"
                        >
                            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                            </svg>
                            "Start Building"
                        </A>
                        
                        <A 
                            href="/gallery"
                            class="inline-flex items-center px-8 py-4 bg-white/10 backdrop-blur-md text-white font-semibold rounded-lg border border-white/20 hover:bg-white/20 transition-all"
                        >
                            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"/>
                            </svg>
                            "Browse Gallery"
                        </A>
                    </div>

                    // Features Grid
                    <div class="grid md:grid-cols-3 gap-8 text-left">
                        <div class="bg-white/5 backdrop-blur-md rounded-lg p-6 border border-white/10">
                            <div class="w-12 h-12 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg flex items-center justify-center mb-4">
                                <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
                                </svg>
                            </div>
                            <h3 class="text-xl font-semibold text-white mb-2">"Lightning Fast"</h3>
                            <p class="text-gray-300">"Cloudflare's global network ensures your ISO builds complete in minutes, not hours."</p>
                        </div>

                        <div class="bg-white/5 backdrop-blur-md rounded-lg p-6 border border-white/10">
                            <div class="w-12 h-12 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg flex items-center justify-center mb-4">
                                <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"/>
                                </svg>
                            </div>
                            <h3 class="text-xl font-semibold text-white mb-2">"Fully Customizable"</h3>
                            <p class="text-gray-300">"Choose from multiple base distributions, custom packages, themes, and configurations."</p>
                        </div>

                        <div class="bg-white/5 backdrop-blur-md rounded-lg p-6 border border-white/10">
                            <div class="w-12 h-12 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg flex items-center justify-center mb-4">
                                <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/>
                                </svg>
                            </div>
                            <h3 class="text-xl font-semibold text-white mb-2">"Secure & Private"</h3>
                            <p class="text-gray-300">"Your builds run in isolated containers with automatic cleanup and secure storage."</p>
                        </div>
                    </div>
                </div>
            </section>

            // Supported Distros Section
            <section class="py-20 px-4 bg-black/20">
                <div class="max-w-6xl mx-auto">
                    <h2 class="text-3xl font-bold text-white text-center mb-12">"Supported Distributions"</h2>
                    <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-6">
                        {[
                            ("Ubuntu", "🟠"),
                            ("Debian", "❤️"),
                            ("Arch", "🔷"),
                            ("Fedora", "🔵"),
                            ("Mint", "🟢"),
                            ("openSUSE", "🦎"),
                        ].iter().map(|(name, icon)| {
                            view! {
                                <div class="bg-white/10 backdrop-blur-md rounded-lg p-6 text-center border border-white/20 hover:bg-white/20 transition-colors">
                                    <div class="text-4xl mb-3">{icon}</div>
                                    <h3 class="text-white font-semibold">{name}</h3>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </section>

            // CTA Section
            <section class="py-20 px-4">
                <div class="max-w-4xl mx-auto text-center">
                    <h2 class="text-3xl font-bold text-white mb-4">"Ready to Build Your Perfect Linux?"</h2>
                    <p class="text-gray-300 mb-8">"Join thousands of users creating custom Linux distributions"</p>
                    <A 
                        href="/create"
                        class="inline-flex items-center px-8 py-4 bg-gradient-to-r from-purple-500 to-pink-500 text-white font-semibold rounded-lg hover:from-purple-600 hover:to-pink-600 transition-all transform hover:scale-105"
                    >
                        "Get Started Now"
                        <svg class="w-5 h-5 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 8l4 4m0 0l-4 4m4-4H3"/>
                        </svg>
                    </A>
                </div>
            </section>
        </div>
    }
}
