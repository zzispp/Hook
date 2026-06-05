use configuration::Settings;
use rbac::{
    application::RbacService,
    infra::{RedisRbacCache, StorageRbacRepository},
};
use storage::connect_database;

use crate::{BackendResult, migration::development, startup};

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
    let rebuild_rbac_cache = command.rebuilds_rbac_cache();
    let connection = database.connection();
    match command {
        MigrationCommand::Up => development::apply(connection).await?,
        MigrationCommand::Fresh | MigrationCommand::Refresh => development::recreate(connection).await?,
        MigrationCommand::Down | MigrationCommand::Reset => development::drop(connection).await?,
        MigrationCommand::Status => print_baseline_status(development::status(connection).await?),
    }
    if rebuild_rbac_cache {
        rebuild_rbac_cache_after_migration(&settings, database).await?;
    }
    Ok(())
}

fn print_baseline_status(status: development::BaselineStatus) {
    println!("baseline tables: {}/{} present", status.existing_tables.len(), status.total_tables);
    println!("baseline migration marker: {}", if status.baseline_applied { "applied" } else { "pending" });
    for table_name in status.existing_tables {
        println!("  {table_name}");
    }
}

async fn rebuild_rbac_cache_after_migration(settings: &Settings, database: storage::Database) -> BackendResult<()> {
    let repository = StorageRbacRepository::new(database);
    let cache = RedisRbacCache::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    let rbac = RbacService::new(repository, cache);
    rbac.rebuild_cache().await?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum BackendCommand {
    Serve,
    Migration(MigrationCommand),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MigrationCommand {
    Up,
    Down,
    Status,
    Fresh,
    Refresh,
    Reset,
}

impl MigrationCommand {
    fn rebuilds_rbac_cache(&self) -> bool {
        matches!(self, Self::Up | Self::Fresh | Self::Refresh)
    }
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
        [] => Ok(MigrationCommand::Up),
        [command] if command == "up" => Ok(MigrationCommand::Up),
        [command] if command == "down" => Ok(MigrationCommand::Down),
        [command] if command == "status" => Ok(MigrationCommand::Status),
        [command] if command == "fresh" => Ok(MigrationCommand::Fresh),
        [command] if command == "refresh" => Ok(MigrationCommand::Refresh),
        [command] if command == "reset" => Ok(MigrationCommand::Reset),
        _ => Err(format!("unsupported migration command: {}", args.join(" ")).into()),
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
    use super::{BackendCommand, MigrationCommand, command_from_args, positional_args};

    #[test]
    fn defaults_to_serve_command() {
        assert_eq!(command_from_args(vec![]).unwrap(), BackendCommand::Serve);
    }

    #[test]
    fn ignores_config_path_when_detecting_command() {
        let args = vec!["--config".into(), "config/config.yaml".into(), "migration".into(), "up".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::Migration(MigrationCommand::Up));
    }

    #[test]
    fn detects_migration_up_command() {
        let args = vec!["migration".into(), "up".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::Migration(MigrationCommand::Up));
    }

    #[test]
    fn detects_migration_down_command() {
        let args = vec!["migration".into(), "down".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::Migration(MigrationCommand::Down));
    }

    #[test]
    fn rejects_migration_steps() {
        let args = vec!["migration".into(), "up".into(), "2".into()];

        assert!(command_from_args(args).is_err());
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
