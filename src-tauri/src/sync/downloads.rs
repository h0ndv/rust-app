use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::db::repo::Repo;
use crate::error::AppError;
use crate::sync::sources::MoodleSource;

const MAX_BYTES: u64 = 100 * 1024 * 1024;

/// Safe name for disk: no separators, no leading dots and no extension loss
/// when trimming.
pub fn safe_name(name: &str) -> String {
    // It keeps the last component: "..\..\x.pdf" cannot escape from
    // the downloads folder.
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);

    let cleaned: String = base
        .chars()
        .map(|character| match character {
            ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other if other.is_control() => '_',
            other => other,
        })
        .collect();

    let trimmed = cleaned.trim_matches(['.', ' ']).to_string();
    let trimmed = if trimmed.is_empty() {
        "archivo".to_string()
    } else {
        trimmed
    };

    if trimmed.chars().count() <= 120 {
        return trimmed;
    }

    let (stem, extension) = match trimmed.rsplit_once('.') {
        Some((stem, extension)) if extension.chars().count() <= 10 => (stem, Some(extension)),
        _ => (trimmed.as_str(), None),
    };

    let budget = 120
        - extension
            .map(|value| value.chars().count() + 1)
            .unwrap_or(0);
    let stem: String = stem.chars().take(budget).collect();

    match extension {
        Some(extension) => format!("{stem}.{extension}"),
        None => stem,
    }
}

pub struct Downloads {
    sender: mpsc::UnboundedSender<String>,
    token: CancellationToken,
}

impl Downloads {
    pub fn new(
        repo: Repo,
        moodle: Arc<dyn MoodleSource>,
        fallback: PathBuf,
        emitter: Option<crate::sync::EventEmitter>,
    ) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<String>();
        let token = CancellationToken::new();
        let worker_token = token.clone();

        tauri::async_runtime::spawn(async move {
            while let Some(file_id) = receiver.recv().await {
                if worker_token.is_cancelled() {
                    break;
                }

                let folder = repo.downloads_dir(&fallback).await.unwrap_or_else(|error| {
                    tracing::warn!(
                        %error,
                        "could not read the downloads folder; using the fallback"
                    );
                    fallback.clone()
                });

                let result = download_one(&repo, moodle.as_ref(), &folder, &file_id).await;
                match result {
                    Ok(path) => {
                        let _ = repo
                            .mark_downloaded(&file_id, &path.display().to_string())
                            .await;
                        emit(&emitter, "download://finished", &file_id);
                        tracing::debug!(file = %file_id, "download ready");
                    }
                    Err(error) => {
                        tracing::warn!(file = %file_id, %error, "download failed");
                        emit(&emitter, "download://error", &file_id);
                    }
                }
            }
        });

        Self { sender, token }
    }

    pub fn enqueue(&self, file_id: String) {
        let _ = self.sender.send(file_id);
    }

    pub fn cancel_all(&self) {
        self.token.cancel();
    }
}

async fn download_one(
    repo: &Repo,
    moodle: &dyn MoodleSource,
    folder: &Path,
    file_id: &str,
) -> Result<PathBuf, AppError> {
    let Some((url, name, _mimetype)) = repo.file_for_download(file_id).await? else {
        return Err(AppError::NotFound);
    };

    let dest = folder.join(safe_name(&name));
    let path = moodle.download(&url, &dest).await?;

    let size = tokio::fs::metadata(&path)
        .await
        .map_err(|e| AppError::Internal(format!("could not read the file: {e}")))?
        .len();

    if size > MAX_BYTES {
        let _ = tokio::fs::remove_file(&path).await;
        return Err(AppError::Internal(format!(
            "the file exceeds {} MB",
            MAX_BYTES / 1024 / 1024
        )));
    }

    Ok(path)
}

fn emit(emitter: &Option<crate::sync::EventEmitter>, event: &str, payload: &str) {
    if let Some(emitter) = emitter {
        emitter(event, serde_json::json!(payload));
    }
}
