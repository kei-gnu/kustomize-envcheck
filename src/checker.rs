use crate::config::Config;
use crate::k8s::{K8sResource, ContainerInfo};
use anyhow::Result;
use regex::Regex;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub app_name: String,
    pub status: CheckStatus,
    pub missing_required: Vec<String>,
    pub missing_optional: Vec<String>,
    pub using_defaults: Vec<String>,
    pub extra_vars: Vec<String>,
    pub validation_errors: Vec<ValidationError>,
    pub passed_vars: Vec<(String, String)>, // (name, value)
}

#[derive(Debug, Clone)]
pub enum CheckStatus {
    Passed,
    Failed,
    Warning,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub var_name: String,
    pub message: String,
}

pub struct EnvChecker {
    config: Config,
}

impl EnvChecker {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn check_resources(&self, resources: &[K8sResource], environment: Option<&str>) -> Vec<CheckResult> {
        resources
            .iter()
            .map(|resource| self.check_resource(resource, environment))
            .collect()
    }

    fn check_resource(&self, resource: &K8sResource, environment: Option<&str>) -> CheckResult {
        let env_name = environment.unwrap_or("development");
        let mut result = CheckResult {
            app_name: resource.name.clone(),
            status: CheckStatus::Passed,
            missing_required: Vec::new(),
            missing_optional: Vec::new(),
            using_defaults: Vec::new(),
            extra_vars: Vec::new(),
            validation_errors: Vec::new(),
            passed_vars: Vec::new(),
        };

        let all_env_vars = self.collect_all_env_vars(&resource.containers);
        let required_vars = self.config.get_required_vars(&resource.name, env_name);
        let optional_vars = self.config.get_optional_vars(&resource.name, env_name);

        for var in &required_vars {
            if !all_env_vars.contains_key(&var.name) {
                result.missing_required.push(var.name.clone());
                result.status = CheckStatus::Failed;
            } else if let Some(value) = all_env_vars.get(&var.name) {
                if let Some(pattern) = &var.pattern {
                    if let Err(e) = self.validate_pattern(&var.name, value, pattern) {
                        result.validation_errors.push(e);
                        result.status = CheckStatus::Failed;
                    } else {
                        result.passed_vars.push((var.name.clone(), value.clone()));
                    }
                } else {
                    result.passed_vars.push((var.name.clone(), value.clone()));
                }
            }
        }

        for var in &optional_vars {
            if !all_env_vars.contains_key(&var.name) {
                if var.default.is_some() {
                    result.using_defaults.push(var.name.clone());
                } else {
                    result.missing_optional.push(var.name.clone());
                    if matches!(result.status, CheckStatus::Passed) {
                        result.status = CheckStatus::Warning;
                    }
                }
            } else if let Some(value) = all_env_vars.get(&var.name) {
                result.passed_vars.push((var.name.clone(), value.clone()));
            }
        }

        let expected_vars: HashSet<String> = required_vars
            .iter()
            .chain(optional_vars.iter())
            .map(|v| v.name.clone())
            .collect();

        for var_name in all_env_vars.keys() {
            if !expected_vars.contains(var_name) {
                result.extra_vars.push(var_name.clone());
            }
        }

        result
    }

    fn collect_all_env_vars(&self, containers: &[ContainerInfo]) -> HashMap<String, String> {
        let mut all_vars = HashMap::new();

        for container in containers {
            for (key, value) in &container.env_vars {
                all_vars.insert(key.clone(), value.clone());
            }
        }

        all_vars
    }

    fn validate_pattern(&self, var_name: &str, value: &str, pattern: &str) -> Result<(), ValidationError> {
        let regex = Regex::new(pattern).map_err(|e| ValidationError {
            var_name: var_name.to_string(),
            message: format!("Invalid regex pattern: {}", e),
        })?;

        if !regex.is_match(value) {
            return Err(ValidationError {
                var_name: var_name.to_string(),
                message: format!("Value '{}' does not match pattern '{}'", value, pattern),
            });
        }

        Ok(())
    }
}