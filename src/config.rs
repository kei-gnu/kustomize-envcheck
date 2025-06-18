use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub environments: HashMap<String, Environment>,
    pub applications: HashMap<String, Application>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Environment {
    pub required_vars: Vec<EnvVar>,
    #[serde(default)]
    pub optional_vars: Vec<EnvVar>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Application {
    pub environments: Vec<String>,
    #[serde(default)]
    pub additional_vars: Vec<EnvVar>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvVar {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn get_required_vars(&self, app_name: &str, env_name: &str) -> Vec<&EnvVar> {
        let mut vars = Vec::new();

        if let Some(env) = self.environments.get(env_name) {
            vars.extend(&env.required_vars);
        }

        if let Some(app) = self.applications.get(app_name) {
            if app.environments.contains(&env_name.to_string()) {
                vars.extend(&app.additional_vars);
            }
        }

        vars
    }

    pub fn get_optional_vars(&self, _app_name: &str, env_name: &str) -> Vec<&EnvVar> {
        let mut vars = Vec::new();

        if let Some(env) = self.environments.get(env_name) {
            vars.extend(&env.optional_vars);
        }

        vars
    }
}