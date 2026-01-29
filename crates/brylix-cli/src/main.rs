//! Brylix CLI - Command line tool for the Brylix framework.
//!
//! # Commands
//!
//! - `brylix new <name>` - Create a new project
//! - `brylix generate <type> <name>` - Generate code
//! - `brylix dev` - Run development server
//! - `brylix build` - Build for Lambda
//! - `brylix deploy` - Deploy to AWS Lambda
//! - `brylix migrate` - Run database migrations

mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "brylix")]
#[command(author, version, about = "CLI tool for the Brylix framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Brylix project
    New {
        /// Project name
        name: String,

        /// Enable multi-tenant mode
        #[arg(long)]
        multi_tenant: bool,

        /// Database type (mysql or postgres)
        #[arg(long, default_value = "mysql")]
        database: String,
    },

    /// Generate code (entity, service, repository, resolver)
    #[command(alias = "g")]
    Generate {
        #[command(subcommand)]
        what: GenerateCommands,
    },

    /// Run development server with hot reload
    Dev {
        /// Port to run on
        #[arg(short, long, default_value = "9000")]
        port: u16,
    },

    /// Build for AWS Lambda deployment
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,

        /// Build for ARM64 (Graviton)
        #[arg(long)]
        arm64: bool,
    },

    /// Deploy to AWS Lambda
    Deploy {
        /// AWS profile to use
        #[arg(long)]
        profile: Option<String>,

        /// Function name
        #[arg(long)]
        function: Option<String>,
    },

    /// Run database migrations
    Migrate {
        /// Generate a new migration
        #[arg(long)]
        generate: Option<String>,

        /// Rollback last migration
        #[arg(long)]
        down: bool,
    },
}

#[derive(Subcommand)]
enum GenerateCommands {
    /// Generate a SeaORM entity
    Entity {
        /// Entity name (singular, PascalCase)
        name: String,
    },

    /// Generate a service
    Service {
        /// Service name
        name: String,
    },

    /// Generate a repository
    Repository {
        /// Repository name
        name: String,
    },

    /// Generate a GraphQL resolver
    Resolver {
        /// Resolver name
        name: String,
    },

    /// Generate all (entity, service, repository, resolver)
    All {
        /// Name
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { name, multi_tenant, database } => {
            commands::new::run(name, *multi_tenant, database);
        }
        Commands::Generate { what } => match what {
            GenerateCommands::Entity { name } => commands::generate::entity(name),
            GenerateCommands::Service { name } => commands::generate::service(name),
            GenerateCommands::Repository { name } => commands::generate::repository(name),
            GenerateCommands::Resolver { name } => commands::generate::resolver(name),
            GenerateCommands::All { name } => commands::generate::all(name),
        },
        Commands::Dev { port } => {
            commands::dev::run(*port);
        }
        Commands::Build { release, arm64 } => {
            commands::build::run(*release, *arm64);
        }
        Commands::Deploy { profile, function } => {
            commands::deploy::run(profile.as_deref(), function.as_deref());
        }
        Commands::Migrate { generate, down } => {
            commands::migrate::run(generate.as_deref(), *down);
        }
    }
}
