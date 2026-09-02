use std::collections::HashMap;
use std::os::windows::process::CommandExt;

use anyhow::bail;
use async_trait::async_trait;
use tokio::process::Command;

use crate::models::installed_packages::UpgradeInfo;
use crate::package_manager::table_parser::parse_table;
use crate::package_manager::winget_manager_trait::WingetManagerTrait;

const CREATE_NO_WINDOW: u32 = 0x08000000;

struct WingetPackageEntry {
    id: String,
    version: String,
    available: Option<String>,
}

impl WingetPackageEntry {
    fn is_upgradable(&self) -> bool {
        self.version != "Unknown" && self.available.is_some()
    }
}

pub struct WingetManager;

impl WingetManager {
    async fn list_entries() -> anyhow::Result<Vec<WingetPackageEntry>> {
        let output = Command::new("winget")
            .args(["list", "--accept-source-agreements"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .await?;

        if !output.status.success() {
            bail!("Failed to list packages");
        }

        let output = String::from_utf8_lossy(&output.stdout);
        let mut output = output.lines();
        let first_line = output.next();
        let first_line = first_line.and_then(|x| x.split('\r').next_back());
        let lines = first_line.into_iter().chain(output);

        let (column_len, cells) = parse_table(lines)?;
        if column_len < 4 {
            bail!("Invalid header: {first_line:?}");
        }

        Ok(cells
            .into_iter()
            .skip(column_len)
            .collect::<Vec<_>>()
            .chunks(column_len)
            .map(|columns| {
                let id = columns[1].clone();
                let version = columns[2].clone();
                let available = if column_len >= 5 {
                    columns.get(3).filter(|s| !s.is_empty()).cloned()
                } else {
                    None
                };
                WingetPackageEntry {
                    id,
                    version,
                    available,
                }
            })
            .collect())
    }
}

#[async_trait]
impl WingetManagerTrait for WingetManager {
    fn is_installed(&self) -> bool {
        std::process::Command::new("winget")
            .arg("--version")
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn is_installed_async(&self) -> bool {
        Command::new("winget")
            .arg("--version")
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn get_installed_packages(
        &self,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        let entries = Self::list_entries().await.map_err(|e| e.to_string())?;
        Ok(entries.into_iter().map(|e| (e.id, e.version)).collect())
    }

    async fn get_upgradable_packages(
        &self,
    ) -> Result<HashMap<String, UpgradeInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let entries = Self::list_entries().await.map_err(|e| e.to_string())?;
        Ok(entries
            .into_iter()
            .filter(|e| e.is_upgradable())
            .map(|e| {
                let available = e.available.clone().unwrap_or_default();
                (e.id, UpgradeInfo::new(&e.version, &available))
            })
            .collect())
    }

    async fn install(
        &self,
        id: &str,
        version: Option<&str>,
        accept_agreements: bool,
        silent: bool,
    ) -> bool {
        let mut args = vec!["install", "--id", id, "--exact"];
        if let Some(v) = version {
            args.push("--version");
            args.push(v);
        }
        if accept_agreements {
            args.push("--accept-source-agreements");
            args.push("--accept-package-agreements");
        }
        if silent {
            args.push("--silent");
        }
        Command::new("winget")
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    async fn uninstall(&self, id: &str, accept_agreements: bool, silent: bool) -> bool {
        let mut args = vec!["uninstall", "--id", id, "--exact"];
        if accept_agreements {
            args.push("--accept-source-agreements");
            args.push("--accept-package-agreements");
        }
        if silent {
            args.push("--silent");
        }
        Command::new("winget")
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    async fn upgrade(&self, id: &str, accept_agreements: bool, silent: bool) -> bool {
        let mut args = vec!["upgrade", "--id", id, "--exact"];
        if accept_agreements {
            args.push("--accept-source-agreements");
            args.push("--accept-package-agreements");
        }
        if silent {
            args.push("--silent");
        }
        Command::new("winget")
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    async fn package_exists(
        &self,
        id: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("winget")
            .args([
                "search",
                "--id",
                id,
                "--exact",
                "--accept-source-agreements",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let text = String::from_utf8_lossy(&output.stdout);
        Ok(text
            .lines()
            .any(|l| l.split_whitespace().any(|t| t.eq_ignore_ascii_case(id))))
    }

    async fn ensure_installed(&self) {
        if self.is_installed_async().await {
            return;
        }
        let ps_command = r#"
            if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
                try {
                    Add-AppxPackage -Register "C:\Program Files\WindowsApps\Microsoft.DesktopAppInstaller_*_x64__8wekyb3d8bbwe\AppxManifest.xml" -DisableDevelopmentMode 2>$null
                } catch {
                    Start-Process -FilePath "ms-windows-store:" -ArgumentList "pdp?ProductId=9NBLGGH4NNS1" -PassThru | Out-Null
                }
            }
        "#;
        let _ = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                ps_command,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .await;
    }
}
