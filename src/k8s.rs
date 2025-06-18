use anyhow::Result;
use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, StatefulSet};
use k8s_openapi::api::core::v1::{Container, EnvVar};
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct K8sResource {
    pub kind: String,
    pub name: String,
    pub containers: Vec<ContainerInfo>,
}

#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub name: String,
    pub env_vars: HashMap<String, String>,
    pub env_from_refs: Vec<String>,
}

pub struct K8sParser;

impl K8sParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_yaml(&self, yaml_content: &str) -> Result<Vec<K8sResource>> {
        let mut resources = Vec::new();

        for document in serde_yaml::Deserializer::from_str(yaml_content) {
            let value = Value::deserialize(document)?;
            
            if let Some(kind) = value.get("kind").and_then(|k| k.as_str()) {
                match kind {
                    "Deployment" => {
                        if let Ok(deployment) = serde_yaml::from_value::<Deployment>(value.clone()) {
                            if let Some(resource) = self.extract_from_deployment(&deployment) {
                                resources.push(resource);
                            }
                        }
                    }
                    "StatefulSet" => {
                        if let Ok(statefulset) = serde_yaml::from_value::<StatefulSet>(value.clone()) {
                            if let Some(resource) = self.extract_from_statefulset(&statefulset) {
                                resources.push(resource);
                            }
                        }
                    }
                    "DaemonSet" => {
                        if let Ok(daemonset) = serde_yaml::from_value::<DaemonSet>(value.clone()) {
                            if let Some(resource) = self.extract_from_daemonset(&daemonset) {
                                resources.push(resource);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(resources)
    }

    fn extract_from_deployment(&self, deployment: &Deployment) -> Option<K8sResource> {
        let name = deployment.metadata.name.clone()?;
        let containers = self.extract_containers(&deployment.spec.as_ref()?.template.spec.as_ref()?.containers);

        Some(K8sResource {
            kind: "Deployment".to_string(),
            name,
            containers,
        })
    }

    fn extract_from_statefulset(&self, statefulset: &StatefulSet) -> Option<K8sResource> {
        let name = statefulset.metadata.name.clone()?;
        let containers = self.extract_containers(&statefulset.spec.as_ref()?.template.spec.as_ref()?.containers);

        Some(K8sResource {
            kind: "StatefulSet".to_string(),
            name,
            containers,
        })
    }

    fn extract_from_daemonset(&self, daemonset: &DaemonSet) -> Option<K8sResource> {
        let name = daemonset.metadata.name.clone()?;
        let containers = self.extract_containers(&daemonset.spec.as_ref()?.template.spec.as_ref()?.containers);

        Some(K8sResource {
            kind: "DaemonSet".to_string(),
            name,
            containers,
        })
    }

    fn extract_containers(&self, containers: &[Container]) -> Vec<ContainerInfo> {
        containers
            .iter()
            .map(|container| {
                let env_vars = self.extract_env_vars(&container.env);
                let env_from_refs = self.extract_env_from(&container.env_from);

                ContainerInfo {
                    name: container.name.clone(),
                    env_vars,
                    env_from_refs,
                }
            })
            .collect()
    }

    fn extract_env_vars(&self, env_vars: &Option<Vec<EnvVar>>) -> HashMap<String, String> {
        let mut map = HashMap::new();

        if let Some(vars) = env_vars {
            for var in vars {
                if let Some(value) = &var.value {
                    map.insert(var.name.clone(), value.clone());
                } else if let Some(value_from) = &var.value_from {
                    let source_description = self.describe_value_from(value_from);
                    map.insert(var.name.clone(), source_description);
                }
            }
        }

        map
    }

    fn describe_value_from(&self, value_from: &k8s_openapi::api::core::v1::EnvVarSource) -> String {
        if let Some(secret_key_ref) = &value_from.secret_key_ref {
            format!("Secret[{}:{}]", secret_key_ref.name, secret_key_ref.key)
        } else if let Some(config_map_key_ref) = &value_from.config_map_key_ref {
            format!("ConfigMap[{}:{}]", config_map_key_ref.name, config_map_key_ref.key)
        } else if let Some(field_ref) = &value_from.field_ref {
            format!("Field[{}]", field_ref.field_path)
        } else if let Some(resource_field_ref) = &value_from.resource_field_ref {
            format!("Resource[{}]", resource_field_ref.resource)
        } else {
            "<unknown source>".to_string()
        }
    }

    fn extract_env_from(&self, env_from: &Option<Vec<k8s_openapi::api::core::v1::EnvFromSource>>) -> Vec<String> {
        let mut refs = Vec::new();

        if let Some(sources) = env_from {
            for source in sources {
                if let Some(config_map_ref) = &source.config_map_ref {
                    refs.push(format!("ConfigMap: {}", config_map_ref.name));
                }
                if let Some(secret_ref) = &source.secret_ref {
                    refs.push(format!("Secret: {}", secret_ref.name));
                }
            }
        }

        refs
    }
}