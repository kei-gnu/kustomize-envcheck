# Kustomize環境変数チェッカー

[English](README.md)

KustomizeでビルドされたKubernetesマニフェストに必要な環境変数が適切に設定されているかをチェックするCLIツールです。

## 機能

- Deployment、StatefulSet、DaemonSetリソースから環境変数を抽出
- 設定ファイルと照合して、不足している必須/オプション変数を特定
- 正規表現を使用したパターン検証のサポート
- 人間が読みやすい色付き出力とCI/CDパイプライン用のJSON出力
- 環境別の設定サポート

## インストール

```bash
# リポジトリをクローン
git clone https://github.com/yourusername/kustomize-envcheck.git
cd kustomize-envcheck

# ソースからビルド
cargo build --release

# PATHにインストール
cargo install --path .
```

## 前提条件

- Rust 1.70以上
- KustomizeがインストールされPATHに含まれていること

## 使い方

### 基本的な使い方

```bash
# 環境変数をチェック
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml

# 特定の環境をチェック
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml --environment production

# JSON形式で出力
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml --output json

# 詳細モード
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml --verbose

# 設定ファイルに定義されていない追加の環境変数を表示
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml --show-extra-vars
```

### 設定ファイル

必須およびオプションの環境変数を定義する`envcheck.yaml`ファイルを作成します：

```yaml
environments:
  development:
    required_vars:
      - name: "DATABASE_URL"
        description: "データベース接続文字列"
        pattern: "^postgresql://.*$"  # オプション: 正規表現パターン検証
      - name: "API_KEY"
        description: "外部APIキー"
    optional_vars:
      - name: "DEBUG"
        description: "デバッグモードフラグ"
        default: "false"
      - name: "LOG_LEVEL"
        description: "ログレベル"
        default: "info"
  
  production:
    required_vars:
      - name: "DATABASE_URL"
        description: "データベース接続文字列"
        pattern: "^postgresql://.*$"
      - name: "API_KEY"
        description: "外部APIキー"
      - name: "REDIS_URL"
        description: "Redis接続文字列"
        pattern: "^redis://.*$"
    optional_vars:
      - name: "LOG_LEVEL"
        description: "ログレベル"
        default: "warn"

applications:
  web-app:
    environments: ["development", "production"]
    additional_vars:
      - name: "PORT"
        description: "HTTPサーバーポート"
        default: "8080"
  worker:
    environments: ["production"]
    additional_vars:
      - name: "QUEUE_NAME"
        description: "ジョブキュー名"
      - name: "WORKER_THREADS"
        description: "ワーカースレッド数"
        default: "4"
```

### Kustomizeディレクトリ構造の例

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

### 出力例

#### テキスト出力

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
  # 追加の変数はデフォルトでは非表示です。表示するには --show-extra-vars を使用してください

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

#### JSON出力

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

## 終了コード

- `0`: すべてのチェックに合格
- `1`: 1つ以上の必須変数が不足

## CI/CD統合

### GitHub Actions

```yaml
- name: Kubernetes環境変数をチェック
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

## 開発

```bash
# テストを実行
cargo test

# デバッグログ付きで実行
RUST_LOG=debug cargo run -- --kustomize-dir ./k8s --config ./envcheck.yaml

# コードをフォーマット
cargo fmt

# リンターを実行
cargo clippy
```

## ライセンス

このプロジェクトはMITライセンスの下でライセンスされています - 詳細はLICENSEファイルをご覧ください。
