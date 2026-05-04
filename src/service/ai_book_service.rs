use std::path::{Path, PathBuf};

use tokio::fs;

use crate::error::error::AppError;
use crate::model::ai_book::AiBookMemory;
use crate::util::hash::md5_hex;
use crate::util::time::now_ts;

#[derive(Clone)]
pub struct AiBookService {
    storage_dir: PathBuf,
}

impl AiBookService {
    pub fn new(storage_dir: &str) -> Self {
        Self {
            storage_dir: PathBuf::from(storage_dir),
        }
    }

    pub async fn get(&self, user_ns: &str, book_url: &str) -> Result<Option<AiBookMemory>, AppError> {
        let path = self.memory_path(user_ns, book_url);
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(path)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        let memory = serde_json::from_str::<AiBookMemory>(&data)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        Ok(Some(memory))
    }

    pub async fn save_for_book(
        &self,
        user_ns: &str,
        book_url: &str,
        mut memory: AiBookMemory,
    ) -> Result<AiBookMemory, AppError> {
        if book_url.trim().is_empty() {
            return Err(AppError::BadRequest("bookUrl required".to_string()));
        }
        if memory.book_url.trim().is_empty() {
            memory.book_url = book_url.to_string();
        }
        if memory.book_url != book_url {
            return Err(AppError::BadRequest("bookUrl mismatch".to_string()));
        }
        if memory.updated_at <= 0 {
            memory.updated_at = now_ts() * 1000;
        }

        let path = self.memory_path(user_ns, book_url);
        ensure_parent(&path).await?;
        let data = serde_json::to_string(&memory)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        fs::write(path, data)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        Ok(memory)
    }

    pub async fn delete(&self, user_ns: &str, book_url: &str) -> Result<bool, AppError> {
        let path = self.memory_path(user_ns, book_url);
        if !path.exists() {
            return Ok(false);
        }
        fs::remove_file(path)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        Ok(true)
    }

    fn memory_path(&self, user_ns: &str, book_url: &str) -> PathBuf {
        self.storage_dir
            .join("data")
            .join(user_ns)
            .join("ai-books")
            .join(format!("{}.json", md5_hex(book_url)))
    }
}

async fn ensure_parent(path: &Path) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    }
    Ok(())
}
