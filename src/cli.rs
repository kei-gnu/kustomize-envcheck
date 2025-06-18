use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "kustomize-envcheck")]
#[command(about = "Check environment variables in Kustomize-built Kubernetes manifests")]
#[command(version)]
pub struct Cli {
    #[arg(short = 'k', long, help = "Path to Kustomize directory")]
    pub kustomize_dir: String,

    #[arg(short = 'c', long, help = "Path to configuration file")]
    pub config: String,

    #[arg(short = 'e', long, help = "Specific environment to check")]
    pub environment: Option<String>,

    #[arg(
        short = 'o',
        long,
        value_enum,
        default_value = "text",
        help = "Output format"
    )]
    pub output: OutputFormat,

    #[arg(short = 'v', long, help = "Verbose output")]
    pub verbose: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}