use clap::{Parser, Subcommand};

mod codegen;
mod config;
mod exec;
mod init;
mod invocation;
mod manifest;

#[derive(Parser)]
#[command(
    name = "euglena",
    about = "Euglena app framework CLI — scaffold, run, build, and test Euglena applications",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new Euglena application in a new directory
    Init {
        /// Name of the new project (used as directory name and cell name)
        name: String,
    },
    /// Run an Euglena application with the `code` interpreter
    Run {
        /// Entry file (default: src/main.code)
        #[arg(default_value = "src/main.code")]
        file: String,
    },
    /// Compile an Euglena application to native binary
    Build {
        /// Entry file (default: src/main.code)
        #[arg(default_value = "src/main.code")]
        file: String,
        /// Enable LLVM optimizations (slower compile, faster runtime)
        #[arg(long)]
        release: bool,
    },
    /// Run all test files in the current project's tests/ directory
    Test,
    /// Manage Code interpreter path used by euglena-cli
    Code {
        #[command(subcommand)]
        command: CodeCommands,
    },
}

#[derive(Subcommand)]
enum CodeCommands {
    /// Set or replace the Code interpreter binary path
    Set {
        /// Absolute or relative path to the Code interpreter binary
        path: String,
    },
    /// Show current configured Code interpreter path
    Show,
    /// Clear configured Code interpreter path
    Clear,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { name } => init::run(&name),
        Commands::Run { file } => exec::run_code("run", Some(&file), false),
        Commands::Build { file, release } => exec::run_code("build", Some(&file), release),
        Commands::Test => exec::run_code("test", None, false),
        Commands::Code { command } => match command {
            CodeCommands::Set { path } => config::set_code_binary_path(&path),
            CodeCommands::Show => config::show_code_binary_path(),
            CodeCommands::Clear => config::clear_code_binary_path(),
        },
    }
}
