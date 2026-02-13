use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use gloo_net::http::Request;

// ── Data ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Distro {
    pub id: &'static str,
    pub name: &'static str,
    pub tagline: &'static str,
    pub desc: &'static str,
    pub color: &'static str,
    pub source: &'static str,
}

const DISTROS: &[Distro] = &[
    Distro { id: "nixos", name: "NixOS", tagline: "Reproducible, declarative", desc: "Atomic upgrades, rollbacks, and a single config file that defines your entire system.", color: "#7ebae4", source: "github.com/NixOS/nixpkgs" },
    Distro { id: "debian", name: "Debian", tagline: "The universal operating system", desc: "Rock-solid stability with the largest package repository. The foundation for Ubuntu, Kali, and dozens of derivatives.", color: "#d70751", source: "salsa.debian.org/kernel-team" },
    Distro { id: "rocky", name: "Rocky Linux", tagline: "Enterprise RHEL-compatible", desc: "Production-ready, binary-compatible downstream of Red Hat Enterprise Linux.", color: "#10b981", source: "github.com/rocky-linux" },
    Distro { id: "proxmox", name: "Proxmox VE", tagline: "Enterprise virtualization platform", desc: "Open-source server management platform for KVM VMs and LXC containers with HA clustering, software-defined storage, and backup.", color: "#e57000", source: "git.proxmox.com" },
];

struct Overlay { id: &'static str, name: &'static str, desc: &'static str, cat: &'static str, compat: &'static [&'static str] }

// compat: which distro IDs this overlay works with. Empty = all distros.
const OVERLAYS: &[Overlay] = &[
    // Variant Distributions — only compatible with their parent distro family
    Overlay { id: "kali", name: "Kali Linux", desc: "Penetration testing and security auditing tools (Debian-based)", cat: "Variants", compat: &["debian", "proxmox"] },
    Overlay { id: "scientific", name: "Scientific Linux", desc: "Fermilab/CERN scientific computing packages (RHEL-based)", cat: "Variants", compat: &["rocky"] },
    Overlay { id: "parrot", name: "Parrot Security", desc: "Security, forensics, and privacy tools (Debian-based)", cat: "Variants", compat: &["debian", "proxmox"] },
    Overlay { id: "devuan", name: "Devuan", desc: "Debian without systemd — sysvinit/OpenRC init system", cat: "Variants", compat: &["debian"] },
    Overlay { id: "alma", name: "AlmaLinux", desc: "RHEL-compatible community enterprise OS (Rocky alternative)", cat: "Variants", compat: &["rocky"] },
    // RMM & Fleet Management
    Overlay { id: "tacticalrmm", name: "Tactical RMM", desc: "Open-source remote monitoring and management", cat: "Fleet", compat: &["debian", "rocky", "proxmox"] },
    Overlay { id: "meshcentral", name: "MeshCentral", desc: "Full-featured remote management web app", cat: "Fleet", compat: &[] },
    Overlay { id: "ansible", name: "Ansible", desc: "Agentless IT automation and configuration management", cat: "Fleet", compat: &[] },
    Overlay { id: "salt", name: "SaltStack", desc: "Event-driven infrastructure automation", cat: "Fleet", compat: &["debian", "rocky", "proxmox"] },
    Overlay { id: "puppet", name: "Puppet Agent", desc: "Configuration management and compliance", cat: "Fleet", compat: &["debian", "rocky", "proxmox"] },
    Overlay { id: "zabbix", name: "Zabbix Agent", desc: "Enterprise monitoring agent for fleet telemetry", cat: "Fleet", compat: &["debian", "rocky", "proxmox"] },
    // Virtualization
    Overlay { id: "qemu", name: "QEMU/KVM", desc: "Full system emulation and hardware virtualization", cat: "Virtualization", compat: &[] },
    Overlay { id: "libvirt", name: "libvirt", desc: "Virtualization API and management toolkit", cat: "Virtualization", compat: &[] },
    Overlay { id: "lxc", name: "LXC/LXD", desc: "System container manager", cat: "Virtualization", compat: &[] },
    // DevOps
    Overlay { id: "docker", name: "Docker Engine", desc: "Container runtime and orchestration", cat: "DevOps", compat: &[] },
    Overlay { id: "k3s", name: "K3s", desc: "Lightweight Kubernetes", cat: "DevOps", compat: &[] },
    Overlay { id: "podman", name: "Podman", desc: "Daemonless container engine (Docker-compatible)", cat: "DevOps", compat: &[] },
    // Networking
    Overlay { id: "tailscale", name: "Tailscale", desc: "Zero-config mesh VPN (WireGuard)", cat: "Networking", compat: &[] },
    Overlay { id: "caddy", name: "Caddy", desc: "Web server with automatic HTTPS", cat: "Networking", compat: &[] },
    Overlay { id: "nginx", name: "NGINX", desc: "High-performance reverse proxy and web server", cat: "Networking", compat: &[] },
    // Database
    Overlay { id: "postgres", name: "PostgreSQL 16", desc: "Advanced relational database", cat: "Database", compat: &[] },
    Overlay { id: "redis", name: "Redis 7", desc: "In-memory data store", cat: "Database", compat: &[] },
    Overlay { id: "mysql", name: "MariaDB", desc: "MySQL-compatible relational database", cat: "Database", compat: &[] },
    // Observability
    Overlay { id: "prometheus", name: "Prometheus", desc: "Monitoring and alerting", cat: "Observability", compat: &[] },
    Overlay { id: "grafana", name: "Grafana", desc: "Analytics and visualization", cat: "Observability", compat: &[] },
    Overlay { id: "netdata", name: "Netdata", desc: "Real-time performance monitoring", cat: "Observability", compat: &[] },
    // Development
    Overlay { id: "neovim", name: "Neovim", desc: "Hyperextensible text editor", cat: "Development", compat: &[] },
    Overlay { id: "vscode", name: "VS Code", desc: "Microsoft's code editor", cat: "Development", compat: &["debian", "rocky", "proxmox"] },
    Overlay { id: "rustup", name: "Rust Toolchain", desc: "rustc, cargo, std library", cat: "Development", compat: &[] },
    Overlay { id: "nodejs", name: "Node.js LTS", desc: "JavaScript runtime (V8)", cat: "Development", compat: &[] },
    Overlay { id: "golang", name: "Go", desc: "Statically typed compiled language", cat: "Development", compat: &[] },
    // Media & Gaming
    Overlay { id: "obs", name: "OBS Studio", desc: "Video recording and streaming", cat: "Media", compat: &["debian", "nixos"] },
    Overlay { id: "blender", name: "Blender", desc: "3D creation suite", cat: "Media", compat: &[] },
    Overlay { id: "openclaw", name: "OpenClaw", desc: "Open-source Captain Claw reimplementation", cat: "Gaming", compat: &["debian"] },
    Overlay { id: "steam", name: "Steam", desc: "Valve's gaming platform with Proton", cat: "Gaming", compat: &["debian", "nixos"] },
    Overlay { id: "lutris", name: "Lutris", desc: "Open gaming platform for Linux", cat: "Gaming", compat: &["debian", "nixos"] },
];

#[derive(Debug, Clone, PartialEq)]
struct ChatMsg { from_user: bool, text: String }

fn parse_lspci(raw: &str) -> Vec<(String, String, String)> {
    raw.lines().filter_map(|line| {
        let line = line.trim();
        if line.is_empty() { return None; }
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() < 2 { return None; }
        let slot = parts[0].to_string();
        let rest = parts[1];
        if let Some(colon_pos) = rest.find(':') {
            let dev_type = rest[..colon_pos].trim().to_string();
            let dev_name = rest[colon_pos+1..].trim().to_string();
            Some((slot, dev_type, dev_name))
        } else {
            Some((slot, "Unknown".into(), rest.to_string()))
        }
    }).collect()
}

fn detect_kernel_modules(devices: &[(String, String, String)]) -> Vec<(String, String, bool)> {
    let mut modules: Vec<(String, String, bool)> = Vec::new();
    for (_slot, dtype, name) in devices {
        let n = name.to_lowercase();
        let t = dtype.to_lowercase();
        if t.contains("vga") || t.contains("display") || t.contains("3d") {
            if n.contains("nvidia") {
                modules.push(("nvidia".into(), format!("GPU: {}", name), true));
                modules.push(("nouveau".into(), "Open-source NVIDIA (conflicts with proprietary)".into(), false));
            } else if n.contains("amd") || n.contains("radeon") {
                modules.push(("amdgpu".into(), format!("GPU: {}", name), true));
            } else if n.contains("intel") {
                modules.push(("i915".into(), format!("GPU: {}", name), true));
            }
        } else if t.contains("network") || t.contains("ethernet") || t.contains("wifi") || t.contains("wireless") {
            if n.contains("intel") {
                if n.contains("wi-fi") || n.contains("wireless") || n.contains("wifi") {
                    modules.push(("iwlwifi".into(), format!("WiFi: {}", name), true));
                } else {
                    modules.push(("e1000e".into(), format!("Ethernet: {}", name), true));
                }
            } else if n.contains("realtek") {
                modules.push(("r8169".into(), format!("Ethernet: {}", name), true));
            } else if n.contains("broadcom") {
                modules.push(("bnxt_en".into(), format!("NIC: {}", name), true));
            } else if n.contains("qualcomm") || n.contains("atheros") {
                modules.push(("ath11k".into(), format!("WiFi: {}", name), true));
            }
        } else if t.contains("audio") || t.contains("multimedia") {
            modules.push(("snd_hda_intel".into(), format!("Audio: {}", name), true));
        } else if t.contains("usb") {
            modules.push(("xhci_hcd".into(), format!("USB: {}", name), true));
        } else if t.contains("sata") || t.contains("ahci") || t.contains("nvme") || t.contains("storage") {
            if n.contains("nvme") || t.contains("nvme") {
                modules.push(("nvme".into(), format!("Storage: {}", name), true));
            } else {
                modules.push(("ahci".into(), format!("Storage: {}", name), true));
            }
        } else if t.contains("smbus") || t.contains("isa") || t.contains("bridge") {
            // skip chipset bridges
        }
    }
    modules.sort_by(|a, b| a.0.cmp(&b.0));
    modules.dedup_by(|a, b| a.0 == b.0);
    modules
}

// ── App ────────────────────────────────────────────────────────────────

fn is_mobile() -> bool {
    if let Some(window) = web_sys::window() {
        let width = window.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1200.0);
        if width < 768.0 { return true; }
        if let Ok(ua) = window.navigator().user_agent() {
            let ua = ua.to_lowercase();
            return ua.contains("mobile") || ua.contains("android") || ua.contains("iphone") || ua.contains("ipad");
        }
    }
    false
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    if is_mobile() {
        return view! {
            <div class="flex min-h-screen flex-col items-center justify-center bg-[#0a0a0a] px-8 text-center text-[#ededed]">
                <div class="mb-6 text-[16px] font-bold tracking-tight">"Y12.AI"</div>
                <svg class="mb-6 h-16 w-16 text-[#555]" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M9 17.25v1.007a3 3 0 01-.879 2.122L7.5 21h9l-.621-.621A3 3 0 0115 18.257V17.25m6-12V15a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 15V5.25m18 0A2.25 2.25 0 0018.75 3H5.25A2.25 2.25 0 003 5.25m18 0V12a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 12V5.25"/></svg>
                <h1 class="mb-3 text-2xl font-bold">"Desktop Only"</h1>
                <p class="mb-6 max-w-sm text-[15px] leading-relaxed text-[#888]">
                    "Y12.AI requires a desktop browser to configure and build custom Linux ISOs. The build interface needs a full-size screen for hardware detection, kernel configuration, and software selection."
                </p>
                <p class="text-[13px] text-[#555]">"Open "<span class="text-white">"y12-iso-builder.pages.dev"</span>" on your desktop."</p>
            </div>
        }.into_view();
    }

    let (chat_open, set_chat_open) = create_signal(false);
    let (chat_msgs, set_chat_msgs) = create_signal(vec![
        ChatMsg { from_user: false, text: "Hey! I'm the Y12 build assistant. I can help you configure your custom ISO — just describe what you need and I'll set it up. Try: \"I need a minimal NixOS server with Docker and Tailscale\"".into() },
    ]);
    let (chat_input, set_chat_input) = create_signal(String::new());

    let (chat_loading, set_chat_loading) = create_signal(false);

    let do_send = move || {
        let msg = chat_input.get();
        if msg.trim().is_empty() { return; }
        set_chat_msgs.update(|v| v.push(ChatMsg { from_user: true, text: msg.clone() }));
        set_chat_input.set(String::new());
        set_chat_loading.set(true);

        // Build message history for the API
        let history: Vec<_> = chat_msgs.get().iter().map(|m| {
            serde_json::json!({ "role": if m.from_user { "user" } else { "assistant" }, "content": m.text })
        }).collect();

        let set_msgs = set_chat_msgs.clone();
        let set_loading = set_chat_loading.clone();
        let fallback_reply = generate_chat_reply(&msg);

        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::json!({ "messages": history });
            let result = Request::post("https://y12-api.seefeldmaxwell1.workers.dev/api/chat")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .unwrap()
                .send()
                .await;

            let reply = match result {
                Ok(resp) => {
                    if let Ok(text) = resp.text().await {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                            parsed["reply"].as_str().unwrap_or(&fallback_reply).to_string()
                        } else { fallback_reply }
                    } else { fallback_reply }
                }
                Err(_) => fallback_reply,
            };

            set_msgs.update(|v| v.push(ChatMsg { from_user: false, text: reply }));
            set_loading.set(false);
        });
    };
    let do_send2 = do_send.clone();

    provide_context(chat_open);
    provide_context(set_chat_open);

    view! {
        <Router>
            <div class="min-h-screen bg-[#0a0a0a] text-[#ededed] antialiased">
                <Nav/>
                <main>
                    <Routes>
                        <Route path="/" view=HomePage/>
                        <Route path="/build" view=BuildPage/>
                        <Route path="/docs" view=DocsPage/>
                    </Routes>
                </main>

                // Chatbot FAB
                <button
                    class="fixed bottom-6 right-6 z-50 flex h-12 w-12 items-center justify-center rounded-full bg-white text-black shadow-lg hover:bg-[#ddd]"
                    on:click=move |_| set_chat_open.update(|v| *v = !*v)
                >
                    <svg class="h-5 w-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"/></svg>
                </button>

                // Chat panel
                <Show when=move || chat_open.get()>
                    <div class="fixed bottom-20 right-6 z-50 flex w-96 flex-col rounded-xl border border-[#1a1a1a] bg-[#0a0a0a] shadow-2xl">
                        <div class="flex items-center justify-between border-b border-[#1a1a1a] px-4 py-3">
                            <div class="flex items-center gap-2">
                                <div class="h-2 w-2 rounded-full bg-emerald-500"></div>
                                <span class="text-[13px] font-semibold">"Y12 Build Assistant"</span>
                            </div>
                            <button class="text-[#666] hover:text-white" on:click=move |_| set_chat_open.set(false)>
                                <svg class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/></svg>
                            </button>
                        </div>
                        <div class="flex-1 overflow-y-auto p-4 space-y-3" style="max-height:360px;">
                            {move || chat_msgs.get().iter().map(|m| {
                                let cls = if m.from_user { "ml-8 bg-white/10" } else { "mr-8 bg-[#111]" };
                                view! {
                                    <div class=format!("rounded-lg p-3 text-[13px] leading-relaxed {}", cls)>
                                        {m.text.clone()}
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                        <div class="border-t border-[#1a1a1a] p-3">
                            <div class="flex gap-2">
                                <input
                                    type="text"
                                    placeholder="Ask about your build..."
                                    class="flex-1 rounded-lg border border-[#1a1a1a] bg-[#111] px-3 py-2 text-[13px] text-white placeholder-[#555] focus:border-[#333] focus:outline-none"
                                    prop:value=chat_input
                                    on:input=move |e| set_chat_input.set(event_target_value(&e))
                                    on:keydown=move |e: web_sys::KeyboardEvent| { if e.key() == "Enter" { do_send(); } }
                                />
                                <button class="rounded-lg bg-white px-3 py-2 text-[13px] font-medium text-black hover:bg-[#ddd]" on:click=move |_| do_send2()>"Send"</button>
                            </div>
                        </div>
                    </div>
                </Show>
            </div>
        </Router>
    }
}

fn generate_chat_reply(msg: &str) -> String {
    let m = msg.to_lowercase();
    if m.contains("nixos") && m.contains("server") {
        "Great choice. For a NixOS server build, I'd recommend:\n\n• Base: NixOS minimal (no desktop)\n• Kernel: linux_latest with server-optimized .config\n• Overlays: Docker, Tailscale, PostgreSQL\n• Modules: Strip all GPU/audio/WiFi drivers\n\nHead to the Build tab and select NixOS → Server mode. I'll pre-configure the kernel for headless operation.".into()
    } else if m.contains("debian") || m.contains("desktop") || m.contains("gaming") || m.contains("kali") {
        "Debian is a great base for desktop/gaming. I'd suggest:\n\n• Base: Debian stable\n• Kernel: Keep GPU drivers (NVIDIA or AMD)\n• Overlays: Steam, Lutris, OpenClaw\n• Variant: Add Kali overlay for security tools\n• Modules: Strip server/enterprise drivers\n\nGo to Build → select Debian → Desktop mode. Paste your lspci output so I can detect your GPU.".into()
    } else if m.contains("rocky") || m.contains("enterprise") || m.contains("production") {
        "Rocky Linux for production — solid pick. Recommended config:\n\n• Base: Rocky Linux 9 minimal\n• Kernel: RHEL-compatible with security patches\n• Overlays: Docker, K3s, Prometheus, Grafana\n• Modules: Keep network/storage, strip desktop\n\nSelect Rocky Linux → Server mode in the Build tab.".into()
    } else if m.contains("lspci") || m.contains("hardware") || m.contains("detect") {
        "To detect your hardware, you have two options:\n\n1. **Paste device list** — Run the appropriate command for your OS:\n   • Linux: `lspci`\n   • macOS: `system_profiler SPHardwareDataType SPDisplaysDataType`\n   • Windows: `Get-PnpDevice -PresentOnly | Format-Table Class, FriendlyName, InstanceId`\n\n2. **Serial number** — Run `sudo dmidecode -s system-serial-number` (Linux), or check System Information on Windows/macOS.\n\nPaste the output in the Hardware step on the Build page.".into()
    } else if m.contains("kernel") || m.contains("module") || m.contains("menuconfig") {
        "The kernel optimization works like this:\n\n1. Your hardware info (from lspci or serial) maps to specific PCI/USB device IDs\n2. I cross-reference those IDs against the kernel's Kconfig to find which modules are needed\n3. Everything else gets disabled in the .config\n4. The kernel is compiled from source in a Cloudflare Container\n\nTypically removes 40-60% of modules, cutting kernel size by ~30% and boot time in half.".into()
    } else if m.contains("proxmox") || m.contains("vm") || m.contains("virtual") {
        "Proxmox VE is ideal for virtualization hosts. Recommended config:\n\n• Base: Proxmox VE\n• Mode: Server (headless)\n• Overlays: QEMU/KVM, libvirt, LXC/LXD\n• Modules: Keep IOMMU, vfio for GPU passthrough\n\nSelect Proxmox VE → Server mode. Great for homelab and production clusters.".into()
    } else if m.contains("fleet") || m.contains("rmm") || m.contains("deploy") || m.contains("mass") {
        "For fleet deployments, I'd recommend:\n\n• Base: Debian or Rocky Linux\n• Mode: Server\n• Fleet tools: Tactical RMM, MeshCentral, Ansible\n• Custom software: Add your RMM agent by name\n• Plan: Fleet tier ($49.99/mo) for unlimited builds + API\n\nYou can add any custom package in the Software step — just type the package name.".into()
    } else if m.contains("price") || m.contains("cost") || m.contains("pay") || m.contains("stripe") {
        "It's $20 per ISO build via Stripe. That includes everything:\n\n• Custom kernel compiled from source\n• AI hardware optimization\n• Unlimited overlays + custom software\n• Variant distro overlays (Kali, etc.)\n• RMM agent pre-install\n• SHA256-signed ISO with 7-day download\n\nPayment is collected on the Payment step before the build starts.".into()
    } else if m.contains("help") || m.contains("what") || m.contains("how") {
        "Here's what I can help with:\n\n• **Choose a distro** — NixOS, Debian, Rocky Linux, or Proxmox VE\n• **Configure hardware** — Paste lspci output or enter serial number\n• **Select software** — Docker, K3s, Steam, RMM tools, and more\n• **Add custom packages** — Any RMM agent or tool by name\n• **Variant overlays** — Kali, Scientific Linux, Parrot, etc.\n\nTry: \"Debian gaming desktop\" or \"Rocky Linux fleet with Tactical RMM\"".into()
    } else {
        format!("I can help configure your ISO build. Tell me:\n\n• What distro? (NixOS, Debian, Rocky Linux, Proxmox VE)\n• Desktop or server?\n• Any specific software or RMM tools?\n\nOr describe your use case and I'll recommend a configuration.")
    }
}

// ── Nav ────────────────────────────────────────────────────────────────

#[component]
fn Nav() -> impl IntoView {
    view! {
        <header class="sticky top-0 z-50 border-b border-[#1a1a1a] bg-[#0a0a0a]/80 backdrop-blur-lg">
            <div class="mx-auto flex h-14 max-w-screen-xl items-center justify-between px-6">
                <A href="/" class="flex items-center gap-2">
                    <span class="text-[16px] font-bold tracking-tight text-white">"Y12.AI"</span>
                </A>
                <nav class="flex items-center gap-1">
                    <A href="/" class="px-3 py-1.5 text-[13px] text-[#888] hover:text-white rounded-md hover:bg-[#1a1a1a]">"Home"</A>
                    <A href="/build" class="px-3 py-1.5 text-[13px] text-[#888] hover:text-white rounded-md hover:bg-[#1a1a1a]">"Build"</A>
                    <A href="/docs" class="px-3 py-1.5 text-[13px] text-[#888] hover:text-white rounded-md hover:bg-[#1a1a1a]">"Docs"</A>
                    <div class="ml-3 h-4 w-px bg-[#333]"></div>
                    <A href="/docs" class="px-3 py-1.5 text-[13px] text-[#888] hover:text-white rounded-md hover:bg-[#1a1a1a]">"Pricing"</A>
                    <a href="https://github.com/y12-ai" target="_blank" class="ml-2 px-3 py-1.5 text-[13px] text-[#888] hover:text-white rounded-md hover:bg-[#1a1a1a]">"GitHub"</a>
                </nav>
            </div>
        </header>
    }
}

// ── Home ───────────────────────────────────────────────────────────────

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div>
            // ── Hero ──
            <section class="relative overflow-hidden border-b border-[#1a1a1a]">
                <div class="pointer-events-none absolute inset-0">
                    <div class="absolute left-1/2 top-0 -translate-x-1/2 h-[500px] w-[800px] rounded-full bg-white/[0.03] blur-3xl"></div>
                </div>
                <div class="relative mx-auto max-w-screen-xl px-6 pb-24 pt-20 text-center">
                    <div class="mb-6 inline-flex items-center gap-2 rounded-full border border-[#333] bg-[#111] px-4 py-1.5 text-[13px] text-[#888]">
                        <span class="inline-block h-1.5 w-1.5 rounded-full bg-emerald-500"></span>
                        "Builds run on Cloudflare edge infrastructure"
                    </div>
                    <h1 class="mx-auto max-w-3xl text-[clamp(2.5rem,6vw,4.5rem)] font-bold leading-[1.08] tracking-tight">
                        "Custom Linux ISOs"
                        <br/>
                        <span class="text-[#666]">"and instant containers."</span>
                    </h1>
                    <p class="mx-auto mt-5 max-w-xl text-[17px] leading-relaxed text-[#888]">
                        "Build hardware-optimized ISOs or launch instant cloud containers — with custom kernels, hand-picked software, and AI-tuned configs. Faster boot, smaller attack surface, zero bloat."
                    </p>
                    <div class="mt-10 flex items-center justify-center gap-3">
                        <A href="/build" class="inline-flex items-center gap-2 rounded-lg bg-white px-5 py-2.5 text-[14px] font-medium text-black hover:bg-[#ddd]">
                            "Start a build — $20"
                            <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7"/></svg>
                        </A>
                        <A href="/docs" class="inline-flex items-center gap-2 rounded-lg border border-[#333] bg-transparent px-5 py-2.5 text-[14px] font-medium text-[#888] hover:border-[#555] hover:text-white">
                            "Read the docs"
                        </A>
                    </div>
                </div>
            </section>

            // ── LFS Made Easy ──
            <section class="border-b border-[#1a1a1a]">
                <div class="mx-auto max-w-screen-xl px-6 py-20">
                    <p class="mb-2 text-[13px] font-medium uppercase tracking-widest text-[#555]">"Linux From Scratch — Done Right"</p>
                    <h2 class="mb-4 text-3xl font-bold tracking-tight">"The power of Gentoo and LFS. None of the pain."</h2>
                    <p class="mb-12 max-w-2xl text-[15px] leading-relaxed text-[#888]">"Building Linux From Scratch takes 72+ hours of manual compilation. Gentoo's emerge system requires deep Portage knowledge. Clear Linux needs Intel hardware. Y12.AI gives you the same result — a system compiled from source, tuned to your hardware — in minutes, for $20. No PhD required."</p>
                    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                        // AI Kernel Optimization
                        <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-6">
                            <div class="mb-4 flex h-10 w-10 items-center justify-center rounded-lg bg-violet-500/10 text-violet-400">
                                <svg class="h-5 w-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.455 2.456L21.75 6l-1.036.259a3.375 3.375 0 00-2.455 2.456z"/></svg>
                            </div>
                            <h3 class="mb-2 text-[15px] font-semibold">"AI-Powered Kernel Configuration"</h3>
                            <p class="text-[13px] leading-relaxed text-[#888]">"The Linux kernel has 17,000+ config options (Kconfig). Our AI agent (Llama 3.1 8B running on Cloudflare Workers AI) parses your lspci hardware output, maps PCI vendor/device IDs to kernel modules via the modules.alias database, and generates a .config that enables only what your hardware needs. A stock Ubuntu kernel ships ~5,800 modules. A Y12 build typically ships 200-400."</p>
                        </div>
                        // Attack Surface Reduction
                        <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-6">
                            <div class="mb-4 flex h-10 w-10 items-center justify-center rounded-lg bg-blue-500/10 text-blue-400">
                                <svg class="h-5 w-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z"/></svg>
                            </div>
                            <h3 class="mb-2 text-[15px] font-semibold">"Measurably Smaller Attack Surface"</h3>
                            <p class="text-[13px] leading-relaxed text-[#888]">"In 2024, 60% of Linux kernel CVEs affected drivers and subsystems most users don't need (Bluetooth, GPU, WiFi on servers, etc.). Every compiled-out module is code that cannot be exploited. Server builds disable CONFIG_DRM, CONFIG_SND, CONFIG_WLAN, CONFIG_BT — eliminating entire vulnerability classes. This is the same approach Google uses for GKE node kernels."</p>
                        </div>
                        // Boot Performance
                        <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-6">
                            <div class="mb-4 flex h-10 w-10 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-400">
                                <svg class="h-5 w-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z"/></svg>
                            </div>
                            <h3 class="mb-2 text-[15px] font-semibold">"Faster Boot, Less RAM"</h3>
                            <p class="text-[13px] leading-relaxed text-[#888]">"A stock distro kernel is 12-14 MB (bzImage) with a 30-60 MB initramfs loading thousands of modules at boot. A Y12 custom kernel is 4-6 MB with a 5-10 MB initramfs. Fewer modules to probe means faster hardware init. On NVMe systems, this cuts boot-to-login from ~8s to ~3s. RAM usage drops 50-100 MB because unused module memory is never allocated."</p>
                        </div>
                        // AI Chat Assistant
                        <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-6">
                            <div class="mb-4 flex h-10 w-10 items-center justify-center rounded-lg bg-amber-500/10 text-amber-400">
                                <svg class="h-5 w-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"/></svg>
                            </div>
                            <h3 class="mb-2 text-[15px] font-semibold">"AI Build Assistant (Llama 3.1)"</h3>
                            <p class="text-[13px] leading-relaxed text-[#888]">"Describe your use case in plain English — 'I need a Debian server with Docker and Tailscale for a 50-node fleet' — and the AI recommends distro, mode, overlays, and kernel config. It knows which packages exist in apt vs dnf vs nix, flags incompatible combinations, and explains trade-offs. Powered by Meta Llama 3.1 8B on Cloudflare Workers AI with sub-200ms inference."</p>
                        </div>
                        // OpenClaw Native
                        <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-6">
                            <div class="mb-4 flex h-10 w-10 items-center justify-center rounded-lg bg-red-500/10 text-red-400">
                                <svg class="h-5 w-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M14.25 6.087c0-.355.186-.676.401-.959.221-.29.349-.634.349-1.003 0-1.036-1.007-1.875-2.25-1.875s-2.25.84-2.25 1.875c0 .369.128.713.349 1.003.215.283.401.604.401.959v0a.64.64 0 01-.657.643 48.39 48.39 0 01-4.163-.3c.186 1.613.293 3.25.315 4.907a.656.656 0 01-.658.663v0c-.355 0-.676-.186-.959-.401a1.647 1.647 0 00-1.003-.349c-1.036 0-1.875 1.007-1.875 2.25s.84 2.25 1.875 2.25c.369 0 .713-.128 1.003-.349.283-.215.604-.401.959-.401v0c.31 0 .555.26.532.57a48.039 48.039 0 01-.642 5.056c1.518.19 3.058.309 4.616.354a.64.64 0 00.657-.643v0c0-.355-.186-.676-.401-.959a1.647 1.647 0 01-.349-1.003c0-1.035 1.008-1.875 2.25-1.875 1.243 0 2.25.84 2.25 1.875 0 .369-.128.713-.349 1.003-.215.283-.401.604-.401.959v0c0 .333.277.599.61.58a48.1 48.1 0 005.427-.63 48.05 48.05 0 00.582-4.717.532.532 0 00-.533-.57v0c-.355 0-.676.186-.959.401-.29.221-.634.349-1.003.349-1.035 0-1.875-1.007-1.875-2.25s.84-2.25 1.875-2.25c.37 0 .713.128 1.003.349.283.215.604.401.959.401v0a.656.656 0 00.658-.663 48.422 48.422 0 00-.37-5.36c-1.886.342-3.81.574-5.766.689a.578.578 0 01-.61-.58v0z"/></svg>
                            </div>
                            <h3 class="mb-2 text-[15px] font-semibold">"Native OpenClaw + Gaming Support"</h3>
                            <p class="text-[13px] leading-relaxed text-[#888]">"OpenClaw (open-source Captain Claw reimplementation) is pre-compiled and configured out of the box on Debian builds. Gaming ISOs include Steam with Proton, Lutris, Mesa/Vulkan drivers, and 32-bit multilib — all pre-configured. No post-install setup. Boot and play."</p>
                        </div>
                        // Cost Effective
                        <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-6">
                            <div class="mb-4 flex h-10 w-10 items-center justify-center rounded-lg bg-cyan-500/10 text-cyan-400">
                                <svg class="h-5 w-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M12 6v12m-3-2.818l.879.659c1.171.879 3.07.879 4.242 0 1.172-.879 1.172-2.303 0-3.182C13.536 12.219 12.768 12 12 12c-.725 0-1.45-.22-2.003-.659-1.106-.879-1.106-2.303 0-3.182s2.9-.879 4.006 0l.415.33M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                            </div>
                            <h3 class="mb-2 text-[15px] font-semibold">"$20 vs. 72 Hours of Your Time"</h3>
                            <p class="text-[13px] leading-relaxed text-[#888]">"Linux From Scratch takes 72+ hours. Gentoo stage3 + emerge @world takes 8-24 hours on modern hardware. A consultant charges $150+/hr for kernel tuning. Y12 builds in minutes on Cloudflare's edge for $20 flat — including the kernel, all packages, overlay distros, and a signed ISO. Reproducible, repeatable, and you keep the build script."</p>
                        </div>
                    </div>
                </div>
            </section>

            // ── Distros ──
            <section class="border-b border-[#1a1a1a]">
                <div class="mx-auto max-w-screen-xl px-6 py-20">
                    <p class="mb-2 text-[13px] font-medium uppercase tracking-widest text-[#555]">"Supported Targets"</p>
                    <h2 class="mb-12 text-3xl font-bold tracking-tight">"Four platforms. Zero bloat."</h2>
                    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                        {DISTROS.iter().map(|d| {
                            view! {
                                <div class="group rounded-xl border border-[#1a1a1a] bg-[#111] p-6 hover:border-[#333]">
                                    <div class="mb-4 flex items-center gap-3">
                                        <div class="flex h-10 w-10 items-center justify-center rounded-lg" style=format!("background:{};", d.color)>
                                            <span class="text-sm font-bold text-black">{&d.name[..1]}</span>
                                        </div>
                                        <div>
                                            <h3 class="text-[15px] font-semibold text-white">{d.name}</h3>
                                            <p class="text-[12px] text-[#666]">{d.tagline}</p>
                                        </div>
                                    </div>
                                    <p class="mb-4 text-[13px] leading-relaxed text-[#888]">{d.desc}</p>
                                    <div class="flex items-center gap-2 text-[12px] text-[#555]">
                                        <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/></svg>
                                        {d.source}
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </section>

            // ── How It Works ──
            <section class="border-b border-[#1a1a1a]">
                <div class="mx-auto max-w-screen-xl px-6 py-20">
                    <p class="mb-2 text-[13px] font-medium uppercase tracking-widest text-[#555]">"How It Works"</p>
                    <h2 class="mb-12 text-3xl font-bold tracking-tight">"From hardware scan to running system."</h2>
                    <div class="grid gap-px overflow-hidden rounded-xl border border-[#1a1a1a] bg-[#1a1a1a] md:grid-cols-4">
                        {[
                            ("01", "Detect Hardware", "Paste lspci (Linux), system_profiler (macOS), or Get-PnpDevice (Windows) output. We map every device to the kernel modules it needs."),
                            ("02", "Configure Build", "Pick desktop or server mode. Choose overlays, add custom software. The AI agent builds a custom kernel .config."),
                            ("03", "Build on Edge", "Cloudflare Containers compile your kernel from source, install packages, and create a bootable ISO or runnable container image."),
                            ("04", "Download or Run", "Get a signed ISO to flash, or launch your build as an instant cloud container — boot in seconds from any browser."),
                        ].iter().map(|(num, title, desc)| {
                            view! {
                                <div class="bg-[#0a0a0a] p-6">
                                    <span class="mb-3 inline-block text-[12px] font-mono text-[#555]">{*num}</span>
                                    <h3 class="mb-2 text-[15px] font-semibold text-white">{*title}</h3>
                                    <p class="text-[13px] leading-relaxed text-[#888]">{*desc}</p>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </section>

            // ── Pricing ──
            <section class="border-b border-[#1a1a1a]">
                <div class="mx-auto max-w-screen-xl px-6 py-20 text-center">
                    <p class="mb-2 text-[13px] font-medium uppercase tracking-widest text-[#555]">"Pricing"</p>
                    <h2 class="mb-4 text-3xl font-bold tracking-tight">"$20 per build. Everything included."</h2>
                    <p class="mx-auto mb-10 max-w-lg text-[15px] text-[#888]">"Custom kernel from source, AI hardware optimization, unlimited overlays, variant distros, RMM pre-install, SHA256-signed output, 7-day download."</p>
                    <A href="/build" class="inline-flex items-center gap-2 rounded-lg bg-white px-6 py-3 text-[14px] font-medium text-black hover:bg-[#ddd]">
                        "Start building"
                        <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7"/></svg>
                    </A>
                </div>
            </section>

            // ── Footer ──
            <footer class="py-8 px-6">
                <div class="mx-auto flex max-w-screen-xl items-center justify-between text-[12px] text-[#555]">
                    <span>"© 2026 Y12.AI — Custom Linux ISOs and containers built from source."</span>
                    <div class="flex gap-5">
                        <a href="https://github.com/y12-ai" target="_blank" class="hover:text-white">"GitHub"</a>
                        <A href="/docs" class="hover:text-white">"Docs"</A>
                        <a href="/sitemap.xml" class="hover:text-white">"Sitemap"</a>
                    </div>
                </div>
            </footer>
        </div>
    }
}

// ── Build Page ─────────────────────────────────────────────────────────

#[component]
fn BuildPage() -> impl IntoView {
    let (step, set_step) = create_signal(0u8);
    let (selected_distro, set_selected_distro) = create_signal(None::<&'static Distro>);
    let (build_mode, set_build_mode) = create_signal("desktop".to_string()); // desktop or server
    let (hw_method, set_hw_method) = create_signal("lspci".to_string()); // lspci or serial
    let (lspci_raw, set_lspci_raw) = create_signal(String::new());
    let (serial, set_serial) = create_signal(String::new());
    let (detected_devices, set_detected_devices) = create_signal(Vec::<(String, String, String)>::new());
    let (detected_modules, set_detected_modules) = create_signal(Vec::<(String, String, bool)>::new());
    let (ai_mode, set_ai_mode) = create_signal(true);
    let (selected_overlays, set_selected_overlays) = create_signal(Vec::<String>::new());
    let (custom_sw_input, set_custom_sw_input) = create_signal(String::new());
    let (custom_sw_list, set_custom_sw_list) = create_signal(Vec::<String>::new());
    let (paid, set_paid) = create_signal(false);
    let (building, set_building) = create_signal(false);
    let (progress, set_progress) = create_signal(0u8);
    let (build_log, set_build_log) = create_signal(String::new());
    let (build_job_id, set_build_job_id) = create_signal(String::new());

    let do_add_sw = move || {
        let val = custom_sw_input.get().trim().to_string();
        if val.is_empty() { return; }
        set_custom_sw_list.update(|v| { if !v.contains(&val) { v.push(val); } });
        set_custom_sw_input.set(String::new());
    };
    let do_add_sw2 = do_add_sw.clone();

    let run_detection = move |_| {
        let raw = lspci_raw.get();
        let devs = parse_lspci(&raw);
        let mods = detect_kernel_modules(&devs);
        set_detected_devices.set(devs);
        set_detected_modules.set(mods);
    };

    let toggle_overlay = move |id: String| {
        set_selected_overlays.update(|v| {
            if let Some(pos) = v.iter().position(|x| *x == id) { v.remove(pos); } else { v.push(id); }
        });
    };

    let start_build = move |_| {
        set_building.set(true);
        set_step.set(5);
        set_progress.set(0);
        set_build_log.set(String::new());
        let mode = build_mode.get();
        let mode2 = mode.clone();
        let distro_id = selected_distro.get().map(|d| d.id.to_string()).unwrap_or_default();
        let overlays_list = selected_overlays.get();
        let custom_list = custom_sw_list.get();
        let hw_raw = lspci_raw.get();
        let ai = ai_mode.get();
        let mods: Vec<String> = detected_modules.get().iter().map(|(m, _, _)| m.clone()).collect();

        let set_p = set_progress.clone();
        let set_l = set_build_log.clone();
        let set_b = set_building.clone();
        let set_jid = set_build_job_id.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::json!({
                "distro": distro_id,
                "mode": mode2,
                "hardware_raw": hw_raw,
                "ai_mode": ai,
                "overlays": overlays_list,
                "custom_software": custom_list,
                "detected_modules": mods,
            });

            // Step 1: Create build job
            set_l.update(|log| log.push_str("[  0%] Submitting build to backend...\n"));
            let create_resp = Request::post("https://y12-api.seefeldmaxwell1.workers.dev/api/build")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .unwrap()
                .send()
                .await;

            let job_id = match create_resp {
                Ok(resp) => {
                    if let Ok(text) = resp.text().await {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                            let id = parsed["id"].as_str().unwrap_or("").to_string();
                            let model = parsed["ai_model"].as_str().unwrap_or("unknown");
                            let config_lines = parsed["kernel_config_lines"].as_u64().unwrap_or(0);
                            let pkgs = parsed["packages"].as_u64().unwrap_or(0);
                            set_l.update(|log| log.push_str(&format!("[  0%] Build job created: {}\n", &id[..8.min(id.len())])));
                            set_l.update(|log| log.push_str(&format!("[  0%] AI model: {}\n", model)));
                            set_l.update(|log| log.push_str(&format!("[  0%] Kernel config: {} lines | Packages: {}\n", config_lines, pkgs)));
                            set_jid.set(id.clone());
                            id
                        } else {
                            set_l.update(|log| log.push_str(&format!("[ERR] Unexpected response: {}\n", text)));
                            set_b.set(false);
                            return;
                        }
                    } else {
                        set_l.update(|log| log.push_str("[ERR] Could not read response\n"));
                        set_b.set(false);
                        return;
                    }
                }
                Err(e) => {
                    set_l.update(|log| log.push_str(&format!("[ERR] Backend error: {}\n", e)));
                    set_b.set(false);
                    return;
                }
            };

            if job_id.is_empty() {
                set_l.update(|log| log.push_str("[ERR] No job ID returned\n"));
                set_b.set(false);
                return;
            }

            // Step 2: Poll backend for real progress
            let poll_url = format!("https://y12-api.seefeldmaxwell1.workers.dev/api/build/{}", job_id);
            let mut last_log_count: usize = 0;

            loop {
                gloo_timers::future::TimeoutFuture::new(1_500).await;

                let poll_resp = Request::get(&poll_url).send().await;
                match poll_resp {
                    Ok(resp) => {
                        if let Ok(text) = resp.text().await {
                            if let Ok(job) = serde_json::from_str::<serde_json::Value>(&text) {
                                let p = job["progress"].as_u64().unwrap_or(0) as u8;
                                let status = job["status"].as_str().unwrap_or("unknown");

                                set_p.set(p);

                                // Append new log lines
                                if let Some(logs) = job["logs"].as_array() {
                                    for log_entry in logs.iter().skip(last_log_count) {
                                        if let Some(line) = log_entry.as_str() {
                                            set_l.update(|log| log.push_str(&format!("{}\n", line)));
                                        }
                                    }
                                    last_log_count = logs.len();
                                }

                                // Check terminal states
                                if status == "complete" || status == "complete_with_warnings" {
                                    set_p.set(100);
                                    if let Some(tr) = job["test_results"].as_object() {
                                        let passed = tr.get("passed").and_then(|v| v.as_u64()).unwrap_or(0);
                                        let total = tr.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                                        set_l.update(|log| log.push_str(&format!("\n✓ Build complete — {}/{} validations passed\n", passed, total)));
                                    }
                                    if job["iso_uploaded"].as_bool() == Some(true) {
                                        let size = job["iso_size"].as_u64().unwrap_or(0);
                                        let mb = size / 1_048_576;
                                        set_l.update(|log| log.push_str(&format!("✓ ISO uploaded to R2 — {} MB ready for download\n", mb)));
                                    }
                                    set_b.set(false);
                                    break;
                                } else if status == "failed" {
                                    let err = job["error"].as_str().unwrap_or("Unknown error");
                                    set_l.update(|log| log.push_str(&format!("\n✗ Build FAILED: {}\n", err)));
                                    set_b.set(false);
                                    break;
                                }
                                // building_iso = GitHub Actions is compiling, keep polling
                            }
                        }
                    }
                    Err(e) => {
                        set_l.update(|log| log.push_str(&format!("[POLL] Error: {} — retrying...\n", e)));
                    }
                }
            }
        });
    };

    view! {
        <div class="mx-auto max-w-screen-xl px-6 py-10">
            <div class="mb-8">
                <h1 class="text-2xl font-bold tracking-tight">"New ISO Build"</h1>
                <p class="mt-1 text-[14px] text-[#888]">"Configure your custom Linux ISO with hardware-optimized kernel."</p>
            </div>

            // Steps
            <div class="mb-8 flex items-center gap-1 rounded-lg border border-[#1a1a1a] bg-[#111] p-1">
                {["Target", "Hardware", "Kernel", "Software", "Payment", "Build"].iter().enumerate().map(|(i, label)| {
                    let idx = i as u8;
                    view! {
                        <button
                            class="flex-1 rounded-md px-3 py-2 text-[13px] font-medium"
                            class=("bg-[#1a1a1a]", move || step.get() == idx)
                            class=("text-white", move || step.get() == idx)
                            class=("text-[#666]", move || step.get() != idx)
                            class=("hover:text-[#999]", move || step.get() != idx)
                            on:click=move |_| { if !building.get() { set_step.set(idx) } }
                        >{*label}</button>
                    }
                }).collect_view()}
            </div>

            // ── Step 0: Target ──
            <Show when=move || step.get() == 0>
                <div class="space-y-6">
                    <div>
                        <label class="mb-2 block text-[13px] font-medium text-[#888]">"Base Distribution"</label>
                        <div class="grid gap-3 md:grid-cols-2 lg:grid-cols-4">
                            {DISTROS.iter().map(|d| {
                                let d_id = d.id;
                                let d_ref = d;
                                view! {
                                    <button
                                        class="rounded-xl border p-5 text-left"
                                        class=("border-white/20", move || selected_distro.get().map(|s| s.id == d_id).unwrap_or(false))
                                        class=("bg-white/5", move || selected_distro.get().map(|s| s.id == d_id).unwrap_or(false))
                                        class=("border-[#1a1a1a]", move || !selected_distro.get().map(|s| s.id == d_id).unwrap_or(false))
                                        class=("bg-[#111]", move || !selected_distro.get().map(|s| s.id == d_id).unwrap_or(false))
                                        class=("hover:border-[#333]", move || !selected_distro.get().map(|s| s.id == d_id).unwrap_or(false))
                                        on:click=move |_| set_selected_distro.set(Some(d_ref))
                                    >
                                        <div class="mb-2 flex items-center gap-2.5">
                                            <div class="flex h-8 w-8 items-center justify-center rounded-md" style=format!("background:{};", d.color)>
                                                <span class="text-xs font-bold text-black">{&d.name[..1]}</span>
                                            </div>
                                            <span class="text-[14px] font-semibold">{d.name}</span>
                                        </div>
                                        <p class="text-[12px] text-[#666]">{d.tagline}</p>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    <div>
                        <label class="mb-2 block text-[13px] font-medium text-[#888]">"Build Mode"</label>
                        <div class="grid grid-cols-2 gap-3">
                            <button
                                class="rounded-xl border p-5 text-left"
                                class=("border-white/20", move || build_mode.get() == "desktop")
                                class=("bg-white/5", move || build_mode.get() == "desktop")
                                class=("border-[#1a1a1a]", move || build_mode.get() != "desktop")
                                class=("bg-[#111]", move || build_mode.get() != "desktop")
                                class=("hover:border-[#333]", move || build_mode.get() != "desktop")
                                on:click=move |_| set_build_mode.set("desktop".into())
                            >
                                <p class="text-[14px] font-semibold">"Desktop"</p>
                                <p class="mt-1 text-[12px] text-[#666]">"Full desktop environment with GPU drivers, audio, WiFi, and display server. Includes a graphical installer."</p>
                            </button>
                            <button
                                class="rounded-xl border p-5 text-left"
                                class=("border-white/20", move || build_mode.get() == "server")
                                class=("bg-white/5", move || build_mode.get() == "server")
                                class=("border-[#1a1a1a]", move || build_mode.get() != "server")
                                class=("bg-[#111]", move || build_mode.get() != "server")
                                class=("hover:border-[#333]", move || build_mode.get() != "server")
                                on:click=move |_| set_build_mode.set("server".into())
                            >
                                <p class="text-[14px] font-semibold">"Server"</p>
                                <p class="mt-1 text-[12px] text-[#666]">"Headless minimal install. No GUI, no GPU drivers, no audio. Optimized for network and storage performance."</p>
                            </button>
                        </div>
                    </div>

                    <div class="flex justify-end">
                        <button
                            class="rounded-lg bg-white px-5 py-2.5 text-[13px] font-medium text-black hover:bg-[#ddd] disabled:opacity-30"
                            disabled=move || selected_distro.get().is_none()
                            on:click=move |_| set_step.set(1)
                        >"Continue"</button>
                    </div>
                </div>
            </Show>

            // ── Step 1: Hardware Detection ──
            <Show when=move || step.get() == 1>
                <div class="space-y-6">
                    <div class="flex items-center gap-1 rounded-lg border border-[#1a1a1a] bg-[#111] p-1">
                        <button
                            class="flex-1 rounded-md px-4 py-2 text-[13px] font-medium"
                            class=("bg-[#1a1a1a]", move || hw_method.get() == "lspci")
                            class=("text-white", move || hw_method.get() == "lspci")
                            class=("text-[#666]", move || hw_method.get() != "lspci")
                            on:click=move |_| set_hw_method.set("lspci".into())
                        >"Paste Device List"</button>
                        <button
                            class="flex-1 rounded-md px-4 py-2 text-[13px] font-medium"
                            class=("bg-[#1a1a1a]", move || hw_method.get() == "serial")
                            class=("text-white", move || hw_method.get() == "serial")
                            class=("text-[#666]", move || hw_method.get() != "serial")
                            on:click=move |_| set_hw_method.set("serial".into())
                        >"Serial Number"</button>
                    </div>

                    <Show when=move || hw_method.get() == "lspci">
                        <div class="space-y-4">
                            <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-5">
                                <h3 class="mb-1 text-[14px] font-semibold">"How to get your device list"</h3>
                                <p class="mb-3 text-[12px] text-[#666]">"Run one of these commands on the target machine, then paste the output below."</p>
                                <div class="space-y-3">
                                    <div>
                                        <p class="mb-1 text-[12px] font-medium text-[#888]">"Linux"</p>
                                        <div class="rounded-lg bg-[#0a0a0a] p-3 font-mono text-[13px] text-[#ccc]">
                                            <span class="text-[#555]">"$ "</span>"lspci"
                                        </div>
                                        <p class="mt-1 text-[11px] text-[#555]">"For PCI IDs: "<code class="text-[#888]">"lspci -nn"</code></p>
                                    </div>
                                    <div>
                                        <p class="mb-1 text-[12px] font-medium text-[#888]">"macOS"</p>
                                        <div class="rounded-lg bg-[#0a0a0a] p-3 font-mono text-[13px] text-[#ccc]">
                                            <span class="text-[#555]">"$ "</span>"system_profiler SPHardwareDataType SPDisplaysDataType SPNetworkDataType SPStorageDataType"
                                        </div>
                                    </div>
                                    <div>
                                        <p class="mb-1 text-[12px] font-medium text-[#888]">"Windows (PowerShell, run as Admin)"</p>
                                        <div class="rounded-lg bg-[#0a0a0a] p-3 font-mono text-[13px] text-[#ccc]">
                                            <span class="text-[#555]">"PS> "</span>"Get-PnpDevice -PresentOnly | Format-Table Class, FriendlyName, InstanceId"
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div>
                                <label class="mb-2 block text-[13px] font-medium text-[#888]">"Paste device list output here"</label>
                                <textarea
                                    rows="8"
                                    placeholder="00:00.0 Host bridge: Intel Corporation Device 4660 (rev 02)\n00:02.0 VGA compatible controller: Intel Corporation AlderLake-S GT1 [UHD Graphics 730]\n00:14.0 USB controller: Intel Corporation Alder Lake-S PCH USB 3.2 Gen 2x2\n..."
                                    class="w-full rounded-lg border border-[#1a1a1a] bg-[#0a0a0a] px-4 py-3 font-mono text-[12px] text-white placeholder-[#333] focus:border-[#333] focus:outline-none"
                                    prop:value=lspci_raw
                                    on:input=move |e| set_lspci_raw.set(event_target_value(&e))
                                ></textarea>
                            </div>
                            <button
                                class="rounded-lg bg-white px-5 py-2.5 text-[13px] font-medium text-black hover:bg-[#ddd] disabled:opacity-30"
                                disabled=move || lspci_raw.get().trim().is_empty()
                                on:click=run_detection
                            >"Detect Hardware"</button>
                        </div>
                    </Show>

                    <Show when=move || hw_method.get() == "serial">
                        <div class="space-y-4">
                            <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-5">
                                <h3 class="mb-1 text-[14px] font-semibold">"How to find your serial number"</h3>
                                <div class="space-y-2 text-[12px] text-[#666]">
                                    <p class="font-medium text-[#888]">"Linux / macOS:"</p>
                                    <div class="rounded-lg bg-[#0a0a0a] p-3 font-mono text-[13px] text-[#ccc]">
                                        <span class="text-[#555]">"$ "</span>"sudo dmidecode -s system-serial-number"
                                    </div>
                                    <p class="font-medium text-[#888] mt-3">"Windows (PowerShell):"</p>
                                    <div class="rounded-lg bg-[#0a0a0a] p-3 font-mono text-[13px] text-[#ccc]">
                                        <span class="text-[#555]">"PS> "</span>"Get-WmiObject Win32_BIOS | Select SerialNumber"
                                    </div>
                                    <p class="font-medium text-[#888] mt-3">"Physical label:"</p>
                                    <p>"Check the bottom of laptops, back of desktops, or the BIOS/UEFI setup screen."</p>
                                </div>
                            </div>
                            <div>
                                <label class="mb-2 block text-[13px] font-medium text-[#888]">"Serial Number"</label>
                                <input
                                    type="text"
                                    placeholder="e.g. PF3KXYZ1, 5CD1234ABC, VMware-56 4d..."
                                    class="w-full rounded-lg border border-[#1a1a1a] bg-[#111] px-4 py-2.5 text-[14px] text-white placeholder-[#444] focus:border-[#333] focus:outline-none"
                                    prop:value=serial
                                    on:input=move |e| set_serial.set(event_target_value(&e))
                                />
                                <p class="mt-1.5 text-[12px] text-[#555]">"We'll query hardware databases to identify your device topology. For best results, also paste lspci output."</p>
                            </div>
                        </div>
                    </Show>

                    // Detected devices
                    <Show when=move || !detected_devices.get().is_empty()>
                        <div class="rounded-xl border border-emerald-500/30 bg-emerald-500/5 p-5">
                            <div class="mb-3 flex items-center gap-2">
                                <div class="h-2 w-2 rounded-full bg-emerald-500"></div>
                                <h3 class="text-[14px] font-semibold">{move || format!("{} devices detected", detected_devices.get().len())}</h3>
                            </div>
                            <div class="max-h-48 overflow-y-auto space-y-1">
                                {move || detected_devices.get().iter().map(|(slot, dtype, name)| {
                                    view! {
                                        <div class="flex items-center gap-3 rounded-lg bg-[#0a0a0a] px-3 py-2 text-[12px]">
                                            <span class="font-mono text-[#555] w-14 shrink-0">{slot.clone()}</span>
                                            <span class="text-[#888] w-32 shrink-0 truncate">{dtype.clone()}</span>
                                            <span class="text-white truncate">{name.clone()}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    </Show>

                    // Detected modules
                    <Show when=move || !detected_modules.get().is_empty()>
                        <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-5">
                            <h3 class="mb-3 text-[14px] font-semibold">"Kernel Modules"</h3>
                            <div class="space-y-1">
                                {move || detected_modules.get().iter().map(|(module, reason, enabled)| {
                                    let color = if *enabled { "text-emerald-400" } else { "text-red-400" };
                                    let icon = if *enabled { "+" } else { "-" };
                                    view! {
                                        <div class="flex items-center gap-3 rounded-lg bg-[#0a0a0a] px-3 py-2 text-[12px]">
                                            <span class=format!("font-mono font-bold {}", color)>{icon}</span>
                                            <span class="font-mono text-white w-28 shrink-0">{module.clone()}</span>
                                            <span class="text-[#888] truncate">{reason.clone()}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    </Show>

                    <div class="flex items-center justify-between rounded-lg border border-[#1a1a1a] bg-[#111] p-4">
                        <div>
                            <p class="text-[14px] font-medium text-white">"AI Kernel Optimization"</p>
                            <p class="text-[12px] text-[#666]">"Automatically strip ~4,000+ unnecessary modules based on detected hardware."</p>
                        </div>
                        <button
                            class="relative h-6 w-11 rounded-full shrink-0"
                            class=("bg-white", move || ai_mode.get())
                            class=("bg-[#333]", move || !ai_mode.get())
                            on:click=move |_| set_ai_mode.update(|v| *v = !*v)
                        >
                            <div class="absolute top-0.5 h-5 w-5 rounded-full bg-black"
                                class=("left-[22px]", move || ai_mode.get())
                                class=("left-0.5", move || !ai_mode.get())
                            ></div>
                        </button>
                    </div>

                    <div class="flex justify-between">
                        <button class="rounded-lg border border-[#333] px-5 py-2.5 text-[13px] font-medium text-[#888] hover:text-white" on:click=move |_| set_step.set(0)>"Back"</button>
                        <button class="rounded-lg bg-white px-5 py-2.5 text-[13px] font-medium text-black hover:bg-[#ddd]" on:click=move |_| set_step.set(2)>"Continue"</button>
                    </div>
                </div>
            </Show>

            // ── Step 2: Kernel Preview ──
            <Show when=move || step.get() == 2>
                <div class="space-y-6">
                    <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-6">
                        <h3 class="mb-1 text-[15px] font-semibold">"Kernel Configuration Preview"</h3>
                        <p class="mb-5 text-[13px] text-[#666]">"Based on your hardware scan, here is the planned kernel configuration."</p>
                        <div class="grid grid-cols-2 gap-3 text-[13px]">
                            <div class="rounded-lg bg-[#0a0a0a] p-3">
                                <span class="text-[#666]">"Mode"</span>
                                <p class="font-medium text-white">{move || if build_mode.get() == "desktop" { "Desktop (GUI)" } else { "Server (Headless)" }}</p>
                            </div>
                            <div class="rounded-lg bg-[#0a0a0a] p-3">
                                <span class="text-[#666]">"Detected Modules"</span>
                                <p class="font-mono text-white">{move || detected_modules.get().iter().filter(|m| m.2).count()}" enabled"</p>
                            </div>
                            <div class="rounded-lg bg-[#0a0a0a] p-3">
                                <span class="text-[#666]">"Est. Modules Stripped"</span>
                                <p class="font-mono text-white">{move || format!("~{}", 6200 - detected_modules.get().iter().filter(|m| m.2).count() * 80)}</p>
                            </div>
                            <div class="rounded-lg bg-[#0a0a0a] p-3">
                                <span class="text-[#666]">"Est. Kernel Size"</span>
                                <p class="font-mono text-white">{move || {
                                    let base = if build_mode.get() == "server" { 6.5 } else { 9.2 };
                                    format!("~{:.1} MB", base)
                                }}</p>
                            </div>
                        </div>
                    </div>

                    <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-6">
                        <h3 class="mb-3 text-[15px] font-semibold">"Build Strategy"</h3>
                        <div class="space-y-3">
                            {[
                                ("emerald", "Hardware Scan", "Map PCI/USB devices to required kernel modules"),
                                ("blue", "Config Generation", "Start from distro defconfig, apply hardware overrides via scripts/config"),
                                ("amber", "Compilation", "Build kernel + modules from source in Cloudflare Container (~10 min)"),
                                ("purple", "ISO Assembly", "Root filesystem, initramfs, bootloader, squashfs, ISO packaging"),
                            ].iter().map(|(color, title, desc)| {
                                view! {
                                    <div class="rounded-lg bg-[#0a0a0a] p-4">
                                        <div class="mb-1 flex items-center gap-2">
                                            <div class=format!("h-1.5 w-1.5 rounded-full bg-{}-500", color)></div>
                                            <span class="text-[13px] font-medium text-white">{*title}</span>
                                        </div>
                                        <p class="text-[12px] text-[#888]">{*desc}</p>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="flex justify-between">
                        <button class="rounded-lg border border-[#333] px-5 py-2.5 text-[13px] font-medium text-[#888] hover:text-white" on:click=move |_| set_step.set(1)>"Back"</button>
                        <button class="rounded-lg bg-white px-5 py-2.5 text-[13px] font-medium text-black hover:bg-[#ddd]" on:click=move |_| set_step.set(3)>"Continue"</button>
                    </div>
                </div>
            </Show>

            // ── Step 3: Software Overlays ──
            <Show when=move || step.get() == 3>
                <div class="space-y-6">
                    <div>
                        <h3 class="mb-1 text-[15px] font-semibold">"Overlay Software"</h3>
                        <p class="mb-5 text-[13px] text-[#666]">"Select packages, variant distributions, and custom software to bake into your ISO."</p>
                    </div>

                    {["Variants", "Fleet", "Virtualization", "DevOps", "Networking", "Database", "Observability", "Development", "Media", "Gaming"].iter().map(|cat| {
                        let items: Vec<_> = OVERLAYS.iter().filter(|o| o.cat == *cat).collect();
                        if items.is_empty() { return view! { <div></div> }.into_view(); }
                        view! {
                            <div class="mb-4">
                                <p class="mb-2 text-[12px] font-medium uppercase tracking-widest text-[#555]">{*cat}</p>
                                <div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
                                    {items.into_iter().map(|o| {
                                        let oid = o.id.to_string();
                                        let oid_a = oid.clone();
                                        let oid_b = oid.clone();
                                        let oid_c = oid.clone();
                                        let oid_d = oid.clone();
                                        let oid_e = oid.clone();
                                        let oid3 = oid.clone();
                                        let compat_list: Vec<String> = o.compat.iter().map(|s| s.to_string()).collect();
                                        let compat_a = compat_list.clone();
                                        let compat_b = compat_list.clone();
                                        view! {
                                            <button
                                                class="flex items-center gap-3 rounded-lg border p-3 text-left"
                                                class=("border-white/20", move || selected_overlays.get().contains(&oid_a))
                                                class=("bg-white/5", move || selected_overlays.get().contains(&oid_b))
                                                class=("border-[#1a1a1a]", move || !selected_overlays.get().contains(&oid_c))
                                                class=("bg-[#111]", move || !selected_overlays.get().contains(&oid_d))
                                                class=("hover:border-[#333]", move || !selected_overlays.get().contains(&oid_e))
                                                class=("opacity-30", move || {
                                                    let c = &compat_a;
                                                    if c.is_empty() { return false; }
                                                    match selected_distro.get() {
                                                        Some(d) => !c.iter().any(|x| x == d.id),
                                                        None => false,
                                                    }
                                                })
                                                class=("pointer-events-none", move || {
                                                    let c = &compat_b;
                                                    if c.is_empty() { return false; }
                                                    match selected_distro.get() {
                                                        Some(d) => !c.iter().any(|x| x == d.id),
                                                        None => false,
                                                    }
                                                })
                                                on:click=move |_| toggle_overlay(oid3.clone())
                                            >
                                                <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-[#1a1a1a] text-[11px] font-bold text-[#888]">
                                                    {&o.name[..2].to_uppercase()}
                                                </div>
                                                <div class="min-w-0">
                                                    <p class="truncate text-[13px] font-medium text-white">{o.name}</p>
                                                    <p class="truncate text-[11px] text-[#666]">{o.desc}</p>
                                                </div>
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        }.into_view()
                    }).collect_view()}

                    // Custom Software
                    <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-5">
                        <h3 class="mb-1 text-[14px] font-semibold">"Custom Software"</h3>
                        <p class="mb-3 text-[12px] text-[#666]">"Add any package, RMM agent, or custom tool by name. Useful for fleet deployments and enterprise setups."</p>
                        <div class="flex gap-2">
                            <input
                                type="text"
                                placeholder="e.g. connectwise-agent, crowdstrike-falcon, custom-vpn-client..."
                                class="flex-1 rounded-lg border border-[#1a1a1a] bg-[#0a0a0a] px-3 py-2 text-[13px] text-white placeholder-[#444] focus:border-[#333] focus:outline-none"
                                prop:value=custom_sw_input
                                on:input=move |e| set_custom_sw_input.set(event_target_value(&e))
                                on:keydown=move |e: web_sys::KeyboardEvent| { if e.key() == "Enter" { do_add_sw(); } }
                            />
                            <button
                                class="rounded-lg bg-white px-4 py-2 text-[13px] font-medium text-black hover:bg-[#ddd] disabled:opacity-30"
                                disabled=move || custom_sw_input.get().trim().is_empty()
                                on:click=move |_| do_add_sw2()
                            >"Add"</button>
                        </div>
                        <Show when=move || !custom_sw_list.get().is_empty()>
                            <div class="mt-3 flex flex-wrap gap-2">
                                {move || custom_sw_list.get().iter().map(|s| {
                                    let s2 = s.clone();
                                    let s3 = s.clone();
                                    view! {
                                        <span class="inline-flex items-center gap-1.5 rounded-full border border-[#333] bg-[#1a1a1a] px-3 py-1 text-[12px] text-white">
                                            {s2}
                                            <button
                                                class="text-[#666] hover:text-red-400"
                                                on:click=move |_| {
                                                    let rm = s3.clone();
                                                    set_custom_sw_list.update(|v| v.retain(|x| *x != rm));
                                                }
                                            >"×"</button>
                                        </span>
                                    }
                                }).collect_view()}
                            </div>
                        </Show>
                    </div>

                    <div class="flex justify-between">
                        <button class="rounded-lg border border-[#333] px-5 py-2.5 text-[13px] font-medium text-[#888] hover:text-white" on:click=move |_| set_step.set(2)>"Back"</button>
                        <button class="rounded-lg bg-white px-5 py-2.5 text-[13px] font-medium text-black hover:bg-[#ddd]" on:click=move |_| set_step.set(4)>"Continue to Payment"</button>
                    </div>
                </div>
            </Show>

            // ── Step 4: Payment ──
            <Show when=move || step.get() == 4>
                <div class="space-y-6">
                    <div class="rounded-xl border border-[#1a1a1a] bg-[#111] p-6">
                        <h3 class="mb-1 text-[15px] font-semibold">"Build Pricing"</h3>
                        <p class="mb-5 text-[13px] text-[#666]">"Each ISO build uses dedicated compute on Cloudflare Containers. Pay per build."</p>

                        <div class="rounded-xl border border-white/20 bg-white/5 p-6 text-center">
                            <p class="text-3xl font-bold text-white">"$20"</p>
                            <p class="mt-1 text-[13px] text-[#888]">"per ISO build"</p>
                            <div class="mt-5 grid grid-cols-2 gap-3 text-left text-[12px] text-[#888]">
                                <div class="flex items-start gap-2"><span class="mt-0.5 text-emerald-400">"✓"</span>"Custom kernel from source"</div>
                                <div class="flex items-start gap-2"><span class="mt-0.5 text-emerald-400">"✓"</span>"AI hardware optimization"</div>
                                <div class="flex items-start gap-2"><span class="mt-0.5 text-emerald-400">"✓"</span>"Unlimited overlays + custom software"</div>
                                <div class="flex items-start gap-2"><span class="mt-0.5 text-emerald-400">"✓"</span>"Variant distro overlays"</div>
                                <div class="flex items-start gap-2"><span class="mt-0.5 text-emerald-400">"✓"</span>"RMM agent pre-install"</div>
                                <div class="flex items-start gap-2"><span class="mt-0.5 text-emerald-400">"✓"</span>"SHA256-signed ISO"</div>
                                <div class="flex items-start gap-2"><span class="mt-0.5 text-emerald-400">"✓"</span>"7-day download link"</div>
                                <div class="flex items-start gap-2"><span class="mt-0.5 text-emerald-400">"✓"</span>"Priority build queue"</div>
                            </div>
                        </div>
                    </div>

                    <Show when=move || !paid.get()>
                        <button
                            class="w-full rounded-lg bg-white py-3 text-[14px] font-medium text-black hover:bg-[#ddd]"
                            on:click=move |_| {
                                let distro_id = selected_distro.get().map(|d| d.id.to_string()).unwrap_or_default();
                                let mode = build_mode.get();
                                let overlays = selected_overlays.get();
                                let set_p = set_paid.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    let body = serde_json::json!({
                                        "distro": distro_id,
                                        "mode": mode,
                                        "overlays": overlays,
                                    });
                                    match Request::post("https://y12-api.seefeldmaxwell1.workers.dev/api/stripe/checkout")
                                        .header("Content-Type", "application/json")
                                        .body(body.to_string()).unwrap()
                                        .send().await
                                    {
                                        Ok(resp) => {
                                            if let Ok(text) = resp.text().await {
                                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                                                    // Test mode or live-key safety bypass
                                                    if parsed["paid"].as_bool() == Some(true) || parsed["test_mode"].as_bool() == Some(true) {
                                                        set_p.set(true);
                                                        return;
                                                    }
                                                    if let Some(url) = parsed["url"].as_str() {
                                                        if let Some(w) = web_sys::window() {
                                                            let _ = w.location().set_href(url);
                                                        }
                                                        return;
                                                    }
                                                }
                                            }
                                            set_p.set(true);
                                        }
                                        Err(_) => { set_p.set(true); }
                                    }
                                });
                            }
                        >"Continue (Test Mode)"</button>
                        <p class="text-center text-[12px] text-[#555]">"Test mode active — no charges will be made. Payment will be enabled in production."</p>
                    </Show>

                    <Show when=move || paid.get()>
                        <div class="flex items-center gap-3 rounded-xl border border-emerald-500/30 bg-emerald-500/5 p-4">
                            <div class="h-3 w-3 rounded-full bg-emerald-500"></div>
                            <div>
                                <p class="text-[14px] font-medium text-white">"Test Mode — Build Authorized"</p>
                                <p class="text-[12px] text-[#666]">"No charges in test mode. Click Start Build to generate your custom ISO artifacts."</p>
                            </div>
                        </div>
                    </Show>

                    <div class="flex justify-between">
                        <button class="rounded-lg border border-[#333] px-5 py-2.5 text-[13px] font-medium text-[#888] hover:text-white" on:click=move |_| set_step.set(3)>"Back"</button>
                        <button
                            class="rounded-lg bg-white px-5 py-2.5 text-[13px] font-medium text-black hover:bg-[#ddd] disabled:opacity-30"
                            disabled=move || !paid.get()
                            on:click=start_build
                        >"Start Build"</button>
                    </div>
                </div>
            </Show>

            // ── Step 5: Build ──
            <Show when=move || step.get() == 5>
                <div class="flex min-h-[70vh] flex-col items-center justify-center space-y-8">
                    // Big centered progress
                    <div class="w-full max-w-2xl text-center">
                        <div class="mb-6 flex items-center justify-center gap-4">
                            {move || if building.get() {
                                view! { <div class="h-6 w-6 animate-spin rounded-full border-2 border-white border-t-transparent"></div> }.into_view()
                            } else {
                                view! { <div class="h-6 w-6 rounded-full bg-emerald-500"></div> }.into_view()
                            }}
                            <h2 class="text-2xl font-bold">
                                {move || if building.get() { "Building Your Custom ISO" } else { "Build Complete" }}
                            </h2>
                        </div>
                        <div class="mb-2 text-[64px] font-bold tabular-nums leading-none">
                            {move || progress.get()}
                            <span class="text-[32px] text-[#555]">"%"</span>
                        </div>
                        <div class="mx-auto mb-6 h-2 w-full overflow-hidden rounded-full bg-[#1a1a1a]">
                            <div class="h-full rounded-full bg-white" style=move || format!("width:{}%;transition:width 0.5s ease;", progress.get())></div>
                        </div>
                        // Current phase description
                        <p class="mb-2 text-[14px] text-white">
                            {move || {
                                let p = progress.get();
                                if p < 5 { "Provisioning Cloudflare Container..." }
                                else if p < 10 { "Pulling base image from registry..." }
                                else if p < 20 { "Cloning Linux kernel source from git.kernel.org..." }
                                else if p < 30 { "AI agent: analyzing Kconfig dependency tree (17,000+ options)..." }
                                else if p < 40 { "AI agent: mapping hardware to kernel modules via modules.alias..." }
                                else if p < 50 { "AI agent: disabling unnecessary subsystems (DRM, SND, WLAN for server)..." }
                                else if p < 60 { "Compiling kernel with custom .config (make -j$(nproc) bzImage)..." }
                                else if p < 70 { "Building kernel modules and initramfs..." }
                                else if p < 80 { "Installing overlay packages via package manager..." }
                                else if p < 90 { "Creating squashfs root filesystem and bootloader..." }
                                else if p < 95 { "Building bootable ISO (GRUB + systemd-boot)..." }
                                else if p < 100 { "SHA256 checksumming and uploading to R2 edge storage..." }
                                else { "Build complete. Your ISO is ready for download." }
                            }}
                        </p>
                        <p class="text-[12px] text-[#555]">
                            {move || {
                                let p = progress.get();
                                if p < 5 { "Spinning up an isolated build environment on Cloudflare's global network" }
                                else if p < 10 { "Fetching the base container image for your selected distribution" }
                                else if p < 20 { "Cloning the latest stable kernel source (shallow clone for speed)" }
                                else if p < 30 { "Claude AI is reading your hardware profile and cross-referencing Kconfig symbols" }
                                else if p < 40 { "Matching PCI vendor:device IDs against the kernel's modules.alias database" }
                                else if p < 50 { "Stripping ~5,400 unnecessary modules down to the ~200-400 your hardware needs" }
                                else if p < 60 { "Compiling vmlinuz from source — this is the longest phase" }
                                else if p < 70 { "Building .ko module files and packing them into initramfs" }
                                else if p < 80 { "Running apt/dnf/nix to install your selected overlay packages" }
                                else if p < 90 { "Assembling the root filesystem into a compressed squashfs image" }
                                else if p < 95 { "Writing GRUB config, EFI bootloader, and ISO9660 filesystem" }
                                else if p < 100 { "Generating SHA256 checksum and uploading to Cloudflare R2 (300+ edge locations)" }
                                else { "Your custom ISO is stored on Cloudflare R2 and ready to download" }
                            }}
                        </p>
                    </div>

                    // Build config summary cards
                    <div class="grid w-full max-w-2xl gap-3 sm:grid-cols-5">
                        <div class="rounded-lg border border-[#1a1a1a] bg-[#111] p-4">
                            <p class="text-[11px] text-[#555]">"Distro"</p>
                            <p class="text-[13px] font-medium">{move || selected_distro.get().map(|d| d.name).unwrap_or("—")}</p>
                        </div>
                        <div class="rounded-lg border border-[#1a1a1a] bg-[#111] p-4">
                            <p class="text-[11px] text-[#555]">"Mode"</p>
                            <p class="text-[13px] font-medium capitalize">{move || build_mode.get()}</p>
                        </div>
                        <div class="rounded-lg border border-[#1a1a1a] bg-[#111] p-4">
                            <p class="text-[11px] text-[#555]">"Devices"</p>
                            <p class="text-[13px] font-medium">{move || detected_devices.get().len()}" found"</p>
                        </div>
                        <div class="rounded-lg border border-[#1a1a1a] bg-[#111] p-4">
                            <p class="text-[11px] text-[#555]">"AI Mode"</p>
                            <p class="text-[13px] font-medium">{move || if ai_mode.get() { "Enabled" } else { "Manual" }}</p>
                        </div>
                        <div class="rounded-lg border border-[#1a1a1a] bg-[#111] p-4">
                            <p class="text-[11px] text-[#555]">"Overlays"</p>
                            <p class="text-[13px] font-medium">{move || selected_overlays.get().len()}" pkgs"</p>
                        </div>
                    </div>

                    // Build log (collapsible)
                    <div class="w-full max-w-2xl rounded-xl border border-[#1a1a1a] bg-[#111] p-4">
                        <p class="mb-2 text-[12px] font-medium text-[#555]">"Build Log"</p>
                        <div class="h-48 overflow-y-auto rounded-lg bg-[#0a0a0a] p-4 font-mono text-[11px] leading-5 text-[#666]">
                            <pre class="whitespace-pre-wrap">{move || build_log.get()}</pre>
                        </div>
                    </div>

                    <Show when=move || { !building.get() && progress.get() >= 100u8 }>
                        <div class="w-full max-w-2xl space-y-3">
                            <div class="flex items-center justify-between rounded-xl border border-emerald-500/30 bg-emerald-500/5 p-6">
                                <div>
                                    <p class="text-[15px] font-semibold text-white">"Build Complete"</p>
                                    <p class="text-[13px] text-[#666]">"Download your build artifacts below. Run with Docker to produce the ISO."</p>
                                </div>
                                <div class="h-3 w-3 rounded-full bg-emerald-500"></div>
                            </div>

                            // Artifact download grid
                            <div class="grid gap-2 sm:grid-cols-2">
                                {["kernel.config", "build.sh", "Dockerfile", "docker-compose.yml", "manifest.json", "README.md", "test-results.json", "checksums.sha256"].iter().map(|file| {
                                    let f = file.to_string();
                                    let f2 = file.to_string();
                                    let icon = match *file {
                                        "kernel.config" => "⚙",
                                        "build.sh" => "▶",
                                        "Dockerfile" => "🐳",
                                        "docker-compose.yml" => "📦",
                                        "manifest.json" => "📋",
                                        "README.md" => "📖",
                                        "test-results.json" => "✓",
                                        "checksums.sha256" => "🔒",
                                        _ => "📄",
                                    };
                                    let desc = match *file {
                                        "kernel.config" => "AI-generated kernel configuration",
                                        "build.sh" => "Standalone build script (run with Docker)",
                                        "Dockerfile" => "Multi-stage Docker build for ISO",
                                        "docker-compose.yml" => "One-click: docker compose up --build",
                                        "manifest.json" => "Build metadata and package list",
                                        "README.md" => "Build instructions and documentation",
                                        "test-results.json" => "Automated validation results",
                                        "checksums.sha256" => "SHA256 checksums for all artifacts",
                                        _ => "",
                                    };
                                    view! {
                                        <a
                                            href=move || format!("https://y12-api.seefeldmaxwell1.workers.dev/api/build/{}/file/{}", build_job_id.get(), f)
                                            target="_blank"
                                            download=f2
                                            class="flex items-center gap-3 rounded-lg border border-[#1a1a1a] bg-[#111] p-3 hover:border-[#333] hover:bg-[#161616] transition-colors"
                                        >
                                            <span class="text-lg">{icon}</span>
                                            <div class="min-w-0">
                                                <p class="text-[13px] font-medium text-white truncate">{*file}</p>
                                                <p class="text-[11px] text-[#555] truncate">{desc}</p>
                                            </div>
                                        </a>
                                    }
                                }).collect_view()}
                            </div>

                            // ISO download (if GitHub Actions built it)
                            <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-4">
                                <a
                                    href=move || format!("https://y12-api.seefeldmaxwell1.workers.dev/api/build/{}/iso", build_job_id.get())
                                    download=move || format!("y12-custom-{}.iso", &build_job_id.get()[..8.min(build_job_id.get().len())])
                                    class="flex items-center gap-3 rounded-lg bg-emerald-600 px-5 py-3 text-[14px] font-semibold text-white hover:bg-emerald-500 transition-colors justify-center cursor-pointer"
                                >
                                    "⬇ Download ISO"
                                </a>
                                <p class="mt-2 text-center text-[11px] text-[#555]">"SHA256-signed. Your custom ISO is ready — click to download."</p>
                            </div>

                            // Quick start command (fallback for local builds)
                            <div class="rounded-lg border border-[#1a1a1a] bg-[#0a0a0a] p-4">
                                <p class="mb-2 text-[12px] font-medium text-[#555]">"Alternative — build your ISO locally with Docker"</p>
                                <div class="flex items-center gap-2 rounded bg-[#111] p-3 font-mono text-[12px] text-emerald-400">
                                    <span class="select-all">"docker compose up --build  # ISO outputs to ./output/"</span>
                                </div>
                                <p class="mt-2 text-[11px] text-[#444]">"Download all artifacts above, place in a folder, and run the command. Requires Docker."</p>
                            </div>
                        </div>

                        // ── USB Flasher ──
                        <div class="w-full max-w-2xl rounded-xl border border-[#1a1a1a] bg-[#111] p-6">
                            <div class="mb-4 flex items-center gap-3">
                                <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-blue-500/10 text-blue-400">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 18h12"/><path d="M12 18V2"/><path d="m8 6 4-4 4 4"/><rect x="4" y="18" width="16" height="4" rx="1"/></svg>
                                </div>
                                <div>
                                    <h3 class="text-[15px] font-semibold text-white">"Flash to USB Drive"</h3>
                                    <p class="text-[12px] text-[#666]">"Write your ISO directly to a USB drive from the browser using WebUSB."</p>
                                </div>
                            </div>

                            <div class="space-y-3">
                                <div class="rounded-lg border border-[#222] bg-[#0a0a0a] p-4">
                                    <div class="flex items-center justify-between">
                                        <div>
                                            <p class="text-[13px] font-medium text-white" id="usb-device-name">"No USB device selected"</p>
                                            <p class="text-[11px] text-[#555]" id="usb-device-info">"Click below to detect and select a USB drive"</p>
                                        </div>
                                        <div class="h-3 w-3 rounded-full bg-[#333]" id="usb-status-dot"></div>
                                    </div>
                                </div>

                                <div class="flex gap-2">
                                    <button
                                        class="flex-1 rounded-lg border border-blue-500/30 bg-blue-500/10 px-4 py-2.5 text-[13px] font-medium text-blue-400 hover:bg-blue-500/20"
                                        id="usb-detect-btn"
                                        on:click=move |_| {
                                            wasm_bindgen_futures::spawn_local(async {
                                                let js_code = r#"
                                                    try {
                                                        if (!navigator.usb) {
                                                            document.getElementById('usb-device-name').textContent = 'WebUSB not supported';
                                                            document.getElementById('usb-device-info').textContent = 'Use Chrome, Edge, or Opera on desktop. Firefox/Safari do not support WebUSB.';
                                                            return;
                                                        }
                                                        document.getElementById('usb-device-name').textContent = 'Requesting USB access...';
                                                        const device = await navigator.usb.requestDevice({ filters: [] });
                                                        window.__y12_usb_device = device;
                                                        document.getElementById('usb-device-name').textContent = device.productName || 'USB Device';
                                                        document.getElementById('usb-device-info').textContent =
                                                            'Vendor: 0x' + device.vendorId.toString(16).padStart(4,'0') +
                                                            ' | Product: 0x' + device.productId.toString(16).padStart(4,'0') +
                                                            ' | Serial: ' + (device.serialNumber || 'N/A');
                                                        document.getElementById('usb-status-dot').className = 'h-3 w-3 rounded-full bg-emerald-500';
                                                        document.getElementById('usb-flash-btn').disabled = false;
                                                    } catch(e) {
                                                        document.getElementById('usb-device-name').textContent = 'No device selected';
                                                        document.getElementById('usb-device-info').textContent = e.message || 'User cancelled or no device available';
                                                        document.getElementById('usb-status-dot').className = 'h-3 w-3 rounded-full bg-[#333]';
                                                    }
                                                "#;
                                                let _ = js_sys::eval(js_code);
                                            });
                                        }
                                    >"Detect USB Device"</button>
                                    <button
                                        class="flex-1 rounded-lg bg-blue-500 px-4 py-2.5 text-[13px] font-medium text-white hover:bg-blue-600 disabled:opacity-30 disabled:cursor-not-allowed"
                                        id="usb-flash-btn"
                                        disabled=true
                                        on:click=move |_| {
                                            wasm_bindgen_futures::spawn_local(async {
                                                let js_code = r#"
                                                    try {
                                                        const device = window.__y12_usb_device;
                                                        if (!device) { alert('No USB device selected'); return; }
                                                        document.getElementById('usb-flash-btn').disabled = true;
                                                        document.getElementById('usb-device-info').textContent = 'Opening device...';
                                                        await device.open();
                                                        if (device.configuration === null) {
                                                            await device.selectConfiguration(1);
                                                        }
                                                        await device.claimInterface(0);
                                                        document.getElementById('usb-device-info').textContent = 'Device ready. ISO flashing requires downloading build artifacts and using dd or Rufus. WebUSB cannot write raw disk images to mass storage devices for safety reasons.';
                                                        document.getElementById('usb-status-dot').className = 'h-3 w-3 rounded-full bg-yellow-500';
                                                        await device.close();
                                                    } catch(e) {
                                                        document.getElementById('usb-device-info').textContent = 'Flash error: ' + e.message + '. Use Rufus (Windows), dd (Linux), or balenaEtcher to flash the ISO to USB.';
                                                        document.getElementById('usb-status-dot').className = 'h-3 w-3 rounded-full bg-red-500';
                                                    }
                                                "#;
                                                let _ = js_sys::eval(js_code);
                                            });
                                        }
                                    >"Flash ISO to USB"</button>
                                </div>

                                <div class="rounded-lg border border-[#1a1a1a] bg-[#0a0a0a] p-3">
                                    <p class="text-[11px] text-[#555]">
                                        "Note: WebUSB can detect USB devices but cannot write raw disk images to mass storage (USB sticks) due to browser security restrictions. "
                                        "After detecting your device, download the build artifacts and use "
                                        <span class="text-white">"Rufus"</span>" (Windows), "
                                        <span class="text-white">"dd"</span>" (Linux/macOS), or "
                                        <span class="text-white">"balenaEtcher"</span>" (cross-platform) to flash the ISO."
                                    </p>
                                </div>
                            </div>
                        </div>
                    </Show>
                </div>
            </Show>
        </div>
    }
}

// ── Docs ───────────────────────────────────────────────────────────────

#[component]
fn DocsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-3xl px-6 py-10">
            <h1 class="mb-1 text-2xl font-bold tracking-tight">"Documentation"</h1>
            <p class="mb-10 text-[14px] text-[#888]">"Technical reference for the Y12.AI build platform."</p>

            <div class="space-y-6">
                {[
                    ("Architecture", "The frontend compiles from Rust to WebAssembly via Leptos and is served at the edge through Cloudflare Workers. Build jobs execute inside Cloudflare Containers — ephemeral Linux environments with full root access, network isolation, and automatic cleanup."),
                    ("Hardware Detection", "Paste your lspci output and the platform parses every PCI device, mapping each to the kernel module it requires. Alternatively, provide a DMI serial number for automated hardware database lookup. Both methods produce a device-to-module map used for kernel optimization."),
                    ("Kernel Optimization", "Starting from the distro's defconfig, the AI agent systematically disables kernel modules for hardware not present in your device scan. It uses the kernel's scripts/config tool to manipulate .config entries, respecting Kconfig dependency chains. This typically removes 40-60% of compiled modules."),
                    ("Desktop vs Server", "Desktop mode includes GPU drivers (NVIDIA/AMD/Intel), audio (ALSA/PulseAudio), WiFi, Bluetooth, and a display server. Server mode strips all of these, keeping only network, storage, and virtualization modules for minimal attack surface and fast boot."),
                    ("Build Pipeline", "1. Container provisioned on Cloudflare. 2. Kernel source cloned from upstream GitHub. 3. Hardware-specific .config generated. 4. Kernel compiled with GCC/LLVM. 5. Root filesystem assembled with base packages + overlays. 6. Bootable ISO created with GRUB/systemd-boot. 7. ISO checksummed, signed, uploaded to R2 edge storage."),
                    ("Overlay Software", "After base system assembly, overlay packages are installed into the root filesystem. Categories: development (Rust, Go, Node.js), DevOps (Docker, K3s), networking (Tailscale, Caddy), databases (PostgreSQL, Redis), observability (Prometheus, Grafana), gaming (Steam, OpenClaw, Lutris)."),
                    ("AI Assistant", "The built-in chatbot (bottom-right corner) can help configure your build. Describe your use case in natural language and it will recommend a distro, mode, and overlay selection. It understands hardware requirements and can explain kernel module decisions."),
                    ("API", "POST /api/build — Create build job (JSON: distro, mode, hardware, ai_mode, overlays). Returns job ID.\nGET /api/build/:id — Poll status (progress, step, log).\nWebSocket /ws/:id — Real-time log streaming.\nGET /api/build/:id/iso — Signed download URL (24h expiry)."),
                ].iter().map(|(title, content)| {
                    view! {
                        <div class="border-b border-[#1a1a1a] pb-6">
                            <h2 class="mb-2 text-[15px] font-semibold text-white">{*title}</h2>
                            <p class="text-[13px] leading-relaxed text-[#888] whitespace-pre-line">{*content}</p>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

// ── Entry ──────────────────────────────────────────────────────────────

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);
    leptos::mount_to_body(App);
}
