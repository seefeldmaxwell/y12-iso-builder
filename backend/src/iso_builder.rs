use crate::models::*;
use crate::AppState;
use anyhow::{Context, Result};
use chrono::Utc;
use std::process::Command;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::fs;
use tokio::process::Command as AsyncCommand;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use tracing::{info, warn, error};

pub struct IsoBuilder {
    work_dir: PathBuf,
}

impl IsoBuilder {
    pub fn new() -> Self {
        Self {
            work_dir: PathBuf::from("/tmp/iso-builder"),
        }
    }

    pub async fn build_iso(&self, job_id: Uuid, config: IsoConfig) -> Result<()> {
        info!("Starting ISO build for job {}: {}", job_id, config.name);
        
        let temp_dir = TempDir::new()?;
        let build_dir = temp_dir.path();
        
        // Update job status to Building
        self.update_job_status(job_id, BuildStatus::Building, 0, "Starting build process").await?;
        
        // Step 1: Prepare base system
        self.prepare_base_system(&config, build_dir).await?;
        self.update_job_status(job_id, BuildStatus::Building, 20, "Base system prepared").await?;
        
        // Step 2: Install packages
        self.install_packages(&config, build_dir).await?;
        self.update_job_status(job_id, BuildStatus::Building, 40, "Packages installed").await?;
        
        // Step 3: Apply customizations
        self.apply_customizations(&config, build_dir).await?;
        self.update_job_status(job_id, BuildStatus::Building, 60, "Customizations applied").await?;
        
        // Step 4: Create ISO
        let iso_path = self.create_iso_image(&config, build_dir).await?;
        self.update_job_status(job_id, BuildStatus::Packaging, 80, "ISO image created").await?;
        
        // Step 5: Upload to storage
        let download_url = self.upload_iso(job_id, &iso_path).await?;
        self.update_job_status(job_id, BuildStatus::Uploading, 90, "ISO uploaded").await?;
        
        // Step 6: Complete
        self.update_job_status(job_id, BuildStatus::Completed, 100, "Build completed successfully").await?;
        self.set_download_url(job_id, download_url).await?;
        
        info!("ISO build completed for job {}", job_id);
        Ok(())
    }

    async fn prepare_base_system(&self, config: &IsoConfig, build_dir: &Path) -> Result<()> {
        info!("Preparing base system for {}", config.distro.name);
        
        let chroot_dir = build_dir.join("chroot");
        fs::create_dir_all(&chroot_dir).await?;
        
        // Use debootstrap for Ubuntu/Debian
        match config.distro.category {
            DistroCategory::Ubuntu | DistroCategory::Debian => {
                let mut cmd = AsyncCommand::new("debootstrap");
                cmd.args(&[
                    "--arch=amd64",
                    "--variant=minbase",
                    &config.distro.id.split('-').next().unwrap_or("ubuntu"),
                    chroot_dir.to_str().unwrap(),
                    "http://archive.ubuntu.com/ubuntu/",
                ]);
                
                let output = cmd.output().await
                    .context("Failed to run debootstrap")?;
                
                if !output.status.success() {
                    return Err(anyhow::anyhow!("debootstrap failed: {}", String::from_utf8_lossy(&output.stderr)));
                }
            }
            DistroCategory::Arch => {
                // Use pacstrap for Arch
                let mut cmd = AsyncCommand::new("pacstrap");
                cmd.args(&["-K", chroot_dir.to_str().unwrap(), "base"]);
                
                let output = cmd.output().await
                    .context("Failed to run pacstrap")?;
                
                if !output.status.success() {
                    return Err(anyhow::anyhow!("pacstrap failed: {}", String::from_utf8_lossy(&output.stderr)));
                }
            }
            DistroCategory::Fedora => {
                // Use dnf for Fedora
                let mut cmd = AsyncCommand::new("dnf");
                cmd.args(&[
                    "install",
                    "-y",
                    "--releasever=39",
                    "--installroot",
                    chroot_dir.to_str().unwrap(),
                    "fedora-release",
                    "coreutils",
                ]);
                
                let output = cmd.output().await
                    .context("Failed to run dnf")?;
                
                if !output.status.success() {
                    return Err(anyhow::anyhow!("dnf failed: {}", String::from_utf8_lossy(&output.stderr)));
                }
            }
            DistroCategory::Custom => {
                return Err(anyhow::anyhow!("Custom distros not yet supported"));
            }
        }
        
        Ok(())
    }

    async fn install_packages(&self, config: &IsoConfig, build_dir: &Path) -> Result<()> {
        info!("Installing packages for {}", config.name);
        
        let chroot_dir = build_dir.join("chroot");
        
        // Install desktop environment
        if let Some(de) = &config.desktop_environment {
            self.install_package_in_chroot(&chroot_dir, de).await?;
        }
        
        // Install additional packages
        for package in &config.packages {
            self.install_package_in_chroot(&chroot_dir, package).await?;
        }
        
        Ok(())
    }

    async fn install_package_in_chroot(&self, chroot_dir: &Path, package: &str) -> Result<()> {
        let package_manager = self.detect_package_manager(chroot_dir).await?;
        
        let mut cmd = AsyncCommand::new("chroot");
        cmd.arg(chroot_dir.to_str().unwrap());
        
        match package_manager {
            "apt" => {
                cmd.args(&["apt-get", "install", "-y", package]);
            }
            "pacman" => {
                cmd.args(&["pacman", "-S", "--noconfirm", package]);
            }
            "dnf" => {
                cmd.args(&["dnf", "install", "-y", package]);
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported package manager: {}", package_manager));
            }
        }
        
        let output = cmd.output().await
            .context(format!("Failed to install package: {}", package))?;
        
        if !output.status.success() {
            warn!("Failed to install package {}: {}", package, String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    async fn detect_package_manager(&self, chroot_dir: &Path) -> Result<String> {
        // Check for package managers in the chroot
        let managers = vec!["apt-get", "pacman", "dnf", "yum"];
        
        for manager in managers {
            let manager_path = chroot_dir.join("usr/bin").join(manager);
            if manager_path.exists() {
                return Ok(manager.to_string());
            }
        }
        
        Err(anyhow::anyhow!("No supported package manager found"))
    }

    async fn apply_customizations(&self, config: &IsoConfig, build_dir: &Path) -> Result<()> {
        info!("Applying customizations for {}", config.name);
        
        let chroot_dir = build_dir.join("chroot");
        
        // Apply theme customizations
        self.apply_theme_customizations(&config.theme, &chroot_dir).await?;
        
        // Run custom scripts
        for script in &config.custom_scripts {
            self.run_custom_script(&chroot_dir, script).await?;
        }
        
        Ok(())
    }

    async fn apply_theme_customizations(&self, theme: &ThemeConfig, chroot_dir: &Path) -> Result<()> {
        // Create theme directories
        let themes_dir = chroot_dir.join("usr/share/themes");
        fs::create_dir_all(&themes_dir).await?;
        
        let icons_dir = chroot_dir.join("usr/share/icons");
        fs::create_dir_all(&icons_dir).await?;
        
        // Apply GTK theme if specified
        if let Some(gtk_theme) = &theme.gtk_theme {
            // This would typically involve downloading and installing the theme
            info!("Setting GTK theme to: {}", gtk_theme);
        }
        
        // Apply icon theme if specified
        if let Some(icon_theme) = &theme.icon_theme {
            info!("Setting icon theme to: {}", icon_theme);
        }
        
        // Apply wallpaper if specified
        if let Some(wallpaper_url) = &theme.wallpaper {
            info!("Setting wallpaper to: {}", wallpaper_url);
            // Download and set wallpaper
        }
        
        Ok(())
    }

    async fn run_custom_script(&self, chroot_dir: &Path, script: &str) -> Result<()> {
        let script_file = chroot_dir.join("tmp/custom-script.sh");
        
        // Write script to file
        let mut file = fs::File::create(&script_file).await?;
        file.write_all(script.as_bytes()).await?;
        file.flush().await?;
        
        // Make script executable
        fs::set_permissions(&script_file, std::fs::Permissions::from_mode(0o755)).await?;
        
        // Run script in chroot
        let mut cmd = AsyncCommand::new("chroot");
        cmd.arg(chroot_dir.to_str().unwrap())
           .arg("/tmp/custom-script.sh");
        
        let output = cmd.output().await
            .context("Failed to run custom script")?;
        
        if !output.status.success() {
            warn!("Custom script failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        // Clean up script
        fs::remove_file(&script_file).await?;
        
        Ok(())
    }

    async fn create_iso_image(&self, config: &IsoConfig, build_dir: &Path) -> Result<PathBuf> {
        info!("Creating ISO image for {}", config.name);
        
        let chroot_dir = build_dir.join("chroot");
        let iso_dir = build_dir.join("iso");
        fs::create_dir_all(&iso_dir).await?;
        
        // Create live system files
        self.create_live_system(&chroot_dir, &iso_dir).await?;
        
        // Create squashfs
        let squashfs_path = iso_dir.join("filesystem.squashfs");
        let mut cmd = AsyncCommand::new("mksquashfs");
        cmd.args(&[
            chroot_dir.to_str().unwrap(),
            squashfs_path.to_str().unwrap(),
            "-e", "boot",
        ]);
        
        let output = cmd.output().await
            .context("Failed to create squashfs")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("mksquashfs failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // Create ISO
        let iso_path = build_dir.join(format!("{}.iso", config.name));
        let mut cmd = AsyncCommand::new("xorriso");
        cmd.args(&[
            "-as", "mkisofs",
            "-iso-level", "3",
            "-full-iso9660-filenames",
            "-volid", &config.name,
            "-appid", "Linux ISO Creator",
            "-publisher", "Linux ISO Creator",
            "-preparer", "Linux ISO Creator",
            "-eltorito-boot", "boot/grub/i386-pc/eltorito.img",
            "-no-emul-boot",
            "-boot-load-size", "4",
            "-boot-info-table",
            "-eltorito-alt-boot",
            "-e", "boot/grub/efi.img",
            "-no-emul-boot",
            "-isohybrid-gpt-basdat",
            "-output", iso_path.to_str().unwrap(),
            iso_dir.to_str().unwrap(),
        ]);
        
        let output = cmd.output().await
            .context("Failed to create ISO")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("xorriso failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(iso_path)
    }

    async fn create_live_system(&self, chroot_dir: &Path, iso_dir: &Path) -> Result<()> {
        // Create boot directory
        let boot_dir = iso_dir.join("boot");
        fs::create_dir_all(&boot_dir).await?;
        
        // Create GRUB configuration
        let grub_dir = boot_dir.join("grub");
        fs::create_dir_all(&grub_dir).await?;
        
        let grub_cfg = r#"set timeout=10
set default=0

menuentry "Linux Live" {
    linux /boot/vmlinuz boot=live quiet splash
    initrd /boot/initrd.img
}
"#;
        
        let grub_cfg_path = grub_dir.join("grub.cfg");
        fs::write(&grub_cfg_path, grub_cfg).await?;
        
        // Copy kernel and initrd if they exist
        let vmlinuz = chroot_dir.join("boot/vmlinuz-*");
        let initrd = chroot_dir.join("boot/initrd.img-*");
        
        // This is simplified - in reality you'd need to find the actual kernel files
        info!("Creating live system boot files");
        
        Ok(())
    }

    async fn upload_iso(&self, job_id: Uuid, iso_path: &Path) -> Result<String> {
        info!("Uploading ISO for job {}", job_id);
        
        // For now, just return a mock URL
        // In production, this would upload to R2, S3, or similar
        let filename = iso_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("custom-linux.iso");
        
        let download_url = format!("https://storage.example.com/isos/{}/{}", job_id, filename);
        
        // Simulate upload
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        Ok(download_url)
    }

    async fn update_job_status(&self, job_id: Uuid, status: BuildStatus, progress: u8, message: &str) -> Result<()> {
        // This would update the job in the database/state
        info!("Job {}: {} - {}% - {}", job_id, status, progress, message);
        Ok(())
    }

    async fn set_download_url(&self, job_id: Uuid, url: String) -> Result<()> {
        // This would update the job with the download URL
        info!("Job {}: Download URL set to {}", job_id, url);
        Ok(())
    }
}
