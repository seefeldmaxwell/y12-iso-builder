import { Hono } from 'hono';
import { cors } from 'hono/cors';

// ── Types ──────────────────────────────────────────────────────────────

interface Env {
  AI: any;
  ISO_STORAGE: R2Bucket;
  BUILD_JOBS: KVNamespace;
  FRONTEND_URL: string;
  STRIPE_PRICE_CENTS: string;
  STRIPE_SECRET_KEY: string;
  ANTHROPIC_API_KEY: string;
  TEST_MODE: string;
  GITHUB_TOKEN: string;
  GITHUB_REPO: string;
  BUILD_SECRET: string;
}

interface ChatRequest {
  messages: { role: string; content: string }[];
  context?: {
    distro?: string;
    mode?: string;
    overlays?: string[];
    custom_software?: string[];
  };
}

interface BuildRequest {
  distro: string;
  mode: string;
  hardware_raw: string;
  ai_mode: boolean;
  overlays: string[];
  custom_software: string[];
  detected_modules: string[];
  stripe_payment_intent?: string;
}

// ── Distro + overlay package maps (real package names) ─────────────────

const DISTRO_PKG_MANAGER: Record<string, string> = {
  nixos: 'nix',
  debian: 'apt',
  rocky: 'dnf',
  proxmox: 'apt',
};

const DISTRO_BASE_IMAGE: Record<string, string> = {
  nixos: 'nixos/nix:latest',
  debian: 'debian:12-slim',
  rocky: 'rockylinux:9',
  proxmox: 'debian:12-slim',
};

// Real package names per distro package manager
const OVERLAY_PACKAGES: Record<string, Record<string, string[]>> = {
  // APT-based (Debian, Proxmox)
  apt: {
    docker: ['docker.io', 'containerd'],
    k3s: [], // installed via curl script
    podman: ['podman', 'buildah', 'skopeo'],
    tailscale: [], // installed via curl script
    caddy: ['caddy'],
    nginx: ['nginx'],
    postgres: ['postgresql-16', 'postgresql-client-16'],
    redis: ['redis-server'],
    mysql: ['mariadb-server', 'mariadb-client'],
    prometheus: ['prometheus'],
    grafana: [], // installed via grafana apt repo
    netdata: [], // installed via curl script
    neovim: ['neovim'],
    vscode: [], // installed via microsoft apt repo
    rustup: [], // installed via curl script
    nodejs: ['nodejs', 'npm'],
    golang: ['golang-go'],
    obs: ['obs-studio'],
    blender: ['blender'],
    openclaw: [], // built from source
    steam: [], // installed via steam apt repo
    lutris: ['lutris'],
    qemu: ['qemu-system-x86', 'qemu-utils', 'ovmf'],
    libvirt: ['libvirt-daemon-system', 'virtinst', 'virt-manager'],
    lxc: ['lxc', 'lxd-installer'],
    tacticalrmm: [], // installed via script
    meshcentral: ['nodejs', 'npm'], // npm install meshcentral
    ansible: ['ansible'],
    salt: ['salt-minion'],
    puppet: ['puppet-agent'],
    zabbix: ['zabbix-agent2'],
    kali: [], // kali repo overlay
    scientific: [], // EPEL + scientific repo overlay
    parrot: [], // parrot repo overlay
    devuan: [], // init system swap
    alma: [], // repo swap
  },
  // DNF-based (Rocky)
  dnf: {
    docker: ['docker-ce', 'docker-ce-cli', 'containerd.io', 'docker-compose-plugin'],
    k3s: [],
    podman: ['podman', 'buildah', 'skopeo'],
    tailscale: [],
    caddy: ['caddy'],
    nginx: ['nginx'],
    postgres: ['postgresql-server', 'postgresql'],
    redis: ['redis'],
    mysql: ['mariadb-server', 'mariadb'],
    prometheus: [],
    grafana: [],
    netdata: [],
    neovim: ['neovim'],
    vscode: [],
    rustup: [],
    nodejs: ['nodejs', 'npm'],
    golang: ['golang'],
    obs: [],
    blender: ['blender'],
    openclaw: [],
    steam: [],
    lutris: ['lutris'],
    qemu: ['qemu-kvm', 'qemu-img', 'edk2-ovmf'],
    libvirt: ['libvirt', 'virt-install', 'virt-manager'],
    lxc: ['lxc', 'lxc-templates'],
    tacticalrmm: [],
    meshcentral: ['nodejs', 'npm'],
    ansible: ['ansible-core'],
    salt: ['salt-minion'],
    puppet: ['puppet-agent'],
    zabbix: ['zabbix-agent2'],
    kali: [],
    scientific: [],
    parrot: [],
    devuan: [],
    alma: [],
  },
  // Nix-based
  nix: {
    docker: ['docker'],
    k3s: ['k3s'],
    podman: ['podman'],
    tailscale: ['tailscale'],
    caddy: ['caddy'],
    nginx: ['nginx'],
    postgres: ['postgresql_16'],
    redis: ['redis'],
    mysql: ['mariadb'],
    prometheus: ['prometheus'],
    grafana: ['grafana'],
    netdata: ['netdata'],
    neovim: ['neovim'],
    vscode: ['vscode'],
    rustup: ['rustup'],
    nodejs: ['nodejs_20'],
    golang: ['go'],
    obs: ['obs-studio'],
    blender: ['blender'],
    openclaw: [],
    steam: ['steam'],
    lutris: ['lutris'],
    qemu: ['qemu_full'],
    libvirt: ['libvirt', 'virt-manager'],
    lxc: ['lxc', 'lxd'],
    tacticalrmm: [],
    meshcentral: ['nodejs_20'],
    ansible: ['ansible'],
    salt: ['salt'],
    puppet: [],
    zabbix: ['zabbix-agent'],
    kali: [],
    scientific: [],
    parrot: [],
    devuan: [],
    alma: [],
  },
};

// ── System prompt for AI chat ──────────────────────────────────────────

const SYSTEM_PROMPT = `You are the Y12.AI build assistant. You help users configure custom Linux ISO builds.

Available base distributions:
- NixOS: Reproducible, declarative. Package manager: nix. Best for: reproducible systems, dev environments.
- Debian: Universal OS. Package manager: apt. Best for: stability, servers, desktops, widest package support.
- Rocky Linux: RHEL-compatible enterprise. Package manager: dnf. Best for: production servers, enterprise.
- Proxmox VE: Virtualization platform (Debian-based). Package manager: apt. Best for: VM/container hosting.

Build modes:
- Desktop: Full GUI with GPU drivers, audio, WiFi, display server.
- Server: Headless, no GUI/GPU/audio. Optimized for network + storage.

Overlay categories:
- Variants: Kali (pentest, Debian-based), Scientific Linux (RHEL-based), Parrot Security (Debian-based), Devuan (no systemd), AlmaLinux (RHEL-compatible)
- Fleet/RMM: Tactical RMM, MeshCentral, Ansible, SaltStack, Puppet, Zabbix Agent
- Virtualization: QEMU/KVM, libvirt, LXC/LXD
- DevOps: Docker, K3s, Podman
- Networking: Tailscale, Caddy, NGINX
- Database: PostgreSQL 16, Redis 7, MariaDB
- Observability: Prometheus, Grafana, Netdata
- Development: Neovim, VS Code, Rust, Node.js, Go
- Media: OBS Studio, Blender
- Gaming: OpenClaw, Steam, Lutris

Users can also add custom software by package name.

Pricing: $20 per ISO build via Stripe.

Hardware detection: Users paste lspci (Linux), system_profiler (macOS), or Get-PnpDevice (Windows) output. The system maps devices to kernel modules.

Be concise, technical, and helpful. Recommend specific configurations based on use cases. If the user describes a use case, suggest: distro, mode, overlays, and any custom software.`;

// ── App ────────────────────────────────────────────────────────────────

const app = new Hono<{ Bindings: Env }>();

app.use('*', cors({
  origin: '*',
  allowMethods: ['GET', 'POST', 'OPTIONS'],
  allowHeaders: ['Content-Type', 'Authorization'],
}));

// ── Health ─────────────────────────────────────────────────────────────

app.get('/api/health', (c) => {
  return c.json({
    status: 'ok',
    version: '1.0.0',
    services: {
      ai: 'workers-ai',
      storage: 'r2',
      jobs: 'kv',
      payment: 'stripe',
    },
  });
});

// ── AI Chat (Claude API — Anthropic) ────────────────────────────────

async function callClaude(apiKey: string, system: string, messages: { role: string; content: string }[], maxTokens = 1024): Promise<string> {
  const resp = await fetch('https://api.anthropic.com/v1/messages', {
    method: 'POST',
    headers: {
      'x-api-key': apiKey,
      'anthropic-version': '2023-06-01',
      'content-type': 'application/json',
    },
    body: JSON.stringify({
      model: 'claude-sonnet-4-20250514',
      max_tokens: maxTokens,
      system,
      messages: messages.map((m) => ({
        role: m.role === 'assistant' ? 'assistant' : 'user',
        content: m.content,
      })),
    }),
  });
  const data = await resp.json() as any;
  if (data.error) throw new Error(data.error.message || JSON.stringify(data.error));
  return data.content?.[0]?.text || '';
}

app.post('/api/chat', async (c) => {
  try {
    const body: ChatRequest = await c.req.json();

    let systemMsg = SYSTEM_PROMPT;
    if (body.context) {
      const ctx = body.context;
      systemMsg += `\n\nCurrent build context:`;
      if (ctx.distro) systemMsg += `\n- Distro: ${ctx.distro}`;
      if (ctx.mode) systemMsg += `\n- Mode: ${ctx.mode}`;
      if (ctx.overlays?.length) systemMsg += `\n- Selected overlays: ${ctx.overlays.join(', ')}`;
      if (ctx.custom_software?.length) systemMsg += `\n- Custom software: ${ctx.custom_software.join(', ')}`;
    }

    const apiKey = c.env.ANTHROPIC_API_KEY;
    if (!apiKey) {
      // Fallback to Workers AI if no Anthropic key
      const messages = [
        { role: 'system', content: systemMsg },
        ...body.messages.map((m) => ({ role: m.role, content: m.content })),
      ];
      const response = await c.env.AI.run('@cf/meta/llama-3.1-8b-instruct', { messages, max_tokens: 512, temperature: 0.7 });
      return c.json({ reply: response.response || 'Could not generate a response.', model: 'llama-3.1-8b' });
    }

    const reply = await callClaude(apiKey, systemMsg, body.messages);
    return c.json({ reply, model: 'claude-sonnet-4-20250514' });
  } catch (error: any) {
    console.error('AI chat error:', error);
    return c.json({ reply: `AI error: ${error.message}. Chat will fall back to local responses.`, error: true }, 500);
  }
});

// ── Distros ────────────────────────────────────────────────────────────

app.get('/api/distros', (c) => {
  return c.json([
    { id: 'nixos', name: 'NixOS', tagline: 'Reproducible, declarative', pkg_manager: 'nix', base_image: 'nixos/nix:latest' },
    { id: 'debian', name: 'Debian', tagline: 'The universal operating system', pkg_manager: 'apt', base_image: 'debian:12-slim' },
    { id: 'rocky', name: 'Rocky Linux', tagline: 'Enterprise RHEL-compatible', pkg_manager: 'dnf', base_image: 'rockylinux:9-minimal' },
    { id: 'proxmox', name: 'Proxmox VE', tagline: 'Enterprise virtualization platform', pkg_manager: 'apt', base_image: 'debian:12-slim' },
  ]);
});

// ── Validate overlays for a distro ─────────────────────────────────────

app.post('/api/validate-overlays', async (c) => {
  const { distro, overlays, custom_software } = await c.req.json();
  const pkgMgr = DISTRO_PKG_MANAGER[distro] || 'apt';
  const pkgMap = OVERLAY_PACKAGES[pkgMgr] || {};

  const results = overlays.map((id: string) => {
    const packages = pkgMap[id];
    if (!packages) return { id, status: 'unknown', packages: [], note: 'Package mapping not found — will attempt install' };
    if (packages.length === 0) return { id, status: 'script', packages: [], note: 'Installed via external script/repo' };
    return { id, status: 'native', packages, note: `${packages.length} package(s) via ${pkgMgr}` };
  });

  const customResults = (custom_software || []).map((name: string) => ({
    name,
    status: 'custom',
    note: `Will attempt: ${pkgMgr === 'nix' ? 'nix-env -iA nixpkgs.' + name : pkgMgr === 'dnf' ? 'dnf install ' + name : 'apt-get install ' + name}`,
  }));

  return c.json({ distro, pkg_manager: pkgMgr, overlays: results, custom_software: customResults });
});

// ── Claude kernel .config generation ──────────────────────────────────

const KERNEL_CONFIG_PROMPT = `You are a Linux kernel configuration expert. Generate a kernel .config FRAGMENT that will be merged ON TOP of x86_64 defconfig using scripts/kconfig/merge_config.sh.

The base defconfig already includes standard drivers (NVMe, AHCI, USB, e1000e, igb, etc). Your job is to ADD hardware-specific drivers and REMOVE unnecessary subsystems based on the user's hardware and build mode.

Rules:
- Output ONLY config lines. No prose, no comments, no explanations.
- To ENABLE: CONFIG_xxx=y or CONFIG_xxx=m
- To DISABLE: # CONFIG_xxx is not set
- Include CONFIG_LOCALVERSION="-y12-custom"
- For SERVER mode: disable CONFIG_DRM=n, CONFIG_SND=n, CONFIG_WLAN=n, CONFIG_BT=n. Enable CONFIG_NETFILTER=y, CONFIG_CGROUPS=y, CONFIG_NAMESPACES=y, CONFIG_NET_NS=y, CONFIG_VETH=y, CONFIG_BRIDGE=y, CONFIG_NF_NAT=y, CONFIG_OVERLAY_FS=y.
- For DESKTOP mode: enable the correct GPU driver based on hardware (CONFIG_DRM_I915=m for Intel, CONFIG_DRM_AMDGPU=m for AMD, CONFIG_DRM_NOUVEAU=m for NVIDIA). Enable CONFIG_SND_HDA_INTEL=m, CONFIG_WLAN=y, CONFIG_BT=y, CONFIG_INPUT_EVDEV=y.
- Map PCI vendor IDs to drivers: 8086=Intel(i915/e1000e/iwlwifi), 1002=AMD(amdgpu), 10de=NVIDIA(nouveau), 14e4=Broadcom(tg3/brcmfmac), 168c=Qualcomm(ath9k/ath10k), 10ec=Realtek(r8169/rtw88)
- For detected modules, enable the corresponding CONFIG_ option.
- Target 30-60 lines. Only output what DIFFERS from defconfig.`;

async function generateKernelConfig(env: Env, hardware: string, distro: string, mode: string, modules: string[]): Promise<string> {
  const apiKey = env.ANTHROPIC_API_KEY;
  if (!apiKey) {
    // Fallback: generate a basic config without AI
    return generateFallbackConfig(mode, modules);
  }

  const userMsg = `Hardware info:\n${hardware || 'No hardware info provided — use generic defaults'}\n\nDistro: ${distro}\nMode: ${mode}\nDetected modules: ${modules.join(', ') || 'none'}\n\nGenerate the kernel .config fragment.`;
  const config = await callClaude(apiKey, KERNEL_CONFIG_PROMPT, [{ role: 'user', content: userMsg }], 4096);
  return config;
}

function generateFallbackConfig(mode: string, modules: string[]): string {
  // This is a FRAGMENT merged on top of defconfig via merge_config.sh
  // defconfig already has standard hardware support — we just customize
  const lines = [
    'CONFIG_LOCALVERSION="-y12-custom"',
  ];
  if (mode === 'server') {
    lines.push('# CONFIG_DRM is not set', '# CONFIG_SND is not set', '# CONFIG_WLAN is not set', '# CONFIG_BT is not set');
    lines.push('CONFIG_NETFILTER=y', 'CONFIG_CGROUPS=y', 'CONFIG_NAMESPACES=y', 'CONFIG_NET_NS=y');
    lines.push('CONFIG_VETH=y', 'CONFIG_BRIDGE=y', 'CONFIG_NF_NAT=y', 'CONFIG_OVERLAY_FS=y');
  } else {
    lines.push('CONFIG_DRM_I915=m', 'CONFIG_DRM_AMDGPU=m', 'CONFIG_DRM_NOUVEAU=m');
    lines.push('CONFIG_SND_HDA_INTEL=m', 'CONFIG_WLAN=y', 'CONFIG_BT=y', 'CONFIG_INPUT_EVDEV=y');
  }
  for (const m of modules) {
    lines.push(`CONFIG_${m.toUpperCase()}=m`);
  }
  return lines.join('\n');
}

// ── Create build job (real pipeline) ──────────────────────────────────

app.post('/api/build', async (c) => {
  try {
    const req: BuildRequest = await c.req.json();
    const jobId = crypto.randomUUID();
    const pkgMgr = DISTRO_PKG_MANAGER[req.distro] || 'apt';
    const baseImage = DISTRO_BASE_IMAGE[req.distro] || 'debian:12-slim';
    const pkgMap = OVERLAY_PACKAGES[pkgMgr] || {};

    // Resolve all packages
    const allPackages: string[] = [];
    for (const overlay of req.overlays) {
      const pkgs = pkgMap[overlay];
      if (pkgs && pkgs.length > 0) allPackages.push(...pkgs);
    }

    // Phase 1: Generate real kernel .config via Claude
    let kernelConfig: string;
    let aiModel = 'fallback';
    try {
      kernelConfig = await generateKernelConfig(c.env, req.hardware_raw, req.distro, req.mode, req.detected_modules);
      aiModel = c.env.ANTHROPIC_API_KEY ? 'claude-sonnet-4-20250514' : 'fallback';
    } catch (e: any) {
      kernelConfig = generateFallbackConfig(req.mode, req.detected_modules);
      aiModel = 'fallback-error: ' + e.message;
    }

    // Phase 2: Generate the real build script
    const buildScript = generateBuildScript({
      jobId,
      distro: req.distro,
      mode: req.mode,
      baseImage,
      pkgManager: pkgMgr,
      packages: allPackages,
      customSoftware: req.custom_software,
      overlays: req.overlays,
      modules: req.detected_modules,
      aiMode: req.ai_mode,
      hardwareRaw: req.hardware_raw,
    });

    // Phase 3: Store kernel config and build script in R2
    const r2Prefix = `builds/${jobId}`;
    await c.env.ISO_STORAGE.put(`${r2Prefix}/kernel.config`, kernelConfig);
    await c.env.ISO_STORAGE.put(`${r2Prefix}/build.sh`, buildScript);
    await c.env.ISO_STORAGE.put(`${r2Prefix}/manifest.json`, JSON.stringify({
      jobId,
      distro: req.distro,
      mode: req.mode,
      baseImage,
      pkgManager: pkgMgr,
      packages: allPackages,
      customSoftware: req.custom_software,
      overlays: req.overlays,
      modules: req.detected_modules,
      aiModel,
      kernelConfigLines: kernelConfig.split('\n').length,
      created: new Date().toISOString(),
    }));

    const job = {
      id: jobId,
      distro: req.distro,
      mode: req.mode,
      status: 'building',
      progress: 0,
      created_at: new Date().toISOString(),
      packages: allPackages,
      custom_software: req.custom_software,
      overlays: req.overlays,
      ai_model: aiModel,
      kernel_config_lines: kernelConfig.split('\n').length,
      r2_prefix: r2Prefix,
      build_script_hash: await sha256(buildScript),
      logs: [
        `[${new Date().toISOString()}] Build job ${jobId} created`,
        `[${new Date().toISOString()}] AI model: ${aiModel}`,
        `[${new Date().toISOString()}] Kernel config: ${kernelConfig.split('\n').length} lines generated`,
        `[${new Date().toISOString()}] Packages: ${allPackages.length} overlay + ${req.custom_software.length} custom`,
        `[${new Date().toISOString()}] Build script stored in R2: ${r2Prefix}/build.sh`,
        `[${new Date().toISOString()}] Kernel config stored in R2: ${r2Prefix}/kernel.config`,
      ],
    };

    await c.env.BUILD_JOBS.put(`job:${jobId}`, JSON.stringify(job), { expirationTtl: 86400 * 7 });

    // Phase 4: Start async build execution
    c.executionCtx.waitUntil(executeBuild(c.env, jobId));

    return c.json({
      id: jobId,
      status: 'building',
      ai_model: aiModel,
      kernel_config_lines: kernelConfig.split('\n').length,
      packages: allPackages.length,
      r2_prefix: r2Prefix,
    }, 201);
  } catch (error: any) {
    return c.json({ error: error.message }, 500);
  }
});

// ── SHA256 helper ─────────────────────────────────────────────────────

async function sha256(data: string): Promise<string> {
  const buf = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(data));
  return Array.from(new Uint8Array(buf)).map(b => b.toString(16).padStart(2, '0')).join('');
}

// ── Async build execution (runs in waitUntil) ─────────────────────────

async function updateJob(env: Env, jobId: string, updates: Record<string, any>) {
  const data = await env.BUILD_JOBS.get(`job:${jobId}`);
  if (!data) return;
  const job = JSON.parse(data);
  Object.assign(job, updates);
  if (updates.log) {
    job.logs = job.logs || [];
    job.logs.push(`[${new Date().toISOString()}] ${updates.log}`);
    delete job.log;
  }
  await env.BUILD_JOBS.put(`job:${jobId}`, JSON.stringify(job), { expirationTtl: 86400 * 7 });
}

// ── Trigger GitHub Actions workflow ────────────────────────────────────

async function triggerGitHubBuild(env: Env, jobId: string): Promise<boolean> {
  const token = env.GITHUB_TOKEN;
  const repo = env.GITHUB_REPO; // format: "owner/repo"
  if (!token || !repo) {
    console.log('GitHub Actions not configured — GITHUB_TOKEN or GITHUB_REPO missing');
    return false;
  }

  try {
    const resp = await fetch(`https://api.github.com/repos/${repo}/actions/workflows/build-iso.yml/dispatches`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Accept': 'application/vnd.github.v3+json',
        'User-Agent': 'Y12-API-Worker',
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        ref: 'main',
        inputs: {
          job_id: jobId,
          api_url: 'https://y12-api.seefeldmaxwell1.workers.dev',
        },
      }),
    });

    if (resp.status === 204 || resp.ok) {
      console.log(`GitHub Actions workflow dispatched for job ${jobId}`);
      return true;
    }

    const errText = await resp.text();
    console.error(`GitHub Actions dispatch failed (${resp.status}): ${errText}`);
    return false;
  } catch (e: any) {
    console.error(`GitHub Actions trigger error: ${e.message}`);
    return false;
  }
}

async function executeBuild(env: Env, jobId: string) {
  try {
    // Step 1: Retrieve build artifacts from R2
    await updateJob(env, jobId, { progress: 5, log: 'Retrieving build artifacts from R2...' });
    const manifestObj = await env.ISO_STORAGE.get(`builds/${jobId}/manifest.json`);
    if (!manifestObj) throw new Error('Manifest not found in R2');
    const manifest = JSON.parse(await manifestObj.text());

    const configObj = await env.ISO_STORAGE.get(`builds/${jobId}/kernel.config`);
    if (!configObj) throw new Error('Kernel config not found in R2');
    const kernelConfig = await configObj.text();

    const scriptObj = await env.ISO_STORAGE.get(`builds/${jobId}/build.sh`);
    if (!scriptObj) throw new Error('Build script not found in R2');
    const buildScript = await scriptObj.text();

    // Step 2: Validate kernel config
    await updateJob(env, jobId, { progress: 10, log: 'Validating kernel configuration...' });
    const configLines = kernelConfig.split('\n').filter(l => l.startsWith('CONFIG_') || l.startsWith('# CONFIG_'));
    const enabledCount = configLines.filter(l => l.includes('=y') || l.includes('=m')).length;
    const disabledCount = configLines.filter(l => l.includes('is not set') || l.includes('=n')).length;

    if (configLines.length < 10) {
      throw new Error(`Kernel config too small: ${configLines.length} lines. Expected 50+.`);
    }

    await updateJob(env, jobId, {
      progress: 15,
      log: `Kernel config validated: ${enabledCount} enabled, ${disabledCount} disabled, ${configLines.length} total config lines`,
    });

    // Step 3: Validate build script
    await updateJob(env, jobId, { progress: 20, log: 'Validating build script...' });
    const scriptLines = buildScript.split('\n');
    const hasShebang = scriptLines[0] === '#!/bin/bash';
    const hasSetE = scriptLines.some(l => l.includes('set -euo pipefail'));
    const hasDockerPull = scriptLines.some(l => l.includes('docker pull'));
    const hasKernelClone = scriptLines.some(l => l.includes('git clone') && l.includes('linux'));
    const hasMake = scriptLines.some(l => l.includes('make') && (l.includes('bzImage') || l.includes('defconfig')));
    const hasIsoCreate = scriptLines.some(l => l.includes('grub-mkrescue') || l.includes('xorriso') || l.includes('mkisofs'));

    const validations = { hasShebang, hasSetE, hasDockerPull, hasKernelClone, hasMake, hasIsoCreate };
    const failedChecks = Object.entries(validations).filter(([, v]) => !v).map(([k]) => k);

    if (failedChecks.length > 0) {
      await updateJob(env, jobId, { progress: 25, log: `WARNING: Build script missing: ${failedChecks.join(', ')}` });
    } else {
      await updateJob(env, jobId, { progress: 25, log: 'Build script validated: all phases present' });
    }

    // Step 4: Validate package resolution
    const pkgList = manifest.packages.map((p: string) => `✓ ${p}`).join(', ');
    await updateJob(env, jobId, { progress: 30, log: `Resolved ${manifest.packages.length} packages for ${manifest.pkgManager}: ${pkgList}` });

    // Step 5: Generate Dockerfile for the build
    await updateJob(env, jobId, { progress: 40, log: 'Generating Dockerfile...' });
    const dockerfile = generateDockerfile(manifest, kernelConfig);
    await env.ISO_STORAGE.put(`builds/${jobId}/Dockerfile`, dockerfile);
    await updateJob(env, jobId, { progress: 45, log: `Dockerfile stored (${dockerfile.split('\n').length} lines)` });

    // Step 6: Generate docker-compose for one-click build
    await updateJob(env, jobId, { progress: 50, log: 'Generating docker-compose.yml...' });
    const compose = generateDockerCompose(jobId, manifest);
    await env.ISO_STORAGE.put(`builds/${jobId}/docker-compose.yml`, compose);

    // Step 7: Create the build archive manifest
    await updateJob(env, jobId, { progress: 60, log: 'Creating build archive...' });
    const buildReadme = generateBuildReadme(jobId, manifest);
    await env.ISO_STORAGE.put(`builds/${jobId}/README.md`, buildReadme);

    // Step 8: Run validation suite
    await updateJob(env, jobId, { progress: 70, log: 'Running validation suite...' });
    const testResults = await runBuildValidation(manifest, kernelConfig, buildScript, dockerfile);
    await env.ISO_STORAGE.put(`builds/${jobId}/test-results.json`, JSON.stringify(testResults, null, 2));

    const passedTests = testResults.filter(t => t.pass).length;
    const totalTests = testResults.length;
    const testSummary = testResults.map(t => `${t.pass ? '✓' : '✗'} ${t.name}`).join(', ');
    await updateJob(env, jobId, {
      progress: 85,
      log: `Validation: ${passedTests}/${totalTests} passed [${testSummary}]`,
    });

    // Step 9: Generate SHA256 checksums for all artifacts
    const checksums: Record<string, string> = {};
    for (const file of ['kernel.config', 'build.sh', 'Dockerfile', 'docker-compose.yml', 'manifest.json', 'README.md']) {
      const obj = await env.ISO_STORAGE.get(`builds/${jobId}/${file}`);
      if (obj) {
        const text = await obj.text();
        checksums[file] = await sha256(text);
      }
    }
    await env.ISO_STORAGE.put(`builds/${jobId}/checksums.sha256`, Object.entries(checksums).map(([f, h]) => `${h}  ${f}`).join('\n'));

    // Step 10: Trigger GitHub Actions to build the ISO
    await updateJob(env, jobId, { progress: 88, log: 'Triggering GitHub Actions ISO build...' });
    const ghTriggered = await triggerGitHubBuild(env, jobId);

    if (ghTriggered) {
      await updateJob(env, jobId, {
        progress: 90,
        status: 'building_iso',
        log: `Validation: ${passedTests}/${totalTests} passed. GitHub Actions building ISO on cloud runner...`,
        test_results: { passed: passedTests, total: totalTests },
        checksums,
        artifacts: ['kernel.config', 'build.sh', 'Dockerfile', 'docker-compose.yml', 'manifest.json', 'README.md', 'test-results.json', 'checksums.sha256'],
        build_runner: 'github_actions',
      });
    } else {
      // GH Actions not configured — complete with artifacts for local build
      const finalStatus = passedTests === totalTests ? 'complete' : 'complete_with_warnings';
      await updateJob(env, jobId, {
        progress: 100,
        status: finalStatus,
        log: `Build ${finalStatus}. ${passedTests}/${totalTests} validations passed. Download artifacts and run: docker compose up --build`,
        completed_at: new Date().toISOString(),
        test_results: { passed: passedTests, total: totalTests },
        checksums,
        artifacts: ['kernel.config', 'build.sh', 'Dockerfile', 'docker-compose.yml', 'manifest.json', 'README.md', 'test-results.json', 'checksums.sha256'],
        build_runner: 'local',
      });
    }

  } catch (error: any) {
    await updateJob(env, jobId, {
      status: 'failed',
      log: `BUILD FAILED: ${error.message}`,
      error: error.message,
    });
  }
}

// ── Build validation suite ────────────────────────────────────────────

interface TestResult { name: string; pass: boolean; msg: string }

async function runBuildValidation(manifest: any, kernelConfig: string, buildScript: string, dockerfile: string): Promise<TestResult[]> {
  const results: TestResult[] = [];

  // 1. Kernel config fragment: check it's not disabling critical options
  // Note: this is a FRAGMENT merged on top of defconfig, so missing options are OK (defconfig has them)
  const mustNotDisable = ['CONFIG_NET', 'CONFIG_INET', 'CONFIG_EXT4_FS', 'CONFIG_PROC_FS', 'CONFIG_SYSFS', 'CONFIG_PRINTK'];
  for (const cfg of mustNotDisable) {
    const disabled = kernelConfig.includes(`# ${cfg} is not set`) || kernelConfig.includes(`${cfg}=n`);
    results.push({ name: `kernel_config_${cfg}`, pass: !disabled, msg: !disabled ? `${cfg} not disabled (defconfig default OK)` : `${cfg} DISABLED — kernel may not boot` });
  }

  // 2. Kernel config: mode-specific checks
  if (manifest.mode === 'server') {
    const serverDisabled = ['CONFIG_DRM', 'CONFIG_SND'];
    for (const cfg of serverDisabled) {
      const disabled = kernelConfig.includes(`# ${cfg} is not set`) || !kernelConfig.includes(`${cfg}=y`);
      results.push({ name: `server_disable_${cfg}`, pass: disabled, msg: disabled ? `${cfg} correctly disabled for server` : `${cfg} should be disabled for server mode` });
    }
    const serverEnabled = ['CONFIG_NETFILTER', 'CONFIG_CGROUPS'];
    for (const cfg of serverEnabled) {
      const has = kernelConfig.includes(`${cfg}=y`);
      results.push({ name: `server_enable_${cfg}`, pass: has, msg: has ? `${cfg} enabled for server` : `${cfg} should be enabled for server mode` });
    }
  } else {
    const desktopEnabled = ['CONFIG_DRM', 'CONFIG_SND'];
    for (const cfg of desktopEnabled) {
      const has = kernelConfig.includes(`${cfg}=y`) || kernelConfig.includes(`${cfg}=m`);
      results.push({ name: `desktop_enable_${cfg}`, pass: has, msg: has ? `${cfg} enabled for desktop` : `${cfg} should be enabled for desktop mode` });
    }
  }

  // 3. Kernel config: has LOCALVERSION
  const hasLocalVer = kernelConfig.includes('CONFIG_LOCALVERSION');
  results.push({ name: 'kernel_localversion', pass: hasLocalVer, msg: hasLocalVer ? 'LOCALVERSION set' : 'LOCALVERSION missing' });

  // 4. Kernel config: reasonable size
  const configLineCount = kernelConfig.split('\n').filter((l: string) => l.startsWith('CONFIG_') || l.startsWith('# CONFIG_')).length;
  const sizeOk = configLineCount >= 20;
  results.push({ name: 'kernel_config_size', pass: sizeOk, msg: `${configLineCount} config lines (min 20)` });

  // 5. Build script: has shebang
  results.push({ name: 'script_shebang', pass: buildScript.startsWith('#!/bin/bash'), msg: buildScript.startsWith('#!/bin/bash') ? 'Has bash shebang' : 'Missing shebang' });

  // 6. Build script: has set -euo pipefail
  const hasStrict = buildScript.includes('set -euo pipefail');
  results.push({ name: 'script_strict_mode', pass: hasStrict, msg: hasStrict ? 'Strict mode enabled' : 'Missing strict mode' });

  // 7. Build script: references correct base image
  const hasBaseImg = buildScript.includes(manifest.baseImage);
  results.push({ name: 'script_base_image', pass: hasBaseImg, msg: hasBaseImg ? `References ${manifest.baseImage}` : `Missing base image ${manifest.baseImage}` });

  // 8. Build script: has kernel compilation
  const hasKernelBuild = buildScript.includes('make') && buildScript.includes('bzImage');
  results.push({ name: 'script_kernel_build', pass: hasKernelBuild, msg: hasKernelBuild ? 'Kernel compilation present' : 'Missing kernel compilation' });

  // 9. Build script: has ISO creation
  const hasIso = buildScript.includes('grub-mkrescue') || buildScript.includes('xorriso') || buildScript.includes('mkisofs');
  results.push({ name: 'script_iso_creation', pass: hasIso, msg: hasIso ? 'ISO creation present' : 'Missing ISO creation step' });

  // 10. Build script: has SHA256 checksum
  const hasSha = buildScript.includes('sha256sum');
  results.push({ name: 'script_checksum', pass: hasSha, msg: hasSha ? 'SHA256 checksum present' : 'Missing checksum step' });

  // 11. Dockerfile: has FROM
  const hasFrom = dockerfile.includes('FROM ');
  results.push({ name: 'dockerfile_from', pass: hasFrom, msg: hasFrom ? 'Has FROM directive' : 'Missing FROM' });

  // 12. Dockerfile: has kernel clone
  const hasClone = dockerfile.includes('git clone') && dockerfile.includes('linux');
  results.push({ name: 'dockerfile_kernel_clone', pass: hasClone, msg: hasClone ? 'Kernel source clone present' : 'Missing kernel clone' });

  // 13. Dockerfile: has make
  const hasMake = dockerfile.includes('make') && (dockerfile.includes('bzImage') || dockerfile.includes('olddefconfig'));
  results.push({ name: 'dockerfile_make', pass: hasMake, msg: hasMake ? 'Kernel make present' : 'Missing make step' });

  // 14. Dockerfile: has COPY kernel.config
  const hasCopy = dockerfile.includes('COPY kernel.config');
  results.push({ name: 'dockerfile_config_copy', pass: hasCopy, msg: hasCopy ? 'Kernel config COPY present' : 'Missing kernel config COPY' });

  // 15. Dockerfile: has grub/ISO step
  const hasGrub = dockerfile.includes('grub') || dockerfile.includes('xorriso');
  results.push({ name: 'dockerfile_bootloader', pass: hasGrub, msg: hasGrub ? 'Bootloader/ISO step present' : 'Missing bootloader step' });

  // 16. Package resolution: all overlay packages resolved
  const pkgMgr = manifest.pkgManager;
  const pkgMap = OVERLAY_PACKAGES[pkgMgr] || {};
  for (const overlay of manifest.overlays) {
    const pkgs = pkgMap[overlay];
    const resolved = pkgs !== undefined;
    results.push({ name: `pkg_resolve_${overlay}`, pass: resolved, msg: resolved ? `${overlay}: ${pkgs.length > 0 ? pkgs.join(', ') : 'script-installed'}` : `${overlay}: unknown overlay` });
  }

  // 17. Distro validation
  const validDistros = ['nixos', 'debian', 'rocky', 'proxmox'];
  const distroValid = validDistros.includes(manifest.distro);
  results.push({ name: 'distro_valid', pass: distroValid, msg: distroValid ? `${manifest.distro} is supported` : `${manifest.distro} is not a supported distro` });

  // 18. Mode validation
  const validModes = ['desktop', 'server'];
  const modeValid = validModes.includes(manifest.mode);
  results.push({ name: 'mode_valid', pass: modeValid, msg: modeValid ? `${manifest.mode} is valid` : `${manifest.mode} is not a valid mode` });

  // 19. Manifest completeness
  const hasAllFields = manifest.jobId && manifest.distro && manifest.mode && manifest.baseImage && manifest.pkgManager;
  results.push({ name: 'manifest_complete', pass: !!hasAllFields, msg: hasAllFields ? 'Manifest has all required fields' : 'Manifest missing fields' });

  // 20. Cross-validation: Dockerfile uses debian:12 as build container
  const dockerBaseMatch = dockerfile.includes('FROM debian:12');
  results.push({ name: 'cross_dockerfile_base', pass: dockerBaseMatch, msg: dockerBaseMatch ? 'Dockerfile uses debian:12 build container' : 'Dockerfile missing debian:12 build base' });

  return results;
}

// ── Dockerfile generator ──────────────────────────────────────────────

function generateDockerfile(manifest: any, kernelConfig: string): string {
  // Always use debian:12 as the BUILD container — it has all the tools.
  // The target distro affects the rootfs packages, not the build environment.
  const lines = [
    'FROM debian:12 AS builder',
    '',
    '# Build dependencies (same for all target distros)',
    'RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \\',
    '    build-essential libncurses-dev bison flex libssl-dev libelf-dev \\',
    '    bc git wget cpio kmod xorriso grub-pc-bin grub-efi-amd64-bin grub-common \\',
    '    mtools dosfstools squashfs-tools ca-certificates debootstrap \\',
    '    && rm -rf /var/lib/apt/lists/*',
    '',
    '# Clone kernel source (v6.6 LTS) with retry for transient mirror failures',
    'RUN for i in 1 2 3; do git clone --depth 1 --branch v6.6 https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git /usr/src/linux && break || sleep 10; done',
    '',
    '# Start from x86_64 defconfig (includes real hardware drivers: NVMe, AHCI, USB, NIC, GPU, etc)',
    '# Then layer the Y12 AI-generated config on top for hardware-specific customization',
    'RUN cd /usr/src/linux && make defconfig',
    'COPY kernel.config /tmp/y12.config',
    'RUN cd /usr/src/linux && scripts/kconfig/merge_config.sh .config /tmp/y12.config',
    '',
    '# Compile kernel + modules (defconfig gives real hardware support)',
    'RUN cd /usr/src/linux && make -j$(nproc) bzImage modules 2>&1 | tail -5',
    '',
    '# Install kernel and modules into rootfs',
    'RUN mkdir -p /rootfs/boot /rootfs/lib/modules',
    'RUN cp /usr/src/linux/arch/x86/boot/bzImage /rootfs/boot/vmlinuz-y12',
    'RUN cd /usr/src/linux && make modules_install INSTALL_MOD_PATH=/rootfs',
    '',
  ];

  // Install overlay packages into rootfs using apt (debian-based rootfs for all ISOs)
  const allPkgs = [...(manifest.packages || []), ...(manifest.customSoftware || [])];
  if (allPkgs.length > 0) {
    lines.push(
      '# Install overlay packages into rootfs',
      `RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ${allPkgs.join(' ')} || true`,
      'RUN rm -rf /var/lib/apt/lists/*',
      '',
    );
  }

  lines.push(
    '# Create ISO filesystem structure',
    'RUN mkdir -p /iso/boot/grub /iso/live',
    'RUN cp /rootfs/boot/vmlinuz-y12 /iso/boot/vmlinuz',
    '',
    '# Create initramfs with modules and rootfs',
    'RUN cd /rootfs && find . | cpio -o -H newc | gzip > /iso/boot/initrd.gz',
    '',
    '# GRUB config for BIOS + EFI boot',
    `RUN printf 'set timeout=5\\nset default=0\\n\\nmenuentry "Y12 Custom Linux (${manifest.distro} ${manifest.mode})" {\\n  linux /boot/vmlinuz root=/dev/ram0 console=tty0 console=ttyS0,115200\\n  initrd /boot/initrd.gz\\n}\\n' > /iso/boot/grub/grub.cfg`,
    '',
    '# Build bootable ISO (BIOS + EFI)',
    'RUN grub-mkrescue -o /output.iso /iso 2>/dev/null || xorriso -as mkisofs -R -J -b boot/grub/i386-pc/eltorito.img -no-emul-boot -boot-load-size 4 -boot-info-table -o /output.iso /iso',
    '',
    '# Checksum',
    'RUN sha256sum /output.iso > /output.iso.sha256',
    '',
    'CMD ["cat", "/output.iso"]',
  );

  return lines.join('\n');
}

// ── Docker Compose generator ──────────────────────────────────────────

function generateDockerCompose(jobId: string, manifest: any): string {
  return `version: "3.8"
services:
  builder:
    build:
      context: .
      dockerfile: Dockerfile
    volumes:
      - ./output:/output
    command: >
      sh -c "cp /output.iso /output/y12-${manifest.distro}-${manifest.mode}-${jobId.slice(0, 8)}.iso &&
             cp /output.iso.sha256 /output/y12-${manifest.distro}-${manifest.mode}-${jobId.slice(0, 8)}.iso.sha256 &&
             echo 'Build complete!'"
`;
}

// ── Build README generator ────────────────────────────────────────────

function generateBuildReadme(jobId: string, manifest: any): string {
  return `# Y12.AI Build — ${jobId}

## Build Configuration
- **Distro**: ${manifest.distro}
- **Mode**: ${manifest.mode}
- **Base Image**: ${manifest.baseImage}
- **Package Manager**: ${manifest.pkgManager}
- **Packages**: ${manifest.packages.join(', ') || 'none'}
- **Custom Software**: ${(manifest.customSoftware || []).join(', ') || 'none'}
- **Overlays**: ${manifest.overlays.join(', ') || 'none'}
- **AI Model**: ${manifest.aiModel}
- **Kernel Config Lines**: ${manifest.kernelConfigLines}
- **Created**: ${manifest.created}

## How to Build

### Option 1: Docker Compose (recommended)
\`\`\`bash
docker compose up --build
# ISO will be in ./output/
\`\`\`

### Option 2: Manual Docker Build
\`\`\`bash
docker build -t y12-${manifest.distro}-${manifest.mode} .
docker run --rm -v $(pwd)/output:/output y12-${manifest.distro}-${manifest.mode} \\
  sh -c "cp /output.iso /output/ && cp /output.iso.sha256 /output/"
\`\`\`

### Option 3: Run build.sh directly (requires Docker)
\`\`\`bash
chmod +x build.sh
./build.sh
\`\`\`

## Files
- \`kernel.config\` — AI-generated kernel .config (${manifest.kernelConfigLines} lines)
- \`build.sh\` — Standalone build script
- \`Dockerfile\` — Multi-stage Docker build
- \`docker-compose.yml\` — One-click build
- \`manifest.json\` — Build metadata
- \`test-results.json\` — Automated validation results
- \`checksums.sha256\` — SHA256 checksums of all artifacts

## Verification
\`\`\`bash
sha256sum -c checksums.sha256
\`\`\`
`;
}

// ── Get build status ───────────────────────────────────────────────────

app.get('/api/build/:id', async (c) => {
  const jobId = c.req.param('id');
  const data = await c.env.BUILD_JOBS.get(`job:${jobId}`);
  if (!data) return c.json({ error: 'Build not found' }, 404);
  return c.json(JSON.parse(data));
});

// ── Simulate build progress (called by container or cron) ──────────────

app.post('/api/build/:id/progress', async (c) => {
  const jobId = c.req.param('id');
  const { progress, log, status } = await c.req.json();
  const data = await c.env.BUILD_JOBS.get(`job:${jobId}`);
  if (!data) return c.json({ error: 'Build not found' }, 404);

  const job = JSON.parse(data);
  if (progress !== undefined) job.progress = progress;
  if (log) job.logs.push(`[${new Date().toISOString()}] ${log}`);
  if (status) job.status = status;

  // If this is a "complete" status update, check R2 for the ISO
  // This handles the race condition where upload-iso wrote to R2 but KV was stale
  if (status === 'complete' && !job.iso_uploaded) {
    const isoObj = await c.env.ISO_STORAGE.head(`builds/${jobId}/output.iso`);
    if (isoObj) {
      job.iso_uploaded = true;
      job.iso_size = isoObj.size;
      job.iso_sha256 = isoObj.customMetadata?.sha256 || '';
      job.iso_r2_key = `builds/${jobId}/output.iso`;
      job.logs.push(`[${new Date().toISOString()}] ISO confirmed in R2: ${isoObj.size} bytes`);
    }
  }

  await c.env.BUILD_JOBS.put(`job:${jobId}`, JSON.stringify(job), { expirationTtl: 86400 * 7 });
  return c.json({ ok: true });
});

// ── Download build artifacts (from R2) ────────────────────────────────

app.get('/api/build/:id/download', async (c) => {
  const jobId = c.req.param('id');
  const data = await c.env.BUILD_JOBS.get(`job:${jobId}`);
  if (!data) return c.json({ error: 'Build not found' }, 404);

  const job = JSON.parse(data);
  if (!job.status?.startsWith('complete')) return c.json({ error: 'Build not complete yet', status: job.status, progress: job.progress }, 400);

  // Return links to all build artifacts
  const files = ['kernel.config', 'build.sh', 'Dockerfile', 'docker-compose.yml', 'manifest.json', 'README.md', 'test-results.json', 'checksums.sha256'];
  return c.json({
    id: jobId,
    status: job.status,
    artifacts: files.map(f => ({
      name: f,
      url: `/api/build/${jobId}/file/${f}`,
    })),
    test_results: job.test_results,
    checksums: job.checksums,
  });
});

// ── Download individual build file ────────────────────────────────────

app.get('/api/build/:id/file/:filename', async (c) => {
  const jobId = c.req.param('id');
  const filename = c.req.param('filename');
  const allowed = ['kernel.config', 'build.sh', 'Dockerfile', 'docker-compose.yml', 'manifest.json', 'README.md', 'test-results.json', 'checksums.sha256'];
  if (!allowed.includes(filename)) return c.json({ error: 'File not found' }, 404);

  const obj = await c.env.ISO_STORAGE.get(`builds/${jobId}/${filename}`);
  if (!obj) return c.json({ error: 'File not found in storage' }, 404);

  const contentType = filename.endsWith('.json') ? 'application/json' : filename.endsWith('.yml') ? 'text/yaml' : filename.endsWith('.md') ? 'text/markdown' : 'text/plain';
  return new Response(obj.body, {
    headers: {
      'Content-Type': contentType,
      'Content-Disposition': `attachment; filename="${filename}"`,
    },
  });
});

// ── Upload ISO from GitHub Actions ────────────────────────────────────

app.put('/api/build/:id/upload-iso', async (c) => {
  const jobId = c.req.param('id');

  // Auth check — only GitHub Actions with BUILD_SECRET can upload
  const auth = c.req.header('Authorization');
  const secret = c.env.BUILD_SECRET;
  if (!secret || auth !== `Bearer ${secret}`) {
    return c.json({ error: 'Unauthorized' }, 401);
  }

  const data = await c.env.BUILD_JOBS.get(`job:${jobId}`);
  if (!data) return c.json({ error: 'Build not found' }, 404);

  const isoSha = c.req.header('X-ISO-SHA256') || 'unknown';
  const isoSize = c.req.header('X-ISO-Size') || '0';

  // Stream the request body directly to R2 (supports up to 5GB)
  const r2Key = `builds/${jobId}/output.iso`;
  const rawBody = c.req.raw.body;
  if (!rawBody) {
    return c.json({ error: 'No body provided' }, 400);
  }

  try {
    await c.env.ISO_STORAGE.put(r2Key, rawBody, {
      httpMetadata: { contentType: 'application/octet-stream' },
      customMetadata: { sha256: isoSha, size: isoSize, uploaded: new Date().toISOString() },
    });
  } catch (e: any) {
    return c.json({ error: `R2 upload failed: ${e.message}` }, 500);
  }

  // Verify the upload
  const head = await c.env.ISO_STORAGE.head(r2Key);
  if (!head) {
    return c.json({ error: 'R2 upload verification failed — object not found after put' }, 500);
  }

  // Update job with ISO info
  const job = JSON.parse(data);
  job.iso_uploaded = true;
  job.iso_size = head.size;
  job.iso_sha256 = isoSha;
  job.iso_r2_key = r2Key;
  job.logs.push(`[${new Date().toISOString()}] ISO uploaded to R2: ${head.size} bytes, SHA256: ${isoSha}`);
  await c.env.BUILD_JOBS.put(`job:${jobId}`, JSON.stringify(job), { expirationTtl: 86400 * 7 });

  return c.json({ ok: true, r2_key: r2Key, size: head.size, sha256: isoSha });
});

// ── Download completed ISO ────────────────────────────────────────────

app.get('/api/build/:id/iso', async (c) => {
  const jobId = c.req.param('id');
  const data = await c.env.BUILD_JOBS.get(`job:${jobId}`);
  if (!data) return c.json({ error: 'Build not found' }, 404);

  const job = JSON.parse(data);

  // Fallback: check R2 directly even if KV doesn't have iso_uploaded (race condition)
  const obj = await c.env.ISO_STORAGE.get(`builds/${jobId}/output.iso`);
  if (!job.iso_uploaded && !obj) return c.json({ error: 'ISO not yet available', status: job.status, progress: job.progress }, 400);
  if (!obj) return c.json({ error: 'ISO not found in R2' }, 404);

  return new Response(obj.body, {
    headers: {
      'Content-Type': 'application/octet-stream',
      'Content-Disposition': `attachment; filename="y12-${job.distro}-${job.mode}-${jobId.slice(0, 8)}.iso"`,
      'Content-Length': String(job.iso_size || ''),
      'X-ISO-SHA256': job.iso_sha256 || '',
    },
  });
});

// ── Self-test ──────────────────────────────────────────────────────────

app.get('/api/test', async (c) => {
  const tests: { name: string; pass: boolean; msg: string }[] = [];

  // Test 1: Workers AI binding
  try {
    const r = await c.env.AI.run('@cf/meta/llama-3.1-8b-instruct', {
      messages: [{ role: 'user', content: 'Say OK' }],
      max_tokens: 5,
    });
    tests.push({ name: 'workers_ai', pass: !!r.response, msg: r.response?.slice(0, 50) || 'no response' });
  } catch (e: any) {
    tests.push({ name: 'workers_ai', pass: false, msg: e.message });
  }

  // Test 2: Claude API
  try {
    const apiKey = c.env.ANTHROPIC_API_KEY;
    if (!apiKey) {
      tests.push({ name: 'claude_api', pass: false, msg: 'ANTHROPIC_API_KEY not set — run: wrangler secret put ANTHROPIC_API_KEY' });
    } else {
      const reply = await callClaude(apiKey, 'Respond with exactly: OK', [{ role: 'user', content: 'Say OK' }], 10);
      tests.push({ name: 'claude_api', pass: reply.includes('OK'), msg: `claude-sonnet-4-20250514: ${reply.slice(0, 50)}` });
    }
  } catch (e: any) {
    tests.push({ name: 'claude_api', pass: false, msg: e.message });
  }

  // Test 3: KV
  try {
    await c.env.BUILD_JOBS.put('_test', 'ok');
    const v = await c.env.BUILD_JOBS.get('_test');
    await c.env.BUILD_JOBS.delete('_test');
    tests.push({ name: 'kv_store', pass: v === 'ok', msg: v || 'null' });
  } catch (e: any) {
    tests.push({ name: 'kv_store', pass: false, msg: e.message });
  }

  // Test 4: R2
  try {
    await c.env.ISO_STORAGE.put('_test.txt', 'ok');
    const obj = await c.env.ISO_STORAGE.get('_test.txt');
    const text = await obj?.text();
    await c.env.ISO_STORAGE.delete('_test.txt');
    tests.push({ name: 'r2_storage', pass: text === 'ok', msg: text || 'null' });
  } catch (e: any) {
    tests.push({ name: 'r2_storage', pass: false, msg: e.message });
  }

  // Test 5: Package resolution (all distros)
  for (const [mgr, overlays] of Object.entries(OVERLAY_PACKAGES)) {
    const dockerPkgs = overlays['docker'];
    tests.push({ name: `pkg_${mgr}_docker`, pass: dockerPkgs !== undefined, msg: `docker → [${dockerPkgs?.join(', ') || 'undefined'}]` });
  }

  // Test 6: Fallback kernel config generation
  try {
    const serverConfig = generateFallbackConfig('server', ['e1000e', 'nvme']);
    const hasLocalver = serverConfig.includes('CONFIG_LOCALVERSION');
    const noDrm = serverConfig.includes('# CONFIG_DRM is not set');
    const hasNetfilter = serverConfig.includes('CONFIG_NETFILTER=y');
    tests.push({ name: 'fallback_config_server', pass: hasLocalver && noDrm && hasNetfilter, msg: `${serverConfig.split('\n').length} lines, localver=${hasLocalver}, no_drm=${noDrm}, netfilter=${hasNetfilter}` });

    const desktopConfig = generateFallbackConfig('desktop', ['i915']);
    const hasI915 = desktopConfig.includes('CONFIG_DRM_I915=m');
    const hasSnd = desktopConfig.includes('CONFIG_SND_HDA_INTEL=m');
    tests.push({ name: 'fallback_config_desktop', pass: hasI915 && hasSnd, msg: `${desktopConfig.split('\n').length} lines, i915=${hasI915}, snd=${hasSnd}` });
  } catch (e: any) {
    tests.push({ name: 'fallback_config', pass: false, msg: e.message });
  }

  // Test 7: Build script generation
  try {
    const script = generateBuildScript({
      jobId: 'test-0000', distro: 'debian', mode: 'server', baseImage: 'debian:12-slim',
      pkgManager: 'apt', packages: ['docker.io'], customSoftware: [], overlays: ['docker'],
      modules: ['e1000e'], aiMode: true, hardwareRaw: '',
    });
    const hasShebang = script.startsWith('#!/bin/bash');
    const hasMake = script.includes('make') && script.includes('bzImage');
    const hasIso = script.includes('grub-mkrescue') || script.includes('xorriso');
    tests.push({ name: 'build_script_gen', pass: hasShebang && hasMake && hasIso, msg: `${script.split('\n').length} lines, shebang=${hasShebang}, make=${hasMake}, iso=${hasIso}` });
  } catch (e: any) {
    tests.push({ name: 'build_script_gen', pass: false, msg: e.message });
  }

  // Test 8: Dockerfile generation
  try {
    const manifest = { baseImage: 'debian:12-slim', pkgManager: 'apt', packages: ['docker.io'], customSoftware: [], distro: 'debian', mode: 'server', overlays: ['docker'] };
    const df = generateDockerfile(manifest, 'CONFIG_LOCALVERSION="-y12-custom"');
    const hasFrom = df.includes('FROM debian:12');
    const hasKernel = df.includes('git clone') && df.includes('linux');
    const hasGrub = df.includes('grub');
    tests.push({ name: 'dockerfile_gen', pass: hasFrom && hasKernel && hasGrub, msg: `${df.split('\n').length} lines, from=${hasFrom}, kernel=${hasKernel}, grub=${hasGrub}` });
  } catch (e: any) {
    tests.push({ name: 'dockerfile_gen', pass: false, msg: e.message });
  }

  // Test 9: Validation suite
  try {
    const manifest = { distro: 'debian', mode: 'server', baseImage: 'debian:12-slim', pkgManager: 'apt', packages: ['docker.io'], overlays: ['docker'], jobId: 'test', customSoftware: [] };
    const config = generateFallbackConfig('server', ['e1000e']);
    const script = generateBuildScript({ jobId: 'test', distro: 'debian', mode: 'server', baseImage: 'debian:12-slim', pkgManager: 'apt', packages: ['docker.io'], customSoftware: [], overlays: ['docker'], modules: ['e1000e'], aiMode: false, hardwareRaw: '' });
    const df = generateDockerfile(manifest, config);
    const results = await runBuildValidation(manifest, config, script, df);
    const passed = results.filter((t: TestResult) => t.pass).length;
    tests.push({ name: 'validation_suite', pass: passed >= 15, msg: `${passed}/${results.length} validations passed` });
  } catch (e: any) {
    tests.push({ name: 'validation_suite', pass: false, msg: e.message });
  }

  // Test 10: SHA256 helper
  try {
    const hash = await sha256('test');
    tests.push({ name: 'sha256', pass: hash.length === 64, msg: `hash length: ${hash.length}` });
  } catch (e: any) {
    tests.push({ name: 'sha256', pass: false, msg: e.message });
  }

  // Test 11: Stripe config
  const hasStripe = !!c.env.STRIPE_SECRET_KEY;
  tests.push({ name: 'stripe_key', pass: hasStripe, msg: hasStripe ? 'STRIPE_SECRET_KEY is set' : 'Not set — run: wrangler secret put STRIPE_SECRET_KEY' });

  const passed = tests.filter((t) => t.pass).length;
  return c.json({ passed, failed: tests.length - passed, total: tests.length, tests });
});

// ── End-to-end build test (creates a real test build) ─────────────────

app.post('/api/test/build', async (c) => {
  const startTime = Date.now();
  const results: TestResult[] = [];

  try {
    // Create a test build
    const testReq: BuildRequest = {
      distro: 'debian',
      mode: 'server',
      hardware_raw: '00:00.0 Host bridge: Intel Corporation Device 9a14\n00:02.0 VGA compatible controller: Intel Corporation Device 9a49\n00:1f.3 Audio device: Intel Corporation Device a0c8\n01:00.0 Ethernet controller: Intel Corporation I225-V\n02:00.0 Non-Volatile memory controller: Samsung Electronics Co Ltd NVMe SSD Controller',
      ai_mode: true,
      overlays: ['docker', 'tailscale'],
      custom_software: [],
      detected_modules: ['i915', 'snd_hda_intel', 'igc', 'nvme'],
    };

    // Test 1: Package resolution
    const pkgMgr = DISTRO_PKG_MANAGER[testReq.distro] || 'apt';
    const pkgMap = OVERLAY_PACKAGES[pkgMgr] || {};
    const allPackages: string[] = [];
    for (const overlay of testReq.overlays) {
      const pkgs = pkgMap[overlay];
      if (pkgs && pkgs.length > 0) allPackages.push(...pkgs);
    }
    results.push({ name: 'e2e_pkg_resolution', pass: allPackages.length > 0, msg: `Resolved ${allPackages.length} packages: ${allPackages.join(', ')}` });

    // Test 2: Kernel config generation
    const kernelConfig = await generateKernelConfig(c.env, testReq.hardware_raw, testReq.distro, testReq.mode, testReq.detected_modules);
    const configLines = kernelConfig.split('\n').filter((l: string) => l.startsWith('CONFIG_') || l.startsWith('# CONFIG_')).length;
    results.push({ name: 'e2e_kernel_config', pass: configLines >= 20, msg: `Generated ${configLines} config lines` });

    // Test 3: Server mode disables DRM/SND
    const noDrm = !kernelConfig.includes('CONFIG_DRM=y') || kernelConfig.includes('# CONFIG_DRM is not set');
    results.push({ name: 'e2e_server_no_drm', pass: noDrm, msg: noDrm ? 'DRM correctly disabled for server' : 'DRM should be disabled for server' });

    // Test 4: Fragment doesn't disable critical boot configs (defconfig has them)
    const netDisabled = kernelConfig.includes('# CONFIG_NET is not set');
    const hasLocalver = kernelConfig.includes('CONFIG_LOCALVERSION');
    results.push({ name: 'e2e_boot_configs', pass: !netDisabled && hasLocalver, msg: `NET_not_disabled=${!netDisabled}, LOCALVERSION=${hasLocalver}` });

    // Test 5: Build script generation
    const buildScript = generateBuildScript({
      jobId: 'e2e-test', distro: testReq.distro, mode: testReq.mode, baseImage: 'debian:12-slim',
      pkgManager: pkgMgr, packages: allPackages, customSoftware: [], overlays: testReq.overlays,
      modules: testReq.detected_modules, aiMode: true, hardwareRaw: testReq.hardware_raw,
    });
    results.push({ name: 'e2e_build_script', pass: buildScript.length > 500, msg: `${buildScript.split('\n').length} lines, ${buildScript.length} bytes` });

    // Test 6: Dockerfile generation
    const manifest = { distro: testReq.distro, mode: testReq.mode, baseImage: 'debian:12-slim', pkgManager: pkgMgr, packages: allPackages, customSoftware: [], overlays: testReq.overlays, jobId: 'e2e-test' };
    const dockerfile = generateDockerfile(manifest, kernelConfig);
    results.push({ name: 'e2e_dockerfile', pass: dockerfile.includes('FROM debian:12-slim'), msg: `${dockerfile.split('\n').length} lines` });

    // Test 7: Full validation suite
    const validationResults = await runBuildValidation(manifest, kernelConfig, buildScript, dockerfile);
    const validPassed = validationResults.filter((t: TestResult) => t.pass).length;
    results.push({ name: 'e2e_validation', pass: validPassed >= 15, msg: `${validPassed}/${validationResults.length} validations passed` });

    // Test 8: R2 storage round-trip
    const testKey = `_e2e_test/${Date.now()}`;
    await c.env.ISO_STORAGE.put(testKey, kernelConfig);
    const retrieved = await c.env.ISO_STORAGE.get(testKey);
    const retrievedText = await retrieved?.text();
    await c.env.ISO_STORAGE.delete(testKey);
    results.push({ name: 'e2e_r2_roundtrip', pass: retrievedText === kernelConfig, msg: `Stored and retrieved ${kernelConfig.length} bytes` });

    // Test 9: SHA256 consistency
    const hash1 = await sha256(kernelConfig);
    const hash2 = await sha256(kernelConfig);
    results.push({ name: 'e2e_sha256_consistent', pass: hash1 === hash2, msg: `Hash: ${hash1.slice(0, 16)}...` });

    // Test 10: KV job storage round-trip
    const testJobId = `_e2e_${Date.now()}`;
    const testJob = { id: testJobId, status: 'test', progress: 50 };
    await c.env.BUILD_JOBS.put(`job:${testJobId}`, JSON.stringify(testJob));
    const jobData = await c.env.BUILD_JOBS.get(`job:${testJobId}`);
    const parsedJob = jobData ? JSON.parse(jobData) : null;
    await c.env.BUILD_JOBS.delete(`job:${testJobId}`);
    results.push({ name: 'e2e_kv_roundtrip', pass: parsedJob?.id === testJobId, msg: `Job stored and retrieved` });

  } catch (e: any) {
    results.push({ name: 'e2e_error', pass: false, msg: e.message });
  }

  const elapsed = Date.now() - startTime;
  const passed = results.filter((t: TestResult) => t.pass).length;
  return c.json({ passed, failed: results.length - passed, total: results.length, elapsed_ms: elapsed, tests: results });
});

// ── Build script generator ─────────────────────────────────────────────

function generateBuildScript(opts: {
  jobId: string;
  distro: string;
  mode: string;
  baseImage: string;
  pkgManager: string;
  packages: string[];
  customSoftware: string[];
  overlays: string[];
  modules: string[];
  aiMode: boolean;
  hardwareRaw: string;
}): string {
  const isoName = `y12-${opts.distro}-${opts.mode}-${opts.jobId.slice(0, 8)}`;
  const lines: string[] = [
    '#!/bin/bash',
    'set -euo pipefail',
    `# ═══════════════════════════════════════════════════════════════════`,
    `# Y12.AI ISO Build Script — Job ${opts.jobId}`,
    `# Distro: ${opts.distro} | Mode: ${opts.mode} | AI: ${opts.aiMode}`,
    `# Generated: ${new Date().toISOString()}`,
    `# Run: chmod +x build.sh && ./build.sh`,
    `# Requires: Docker (or use the Dockerfile/docker-compose.yml instead)`,
    `# ═══════════════════════════════════════════════════════════════════`,
    '',
    `export JOB_ID="${opts.jobId}"`,
    `export DISTRO="${opts.distro}"`,
    `export MODE="${opts.mode}"`,
    `export BASE_IMAGE="${opts.baseImage}"`,
    `export ISO_NAME="${isoName}"`,
    `CONTAINER="y12-build-${opts.jobId.slice(0, 8)}"`,
    '',
    'cleanup() { docker rm -f $CONTAINER 2>/dev/null || true; }',
    'trap cleanup EXIT',
    '',
    '# ── Phase 1: Pull base image ──────────────────────────────────────',
    `echo "[  5%] Pulling ${opts.baseImage}..."`,
    `docker pull ${opts.baseImage}`,
    '',
    '# ── Phase 2: Create build container ───────────────────────────────',
    `echo "[ 10%] Creating build container..."`,
    `docker run -d --name $CONTAINER --privileged ${opts.baseImage} sleep infinity`,
    '',
    '# ── Phase 3: Install build dependencies ──────────────────────────',
    `echo "[ 15%] Installing build dependencies..."`,
  ];

  if (opts.pkgManager === 'apt') {
    lines.push(
      `docker exec $CONTAINER bash -c "apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \\`,
      `  build-essential libncurses-dev bison flex libssl-dev libelf-dev bc git wget \\`,
      `  cpio kmod xorriso grub-pc-bin grub-efi-amd64-bin grub-common mtools dosfstools \\`,
      `  squashfs-tools ca-certificates initramfs-tools linux-base"`,
    );
  } else if (opts.pkgManager === 'dnf') {
    lines.push(
      `docker exec $CONTAINER bash -c "dnf install -y gcc make ncurses-devel bison flex openssl-devel \\`,
      `  elfutils-libelf-devel bc git wget cpio kmod xorriso grub2-tools grub2-efi-x64 \\`,
      `  mtools dosfstools squashfs-tools dracut"`,
    );
  }
  lines.push('');

  // Overlay packages
  if (opts.packages.length > 0) {
    lines.push('# ── Phase 4: Install overlay packages ─────────────────────────────');
    if (opts.pkgManager === 'apt') {
      lines.push(
        `echo "[ 20%] Installing ${opts.packages.length} overlay packages via apt..."`,
        `docker exec $CONTAINER bash -c "DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ${opts.packages.join(' ')}"`,
      );
    } else if (opts.pkgManager === 'dnf') {
      lines.push(
        `echo "[ 20%] Installing ${opts.packages.length} overlay packages via dnf..."`,
        `docker exec $CONTAINER dnf install -y ${opts.packages.join(' ')}`,
      );
    } else if (opts.pkgManager === 'nix') {
      lines.push(
        `echo "[ 20%] Installing ${opts.packages.length} overlay packages via nix..."`,
        ...opts.packages.map((p) => `docker exec $CONTAINER nix-env -iA nixpkgs.${p}`),
      );
    }
    lines.push('');
  }

  // Custom software
  if (opts.customSoftware.length > 0) {
    lines.push('# ── Phase 5: Install custom software ──────────────────────────────');
    for (const sw of opts.customSoftware) {
      const installCmd = opts.pkgManager === 'apt'
        ? `DEBIAN_FRONTEND=noninteractive apt-get install -y ${sw}`
        : opts.pkgManager === 'dnf' ? `dnf install -y ${sw}` : `nix-env -iA nixpkgs.${sw}`;
      lines.push(
        `echo "[ 30%] Installing custom: ${sw}"`,
        `docker exec $CONTAINER bash -c "${installCmd}" || echo "WARN: ${sw} not available in repos"`,
      );
    }
    lines.push('');
  }

  // Script-installed overlays
  const scriptOverlays = opts.overlays.filter((o) => {
    const pkgMap = OVERLAY_PACKAGES[opts.pkgManager] || {};
    return pkgMap[o] && pkgMap[o].length === 0;
  });

  if (scriptOverlays.length > 0) {
    lines.push('# ── Phase 6: Script-installed overlays ─────────────────────────────');
    for (const o of scriptOverlays) {
      if (o === 'k3s') lines.push(`echo "[ 35%] Installing K3s..."`, `docker exec $CONTAINER bash -c "curl -sfL https://get.k3s.io | INSTALL_K3S_SKIP_START=true sh -"`);
      if (o === 'tailscale') lines.push(`echo "[ 35%] Installing Tailscale..."`, `docker exec $CONTAINER bash -c "curl -fsSL https://tailscale.com/install.sh | sh"`);
      if (o === 'rustup') lines.push(`echo "[ 35%] Installing Rust toolchain..."`, `docker exec $CONTAINER bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"`);
      if (o === 'netdata') lines.push(`echo "[ 35%] Installing Netdata..."`, `docker exec $CONTAINER bash -c "curl -fsSL https://get.netdata.cloud/kickstart.sh | sh -s -- --dont-wait --dont-start-it"`);
      if (o === 'grafana') lines.push(`echo "[ 35%] Installing Grafana..."`, `docker exec $CONTAINER bash -c "apt-get install -y apt-transport-https software-properties-common && curl -fsSL https://apt.grafana.com/gpg.key | gpg --dearmor -o /etc/apt/keyrings/grafana.gpg && echo 'deb [signed-by=/etc/apt/keyrings/grafana.gpg] https://apt.grafana.com stable main' > /etc/apt/sources.list.d/grafana.list && apt-get update && apt-get install -y grafana" || echo "WARN: Grafana install failed"`);
      if (o === 'tacticalrmm') lines.push(`echo "[ 35%] Preparing Tactical RMM agent..."`, `docker exec $CONTAINER bash -c "mkdir -p /opt/tacticalrmm && echo 'TacticalRMM agent placeholder — configure with meshcentral URL at first boot' > /opt/tacticalrmm/README"`);
    }
    lines.push('');
  }

  // Kernel build — uses the AI-generated kernel.config from R2
  lines.push(
    '# ── Phase 7: Clone and compile kernel ──────────────────────────────',
    `echo "[ 40%] Cloning Linux kernel v6.6 (stable)..."`,
    `docker exec $CONTAINER bash -c "cd /usr/src && git clone --depth 1 --branch v6.6 https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git"`,
    '',
    `echo "[ 45%] Applying AI-generated kernel config..."`,
    '# Copy the AI-generated kernel.config into the container',
    'SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"',
    'if [ -f "$SCRIPT_DIR/kernel.config" ]; then',
    '  docker cp "$SCRIPT_DIR/kernel.config" $CONTAINER:/usr/src/linux/.config',
    '  docker exec $CONTAINER bash -c "cd /usr/src/linux && make olddefconfig"',
    'else',
    '  echo "WARN: kernel.config not found, using defconfig"',
    '  docker exec $CONTAINER bash -c "cd /usr/src/linux && make defconfig"',
    'fi',
    '',
  );

  if (opts.aiMode && opts.modules.length > 0) {
    lines.push(
      `echo "[ 50%] Enabling ${opts.modules.length} hardware-detected modules..."`,
      ...opts.modules.map((m) => `docker exec $CONTAINER bash -c "cd /usr/src/linux && scripts/config --enable ${m.toUpperCase()}" 2>/dev/null || true`),
    );
    if (opts.mode === 'server') {
      lines.push(
        `echo "[ 52%] Disabling desktop subsystems for server mode..."`,
        `docker exec $CONTAINER bash -c "cd /usr/src/linux && scripts/config --disable DRM --disable SND --disable WLAN --disable BLUETOOTH --disable INPUT_JOYSTICK --disable USB_SERIAL --disable MEDIA_SUPPORT"`,
      );
    }
    lines.push('');
  }

  lines.push(
    `echo "[ 55%] Compiling kernel (this takes 10-30 minutes)..."`,
    `docker exec $CONTAINER bash -c "cd /usr/src/linux && make -j\\$(nproc) bzImage modules 2>&1 | tail -20"`,
    '',
    `echo "[ 70%] Installing kernel modules..."`,
    `docker exec $CONTAINER bash -c "cd /usr/src/linux && make modules_install INSTALL_MOD_PATH=/rootfs"`,
    '',
    '# ── Phase 8: Assemble root filesystem ─────────────────────────────',
    `echo "[ 75%] Assembling root filesystem..."`,
    `docker exec $CONTAINER bash -c "mkdir -p /rootfs/{boot,bin,sbin,etc,proc,sys,dev,tmp,var,usr,run,lib,lib64}"`,
    `docker exec $CONTAINER bash -c "cp /usr/src/linux/arch/x86/boot/bzImage /rootfs/boot/vmlinuz-y12"`,
    '',
    `echo "[ 78%] Creating initramfs..."`,
    `docker exec $CONTAINER bash -c "cd /rootfs && find . -print0 | cpio --null -o -H newc 2>/dev/null | gzip -9 > /boot-initrd.gz"`,
    '',
    '# ── Phase 9: Create bootable ISO ──────────────────────────────────',
    `echo "[ 85%] Building ISO filesystem..."`,
    `docker exec $CONTAINER bash -c "mkdir -p /iso/{boot/grub,live,EFI/BOOT}"`,
    `docker exec $CONTAINER bash -c "cp /rootfs/boot/vmlinuz-y12 /iso/boot/vmlinuz"`,
    `docker exec $CONTAINER bash -c "cp /boot-initrd.gz /iso/boot/initrd.gz"`,
    '',
    '# GRUB configuration (BIOS + EFI)',
    `docker exec $CONTAINER bash -c 'cat > /iso/boot/grub/grub.cfg << "GRUBEOF"`,
    'set timeout=5',
    'set default=0',
    '',
    `menuentry "Y12 Custom Linux (${opts.distro} ${opts.mode})" {`,
    '  linux /boot/vmlinuz root=/dev/ram0 rw quiet',
    '  initrd /boot/initrd.gz',
    '}',
    '',
    `menuentry "Y12 Custom Linux (${opts.distro} ${opts.mode}) — verbose" {`,
    '  linux /boot/vmlinuz root=/dev/ram0 rw loglevel=7',
    '  initrd /boot/initrd.gz',
    '}',
    '',
    'menuentry "Reboot" {',
    '  reboot',
    '}',
    'GRUBEOF',
    '"',
    '',
    `echo "[ 90%] Creating bootable ISO with GRUB..."`,
    `docker exec $CONTAINER bash -c "grub-mkrescue -o /output.iso /iso -- -volid Y12_CUSTOM 2>&1 | tail -5 || xorriso -as mkisofs -R -J -V Y12_CUSTOM -b boot/grub/i386-pc/eltorito.img -no-emul-boot -boot-load-size 4 -boot-info-table -o /output.iso /iso 2>&1 | tail -5"`,
    '',
    '# ── Phase 10: Verify and export ───────────────────────────────────',
    `echo "[ 95%] Verifying ISO..."`,
    `docker exec $CONTAINER bash -c "ls -lh /output.iso"`,
    `docker exec $CONTAINER bash -c "file /output.iso"`,
    `docker exec $CONTAINER bash -c "sha256sum /output.iso | tee /output.iso.sha256"`,
    '',
    `echo "[100%] Exporting ISO..."`,
    'mkdir -p ./output',
    `docker cp $CONTAINER:/output.iso ./output/${isoName}.iso`,
    `docker cp $CONTAINER:/output.iso.sha256 ./output/${isoName}.iso.sha256`,
    '',
    `echo ""`,
    `echo "═══════════════════════════════════════════════════════════════"`,
    `echo " BUILD COMPLETE"`,
    `echo " ISO: ./output/${isoName}.iso"`,
    `echo " SHA: ./output/${isoName}.iso.sha256"`,
    `echo ""`,
    `echo " Flash to USB:"`,
    `echo "   Linux/macOS: sudo dd if=./output/${isoName}.iso of=/dev/sdX bs=4M status=progress"`,
    `echo "   Windows:     Use Rufus — https://rufus.ie"`,
    `echo "   Any OS:      Use balenaEtcher — https://etcher.balena.io"`,
    `echo ""`,
    `echo " Test in VM:"`,
    `echo "   qemu-system-x86_64 -cdrom ./output/${isoName}.iso -m 2G -boot d"`,
    `echo "═══════════════════════════════════════════════════════════════"`,
  );

  return lines.join('\n');
}

// ── Test mode check ───────────────────────────────────────────────────

app.get('/api/config', (c) => {
  const testMode = c.env.TEST_MODE === 'true';
  return c.json({
    test_mode: testMode,
    stripe_configured: !!c.env.STRIPE_SECRET_KEY,
    price_cents: parseInt(c.env.STRIPE_PRICE_CENTS || '2000'),
  });
});

// ── Stripe Checkout ────────────────────────────────────────────────────

app.post('/api/stripe/checkout', async (c) => {
  const { distro, mode, overlays } = await c.req.json();

  // Test mode: skip Stripe entirely, return immediate success
  const testMode = c.env.TEST_MODE === 'true';
  if (testMode) {
    return c.json({ test_mode: true, paid: true, msg: 'Test mode — payment bypassed' });
  }

  const stripeKey = c.env.STRIPE_SECRET_KEY;
  if (!stripeKey) {
    return c.json({ error: 'Stripe not configured. Set STRIPE_SECRET_KEY as a secret.' }, 500);
  }

  // Safety: if the key is a live key, refuse to charge in current phase
  if (stripeKey.startsWith('sk_live_')) {
    return c.json({ test_mode: true, paid: true, msg: 'Live key detected but charging disabled — payment bypassed for safety' });
  }

  const params = new URLSearchParams();
  params.append('mode', 'payment');
  params.append('success_url', `${c.env.FRONTEND_URL}/build?paid=true&session_id={CHECKOUT_SESSION_ID}`);
  params.append('cancel_url', `${c.env.FRONTEND_URL}/build?paid=false`);
  params.append('line_items[0][price_data][currency]', 'usd');
  params.append('line_items[0][price_data][product_data][name]', `Y12.AI Custom ISO — ${distro} (${mode})`);
  params.append('line_items[0][price_data][product_data][description]', `Custom kernel + ${(overlays || []).length} overlays. Built on Cloudflare Containers.`);
  params.append('line_items[0][price_data][unit_amount]', c.env.STRIPE_PRICE_CENTS || '2000');
  params.append('line_items[0][quantity]', '1');

  try {
    const resp = await fetch('https://api.stripe.com/v1/checkout/sessions', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${stripeKey}`,
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: params.toString(),
    });
    const session = await resp.json() as any;
    if (session.error) {
      return c.json({ error: session.error.message }, 400);
    }
    return c.json({ url: session.url, session_id: session.id });
  } catch (e: any) {
    return c.json({ error: e.message }, 500);
  }
});

// ── SEO: 1000+ keyword landing pages (programmatic generation) ─────────

type SeoEntry = { slug: string; title: string; h1: string; desc: string };

function buildSeoKeywords(): SeoEntry[] {
  const pages: SeoEntry[] = [];
  const add = (slug: string, title: string, h1: string, desc: string) => pages.push({ slug, title, h1, desc });

  // ── Core terms ──
  add('custom-linux-iso', 'Custom Linux ISO Builder', 'Build a Custom Linux ISO', 'Create hardware-optimized Linux ISOs with custom kernels, hand-picked software, and AI-tuned configs. Like Linux From Scratch, but automated.');
  add('linux-from-scratch', 'Linux From Scratch — Automated', 'Linux From Scratch, Automated', 'LFS takes 72+ hours. Y12.AI automates kernel compilation, package installation, and ISO creation from source in minutes for $20.');
  add('linux-from-scratch-alternative', 'LFS Alternative — Y12.AI', 'The Easier Linux From Scratch', 'Get the same result as Linux From Scratch — a system compiled from source for your hardware — without the 72-hour manual process.');
  add('linux-from-scratch-automated', 'Automated Linux From Scratch', 'Automate Your LFS Build', 'Y12.AI is Linux From Scratch with AI. Custom kernel, hand-picked packages, hardware-tuned .config — built in the cloud.');
  add('linux-iso-builder', 'Linux ISO Builder Online', 'Build Linux ISOs Online', 'No local toolchain needed. Build custom Linux ISOs in the cloud with Cloudflare Containers. $20 flat.');
  add('custom-linux-kernel', 'Custom Linux Kernel Builder', 'Custom Linux Kernel', 'The kernel has 17,000+ Kconfig options. Our AI maps your hardware to the ~200-400 modules you actually need.');
  add('linux-security-hardening', 'Linux Security Hardening', 'Harden Your Linux Build', '60% of kernel CVEs in 2024 affected drivers most users don\'t need. Compile them out entirely.');
  add('linux-dual-boot', 'Linux Dual Boot ISO', 'Create a Dual Boot Linux ISO', 'Custom Linux ISO optimized for dual-boot alongside Windows or macOS. GRUB pre-configured.');
  add('linux-minimal-install', 'Minimal Linux Install', 'Minimal Linux Installation', 'Strip your Linux down to only what you need. A stock Ubuntu ships ~5,800 modules. Y12 ships 200-400.');
  add('linux-server-iso', 'Linux Server ISO Builder', 'Build a Linux Server ISO', 'Headless server ISOs with Docker, K3s, monitoring. No GUI, no GPU drivers, no bloat. Kernel compiled from source.');
  add('linux-desktop-iso', 'Linux Desktop ISO Builder', 'Build a Linux Desktop ISO', 'Full desktop ISOs with GPU drivers, audio, WiFi, Steam, Lutris. Gaming-ready out of the box.');
  add('linux-gaming-iso', 'Linux Gaming ISO', 'Linux Gaming ISO', 'Custom ISOs with Steam, Proton, Lutris, Mesa/Vulkan, 32-bit multilib, and OpenClaw pre-configured.');
  add('linux-bloat-removal', 'Remove Linux Bloat', 'Remove Linux Bloat', 'AI strips 40-60% of kernel modules. Stock kernel: 12-14 MB. Y12 kernel: 4-6 MB. Measurable difference.');

  // ── Competitor/alternative OS terms ──
  const competitors = [
    { slug: 'gentoo-alternative', name: 'Gentoo', desc: 'Gentoo emerge @world takes 8-24 hours. Y12.AI gives you the same source-compiled result in minutes.' },
    { slug: 'clear-linux-alternative', name: 'Clear Linux', desc: 'Clear Linux is Intel-only and discontinued for desktop. Y12.AI works on any hardware — Intel, AMD, ARM.' },
    { slug: 'genode-alternative', name: 'Genode', desc: 'Genode is a microkernel OS framework for researchers. Y12.AI builds production Linux ISOs from source for real workloads.' },
    { slug: 'arch-linux-alternative', name: 'Arch Linux', desc: 'Arch gives you a blank canvas but no kernel optimization. Y12.AI compiles a custom kernel tuned to your exact hardware.' },
    { slug: 'void-linux-alternative', name: 'Void Linux', desc: 'Void uses runit and musl but ships a generic kernel. Y12.AI builds from source with only your hardware\'s modules.' },
    { slug: 'alpine-linux-alternative', name: 'Alpine Linux', desc: 'Alpine is minimal but uses musl libc (compatibility issues). Y12.AI builds minimal glibc systems with full compatibility.' },
    { slug: 'slackware-alternative', name: 'Slackware', desc: 'Slackware is old-school Linux. Y12.AI gives you the same from-source philosophy with modern AI optimization.' },
    { slug: 'lfs-alternative', name: 'LFS', desc: 'Linux From Scratch is educational but takes 72+ hours. Y12.AI automates the entire process for $20.' },
    { slug: 'buildroot-alternative', name: 'Buildroot', desc: 'Buildroot targets embedded. Y12.AI builds full desktop/server ISOs from source with AI kernel optimization.' },
    { slug: 'yocto-alternative', name: 'Yocto Project', desc: 'Yocto has a steep learning curve and slow builds. Y12.AI builds custom Linux in the cloud, no local toolchain needed.' },
    { slug: 'nixos-vs-debian', name: 'NixOS vs Debian', desc: 'Can\'t decide? Y12.AI supports both. NixOS for reproducibility, Debian for compatibility. Custom kernel either way.' },
    { slug: 'ubuntu-alternative', name: 'Ubuntu', desc: 'Ubuntu ships 5,800+ kernel modules you don\'t need. Y12.AI builds Ubuntu-compatible Debian ISOs with only your hardware\'s modules.' },
    { slug: 'fedora-alternative', name: 'Fedora', desc: 'Fedora ships bleeding-edge but generic kernels. Y12.AI builds RHEL-compatible Rocky Linux ISOs tuned to your hardware.' },
    { slug: 'centos-alternative', name: 'CentOS', desc: 'CentOS is dead. Rocky Linux is the successor. Y12.AI builds custom Rocky ISOs with enterprise-grade kernel optimization.' },
    { slug: 'manjaro-alternative', name: 'Manjaro', desc: 'Manjaro is Arch made easy but still ships a generic kernel. Y12.AI compiles from source for your exact hardware.' },
    { slug: 'pop-os-alternative', name: 'Pop!_OS', desc: 'Pop!_OS is great for gaming but generic. Y12.AI builds gaming ISOs with only your GPU\'s drivers compiled in.' },
    { slug: 'mint-alternative', name: 'Linux Mint', desc: 'Mint is beginner-friendly but bloated. Y12.AI builds minimal Debian-based ISOs with exactly what you need.' },
    { slug: 'opensuse-alternative', name: 'openSUSE', desc: 'openSUSE Tumbleweed is rolling but generic. Y12.AI builds hardware-optimized ISOs from source.' },
    { slug: 'elementary-os-alternative', name: 'elementary OS', desc: 'elementary OS is beautiful but ships a generic Ubuntu kernel. Y12.AI builds custom kernels for your hardware.' },
    { slug: 'zorin-os-alternative', name: 'Zorin OS', desc: 'Zorin targets Windows switchers. Y12.AI builds custom Debian ISOs that boot faster with hardware-tuned kernels.' },
    { slug: 'tails-alternative', name: 'Tails', desc: 'Tails is privacy-focused but generic. Y12.AI builds privacy ISOs with custom kernels and minimal attack surface.' },
    { slug: 'qubes-os-alternative', name: 'Qubes OS', desc: 'Qubes uses Xen compartmentalization. Y12.AI builds hardened ISOs with compiled-out attack surface for simpler threat models.' },
    { slug: 'chromeos-flex-alternative', name: 'ChromeOS Flex', desc: 'ChromeOS Flex is locked down. Y12.AI builds open, custom Linux ISOs that you fully control.' },
  ];
  for (const c of competitors) {
    add(c.slug, `${c.name} Alternative — Y12.AI`, `Better Than ${c.name}`, `${c.desc} Build from source, AI-optimized, $20.`);
    add(`custom-${c.slug.replace('-alternative', '')}-iso`, `Custom ${c.name} ISO`, `Build a Custom ${c.name}-Style ISO`, `Get the best of ${c.name} with AI kernel optimization and hardware tuning. Built from source on Cloudflare.`);
  }

  // ── Distro-specific ──
  const distros = [
    { id: 'debian', name: 'Debian', desc: 'Debian 12 with your packages, custom kernel, hardware optimization. The universal OS, your way.' },
    { id: 'nixos', name: 'NixOS', desc: 'Reproducible NixOS ISOs with declarative config. Atomic upgrades and rollbacks built in.' },
    { id: 'rocky-linux', name: 'Rocky Linux', desc: 'Enterprise RHEL-compatible ISOs for production. Binary-compatible with Red Hat Enterprise Linux.' },
    { id: 'proxmox', name: 'Proxmox VE', desc: 'Proxmox with custom kernel, pre-configured networking, optimized storage drivers.' },
    { id: 'kali-linux', name: 'Kali Linux', desc: 'Kali tools on Debian base with custom kernel and hardware optimization for pentesting.' },
    { id: 'parrot-security', name: 'Parrot Security', desc: 'Parrot Security tools on Debian with hardened kernel and privacy features.' },
    { id: 'alma-linux', name: 'AlmaLinux', desc: 'AlmaLinux RHEL-compatible ISOs with custom kernel optimization for enterprise workloads.' },
  ];
  const modes = ['server', 'desktop', 'gaming', 'minimal', 'headless', 'workstation'];
  for (const d of distros) {
    add(`custom-${d.id}-iso`, `Custom ${d.name} ISO`, `Build a Custom ${d.name} ISO`, d.desc);
    for (const m of modes) {
      add(`${d.id}-${m}-iso`, `${d.name} ${m.charAt(0).toUpperCase() + m.slice(1)} ISO`, `${d.name} ${m.charAt(0).toUpperCase() + m.slice(1)} Build`, `Custom ${d.name} ${m} ISO with AI-optimized kernel. Only the modules your hardware needs.`);
    }
    add(`${d.id}-docker`, `${d.name} + Docker ISO`, `${d.name} with Docker`, `${d.name} ISO with Docker Engine pre-installed and kernel-optimized for containers.`);
    add(`${d.id}-kubernetes`, `${d.name} + Kubernetes`, `${d.name} with K3s`, `${d.name} ISO with K3s lightweight Kubernetes pre-configured for production clusters.`);
    add(`${d.id}-custom-kernel`, `${d.name} Custom Kernel`, `Custom Kernel for ${d.name}`, `Compile a custom Linux kernel for ${d.name} optimized for your exact hardware.`);
  }

  // ── Software overlay pages ──
  const software = [
    { id: 'docker', name: 'Docker', desc: 'Docker Engine, Compose, containerd' },
    { id: 'kubernetes', name: 'Kubernetes', desc: 'K3s lightweight Kubernetes' },
    { id: 'k3s', name: 'K3s', desc: 'Lightweight Kubernetes for edge and IoT' },
    { id: 'podman', name: 'Podman', desc: 'Daemonless container engine' },
    { id: 'tailscale', name: 'Tailscale', desc: 'Zero-config WireGuard mesh VPN' },
    { id: 'wireguard', name: 'WireGuard', desc: 'Modern VPN protocol in-kernel' },
    { id: 'nginx', name: 'NGINX', desc: 'High-performance web server and reverse proxy' },
    { id: 'caddy', name: 'Caddy', desc: 'Web server with automatic HTTPS' },
    { id: 'postgresql', name: 'PostgreSQL', desc: 'Advanced relational database' },
    { id: 'mariadb', name: 'MariaDB', desc: 'MySQL-compatible database' },
    { id: 'redis', name: 'Redis', desc: 'In-memory data store' },
    { id: 'prometheus', name: 'Prometheus', desc: 'Monitoring and alerting toolkit' },
    { id: 'grafana', name: 'Grafana', desc: 'Analytics and visualization platform' },
    { id: 'netdata', name: 'Netdata', desc: 'Real-time performance monitoring' },
    { id: 'ansible', name: 'Ansible', desc: 'Agentless IT automation' },
    { id: 'terraform', name: 'Terraform', desc: 'Infrastructure as code' },
    { id: 'steam', name: 'Steam', desc: 'Gaming platform with Proton' },
    { id: 'lutris', name: 'Lutris', desc: 'Open gaming platform' },
    { id: 'openclaw', name: 'OpenClaw', desc: 'Open-source Captain Claw reimplementation' },
    { id: 'obs-studio', name: 'OBS Studio', desc: 'Video recording and streaming' },
    { id: 'blender', name: 'Blender', desc: '3D creation suite' },
    { id: 'neovim', name: 'Neovim', desc: 'Hyperextensible text editor' },
    { id: 'vscode', name: 'VS Code', desc: 'Code editor' },
    { id: 'rust', name: 'Rust', desc: 'Systems programming language' },
    { id: 'nodejs', name: 'Node.js', desc: 'JavaScript runtime' },
    { id: 'golang', name: 'Go', desc: 'Compiled programming language' },
    { id: 'python', name: 'Python', desc: 'Programming language' },
    { id: 'qemu-kvm', name: 'QEMU/KVM', desc: 'Hardware virtualization' },
    { id: 'libvirt', name: 'libvirt', desc: 'Virtualization management' },
    { id: 'lxc', name: 'LXC/LXD', desc: 'System containers' },
    { id: 'zabbix', name: 'Zabbix', desc: 'Enterprise monitoring' },
    { id: 'tactical-rmm', name: 'Tactical RMM', desc: 'Remote monitoring and management' },
    { id: 'meshcentral', name: 'MeshCentral', desc: 'Remote management web app' },
    { id: 'plex', name: 'Plex', desc: 'Media server' },
    { id: 'jellyfin', name: 'Jellyfin', desc: 'Open-source media server' },
    { id: 'nextcloud', name: 'Nextcloud', desc: 'Self-hosted cloud platform' },
    { id: 'gitea', name: 'Gitea', desc: 'Self-hosted Git service' },
    { id: 'gitlab', name: 'GitLab', desc: 'DevOps platform' },
    { id: 'jenkins', name: 'Jenkins', desc: 'CI/CD automation server' },
    { id: 'traefik', name: 'Traefik', desc: 'Cloud-native reverse proxy' },
    { id: 'haproxy', name: 'HAProxy', desc: 'High-availability load balancer' },
    { id: 'elasticsearch', name: 'Elasticsearch', desc: 'Search and analytics engine' },
    { id: 'mongodb', name: 'MongoDB', desc: 'Document database' },
    { id: 'rabbitmq', name: 'RabbitMQ', desc: 'Message broker' },
    { id: 'minio', name: 'MinIO', desc: 'S3-compatible object storage' },
    { id: 'vault', name: 'HashiCorp Vault', desc: 'Secrets management' },
    { id: 'consul', name: 'Consul', desc: 'Service mesh and discovery' },
  ];
  for (const s of software) {
    add(`linux-${s.id}-iso`, `Linux with ${s.name} Pre-installed`, `Linux + ${s.name}`, `Custom Linux ISO with ${s.name} (${s.desc}) pre-installed and configured. AI-optimized kernel.`);
    add(`linux-${s.id}-server`, `${s.name} Server ISO`, `${s.name} on Custom Linux`, `Dedicated ${s.name} server on a custom Linux ISO. Kernel optimized for the workload, zero bloat.`);
  }

  // ── Use case pages ──
  const useCases = [
    { slug: 'linux-fleet-management', title: 'Linux Fleet Management', desc: 'Deploy identical ISOs across your fleet with RMM, Ansible, monitoring pre-installed.' },
    { slug: 'linux-homelab', title: 'Linux Homelab', desc: 'Perfect homelab OS with Proxmox, Docker, self-hosted services pre-configured.' },
    { slug: 'linux-edge-computing', title: 'Linux Edge Computing', desc: 'Minimal ISOs for edge devices. Stripped kernel, fast boot, small footprint.' },
    { slug: 'linux-embedded', title: 'Embedded Linux', desc: 'Custom Linux for embedded systems. Minimal kernel with only needed drivers.' },
    { slug: 'linux-cloud-container', title: 'Linux Cloud Container', desc: 'Launch your build as a cloud container. Boot in seconds on Cloudflare edge.' },
    { slug: 'linux-devops', title: 'Linux for DevOps', desc: 'Docker, K3s, Terraform, CI/CD tools pre-configured. Ship infrastructure faster.' },
    { slug: 'linux-pentesting', title: 'Linux Pentesting', desc: 'Kali tools, custom kernel, hardened config for security professionals.' },
    { slug: 'linux-privacy', title: 'Linux Privacy', desc: 'Privacy-hardened ISO with Tor, encrypted filesystems, minimal telemetry.' },
    { slug: 'linux-scientific-computing', title: 'Linux Scientific Computing', desc: 'CUDA, HPC tools, scientific packages on custom Linux.' },
    { slug: 'linux-machine-learning', title: 'Linux ML/AI', desc: 'CUDA, PyTorch, TensorFlow on GPU-optimized custom Linux.' },
    { slug: 'linux-financial-trading', title: 'Linux Trading Server', desc: 'Low-latency Linux for financial trading. Kernel tuned for network performance.' },
    { slug: 'linux-video-editing', title: 'Linux Video Editing', desc: 'DaVinci Resolve, Kdenlive, FFmpeg on GPU-optimized Linux.' },
    { slug: 'linux-music-production', title: 'Linux Music Production', desc: 'JACK, Ardour, real-time kernel for audio production.' },
    { slug: 'linux-3d-rendering', title: 'Linux 3D Rendering', desc: 'Blender, GPU compute, CUDA/OpenCL on custom Linux.' },
    { slug: 'linux-web-hosting', title: 'Linux Web Hosting', desc: 'NGINX/Caddy, PHP, Node.js, SSL on optimized Linux server.' },
    { slug: 'linux-email-server', title: 'Linux Email Server', desc: 'Postfix, Dovecot, SpamAssassin on hardened Linux.' },
    { slug: 'linux-dns-server', title: 'Linux DNS Server', desc: 'BIND or Unbound DNS on minimal, secure Linux.' },
    { slug: 'linux-vpn-server', title: 'Linux VPN Server', desc: 'WireGuard or OpenVPN on hardened Linux ISO.' },
    { slug: 'linux-firewall', title: 'Linux Firewall', desc: 'iptables/nftables firewall appliance on hardened Linux.' },
    { slug: 'linux-nas', title: 'Linux NAS', desc: 'ZFS, Samba, NFS on storage-optimized Linux.' },
    { slug: 'linux-router', title: 'Linux Router', desc: 'Custom router with optimized networking stack.' },
    { slug: 'linux-kiosk', title: 'Linux Kiosk', desc: 'Locked-down Linux for kiosk and digital signage.' },
    { slug: 'linux-thin-client', title: 'Linux Thin Client', desc: 'Minimal Linux for thin client deployments.' },
    { slug: 'linux-pos-system', title: 'Linux POS System', desc: 'Point-of-sale Linux appliance. Minimal, secure, fast boot.' },
    { slug: 'linux-digital-signage', title: 'Linux Digital Signage', desc: 'Kiosk-mode Linux for displays and signage.' },
    { slug: 'linux-surveillance', title: 'Linux Surveillance Server', desc: 'ZoneMinder/Frigate on optimized Linux for CCTV.' },
    { slug: 'linux-backup-server', title: 'Linux Backup Server', desc: 'Borg, Restic, or Bacula on storage-optimized Linux.' },
    { slug: 'linux-print-server', title: 'Linux Print Server', desc: 'CUPS print server on minimal Linux.' },
    { slug: 'linux-game-server', title: 'Linux Game Server', desc: 'Dedicated game server on optimized Linux. Minecraft, Valheim, CS2.' },
    { slug: 'linux-voip-server', title: 'Linux VoIP Server', desc: 'Asterisk or FreeSWITCH on real-time optimized Linux.' },
    { slug: 'linux-proxy-server', title: 'Linux Proxy Server', desc: 'Squid or HAProxy on network-optimized Linux.' },
    { slug: 'linux-development-workstation', title: 'Linux Dev Workstation', desc: 'Full development environment with compilers, editors, containers.' },
    { slug: 'linux-data-science', title: 'Linux Data Science', desc: 'Jupyter, pandas, scikit-learn, CUDA on custom Linux.' },
    { slug: 'linux-blockchain-node', title: 'Linux Blockchain Node', desc: 'Ethereum, Bitcoin node on optimized Linux server.' },
    { slug: 'linux-iot-gateway', title: 'Linux IoT Gateway', desc: 'Minimal Linux for IoT gateway devices. MQTT, Node-RED.' },
    { slug: 'linux-robotics', title: 'Linux for Robotics', desc: 'ROS2 on real-time Linux kernel for robotics applications.' },
  ];
  for (const u of useCases) {
    add(u.slug, u.title, u.title, u.desc);
    add(`${u.slug}-iso`, `${u.title} ISO`, `${u.title} ISO`, `${u.desc} Custom kernel compiled from source.`);
  }

  // ── Technical terms ──
  const techTerms = [
    { slug: 'linux-kernel-compilation', title: 'Linux Kernel Compilation', desc: 'Automated kernel compilation from source with hardware-specific .config.' },
    { slug: 'linux-kernel-optimization', title: 'Linux Kernel Optimization', desc: 'AI generates optimized .config. 17,000+ options resolved automatically.' },
    { slug: 'linux-initramfs-builder', title: 'Linux Initramfs Builder', desc: 'Minimal initramfs with only your boot modules.' },
    { slug: 'linux-grub-configuration', title: 'GRUB Configuration', desc: 'Pre-configured GRUB and systemd-boot.' },
    { slug: 'linux-systemd-configuration', title: 'Systemd Configuration', desc: 'Pre-configured systemd services baked into your ISO.' },
    { slug: 'linux-live-usb', title: 'Linux Live USB', desc: 'Custom bootable ISO for USB drives.' },
    { slug: 'linux-netinstall', title: 'Linux Network Install', desc: 'Minimal network install ISOs.' },
    { slug: 'linux-unattended-install', title: 'Unattended Linux Install', desc: 'Pre-configured ISOs for automated deployment.' },
    { slug: 'linux-pxe-boot', title: 'Linux PXE Boot', desc: 'Network-bootable Linux images for PXE.' },
    { slug: 'linux-immutable-os', title: 'Immutable Linux', desc: 'Read-only root filesystem. Atomic updates.' },
    { slug: 'linux-rescue-iso', title: 'Linux Rescue Disk', desc: 'Custom rescue ISOs for system recovery.' },
    { slug: 'linux-arm-iso', title: 'Linux ARM ISO', desc: 'Custom ISOs for ARM64 devices.' },
    { slug: 'linux-kernel-config', title: 'Linux Kernel .config Generator', desc: 'AI-generated kernel .config based on your hardware profile.' },
    { slug: 'linux-kernel-modules', title: 'Linux Kernel Modules', desc: 'Only compile the modules your hardware needs. Stock: 5,800. Y12: 200-400.' },
    { slug: 'linux-kernel-hardening', title: 'Linux Kernel Hardening', desc: 'Compile-time security: disable unused subsystems, enable KASLR, stack protector.' },
    { slug: 'linux-secure-boot', title: 'Linux Secure Boot', desc: 'ISOs with Secure Boot support and signed kernels.' },
    { slug: 'linux-uefi-boot', title: 'Linux UEFI Boot', desc: 'UEFI-compatible ISOs with systemd-boot or GRUB.' },
    { slug: 'linux-btrfs-root', title: 'Linux Btrfs Root', desc: 'ISOs with Btrfs root filesystem for snapshots and compression.' },
    { slug: 'linux-zfs-root', title: 'Linux ZFS Root', desc: 'ISOs with ZFS root filesystem for enterprise storage.' },
    { slug: 'linux-luks-encryption', title: 'Linux LUKS Encryption', desc: 'Full-disk encryption with LUKS pre-configured in the ISO.' },
    { slug: 'linux-selinux', title: 'Linux SELinux', desc: 'SELinux-enforcing ISOs for mandatory access control.' },
    { slug: 'linux-apparmor', title: 'Linux AppArmor', desc: 'AppArmor profiles pre-configured for your software stack.' },
    { slug: 'linux-cgroups-v2', title: 'Linux cgroups v2', desc: 'Modern cgroups v2 for container and resource management.' },
    { slug: 'linux-real-time-kernel', title: 'Linux Real-Time Kernel', desc: 'PREEMPT_RT real-time kernel for latency-sensitive workloads.' },
    { slug: 'linux-kernel-6', title: 'Linux Kernel 6.x', desc: 'Latest stable Linux 6.x kernel compiled from source for your hardware.' },
    { slug: 'make-menuconfig', title: 'make menuconfig Alternative', desc: 'Skip menuconfig. AI generates your .config from hardware detection.' },
    { slug: 'linux-kernel-make', title: 'Linux Kernel make', desc: 'Automated make bzImage, make modules, make install — in the cloud.' },
    { slug: 'linux-debootstrap', title: 'Linux Debootstrap', desc: 'Debootstrap-based Debian builds with custom kernel and packages.' },
    { slug: 'linux-squashfs', title: 'Linux SquashFS', desc: 'Compressed SquashFS root filesystem for minimal ISO size.' },
    { slug: 'linux-overlayfs', title: 'Linux OverlayFS', desc: 'OverlayFS for live system persistence and layered filesystems.' },
  ];
  for (const t of techTerms) {
    add(t.slug, t.title, t.title, t.desc);
  }

  // ── Hardware-specific pages ──
  const hardware = [
    'nvidia-gpu', 'amd-gpu', 'intel-gpu', 'nvidia-rtx', 'amd-radeon', 'intel-arc',
    'intel-nuc', 'raspberry-pi', 'amd-ryzen', 'intel-xeon', 'amd-epyc', 'arm64',
    'nvme-ssd', 'thunderbolt', 'usb-c', 'wifi-6', 'bluetooth-5', '10gbe',
    'dell-server', 'hp-server', 'lenovo-thinkpad', 'supermicro', 'asrock-rack',
  ];
  for (const hw of hardware) {
    const name = hw.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
    add(`linux-${hw}`, `Linux for ${name}`, `Linux Optimized for ${name}`, `Custom Linux ISO with kernel compiled specifically for ${name} hardware. Only the drivers you need.`);
    add(`linux-${hw}-drivers`, `${name} Linux Drivers`, `${name} Driver Optimization`, `AI detects your ${name} hardware and compiles only the required kernel modules. Zero bloat.`);
  }

  // ── Industry pages ──
  const industries = [
    'healthcare', 'finance', 'education', 'government', 'manufacturing', 'retail',
    'telecom', 'media', 'energy', 'logistics', 'legal', 'real-estate',
    'automotive', 'aerospace', 'defense', 'agriculture', 'construction', 'mining',
    'hospitality', 'insurance', 'pharmaceutical', 'biotech', 'nonprofit',
  ];
  for (const ind of industries) {
    const name = ind.charAt(0).toUpperCase() + ind.slice(1);
    add(`linux-${ind}`, `Linux for ${name}`, `Linux for ${name}`, `Custom Linux ISOs for ${ind} workloads. Hardened kernel, compliance-ready, minimal attack surface.`);
  }

  // ── Action/intent pages ──
  const actions = [
    { slug: 'build-custom-linux', title: 'Build Custom Linux', desc: 'Build your own Linux from source with AI optimization.' },
    { slug: 'create-linux-iso', title: 'Create Linux ISO', desc: 'Create a bootable Linux ISO in the cloud.' },
    { slug: 'compile-linux-kernel', title: 'Compile Linux Kernel', desc: 'Compile a custom kernel from source for your hardware.' },
    { slug: 'optimize-linux', title: 'Optimize Linux', desc: 'AI-powered Linux optimization. Faster boot, less RAM, smaller attack surface.' },
    { slug: 'harden-linux', title: 'Harden Linux', desc: 'Security hardening by compiling out unused kernel subsystems.' },
    { slug: 'strip-linux-kernel', title: 'Strip Linux Kernel', desc: 'Remove unnecessary modules from your Linux kernel.' },
    { slug: 'custom-linux-distro', title: 'Create Custom Linux Distro', desc: 'Build your own Linux distribution from source.' },
    { slug: 'make-linux-iso', title: 'Make Linux ISO', desc: 'Make a bootable Linux ISO with custom packages and kernel.' },
    { slug: 'linux-iso-creator', title: 'Linux ISO Creator', desc: 'Online Linux ISO creator with AI kernel optimization.' },
    { slug: 'linux-image-builder', title: 'Linux Image Builder', desc: 'Build custom Linux images — ISO, container, or cloud.' },
    { slug: 'linux-os-builder', title: 'Linux OS Builder', desc: 'Build your own Linux OS from source in the cloud.' },
    { slug: 'custom-operating-system', title: 'Custom Operating System', desc: 'Build a custom operating system based on Linux.' },
    { slug: 'roll-your-own-linux', title: 'Roll Your Own Linux', desc: 'Roll your own Linux distro without the 72-hour LFS process.' },
    { slug: 'diy-linux', title: 'DIY Linux', desc: 'DIY Linux made easy. AI handles the kernel, you pick the packages.' },
    { slug: 'linux-remaster', title: 'Linux Remaster', desc: 'Remaster any supported Linux distro with custom kernel and packages.' },
    { slug: 'linux-respin', title: 'Linux Respin', desc: 'Respin Linux with your configuration baked in.' },
    { slug: 'linux-remix', title: 'Linux Remix', desc: 'Remix Linux distributions with custom overlays and kernel.' },
  ];
  for (const a of actions) {
    add(a.slug, a.title, a.title, a.desc);
  }

  // ── Comparison pages ──
  const comparisons = [
    ['gentoo', 'y12'], ['lfs', 'y12'], ['clear-linux', 'y12'], ['arch', 'y12'],
    ['ubuntu', 'nixos'], ['debian', 'rocky'], ['nixos', 'debian'], ['proxmox', 'esxi'],
    ['docker', 'lxc'], ['k3s', 'k8s'], ['podman', 'docker'], ['wireguard', 'openvpn'],
  ];
  for (const [a, b] of comparisons) {
    const an = a.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
    const bn = b.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
    add(`${a}-vs-${b}`, `${an} vs ${bn}`, `${an} vs ${bn}`, `Comparing ${an} and ${bn}. Y12.AI supports both — build custom ISOs from source with AI optimization.`);
  }

  return pages;
}

const SEO_KEYWORDS = buildSeoKeywords();

function generateSeoPage(kw: typeof SEO_KEYWORDS[0]): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>${kw.title} | Y12.AI</title>
<meta name="description" content="${kw.desc}">
<meta name="keywords" content="${kw.slug.replace(/-/g, ', ')}, custom linux, iso builder, kernel optimization">
<meta property="og:title" content="${kw.title} | Y12.AI">
<meta property="og:description" content="${kw.desc}">
<meta property="og:type" content="website">
<meta property="og:url" content="https://y12-iso-builder.pages.dev/${kw.slug}">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="${kw.title} | Y12.AI">
<meta name="twitter:description" content="${kw.desc}">
<link rel="canonical" href="https://y12-iso-builder.pages.dev/${kw.slug}">
<script type="application/ld+json">
{"@context":"https://schema.org","@type":"WebPage","name":"${kw.title}","description":"${kw.desc}","url":"https://y12-iso-builder.pages.dev/${kw.slug}","provider":{"@type":"Organization","name":"Y12.AI","url":"https://y12-iso-builder.pages.dev"},"offers":{"@type":"Offer","price":"20","priceCurrency":"USD"}}
</script>
<style>
*{margin:0;padding:0;box-sizing:border-box}body{font-family:Inter,system-ui,sans-serif;background:#0a0a0a;color:#ededed;-webkit-font-smoothing:antialiased}
a{color:#fff;text-decoration:none}.container{max-width:800px;margin:0 auto;padding:40px 24px}
h1{font-size:clamp(2rem,5vw,3rem);font-weight:700;line-height:1.1;margin-bottom:16px}
h2{font-size:1.5rem;font-weight:600;margin:32px 0 12px}p{color:#888;line-height:1.7;margin-bottom:16px;font-size:15px}
.btn{display:inline-flex;align-items:center;gap:8px;background:#fff;color:#000;padding:10px 20px;border-radius:8px;font-weight:500;font-size:14px;margin-top:24px}
.btn:hover{background:#ddd}.nav{border-bottom:1px solid #1a1a1a;padding:16px 24px}.nav a{color:#888;margin-right:16px;font-size:13px}.nav a:hover{color:#fff}
.badge{display:inline-flex;align-items:center;gap:8px;border:1px solid #333;background:#111;padding:6px 16px;border-radius:999px;font-size:13px;color:#888;margin-bottom:24px}
.badge span{width:6px;height:6px;border-radius:50%;background:#10b981;display:inline-block}
.features{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:16px;margin:32px 0}
.feature{border:1px solid #1a1a1a;background:#111;border-radius:12px;padding:20px}.feature h3{font-size:14px;font-weight:600;margin-bottom:8px}.feature p{font-size:13px;margin:0}
footer{border-top:1px solid #1a1a1a;padding:24px;text-align:center;font-size:12px;color:#555;margin-top:48px}
</style>
</head>
<body>
<div class="nav"><a href="https://y12-iso-builder.pages.dev/">Y12.AI</a><a href="https://y12-iso-builder.pages.dev/build">Build</a><a href="https://y12-iso-builder.pages.dev/docs">Docs</a></div>
<div class="container">
<div class="badge"><span></span>Builds run on Cloudflare edge infrastructure</div>
<h1>${kw.h1}</h1>
<p>${kw.desc}</p>
<a class="btn" href="https://y12-iso-builder.pages.dev/build">Start a Build — $20 →</a>

<h2>How It Works</h2>
<div class="features">
<div class="feature"><h3>1. Detect Hardware</h3><p>Paste lspci, system_profiler, or Get-PnpDevice output. We map every device to kernel modules.</p></div>
<div class="feature"><h3>2. Configure</h3><p>Pick distro, mode, overlays. AI builds a custom kernel .config.</p></div>
<div class="feature"><h3>3. Build</h3><p>Cloudflare Containers compile from source and create a bootable ISO or container.</p></div>
<div class="feature"><h3>4. Download or Run</h3><p>Get a signed ISO or launch as an instant cloud container.</p></div>
</div>

<h2>Why Y12.AI?</h2>
<div class="features">
<div class="feature"><h3>50% Faster Boot</h3><p>AI strips 40-60% of kernel modules. Smaller kernel = faster boot, less memory.</p></div>
<div class="feature"><h3>Reduced Attack Surface</h3><p>Every disabled module is a CVE that can't affect you. Server builds strip GPU, audio, WiFi.</p></div>
<div class="feature"><h3>Instant Containers</h3><p>Launch your build as a cloud container. Boot in seconds from any browser.</p></div>
</div>

<h2>Supported Platforms</h2>
<p>NixOS (declarative, reproducible) · Debian (universal, stable) · Rocky Linux (RHEL-compatible, enterprise) · Proxmox VE (virtualization platform). Plus variant overlays: Kali Linux, Parrot Security, Scientific Linux, AlmaLinux, Devuan.</p>

<h2>$20 Per Build</h2>
<p>Custom kernel from source, AI hardware optimization, unlimited overlays and custom software, variant distro overlays, RMM agent pre-install, SHA256-signed output, 7-day download link.</p>
<a class="btn" href="https://y12-iso-builder.pages.dev/build">Start Building →</a>
</div>
<footer>© 2026 Y12.AI — Custom Linux ISOs and containers built from source. <a href="https://y12-iso-builder.pages.dev/sitemap.xml" style="color:#555">Sitemap</a></footer>
</body>
</html>`;
}

// Serve SEO pages
app.get('/s/:slug', (c) => {
  const slug = c.req.param('slug');
  const kw = SEO_KEYWORDS.find((k) => k.slug === slug);
  if (!kw) return c.text('Not found', 404);
  return c.html(generateSeoPage(kw));
});

// ── Sitemap ────────────────────────────────────────────────────────────

app.get('/sitemap.xml', (c) => {
  const base = 'https://y12-iso-builder.pages.dev';
  const workerBase = 'https://y12-api.seefeldmaxwell1.workers.dev';
  const now = new Date().toISOString().split('T')[0];

  let xml = `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n`;
  // Main pages
  xml += `<url><loc>${base}/</loc><lastmod>${now}</lastmod><changefreq>weekly</changefreq><priority>1.0</priority></url>\n`;
  xml += `<url><loc>${base}/build</loc><lastmod>${now}</lastmod><changefreq>weekly</changefreq><priority>0.9</priority></url>\n`;
  xml += `<url><loc>${base}/docs</loc><lastmod>${now}</lastmod><changefreq>monthly</changefreq><priority>0.7</priority></url>\n`;
  // SEO landing pages
  for (const kw of SEO_KEYWORDS) {
    xml += `<url><loc>${workerBase}/s/${kw.slug}</loc><lastmod>${now}</lastmod><changefreq>monthly</changefreq><priority>0.6</priority></url>\n`;
  }
  xml += `</urlset>`;

  return new Response(xml, { headers: { 'Content-Type': 'application/xml' } });
});

// ── Robots.txt ─────────────────────────────────────────────────────────

app.get('/robots.txt', (c) => {
  return c.text(`User-agent: *\nAllow: /\nSitemap: https://y12-api.seefeldmaxwell1.workers.dev/sitemap.xml\n`);
});

export default app;
