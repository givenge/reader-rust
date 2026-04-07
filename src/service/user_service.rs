use crate::app::config::AppConfig;
use crate::error::error::AppError;
use crate::model::user::User;
use crate::util::crypto::{gen_encrypted_password, random_string};
use crate::util::time::now_ts;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

const TOKEN_TTL_MS: i64 = 7 * 86_400 * 1000;

#[derive(Clone)]
pub struct UserService {
    cfg: AppConfig,
    users_path: PathBuf,
    data_root: PathBuf,
}

impl UserService {
    pub fn new(cfg: AppConfig) -> Self {
        let data_root = PathBuf::from(&cfg.storage_dir).join("data");
        let users_path = data_root.join("users.json");
        Self {
            cfg,
            users_path,
            data_root,
        }
    }

    pub fn secure_enabled(&self) -> bool {
        self.cfg.secure
    }

    pub fn secure_key_required(&self) -> bool {
        !self.cfg.secure_key.is_empty()
    }

    pub fn secure_key_matches(&self, key: &str) -> bool {
        !self.cfg.secure_key.is_empty() && self.cfg.secure_key == key
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
        is_login: bool,
        code: Option<&str>,
    ) -> Result<Value, AppError> {
        let mut users = self.load_users().await?;

        // Check if this is the first user (will be admin)
        let is_first_user = users.is_empty();

        if let Some(user) = users.get_mut(username) {
            if !is_login {
                return Err(AppError::BadRequest("用户名已被占用".to_string()));
            }
            let encrypted = gen_encrypted_password(password, &user.salt);
            if encrypted != user.password {
                return Err(AppError::BadRequest("密码错误".to_string()));
            }
            let login_data = self.save_user_session(user, true);
            self.save_users(&users).await?;
            return Ok(login_data);
        }

        if is_login {
            return Err(AppError::BadRequest("用户不存在".to_string()));
        }
        self.validate_new_user(username, password, code)?;
        if users.len() as u32 >= self.cfg.user_limit {
            return Err(AppError::BadRequest("超过用户数上限".to_string()));
        }

        let salt = random_string(8);
        let encrypted = gen_encrypted_password(password, &salt);
        let mut user = User {
            username: username.to_string(),
            password: encrypted,
            salt,
            token: "".to_string(),
            last_login_at: now_ts() * 1000,
            created_at: now_ts() * 1000,
            enable_webdav: false,
            token_map: None,
            enable_local_store: false,
            is_admin: is_first_user, // First user is admin
        };
        let login_data = self.save_user_session(&mut user, true);
        users.insert(username.to_string(), user);
        self.save_users(&users).await?;
        Ok(login_data)
    }

    pub async fn logout(&self, access_token: &str) -> Result<(), AppError> {
        let (username, token) = parse_access_token(access_token)?;
        let mut users = self.load_users().await?;
        let user = users
            .get_mut(&username)
            .ok_or_else(|| AppError::BadRequest("系统错误".to_string()))?;
        let (mut map, _) = self.collect_active_sessions(user, now_ms());
        map.remove(&token);
        user.token_map = Some(map);
        if user.token == token {
            user.token = "".to_string();
        }
        self.save_users(&users).await?;
        Ok(())
    }

    pub async fn get_user_info(
        &self,
        access_token: Option<&str>,
    ) -> Result<(Option<Value>, bool, bool), AppError> {
        if !self.cfg.secure {
            return Ok((None, false, self.secure_key_required()));
        }
        if let Some(token) = access_token {
            if let Some(user) = self.check_auth(token).await? {
                return Ok((
                    Some(self.format_user(&user)),
                    true,
                    self.secure_key_required(),
                ));
            }
        }
        Ok((None, true, self.secure_key_required()))
    }

    pub async fn save_user_config(&self, user_ns: &str, config: Value) -> Result<(), AppError> {
        let dir = self.data_root.join(user_ns);
        fs::create_dir_all(&dir)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        let path = dir.join("userConfig.json");
        let mut cfg = config;
        if let Some(obj) = cfg.as_object_mut() {
            obj.insert("@updateTime".to_string(), Value::from(now_ts() * 1000));
        }
        let data = serde_json::to_string(&cfg).map_err(|e| AppError::BadRequest(e.to_string()))?;
        fs::write(path, data)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }

    pub async fn get_user_config(&self, user_ns: &str) -> Result<Value, AppError> {
        let path = self.data_root.join(user_ns).join("userConfig.json");
        if !path.exists() {
            return Err(AppError::BadRequest("没有备份文件".to_string()));
        }
        let data = fs::read_to_string(path)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        let v: Value =
            serde_json::from_str(&data).map_err(|e| AppError::BadRequest(e.to_string()))?;
        Ok(v)
    }

    pub async fn get_user_list(&self) -> Result<Vec<Value>, AppError> {
        let users = self.load_users().await?;
        Ok(users.values().map(|u| self.format_user(u)).collect())
    }

    pub async fn add_user(&self, username: &str, password: &str) -> Result<Vec<Value>, AppError> {
        self.validate_new_user(username, password, None)?;
        let mut users = self.load_users().await?;
        if users.contains_key(username) {
            return Err(AppError::BadRequest("用户已存在".to_string()));
        }
        if users.len() as u32 >= self.cfg.user_limit {
            return Err(AppError::BadRequest("超过用户数上限".to_string()));
        }
        let salt = random_string(8);
        let encrypted = gen_encrypted_password(password, &salt);
        let user = User {
            username: username.to_string(),
            password: encrypted,
            salt,
            token: "".to_string(),
            last_login_at: now_ts() * 1000,
            created_at: now_ts() * 1000,
            enable_webdav: false,
            token_map: None,
            enable_local_store: false,
            is_admin: false,
        };
        users.insert(username.to_string(), user);
        self.save_users(&users).await?;
        Ok(users.values().map(|u| self.format_user(u)).collect())
    }

    pub async fn reset_password(&self, username: &str, password: &str) -> Result<(), AppError> {
        if password.len() < 8 {
            return Err(AppError::BadRequest("密码不能低于8位".to_string()));
        }
        if username == "default" {
            return Err(AppError::BadRequest("用户不存在".to_string()));
        }
        let mut users = self.load_users().await?;
        let user = users
            .get_mut(username)
            .ok_or_else(|| AppError::BadRequest("用户不存在".to_string()))?;
        let salt = random_string(8);
        let encrypted = gen_encrypted_password(password, &salt);
        user.salt = salt;
        user.password = encrypted;
        self.save_users(&users).await?;
        Ok(())
    }

    pub async fn change_password(
        &self,
        access_token: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        if new_password.len() < 8 {
            return Err(AppError::BadRequest("密码不能低于8位".to_string()));
        }
        let (username, token) = parse_access_token(access_token)?;
        let mut users = self.load_users().await?;
        let user = users
            .get_mut(&username)
            .ok_or_else(|| AppError::BadRequest("用户不存在".to_string()))?;
        let encrypted_old = gen_encrypted_password(old_password, &user.salt);
        if encrypted_old != user.password {
            return Err(AppError::BadRequest("当前密码错误".to_string()));
        }

        let salt = random_string(8);
        let encrypted = gen_encrypted_password(new_password, &salt);
        user.salt = salt;
        user.password = encrypted;
        let now = now_ms();
        let (mut map, _) = self.collect_active_sessions(user, now);
        map.retain(|k, v| k == &token && *v > now);
        map.insert(token.clone(), session_expire_at(now));
        user.token = token.clone();
        user.last_login_at = now;
        user.token_map = Some(map);
        self.save_users(&users).await?;
        Ok(())
    }

    pub async fn delete_users(&self, usernames: &[String]) -> Result<Vec<Value>, AppError> {
        let mut users = self.load_users().await?;
        for u in usernames {
            users.remove(u);
            let user_dir = self.data_root.join(u);
            if user_dir.exists() {
                let _ = fs::remove_dir_all(user_dir).await;
            }
        }
        self.save_users(&users).await?;
        Ok(users.values().map(|u| self.format_user(u)).collect())
    }

    pub async fn update_user(
        &self,
        username: &str,
        enable_webdav: Option<bool>,
        enable_local_store: Option<bool>,
    ) -> Result<Vec<Value>, AppError> {
        let mut users = self.load_users().await?;
        let user = users
            .get_mut(username)
            .ok_or_else(|| AppError::BadRequest("用户不存在".to_string()))?;
        if let Some(v) = enable_webdav {
            user.enable_webdav = v;
        }
        if let Some(v) = enable_local_store {
            user.enable_local_store = v;
        }
        self.save_users(&users).await?;
        Ok(users.values().map(|u| self.format_user(u)).collect())
    }

    pub async fn check_auth(&self, access_token: &str) -> Result<Option<User>, AppError> {
        if !self.cfg.secure {
            return Ok(None);
        }
        let (username, token) = parse_access_token(access_token)?;
        let mut users = self.load_users().await?;
        let user = match users.get_mut(&username) {
            Some(u) => u,
            None => return Ok(None),
        };
        let now = now_ms();
        let (mut map, _) = self.collect_active_sessions(user, now);
        if map.contains_key(&token) {
            map.insert(token.clone(), session_expire_at(now));
            user.token = token.clone();
            user.last_login_at = now;
            user.token_map = Some(map);
            let user_clone = user.clone();
            self.save_users(&users).await?;
            return Ok(Some(user_clone));
        }
        user.token_map = Some(map);
        Ok(None)
    }

    /// Check if user is admin (either by is_admin flag or by secure key)
    pub async fn is_admin(
        &self,
        access_token: Option<&str>,
        secure_key: Option<&str>,
    ) -> Result<bool, AppError> {
        // Check secure key first
        if let Some(key) = secure_key {
            if self.secure_key_matches(key) {
                return Ok(true);
            }
        }
        // Check if user is admin
        if let Some(token) = access_token {
            if let Ok(Some(user)) = self.check_auth(token).await {
                return Ok(user.is_admin);
            }
        }
        Ok(false)
    }

    pub async fn resolve_user_ns_with_override(
        &self,
        access_token: Option<&str>,
        secure_key: Option<&str>,
        user_ns: Option<&str>,
    ) -> Result<String, AppError> {
        if !self.cfg.secure {
            return Ok("default".to_string());
        }
        if let Some(key) = secure_key {
            if self.secure_key_matches(key) {
                if let Some(ns) = user_ns {
                    let ns = ns.trim();
                    if !ns.is_empty() {
                        return Ok(ns.to_string());
                    }
                }
                return Ok("default".to_string());
            }
        }
        if let Some(token) = access_token {
            if let Ok(Some(user)) = self.check_auth(token).await {
                return Ok(user.username);
            }
        }
        Err(AppError::BadRequest("NEED_LOGIN".to_string()))
    }

    fn save_user_session(&self, user: &mut User, regenerate_token: bool) -> Value {
        user.last_login_at = now_ms();
        let (mut map, _) = self.collect_active_sessions(user, user.last_login_at);
        if regenerate_token {
            user.token = self.generate_session_token(&user.username);
            map.insert(user.token.clone(), session_expire_at(user.last_login_at));
        }
        user.token_map = Some(map);
        self.format_user(user)
    }

    fn format_user(&self, user: &User) -> Value {
        serde_json::json!({
            "username": user.username,
            "lastLoginAt": user.last_login_at,
            "accessToken": format!("{}:{}", user.username, user.token),
            "enableWebdav": user.enable_webdav,
            "enableLocalStore": user.enable_local_store,
            "createdAt": user.created_at,
            "isAdmin": user.is_admin,
        })
    }

    fn collect_active_sessions(&self, user: &User, now: i64) -> (HashMap<String, i64>, bool) {
        let mut changed = false;
        let mut map = user.token_map.clone().unwrap_or_default();
        map.retain(|_, exp| {
            let keep = *exp > now;
            if !keep {
                changed = true;
            }
            keep
        });

        if !user.token.is_empty() {
            let legacy_expire_at = user.last_login_at.saturating_add(TOKEN_TTL_MS);
            if legacy_expire_at > now {
                match map.get_mut(&user.token) {
                    Some(exp) if *exp < legacy_expire_at => {
                        *exp = legacy_expire_at;
                        changed = true;
                    }
                    Some(_) => {}
                    None => {
                        map.insert(user.token.clone(), legacy_expire_at);
                        changed = true;
                    }
                }
            }
        }

        (map, changed)
    }

    fn generate_session_token(&self, username: &str) -> String {
        gen_encrypted_password(username, &format!("{}{}", now_ms(), random_string(8)))
    }

    pub async fn verify_basic_webdav(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<User>, AppError> {
        if !self.cfg.secure {
            return Ok(None);
        }
        let users = self.load_users().await?;
        let user = match users.get(username) {
            Some(u) => u,
            None => return Ok(None),
        };
        if !user.enable_webdav {
            return Ok(None);
        }
        let encrypted = gen_encrypted_password(password, &user.salt);
        if encrypted != user.password {
            return Ok(None);
        }
        Ok(Some(user.clone()))
    }

    pub async fn require_webdav_user(
        &self,
        access_token: Option<&str>,
    ) -> Result<String, AppError> {
        if !self.cfg.secure {
            return Err(AppError::BadRequest("仅安全模式支持WebDAV功能".to_string()));
        }
        let token = access_token.ok_or_else(|| AppError::BadRequest("NEED_LOGIN".to_string()))?;
        let user = self
            .check_auth(token)
            .await?
            .ok_or_else(|| AppError::BadRequest("NEED_LOGIN".to_string()))?;
        if !user.enable_webdav {
            return Err(AppError::BadRequest("未开启webdav功能".to_string()));
        }
        Ok(user.username)
    }

    async fn load_users(&self) -> Result<HashMap<String, User>, AppError> {
        if !self.users_path.exists() {
            return Ok(HashMap::new());
        }
        let data = fs::read_to_string(&self.users_path)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        let map: HashMap<String, User> =
            serde_json::from_str(&data).map_err(|e| AppError::BadRequest(e.to_string()))?;
        Ok(map)
    }

    async fn save_users(&self, users: &HashMap<String, User>) -> Result<(), AppError> {
        if let Some(parent) = self.users_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::Internal(e.into()))?;
        }
        let data = serde_json::to_string(users).map_err(|e| AppError::BadRequest(e.to_string()))?;
        fs::write(&self.users_path, data)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }

    fn validate_new_user(
        &self,
        username: &str,
        password: &str,
        code: Option<&str>,
    ) -> Result<(), AppError> {
        if username.is_empty() {
            return Err(AppError::BadRequest("请输入用户名".to_string()));
        }
        if password.is_empty() {
            return Err(AppError::BadRequest("请输入密码".to_string()));
        }
        if username.len() < 5 {
            return Err(AppError::BadRequest("用户名不能低于5位".to_string()));
        }
        if password.len() < 8 {
            return Err(AppError::BadRequest("密码不能低于8位".to_string()));
        }
        if username == "default" {
            return Err(AppError::BadRequest("用户名不能为非法字符".to_string()));
        }
        let re = regex::Regex::new("^[a-z0-9]+$").unwrap();
        if !re.is_match(username) {
            return Err(AppError::BadRequest(
                "用户名只能由字母和数字组成".to_string(),
            ));
        }
        if !self.cfg.invite_code.is_empty() {
            let c = code.unwrap_or("");
            if c.is_empty() {
                return Err(AppError::BadRequest("请输入邀请码".to_string()));
            }
            if c != self.cfg.invite_code {
                return Err(AppError::BadRequest("邀请码错误".to_string()));
            }
        }
        Ok(())
    }
}

fn parse_access_token(access_token: &str) -> Result<(String, String), AppError> {
    let parts: Vec<&str> = access_token.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(AppError::BadRequest("NEED_LOGIN".to_string()));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn now_ms() -> i64 {
    now_ts() * 1000
}

fn session_expire_at(now: i64) -> i64 {
    now.saturating_add(TOKEN_TTL_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_user_service() -> (UserService, PathBuf) {
        let temp_dir =
            std::env::temp_dir().join(format!("reader-rust-user-service-{}", random_string(8)));
        let cfg = AppConfig {
            secure: true,
            storage_dir: temp_dir.to_string_lossy().to_string(),
            ..AppConfig::default()
        };
        (UserService::new(cfg), temp_dir)
    }

    #[tokio::test]
    async fn login_keeps_legacy_session_and_supports_multi_device() {
        let (service, temp_dir) = create_user_service();

        let first_login = service
            .login("reader1", "password123", false, None)
            .await
            .unwrap();
        let first_access_token = first_login["accessToken"].as_str().unwrap().to_string();
        let (_, first_raw_token) = parse_access_token(&first_access_token).unwrap();

        let mut users = service.load_users().await.unwrap();
        let user = users.get_mut("reader1").unwrap();
        user.token = first_raw_token.clone();
        user.token_map = None;
        service.save_users(&users).await.unwrap();

        let second_login = service
            .login("reader1", "password123", true, None)
            .await
            .unwrap();
        let second_access_token = second_login["accessToken"].as_str().unwrap().to_string();
        let (_, second_raw_token) = parse_access_token(&second_access_token).unwrap();

        assert_ne!(first_access_token, second_access_token);
        assert!(service
            .check_auth(&first_access_token)
            .await
            .unwrap()
            .is_some());
        assert!(service
            .check_auth(&second_access_token)
            .await
            .unwrap()
            .is_some());

        let users = service.load_users().await.unwrap();
        let token_map = users
            .get("reader1")
            .and_then(|user| user.token_map.as_ref())
            .unwrap();
        assert!(token_map.contains_key(&first_raw_token));
        assert!(token_map.contains_key(&second_raw_token));

        let _ = fs::remove_dir_all(temp_dir).await;
    }

    #[tokio::test]
    async fn get_user_info_refreshes_current_session_and_returns_current_token() {
        let (service, temp_dir) = create_user_service();

        let login = service
            .login("reader2", "password123", false, None)
            .await
            .unwrap();
        let access_token = login["accessToken"].as_str().unwrap().to_string();
        let (_, raw_token) = parse_access_token(&access_token).unwrap();

        let short_expire_at = now_ms() + 1000;
        let mut users = service.load_users().await.unwrap();
        let user = users.get_mut("reader2").unwrap();
        user.token_map
            .as_mut()
            .unwrap()
            .insert(raw_token.clone(), short_expire_at);
        service.save_users(&users).await.unwrap();

        let (user_info, secure, secure_key_required) =
            service.get_user_info(Some(&access_token)).await.unwrap();

        assert!(secure);
        assert!(!secure_key_required);
        assert_eq!(
            user_info.unwrap()["accessToken"].as_str().unwrap(),
            access_token
        );

        let users = service.load_users().await.unwrap();
        let expire_at = users
            .get("reader2")
            .and_then(|user| user.token_map.as_ref())
            .and_then(|map| map.get(&raw_token))
            .copied()
            .unwrap();
        assert!(expire_at > short_expire_at);

        let _ = fs::remove_dir_all(temp_dir).await;
    }
}
