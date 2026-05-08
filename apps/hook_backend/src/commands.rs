use configuration::Settings;
use sea_orm_migration::MigratorTrait;
use storage::connect_database;

use crate::{BackendResult, migration::Migrator, startup};

pub async fn run() -> BackendResult<()> {
    let settings = Settings::load()?;
    init_tracing(&settings)?;

    match command_from_args(std::env::args().skip(1).collect())? {
        BackendCommand::Serve => startup::serve(settings).await,
        BackendCommand::Migration(command) => run_migration(settings, command).await,
    }
}

fn init_tracing(settings: &Settings) -> BackendResult<()> {
    hook_tracing::init_global_subscriber(hook_tracing::TracingConfig {
        log_level: settings.tracing_log_level()?,
    })?;
    Ok(())
}

async fn run_migration(settings: Settings, command: MigrationCommand) -> BackendResult<()> {
    let database = connect_database(&settings.database_url()?).await?;
    let connection = database.connection();
    match command {
        MigrationCommand::Up(steps) => Migrator::up(connection, steps).await?,
        MigrationCommand::Down(steps) => Migrator::down(connection, steps).await?,
        MigrationCommand::Status => Migrator::status(connection).await?,
        MigrationCommand::Fresh => Migrator::fresh(connection).await?,
        MigrationCommand::Refresh => Migrator::refresh(connection).await?,
        MigrationCommand::Reset => Migrator::reset(connection).await?,
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum BackendCommand {
    Serve,
    Migration(MigrationCommand),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MigrationCommand {
    Up(Option<u32>),
    Down(Option<u32>),
    Status,
    Fresh,
    Refresh,
    Reset,
}

fn command_from_args(args: Vec<String>) -> BackendResult<BackendCommand> {
    let positionals = positional_args(args)?;
    match positionals.as_slice() {
        [] => Ok(BackendCommand::Serve),
        [migration, args @ ..] if migration == "migration" => Ok(BackendCommand::Migration(migration_command(args)?)),
        _ => Err(format!("unsupported backend command: {}", positionals.join(" ")).into()),
    }
}

fn migration_command(args: &[String]) -> BackendResult<MigrationCommand> {
    match args {
        [] => Ok(MigrationCommand::Up(None)),
        [command] if command == "up" => Ok(MigrationCommand::Up(None)),
        [command, steps] if command == "up" => Ok(MigrationCommand::Up(Some(parse_steps(steps)?))),
        [command] if command == "down" => Ok(MigrationCommand::Down(Some(1))),
        [command, steps] if command == "down" => Ok(MigrationCommand::Down(Some(parse_steps(steps)?))),
        [command] if command == "status" => Ok(MigrationCommand::Status),
        [command] if command == "fresh" => Ok(MigrationCommand::Fresh),
        [command] if command == "refresh" => Ok(MigrationCommand::Refresh),
        [command] if command == "reset" => Ok(MigrationCommand::Reset),
        _ => Err(format!("unsupported migration command: {}", args.join(" ")).into()),
    }
}

fn parse_steps(value: &str) -> BackendResult<u32> {
    value
        .parse::<u32>()
        .map_err(|error| format!("invalid migration step count '{value}': {error}").into())
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
    use super::{BackendCommand, MigrationCommand, command_from_args, positional_args};

    #[test]
    fn defaults_to_serve_command() {
        assert_eq!(command_from_args(vec![]).unwrap(), BackendCommand::Serve);
    }

    #[test]
    fn ignores_config_path_when_detecting_command() {
        let args = vec!["--config".into(), "config/config.yaml".into(), "migration".into(), "up".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::Migration(MigrationCommand::Up(None)));
    }

    #[test]
    fn detects_migration_up_command() {
        let args = vec!["migration".into(), "up".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::Migration(MigrationCommand::Up(None)));
    }

    #[test]
    fn detects_migration_down_command() {
        let args = vec!["migration".into(), "down".into(), "2".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::Migration(MigrationCommand::Down(Some(2))));
    }

    #[test]
    fn detects_migration_status_command() {
        let args = vec!["migration".into(), "status".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::Migration(MigrationCommand::Status));
    }

    #[test]
    fn rejects_schema_commands() {
        let args = vec!["schema".into(), "push".into()];

        assert!(command_from_args(args).is_err());
    }

    #[test]
    fn rejects_missing_config_path() {
        let args = vec!["--config".into()];

        assert!(positional_args(args).is_err());
    }
}
