use std::{io, sync::Arc, time::Duration};

use storage::{
    Database,
    provider::{ProviderStore, RequestPayloadKey, RequestPayloadOwner, RequestPayloadPendingInput, RequestPayloadStoreInput, request_payload_data},
};
use tokio::sync::{Semaphore, mpsc};

use super::LlmProxyError;

const WRITER_QUEUE_CAPACITY: usize = 1_000;
const WRITER_CONCURRENCY: usize = 2;
const WRITE_TIMEOUT_SECONDS: u64 = 15;

#[derive(Clone)]
pub(super) struct RequestPayloadWriter {
    sender: mpsc::Sender<RequestPayloadJob>,
}

#[derive(Clone, Debug)]
pub(super) struct RequestPayloadJob {
    pub owner: RequestPayloadOwner,
    pub kind: &'static str,
    pub payload: serde_json::Value,
    key: Option<RequestPayloadKey>,
}

impl RequestPayloadJob {
    fn with_key(self, key: RequestPayloadKey) -> Self {
        Self { key: Some(key), ..self }
    }
}

impl RequestPayloadWriter {
    pub(super) fn spawn(database: Database) -> Self {
        let (sender, receiver) = mpsc::channel(WRITER_QUEUE_CAPACITY);
        spawn_worker(database, receiver);
        Self { sender }
    }

    pub(super) async fn enqueue(&self, database: Database, job: RequestPayloadJob) -> Result<(), LlmProxyError> {
        let store = ProviderStore::new(database);
        let key = store
            .create_pending_request_payload(RequestPayloadPendingInput {
                owner: job.owner.clone(),
                kind: job.kind.to_owned(),
                payload: job.payload.clone(),
            })
            .await?;
        if self.sender.try_send(job.with_key(key.clone())).is_err() {
            let message = "request payload writer queue full".to_owned();
            store.mark_request_payload_failed(key, message.clone()).await?;
            return Err(LlmProxyError::Infrastructure(message));
        }
        Ok(())
    }
}

fn spawn_worker(database: Database, mut receiver: mpsc::Receiver<RequestPayloadJob>) {
    tokio::spawn(async move {
        let semaphore = Arc::new(Semaphore::new(WRITER_CONCURRENCY));
        while let Some(job) = receiver.recv().await {
            let Ok(permit) = semaphore.clone().acquire_owned().await else {
                hook_tracing::error("request payload writer semaphore closed", &io::Error::other("request payload worker stopped"));
                return;
            };
            let worker_database = database.clone();
            tokio::spawn(async move {
                let _permit = permit;
                if let Err(error) = write_job(worker_database, job).await {
                    hook_tracing::error("request payload writer failed", &error);
                }
            });
        }
    });
}

async fn write_job(database: Database, job: RequestPayloadJob) -> Result<(), LlmProxyError> {
    let key = job
        .key
        .clone()
        .ok_or_else(|| LlmProxyError::Infrastructure("request payload job missing storage key".into()))?;
    match tokio::time::timeout(Duration::from_secs(WRITE_TIMEOUT_SECONDS), store_job(database.clone(), job)).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => {
            mark_failed(database, key, error.to_string()).await?;
            Err(error)
        }
        Err(_) => {
            mark_failed(database, key, format!("request payload write exceeded {WRITE_TIMEOUT_SECONDS} seconds")).await?;
            Ok(())
        }
    }
}

async fn mark_failed(database: Database, key: RequestPayloadKey, message: String) -> Result<(), LlmProxyError> {
    ProviderStore::new(database).mark_request_payload_failed(key, message).await?;
    Ok(())
}

async fn store_job(database: Database, job: RequestPayloadJob) -> Result<(), LlmProxyError> {
    let data = request_payload_data(&job.payload)?;
    ProviderStore::new(database)
        .store_request_payload(RequestPayloadStoreInput {
            key: job
                .key
                .ok_or_else(|| LlmProxyError::Infrastructure("request payload job missing storage key".into()))?,
            data,
        })
        .await?;
    Ok(())
}

pub(super) fn payload_job(owner: RequestPayloadOwner, kind: &'static str, payload: serde_json::Value) -> RequestPayloadJob {
    RequestPayloadJob {
        owner,
        kind,
        payload,
        key: None,
    }
}

#[cfg(test)]
mod tests {
    use storage::provider::{OWNER_REQUEST_RECORD, RequestPayloadOwner};

    use super::payload_job;

    #[test]
    fn payload_job_keeps_owner_kind_and_payload() {
        let owner = RequestPayloadOwner {
            owner_type: OWNER_REQUEST_RECORD.to_owned(),
            owner_id: "request-1".to_owned(),
        };
        let payload = serde_json::json!({"model": "gpt"});

        let job = payload_job(owner.clone(), "request_body", payload.clone());

        assert_eq!(job.owner, owner);
        assert_eq!(job.kind, "request_body");
        assert_eq!(job.payload, payload);
    }
}
