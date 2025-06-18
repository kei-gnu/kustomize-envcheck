# Kustomize Environment Variable Checker

## プロジェクト概要

KustomizeでビルドされたKubernetes YAMLファイルから環境変数を抽出し、必要な環境変数が設定されているかをチェックするRust製CLIツールです。

## 機能要件

### 基本機能
- Kustomizeディレクトリを指定して`kustomize build`を実行
- 生成されたYAMLからDeployment、StatefulSet、DaemonSetリソースを解析
- 各コンテナの環境変数（env、envFrom）を抽出
- 設定ファイルで定義された必要な環境変数との照合
- 不足している環境変数と余分な環境変数を報告

### 出力形式
- 人間が読みやすいテキスト形式（デフォルト）
- JSON形式（CI/CDパイプライン用）
- 終了コード：問題がある場合は非ゼロ

### 設定ファイル
- YAML形式で必要な環境変数を定義
- アプリケーション別、環境別の設定をサポート
- 必須/オプショナルの区別
- 正規表現による値の検証（オプション）

## 技術スタック

### 言語・フレームワーク
- Rust 2021 Edition
- Tokio（非同期ランタイム）

### 主要な依存関係
- `clap` - CLI引数解析
- `serde` + `serde_yaml` - YAML シリアライゼーション
- `tokio` - 非同期処理
- `anyhow` - エラーハンドリング
- `k8s-openapi` - Kubernetes リソース定義
- `regex` - 正規表現による値検証

## CLI インターフェース

```bash
# 基本的な使用方法
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml

# JSON出力
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml --output json

# 特定の環境のみチェック
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml --environment production

# Verboseモード
kustomize-envcheck --kustomize-dir ./k8s --config ./envcheck.yaml --verbose
```

## 設定ファイル例

```yaml
# envcheck.yaml
environments:
  development:
    required_vars:
      - name: "DATABASE_URL"
        description: "Database connection string"
      - name: "API_KEY"
        description: "External API key"
    optional_vars:
      - name: "DEBUG"
        description: "Debug mode flag"
        default: "false"
  
  production:
    required_vars:
      - name: "DATABASE_URL"
        description: "Database connection string"
      - name: "API_KEY"
        description: "External API key"
      - name: "REDIS_URL"
        description: "Redis connection string"
    optional_vars:
      - name: "LOG_LEVEL"
        description: "Logging level"
        default: "info"

applications:
  web-app:
    environments: ["development", "production"]
  worker:
    environments: ["production"]
    additional_vars:
      - name: "QUEUE_NAME"
        description: "Job queue name"
```

## 出力例

### テキスト形式
```
✓ Checking environment variables for kustomize directory: ./k8s
✓ Found 2 deployments: web-app, worker

Application: web-app
  ✓ DATABASE_URL: postgresql://localhost:5432/myapp
  ✗ API_KEY: Missing (External API key)
  ⚠ DEBUG: Not set, using default: false
  ✓ PORT: 8080

Application: worker  
  ✓ DATABASE_URL: postgresql://localhost:5432/myapp
  ✓ QUEUE_NAME: job-queue
  ✗ REDIS_URL: Missing (Redis connection string)
  ✓ WORKER_THREADS: 4

Summary:
- 2 missing required variables
- 1 optional variable using default
```

### JSON形式
```json
{
  "status": "failed",
  "summary": {
    "total_applications": 2,
    "missing_required": 2,
    "missing_optional": 1
  },
  "applications": [
    {
      "name": "web-app",
      "status": "failed",
      "missing_required": ["API_KEY"],
      "missing_optional": [],
      "using_defaults": ["DEBUG"]
    }
  ]
}
```

## プロジェクト構造

```
kustomize-envcheck/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── cli.rs          # CLI引数定義
│   ├── config.rs       # 設定ファイル処理
│   ├── kustomize.rs    # Kustomizeビルド実行
│   ├── k8s.rs          # Kubernetes YAML解析
│   ├── checker.rs      # 環境変数チェックロジック
│   └── output.rs       # 出力フォーマット
├── tests/
│   ├── integration/
│   └── fixtures/
└── examples/
    ├── envcheck.yaml
    └── k8s/
```

## 開発方針

### エラーハンドリング
- `anyhow`を使用してエラーチェーンを適切に管理
- ユーザーフレンドリーなエラーメッセージ
- デバッグ情報は`--verbose`フラグでのみ表示

### テスト戦略
- 単体テスト：各モジュールの機能
- 統合テスト：実際のKustomizeプロジェクトを使用
- テストフィクスチャ：様々なKubernetesリソースパターン

### パフォーマンス
- 大きなYAMLファイルの効率的な解析
- 並列処理による高速化（複数アプリケーション）
- メモリ使用量の最適化

## 配布

- GitHub Releases でバイナリ配布
- `cargo install` でのインストール対応
- Dockerイメージ（CI/CD用）

## 将来の拡張予定

- **環境変数値の検証機能**
  - 正規表現パターンマッチング
  - 値の候補リスト検証
  - 数値範囲チェック
  - URL形式検証
- Helm chart対応
- より多くのKubernetesリソースタイプ（Job、CronJobなど）
- IDE統合（LSP対応）
- Web UI（オプション）
