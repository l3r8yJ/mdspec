use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "mdspec",
    version,
    about = "Validate structured Markdown endpoint documentation"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(
        about = "Check Markdown files, directories, or glob patterns (exit: 0 clean, 1 violations, 2 operational error)"
    )]
    Check {
        #[arg(value_name = "INPUT", required = true, num_args = 1..)]
        inputs: Vec<PathBuf>,
        #[arg(
            long,
            value_name = "GLOB",
            help = "Exclude matching files and directory subtrees; repeatable"
        )]
        exclude: Vec<String>,
        #[arg(long, help = "Report unused definitions and treat warnings as errors")]
        strict: bool,
        #[arg(long, value_enum, default_value = "text")]
        format: Format,
        #[arg(long, value_name = "FILE", help = "Read this TOML configuration file")]
        config: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Format {
    Text,
    Json,
}
