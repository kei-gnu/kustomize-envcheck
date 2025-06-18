mod cli;
mod config;
mod kustomize;
mod k8s;
mod checker;
mod output;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use config::Config;
use checker::EnvChecker;
use kustomize::KustomizeBuilder;
use k8s::K8sParser;
use output::OutputFormatter;
use std::path::Path;
use std::process;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {:#}", e);
        process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("Verbose mode enabled");
    }

    KustomizeBuilder::check_kustomize_installed()
        .context("Kustomize check failed")?;

    let config_path = Path::new(&cli.config);
    let config = Config::from_file(config_path)
        .with_context(|| format!("Failed to load config from {}", cli.config))?;

    let kustomize_dir = Path::new(&cli.kustomize_dir);
    if !kustomize_dir.exists() {
        anyhow::bail!("Kustomize directory does not exist: {}", cli.kustomize_dir);
    }

    if cli.verbose {
        eprintln!("Building Kustomize manifests from: {}", cli.kustomize_dir);
    }

    let kustomize_builder = KustomizeBuilder::new();
    let yaml_content = kustomize_builder.build(kustomize_dir).await
        .with_context(|| format!("Failed to build Kustomize from {}", cli.kustomize_dir))?;

    if cli.verbose {
        eprintln!("Parsing Kubernetes resources...");
    }

    let parser = K8sParser::new();
    let resources = parser.parse_yaml(&yaml_content)
        .context("Failed to parse Kubernetes YAML")?;

    if cli.verbose {
        eprintln!("Found {} resources", resources.len());
    }

    let checker = EnvChecker::new(config);
    let results = checker.check_resources(&resources, cli.environment.as_deref());

    let formatter = OutputFormatter::new(cli.show_extra_vars, cli.verbose);
    let output = formatter.format(&results, &cli.output)?;

    println!("{}", output);

    let has_failures = results.iter().any(|r| matches!(r.status, checker::CheckStatus::Failed));
    if has_failures {
        process::exit(1);
    }

    Ok(())
}
