use crate::error::error::AppError;
use crate::model::book_source::BookSource;
use crate::storage::db::repo::BookSourceRepo;
use std::path::PathBuf;
use tokio::fs;

#[derive(Clone)]
pub struct BookSourceService {
    repo: BookSourceRepo,
    default_owner_path: PathBuf,
}

impl BookSourceService {
    pub fn new(repo: BookSourceRepo, storage_dir: &str) -> Self {
        let default_owner_path = PathBuf::from(storage_dir)
            .join("data")
            .join("__default__")
            .join("defaultBookSourceOwner.txt");
        Self { repo, default_owner_path }
    }

    pub async fn save(&self, user_ns: &str, source: BookSource) -> Result<(), AppError> {
        let json = serde_json::to_string(&source).map_err(|e| AppError::BadRequest(e.to_string()))?;
        self.repo.upsert(user_ns, &source, &json).await
    }

    pub async fn save_many(&self, user_ns: &str, sources: Vec<BookSource>) -> Result<(), AppError> {
        for s in sources {
            self.save(user_ns, s).await?;
        }
        Ok(())
    }

    pub async fn get(&self, user_ns: &str, book_source_url: &str) -> Result<Option<BookSource>, AppError> {
        let json = self.repo.get(user_ns, book_source_url).await?;
        if let Some(j) = json {
            let source: BookSource = serde_json::from_str(&j).map_err(|e| AppError::BadRequest(e.to_string()))?;
            Ok(Some(source))
        } else {
            Ok(None)
        }
    }

    pub async fn list(&self, user_ns: &str) -> Result<Vec<BookSource>, AppError> {
        let rows = self.repo.list(user_ns).await?;
        let mut out = Vec::with_capacity(rows.len());
        for j in rows {
            if let Ok(s) = serde_json::from_str::<BookSource>(&j) {
                out.push(s);
            }
        }
        Ok(out)
    }

    pub async fn delete(&self, user_ns: &str, book_source_url: &str) -> Result<(), AppError> {
        self.repo.delete(user_ns, book_source_url).await
    }

    pub async fn delete_all(&self, user_ns: &str) -> Result<(), AppError> {
        self.repo.delete_all(user_ns).await
    }

    /// Copy sources from one user to another (used for setting default sources)
    pub async fn copy_to(&self, from_ns: &str, to_ns: &str) -> Result<i64, AppError> {
        self.repo.copy_to(from_ns, to_ns).await
    }

    /// Set a user's sources as the default sources (for new users)
    pub async fn set_as_default(&self, from_ns: &str) -> Result<i64, AppError> {
        let count = self.copy_to(from_ns, "__default__").await?;
        if let Some(dir) = self.default_owner_path.parent() {
            fs::create_dir_all(dir)
                .await
                .map_err(|e| AppError::Internal(e.into()))?;
        }
        fs::write(&self.default_owner_path, from_ns)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        Ok(count)
    }

    /// Copy default sources to a new user
    pub async fn copy_default_to_user(&self, to_ns: &str) -> Result<i64, AppError> {
        let defaults = self.list("__default__").await?;
        if defaults.is_empty() {
            return Ok(0);
        }
        let count = defaults.len() as i64;
        self.save_many(to_ns, defaults).await?;
        Ok(count)
    }

    pub async fn get_default_owner(&self) -> Result<Option<String>, AppError> {
        match fs::read_to_string(&self.default_owner_path).await {
            Ok(value) => {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(trimmed.to_string()))
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(AppError::Internal(err.into())),
        }
    }
}
