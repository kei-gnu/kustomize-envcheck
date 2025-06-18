# Kustomize Environment Variable Checker

[日本語版](README.ja.md)

A CLI tool to check if required environment variables are properly configured in Kubernetes manifests built by Kustomize.

## Features

- Extract environment variables from Deployment, StatefulSet, and DaemonSet resources
- Check against a configuration file to identify missing required/optional variables
- Support for pattern validation using regular expressions
- Human-readable colored output and JSON output for CI/CD pipelines
- Environment-specific configuration support

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/kustomize-envcheck.git
cd kustomize-envcheck

# Build from source
cargo build --release

# Install to your PATH
cargo install --path .
```

## Prerequisites

- Rust 1.70 or higher
- Kustomize installed and available in PATH

## Usage

### Basic usage

```bash
# Check environment variables
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml

# Check specific environment
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml --environment production

# Output as JSON
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml --output json

# Verbose mode
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml --verbose

# Show extra environment variables not defined in config
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml --show-extra-vars
```

### Configuration file

Create an `envcheck.yaml` file to define required and optional environment variables:

```yaml
environments:
  development:
    required_vars:
      - name: "DATABASE_URL"
        description: "Database connection string"
        pattern: "^postgresql://.*$"  # Optional: regex pattern validation
      - name: "API_KEY"
        description: "External API key"
    optional_vars:
      - name: "DEBUG"
        description: "Debug mode flag"
        default: "false"
      - name: "LOG_LEVEL"
        description: "Logging level"
        default: "info"
  
  production:
    required_vars:
      - name: "DATABASE_URL"
        description: "Database connection string"
        pattern: "^postgresql://.*$"
      - name: "API_KEY"
        description: "External API key"
      - name: "REDIS_URL"
        description: "Redis connection string"
        pattern: "^redis://.*$"
    optional_vars:
      - name: "LOG_LEVEL"
        description: "Logging level"
        default: "warn"

applications:
  web-app:
    environments: ["development", "production"]
    additional_vars:
      - name: "PORT"
        description: "HTTP server port"
        default: "8080"
  worker:
    environments: ["production"]
    additional_vars:
      - name: "QUEUE_NAME"
        description: "Job queue name"
      - name: "WORKER_THREADS"
        description: "Number of worker threads"
        default: "4"
```

### Example Kustomize structure

```
k8s/
├── base/
│   ├── deployment.yaml
│   ├── service.yaml
│   └── kustomization.yaml
├── overlays/
│   ├── development/
│   │   ├── kustomization.yaml
│   │   └── config-map.yaml
│   └── production/
│       ├── kustomization.yaml
│       ├── config-map.yaml
│       └── secret.yaml
└── kustomization.yaml
```

### Output examples

#### Text output

```
Environment Variable Check Results
==================================================

Application: web-app
  ✓ Status: Passed
  ✓ DATABASE_URL: postgresql://localhost:5432/myapp
  ✓ API_KEY: <from source>
  ⚠ Using default values:
    - DEBUG
    - PORT
  # Extra variables are hidden by default, use --show-extra-vars to display them

Application: worker
  ✗ Status: Failed
  ✗ Missing required variables:
    - REDIS_URL
  ✓ QUEUE_NAME: job-queue
  ⚠ Using default values:
    - WORKER_THREADS

Summary
--------------------------------------------------
Total applications: 2
Failed: 1 | Warnings: 0 | Passed: 1
Missing required variables: 1
Missing optional variables: 0
```

#### JSON output

```json
{
  "status": "failed",
  "summary": {
    "total_applications": 2,
    "missing_required": 1,
    "missing_optional": 0
  },
  "applications": [
    {
      "name": "web-app",
      "status": "passed",
      "missing_required": [],
      "missing_optional": [],
      "using_defaults": ["DEBUG", "PORT"],
      "extra_vars": ["KUBERNETES_SERVICE_HOST", "KUBERNETES_SERVICE_PORT"]
    },
    {
      "name": "worker",
      "status": "failed",
      "missing_required": ["REDIS_URL"],
      "missing_optional": [],
      "using_defaults": ["WORKER_THREADS"],
      "extra_vars": []
    }
  ]
}
```

## Exit codes

- `0`: All checks passed
- `1`: One or more required variables are missing

## CI/CD Integration

### GitHub Actions

```yaml
- name: Check Kubernetes Environment Variables
  run: |
    kustomize-envcheck \
      --kustomize-dir ./k8s/overlays/production \
      --config ./envcheck.yaml \
      --environment production \
      --output json
```

### GitLab CI

```yaml
check-env-vars:
  script:
    - kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml --output json
  only:
    - merge_requests
```

## Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- --kustomize-dir ./k8s --config ./envcheck.yaml

# Format code
cargo fmt

# Run linter
cargo clippy
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
