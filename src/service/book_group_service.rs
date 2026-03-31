use tokio::fs;
use std::path::PathBuf;
use crate::error::error::AppError;
use crate::model::book_group::BookGroup;

pub struct BookGroupService {
    storage_dir: String,
}

impl BookGroupService {
    pub fn new(storage_dir: &str) -> Self {
        Self {
            storage_dir: storage_dir.to_string(),
        }
    }

    fn file_path(&self, user_ns: &str) -> PathBuf {
        PathBuf::from(&self.storage_dir)
            .join("data")
            .join(user_ns)
            .join("book_groups.json")
    }

    pub async fn get_groups(&self, user_ns: &str) -> Result<Vec<BookGroup>, AppError> {
        let path = self.file_path(user_ns);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(path).await.map_err(|e| AppError::Internal(e.into()))?;
        let groups: Vec<BookGroup> = serde_json::from_str(&data).unwrap_or_default();
        Ok(groups)
    }

    pub async fn save_groups(&self, user_ns: &str, groups: &Vec<BookGroup>) -> Result<(), AppError> {
        let path = self.file_path(user_ns);
        if let Some(p) = path.parent() {
            if !p.exists() {
                fs::create_dir_all(p).await.map_err(|e| AppError::Internal(e.into()))?;
            }
        }
        let data = serde_json::to_string(groups).map_err(|e| AppError::Internal(e.into()))?;
        fs::write(path, data).await.map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }

    pub async fn save_group(&self, user_ns: &str, mut group: BookGroup) -> Result<(), AppError> {
        let mut groups = self.get_groups(user_ns).await?;
        if group.group_id == 0 {
            // Generate a simple id based on max id + 1 or timestamp
            let max_id = groups.iter().map(|g| g.group_id).max().unwrap_or(0);
            group.group_id = max_id + 1;
        }
        let mut found = false;
        for g in &mut groups {
            if g.group_id == group.group_id {
                g.group_name = group.group_name.clone();
                g.order_no = group.order_no;
                found = true;
                break;
            }
        }
        if !found {
            groups.push(group);
        }
        self.save_groups(user_ns, &groups).await?;
        Ok(())
    }

    pub async fn delete_group(&self, user_ns: &str, group_id: i64) -> Result<(), AppError> {
        let mut groups = self.get_groups(user_ns).await?;
        groups.retain(|g| g.group_id != group_id);
        self.save_groups(user_ns, &groups).await?;
        Ok(())
    }
}
