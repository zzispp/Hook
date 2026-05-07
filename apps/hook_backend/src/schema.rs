use std::collections::BTreeSet;

use configuration::Settings;
use storage::{DatabaseConnectOptions, connect_database};
use tokio_postgres::NoTls;

use crate::BackendResult;

#[derive(Debug, PartialEq, Eq)]
enum BootstrapPlan {
    Push,
    Ready,
}

pub async fn bootstrap(settings: Settings) -> BackendResult<()> {
    let database_url = settings.database_url()?;
    let database = connect_database(&database_url, DatabaseConnectOptions { push_schema: false }).await?;
    let table_names = database.table_names();
    let existing = existing_tables(&database_url, &table_names).await?;

    match bootstrap_plan(&table_names, &existing)? {
        BootstrapPlan::Push => {
            database.push_schema().await?;
            tracing::info!("database schema bootstrapped");
        }
        BootstrapPlan::Ready => tracing::info!("database schema already exists"),
    }

    Ok(())
}

async fn existing_tables(database_url: &str, table_names: &[String]) -> BackendResult<BTreeSet<String>> {
    let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            tracing::error!(%error, "postgres connection error");
        }
    });

    let rows = client
        .query(
            "select table_name from information_schema.tables where table_schema = 'public' and table_name = any($1)",
            &[&table_names],
        )
        .await?;

    Ok(rows.into_iter().map(|row| row.get::<_, String>("table_name")).collect())
}

fn bootstrap_plan(table_names: &[String], existing: &BTreeSet<String>) -> BackendResult<BootstrapPlan> {
    if existing.is_empty() {
        return Ok(BootstrapPlan::Push);
    }

    let expected = table_names.iter().cloned().collect::<BTreeSet<_>>();
    if expected == *existing {
        return Ok(BootstrapPlan::Ready);
    }

    let missing = expected.difference(existing).cloned().collect::<Vec<_>>();
    let present = existing.iter().cloned().collect::<Vec<_>>();
    Err(format!(
        "partial database schema detected; present tables: {}, missing tables: {}",
        present.join(", "),
        missing.join(", ")
    )
    .into())
}

#[cfg(test)]
mod tests {
    use super::{BootstrapPlan, bootstrap_plan};
    use std::collections::BTreeSet;

    #[test]
    fn empty_database_pushes_schema() {
        let table_names = table_names();
        let existing = BTreeSet::new();

        let plan = bootstrap_plan(&table_names, &existing).unwrap();

        assert_eq!(plan, BootstrapPlan::Push);
    }

    #[test]
    fn complete_database_is_ready() {
        let table_names = table_names();
        let existing = BTreeSet::from(["users".into(), "role_records".into()]);

        let plan = bootstrap_plan(&table_names, &existing).unwrap();

        assert_eq!(plan, BootstrapPlan::Ready);
    }

    #[test]
    fn partial_database_errors() {
        let table_names = table_names();
        let existing = BTreeSet::from(["users".into()]);

        let result = bootstrap_plan(&table_names, &existing);

        assert!(result.is_err());
    }

    fn table_names() -> Vec<String> {
        vec!["users".into(), "role_records".into()]
    }
}
