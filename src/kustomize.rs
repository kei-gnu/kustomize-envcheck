use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub struct KustomizeBuilder;

impl KustomizeBuilder {
    pub fn new() -> Self {
        Self
    }

    pub async fn build(&self, dir: &Path) -> Result<String> {
        let output = Command::new("kustomize")
            .arg("build")
            .arg(dir)
            .output()
            .context("Failed to execute kustomize build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Kustomize build failed: {}", stderr);
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Failed to parse kustomize output as UTF-8")?;

        Ok(stdout)
    }

    pub fn check_kustomize_installed() -> Result<()> {
        let output = Command::new("kustomize")
            .arg("version")
            .output()
            .context("Failed to check kustomize installation")?;

        if !output.status.success() {
            anyhow::bail!("Kustomize is not installed or not in PATH");
        }

        Ok(())
    }
}