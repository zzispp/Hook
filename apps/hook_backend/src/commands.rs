use configuration::Settings;
use storage::{DatabaseConnectOptions, connect_database};
use toasty_cli::{Config as ToastyCliConfig, MigrationConfig, ToastyCli};

use crate::{BackendResult, schema, startup};

pub async fn run() -> BackendResult<()> {
    let settings = Settings::load()?;
    match command_from_args(std::env::args().skip(1).collect())? {
        BackendCommand::Serve => startup::serve(settings).await,
        BackendCommand::SchemaBootstrap => schema::bootstrap(settings).await,
        BackendCommand::SchemaPush => push_schema(settings).await,
        BackendCommand::Migration(args) => run_migration(settings, args).await,
    }
}

async fn push_schema(settings: Settings) -> BackendResult<()> {
    let database_url = settings.database_url()?;
    let database = connect_database(&database_url, DatabaseConnectOptions { push_schema: false }).await?;

    database.push_schema().await?;
    tracing::info!("database schema pushed");
    Ok(())
}

async fn run_migration(settings: Settings, args: Vec<String>) -> BackendResult<()> {
    let database_url = settings.database_url()?;
    let database = connect_database(&database_url, DatabaseConnectOptions { push_schema: false }).await?;
    let config = ToastyCliConfig::new().migration(MigrationConfig::new().path("toasty"));
    let cli = ToastyCli::with_config(database.into_inner(), config);

    let mut toasty_args = vec!["toasty".to_owned(), "migration".to_owned()];
    toasty_args.extend(args);
    cli.parse_from(toasty_args).await?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum BackendCommand {
    Serve,
    SchemaBootstrap,
    SchemaPush,
    Migration(Vec<String>),
}

fn command_from_args(args: Vec<String>) -> BackendResult<BackendCommand> {
    let positionals = positional_args(args)?;
    match positionals.as_slice() {
        [] => Ok(BackendCommand::Serve),
        [schema, bootstrap] if schema == "schema" && bootstrap == "bootstrap" => Ok(BackendCommand::SchemaBootstrap),
        [schema, push] if schema == "schema" && push == "push" => Ok(BackendCommand::SchemaPush),
        [migration, args @ ..] if migration == "migration" => Ok(BackendCommand::Migration(args.to_vec())),
        _ => Err(format!("unsupported backend command: {}", positionals.join(" ")).into()),
    }
}

fn positional_args(args: Vec<String>) -> BackendResult<Vec<String>> {
    let mut positionals = Vec::new();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if arg == "--config" {
            args.next().ok_or("--config requires a file path")?;
            continue;
        }
        positionals.push(arg);
    }

    Ok(positionals)
}

#[cfg(test)]
mod tests {
    use super::{BackendCommand, command_from_args, positional_args};

    #[test]
    fn defaults_to_serve_command() {
        assert_eq!(command_from_args(vec![]).unwrap(), BackendCommand::Serve);
    }

    #[test]
    fn detects_schema_push_command() {
        let args = vec!["schema".into(), "push".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::SchemaPush);
    }

    #[test]
    fn detects_schema_bootstrap_command() {
        let args = vec!["schema".into(), "bootstrap".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::SchemaBootstrap);
    }

    #[test]
    fn ignores_config_path_when_detecting_command() {
        let args = vec!["--config".into(), "config/config.yaml".into(), "schema".into(), "push".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::SchemaPush);
    }

    #[test]
    fn detects_migration_command() {
        let args = vec!["migration".into(), "apply".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::Migration(vec!["apply".into()]));
    }

    #[test]
    fn rejects_unknown_command() {
        let args = vec!["schem".into(), "push".into()];

        assert!(command_from_args(args).is_err());
    }

    #[test]
    fn rejects_missing_config_path() {
        let args = vec!["--config".into()];

        assert!(positional_args(args).is_err());
    }
}
