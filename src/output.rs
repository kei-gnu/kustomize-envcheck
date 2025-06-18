use crate::checker::{CheckResult, CheckStatus};
use crate::cli::OutputFormat;
use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutput {
    pub status: String,
    pub summary: Summary,
    pub applications: Vec<ApplicationResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub total_applications: usize,
    pub missing_required: usize,
    pub missing_optional: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationResult {
    pub name: String,
    pub status: String,
    pub missing_required: Vec<String>,
    pub missing_optional: Vec<String>,
    pub using_defaults: Vec<String>,
    pub extra_vars: Vec<String>,
}

pub struct OutputFormatter {
    show_extra_vars: bool,
}

impl OutputFormatter {
    pub fn new(show_extra_vars: bool) -> Self {
        Self { show_extra_vars }
    }

    pub fn format(&self, results: &[CheckResult], format: &OutputFormat) -> Result<String> {
        match format {
            OutputFormat::Text => self.format_text(results),
            OutputFormat::Json => self.format_json(results),
        }
    }

    fn format_text(&self, results: &[CheckResult]) -> Result<String> {
        let mut output = String::new();

        writeln!(&mut output, "{}", "Environment Variable Check Results".bold())?;
        writeln!(&mut output, "{}", "=".repeat(50))?;
        writeln!(&mut output)?;

        for result in results {
            writeln!(&mut output, "{} {}", "Application:".bold(), result.app_name)?;
            
            let status_icon = match result.status {
                CheckStatus::Passed => "✓".green(),
                CheckStatus::Failed => "✗".red(),
                CheckStatus::Warning => "⚠".yellow(),
            };
            
            writeln!(&mut output, "  {} Status: {:?}", status_icon, result.status)?;
            
            if !result.missing_required.is_empty() {
                writeln!(&mut output, "  {} Missing required variables:", "✗".red())?;
                for var in &result.missing_required {
                    writeln!(&mut output, "    - {}", var.red())?;
                }
            }
            
            if !result.missing_optional.is_empty() {
                writeln!(&mut output, "  {} Missing optional variables:", "⚠".yellow())?;
                for var in &result.missing_optional {
                    writeln!(&mut output, "    - {}", var.yellow())?;
                }
            }
            
            if !result.using_defaults.is_empty() {
                writeln!(&mut output, "  {} Using default values:", "ℹ".blue())?;
                for var in &result.using_defaults {
                    writeln!(&mut output, "    - {}", var)?;
                }
            }
            
            if self.show_extra_vars && !result.extra_vars.is_empty() {
                writeln!(&mut output, "  {} Extra variables (not in config):", "ℹ".blue())?;
                for var in &result.extra_vars {
                    writeln!(&mut output, "    - {}", var)?;
                }
            }
            
            if !result.validation_errors.is_empty() {
                writeln!(&mut output, "  {} Validation errors:", "✗".red())?;
                for error in &result.validation_errors {
                    writeln!(&mut output, "    - {}: {}", error.var_name, error.message)?;
                }
            }
            
            writeln!(&mut output)?;
        }

        writeln!(&mut output, "{}", "Summary".bold())?;
        writeln!(&mut output, "{}", "-".repeat(50))?;
        
        let total_apps = results.len();
        let failed_apps = results.iter().filter(|r| matches!(r.status, CheckStatus::Failed)).count();
        let warning_apps = results.iter().filter(|r| matches!(r.status, CheckStatus::Warning)).count();
        let total_missing_required: usize = results.iter().map(|r| r.missing_required.len()).sum();
        let total_missing_optional: usize = results.iter().map(|r| r.missing_optional.len()).sum();
        
        writeln!(&mut output, "Total applications: {}", total_apps)?;
        writeln!(&mut output, "Failed: {} | Warnings: {} | Passed: {}", 
            failed_apps.to_string().red(),
            warning_apps.to_string().yellow(),
            (total_apps - failed_apps - warning_apps).to_string().green()
        )?;
        writeln!(&mut output, "Missing required variables: {}", total_missing_required.to_string().red())?;
        writeln!(&mut output, "Missing optional variables: {}", total_missing_optional.to_string().yellow())?;

        Ok(output)
    }

    fn format_json(&self, results: &[CheckResult]) -> Result<String> {
        let total_missing_required: usize = results.iter().map(|r| r.missing_required.len()).sum();
        let total_missing_optional: usize = results.iter().map(|r| r.missing_optional.len()).sum();
        
        let overall_status = if results.iter().any(|r| matches!(r.status, CheckStatus::Failed)) {
            "failed"
        } else if results.iter().any(|r| matches!(r.status, CheckStatus::Warning)) {
            "warning"
        } else {
            "passed"
        };

        let json_output = JsonOutput {
            status: overall_status.to_string(),
            summary: Summary {
                total_applications: results.len(),
                missing_required: total_missing_required,
                missing_optional: total_missing_optional,
            },
            applications: results
                .iter()
                .map(|r| ApplicationResult {
                    name: r.app_name.clone(),
                    status: format!("{:?}", r.status).to_lowercase(),
                    missing_required: r.missing_required.clone(),
                    missing_optional: r.missing_optional.clone(),
                    using_defaults: r.using_defaults.clone(),
                    extra_vars: if self.show_extra_vars { r.extra_vars.clone() } else { vec![] },
                })
                .collect(),
        };

        Ok(serde_json::to_string_pretty(&json_output)?)
    }
}