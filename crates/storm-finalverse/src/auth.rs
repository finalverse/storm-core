// File: crates/storm-finalverse/src/auth.rs
// Authentication and authorization for Finalverse protocol
// Handles token management, session validation, and security

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use tracing::{info, warn, debug};
use anyhow::Result;

/// Authentication manager for Finalverse connections
pub struct FinalverseAuth {
    /// Active sessions keyed by session ID
    sessions: HashMap<String, Session>,
    /// User credentials cache
    user_cache: HashMap<String, UserInfo>,
    /// Authentication configuration
    config: AuthConfig,
}

/// User session information
#[derive(Debug, Clone)]
pub struct Session {
    /// Unique session identifier
    pub session_id: String,
    /// User ID associated with this session
    pub user_id: String,
    /// Authentication token
    pub token: String,
    /// Session creation timestamp
    pub created_at: SystemTime,
    /// Last activity timestamp
    pub last_activity: SystemTime,
    /// Session expiration time
    pub expires_at: SystemTime,
    /// Permissions granted to this session
    pub permissions: Vec<Permission>,
    /// Client information
    pub client_info: ClientInfo,
}

/// User information and credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// Unique user identifier
    pub user_id: String,
    /// Username for authentication
    pub username: String,
    /// Display name for the user
    pub display_name: String,
    /// Email address (optional)
    pub email: Option<String>,
    /// Password hash (not stored in plaintext)
    pub password_hash: String,
    /// Account creation timestamp
    pub created_at: i64,
    /// Last login timestamp
    pub last_login: Option<i64>,
    /// Account status
    pub status: AccountStatus,
    /// User roles and permissions
    pub roles: Vec<String>,
    /// User preferences
    pub preferences: HashMap<String, serde_json::Value>,
}

/// Client information for session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client application name
    pub client_name: String,
    /// Client version
    pub client_version: String,
    /// Platform/OS information
    pub platform: String,
    /// Client IP address
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
}

/// Account status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    /// Account is active and can be used
    Active,
    /// Account is suspended temporarily
    Suspended,
    /// Account is banned permanently
    Banned,
    /// Account is pending verification
    PendingVerification,
    /// Account has been deleted
    Deleted,
}

impl PartialEq for AccountStatus {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (AccountStatus::Active, AccountStatus::Active)
            | (AccountStatus::Suspended, AccountStatus::Suspended)
            | (AccountStatus::Banned, AccountStatus::Banned)
            | (AccountStatus::PendingVerification, AccountStatus::PendingVerification)
            | (AccountStatus::Deleted, AccountStatus::Deleted)
        )
    }
}

/// Permission types for authorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Basic read access to world data
    ReadWorld,
    /// Write access to modify world objects
    WriteWorld,
    /// Administrative access to manage world
    AdminWorld,
    /// Send chat messages
    Chat,
    /// Moderate chat and users
    ModerateChat,
    /// Create and modify avatars
    ManageAvatar,
    /// Upload and manage assets
    ManageAssets,
    /// Use AI features
    UseAi,
    /// Administrative AI features
    AdminAi,
    /// System administration
    SystemAdmin,
}

// Add required traits for Permission
impl PartialOrd for Permission {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Permission {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Use a simple ordering based on the variant index
        let self_index = match self {
            Permission::ReadWorld => 0,
            Permission::WriteWorld => 1,
            Permission::AdminWorld => 2,
            Permission::Chat => 3,
            Permission::ModerateChat => 4,
            Permission::ManageAvatar => 5,
            Permission::ManageAssets => 6,
            Permission::UseAi => 7,
            Permission::AdminAi => 8,
            Permission::SystemAdmin => 9,
        };

        let other_index = match other {
            Permission::ReadWorld => 0,
            Permission::WriteWorld => 1,
            Permission::AdminWorld => 2,
            Permission::Chat => 3,
            Permission::ModerateChat => 4,
            Permission::ManageAvatar => 5,
            Permission::ManageAssets => 6,
            Permission::UseAi => 7,
            Permission::AdminAi => 8,
            Permission::SystemAdmin => 9,
        };

        self_index.cmp(&other_index)
    }
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Session timeout duration
    pub session_timeout: Duration,
    /// Token refresh interval
    pub refresh_interval: Duration,
    /// Maximum concurrent sessions per user
    pub max_sessions_per_user: u32,
    /// Password minimum length
    pub min_password_length: usize,
    /// Enable two-factor authentication
    pub enable_2fa: bool,
    /// JWT secret for token signing
    pub jwt_secret: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            session_timeout: Duration::from_secs(24 * 60 * 60), // 24 hours in seconds
            refresh_interval: Duration::from_secs(60 * 60),     // 1 hour in seconds
            max_sessions_per_user: 5,
            min_password_length: 8,
            enable_2fa: false,
            jwt_secret: "default_secret_change_in_production".to_string(),
        }
    }
}

/// Authentication request payload
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
    pub client_info: ClientInfo,
    pub two_factor_code: Option<String>,
}

/// Authentication response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub session_id: Option<String>,
    pub token: Option<String>,
    pub user_info: Option<UserInfo>,
    pub permissions: Option<Vec<Permission>>,
    pub expires_at: Option<i64>,
    pub error: Option<String>,
}

/// Token validation result
#[derive(Debug)]
pub struct TokenValidation {
    pub valid: bool,
    pub session: Option<Session>,
    pub error: Option<String>,
}

impl FinalverseAuth {
    /// Create a new authentication manager
    pub fn new() -> Self {
        Self::with_config(AuthConfig::default())
    }

    /// Create authentication manager with custom configuration
    pub fn with_config(config: AuthConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            user_cache: HashMap::new(),
            config,
        }
    }

    /// Authenticate a user with username and password
    pub async fn authenticate(&mut self, request: &AuthRequest) -> Result<AuthResponse> {
        info!("Authentication attempt for user: {}", request.username);

        // Validate input
        if request.username.is_empty() || request.password.is_empty() {
            return Ok(AuthResponse {
                success: false,
                session_id: None,
                token: None,
                user_info: None,
                permissions: None,
                expires_at: None,
                error: Some("Username and password are required".to_string()),
            });
        }

        // Look up user (in production, this would query a database)
        let user_info = match self.get_user_info(&request.username).await {
            Some(user) => user,
            None => {
                warn!("Authentication failed: user not found: {}", request.username);
                return Ok(AuthResponse {
                    success: false,
                    session_id: None,
                    token: None,
                    user_info: None,
                    permissions: None,
                    expires_at: None,
                    error: Some("Invalid username or password".to_string()),
                });
            }
        };

        // Check account status
        if user_info.status != AccountStatus::Active {
            warn!("Authentication failed: account not active: {} ({:?})",
                request.username, user_info.status);
            return Ok(AuthResponse {
                success: false,
                session_id: None,
                token: None,
                user_info: None,
                permissions: None,
                expires_at: None,
                error: Some("Account is not active".to_string()),
            });
        }

        // Verify password
        if !self.verify_password(&request.password, &user_info.password_hash) {
            warn!("Authentication failed: invalid password for user: {}", request.username);
            return Ok(AuthResponse {
                success: false,
                session_id: None,
                token: None,
                user_info: None,
                permissions: None,
                expires_at: None,
                error: Some("Invalid username or password".to_string()),
            });
        }

        // Check 2FA if enabled
        if self.config.enable_2fa {
            if let Some(code) = &request.two_factor_code {
                if !self.verify_2fa_code(&user_info.user_id, code) {
                    warn!("Authentication failed: invalid 2FA code for user: {}", request.username);
                    return Ok(AuthResponse {
                        success: false,
                        session_id: None,
                        token: None,
                        user_info: None,
                        permissions: None,
                        expires_at: None,
                        error: Some("Invalid two-factor authentication code".to_string()),
                    });
                }
            } else {
                return Ok(AuthResponse {
                    success: false,
                    session_id: None,
                    token: None,
                    user_info: None,
                    permissions: None,
                    expires_at: None,
                    error: Some("Two-factor authentication code required".to_string()),
                });
            }
        }

        // Check session limits
        if self.get_user_session_count(&user_info.user_id) >= self.config.max_sessions_per_user {
            warn!("Authentication failed: session limit reached for user: {}", request.username);
            return Ok(AuthResponse {
                success: false,
                session_id: None,
                token: None,
                user_info: None,
                permissions: None,
                expires_at: None,
                error: Some("Maximum number of sessions reached".to_string()),
            });
        }

        // Create new session
        let session = self.create_session(&user_info, &request.client_info)?;

        info!("Authentication successful for user: {} (session: {})",
            request.username, session.session_id);

        Ok(AuthResponse {
            success: true,
            session_id: Some(session.session_id.clone()),
            token: Some(session.token.clone()),
            user_info: Some(user_info),
            permissions: Some(session.permissions.clone()),
            expires_at: Some(session.expires_at.duration_since(UNIX_EPOCH)?.as_secs() as i64),
            error: None,
        })
    }

    /// Validate an authentication token - fixed borrowing issue
    pub fn validate_token(&mut self, token: &str) -> TokenValidation {
        // First pass: find the session ID that matches the token
        let mut matching_session_id = None;
        for (session_id, session) in &self.sessions {
            if session.token == token {
                matching_session_id = Some(session_id.clone());
                break;
            }
        }

        // Second pass: update the matching session if found
        if let Some(session_id) = matching_session_id {
            if let Some(session) = self.sessions.get_mut(&session_id) {
                // Check if session has expired
                if SystemTime::now() > session.expires_at {
                    debug!("Token validation failed: session expired for {}", session.session_id);
                    return TokenValidation {
                        valid: false,
                        session: None,
                        error: Some("Session expired".to_string()),
                    };
                }

                // Update last activity
                session.last_activity = SystemTime::now();

                debug!("Token validation successful for session: {}", session.session_id);
                return TokenValidation {
                    valid: true,
                    session: Some(session.clone()),
                    error: None,
                };
            }
        }

        debug!("Token validation failed: token not found");
        TokenValidation {
            valid: false,
            session: None,
            error: Some("Invalid token".to_string()),
        }
    }

    /// Check if a user has a specific permission
    pub fn has_permission(&self, session_id: &str, permission: &Permission) -> bool {
        if let Some(session) = self.sessions.get(session_id) {
            session.permissions.contains(permission)
        } else {
            false
        }
    }

    /// Revoke a session (logout)
    pub fn revoke_session(&mut self, session_id: &str) -> Result<()> {
        if let Some(session) = self.sessions.remove(session_id) {
            info!("Session revoked: {} (user: {})", session_id, session.user_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }

    /// Revoke all sessions for a user
    pub fn revoke_user_sessions(&mut self, user_id: &str) -> Result<u32> {
        let mut revoked_count = 0;
        self.sessions.retain(|_, session| {
            if session.user_id == user_id {
                revoked_count += 1;
                false
            } else {
                true
            }
        });

        info!("Revoked {} sessions for user: {}", revoked_count, user_id);
        Ok(revoked_count)
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) -> u32 {
        let now = SystemTime::now();
        let initial_count = self.sessions.len();

        self.sessions.retain(|session_id, session| {
            if now > session.expires_at {
                debug!("Cleaning up expired session: {}", session_id);
                false
            } else {
                true
            }
        });

        let cleaned_count = initial_count - self.sessions.len();
        if cleaned_count > 0 {
            info!("Cleaned up {} expired sessions", cleaned_count);
        }

        cleaned_count as u32
    }

    /// Refresh a session token - fixed borrowing issue
    pub fn refresh_session(&mut self, session_id: &str) -> Result<String> {
        // First, get the session info we need without holding a mutable borrow
        let (user_id, created_at, current_token) = {
            let session = self.sessions.get(session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;
            (session.user_id.clone(), session.created_at, session.token.clone())
        };

        // Check if refresh is needed
        let time_since_creation = SystemTime::now().duration_since(created_at)?;
        if time_since_creation < self.config.refresh_interval {
            return Ok(current_token); // No refresh needed yet
        }

        // Generate new token (now we can borrow self immutably)
        let new_token = self.generate_token(&user_id)?;

        // Now update the session with the new token
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        session.token = new_token.clone();
        session.last_activity = SystemTime::now();

        debug!("Session token refreshed for: {}", session_id);
        Ok(new_token)
    }

    /// Get active session count
    pub fn active_session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get session count for a specific user
    pub fn get_user_session_count(&self, user_id: &str) -> u32 {
        self.sessions.values()
            .filter(|session| session.user_id == user_id)
            .count() as u32
    }

    /// Get all sessions for a user
    pub fn get_user_sessions(&self, user_id: &str) -> Vec<&Session> {
        self.sessions.values()
            .filter(|session| session.user_id == user_id)
            .collect()
    }

    /// Create a new session for a user
    fn create_session(&mut self, user_info: &UserInfo, client_info: &ClientInfo) -> Result<Session> {
        let session_id = Uuid::new_v4().to_string();
        let token = self.generate_token(&user_info.user_id)?;
        let now = SystemTime::now();
        let expires_at = now + self.config.session_timeout;

        // Determine permissions based on user roles
        let permissions = self.get_permissions_for_roles(&user_info.roles);

        let session = Session {
            session_id: session_id.clone(),
            user_id: user_info.user_id.clone(),
            token,
            created_at: now,
            last_activity: now,
            expires_at,
            permissions,
            client_info: client_info.clone(),
        };

        self.sessions.insert(session_id, session.clone());
        Ok(session)
    }

    /// Generate a secure authentication token
    fn generate_token(&self, user_id: &str) -> Result<String> {
        use base64::{Engine as _, engine::general_purpose};

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let random_bytes = *Uuid::new_v4().as_bytes();

        let token_data = format!("{}:{}:{}", user_id, timestamp,
                                 general_purpose::STANDARD.encode(random_bytes));
        let mut hasher = Sha256::new();
        hasher.update(token_data.as_bytes());
        hasher.update(self.config.jwt_secret.as_bytes());

        let hash = hasher.finalize();
        Ok(general_purpose::STANDARD.encode(hash))
    }

    /// Verify a password against its hash
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        use base64::{Engine as _, engine::general_purpose};
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let computed_hash = general_purpose::STANDARD.encode(hasher.finalize());
        computed_hash == hash
    }

    /// Hash a password for storage
    pub fn hash_password(&self, password: &str) -> String {
        use base64::{Engine as _, engine::general_purpose};
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        general_purpose::STANDARD.encode(hasher.finalize())
    }

    /// Verify a 2FA code
    fn verify_2fa_code(&self, _user_id: &str, _code: &str) -> bool {
        // In production, implement proper TOTP verification
        // For now, just return true as a placeholder
        true
    }

    /// Get user information by username - fixed borrowing issue
    async fn get_user_info(&self, username: &str) -> Option<UserInfo> {
        // Check cache first
        if let Some(user) = self.user_cache.get(username) {
            return Some(user.clone());
        }

        // In production, this would query a database
        // For now, return a mock user for testing
        if username == "testuser" {
            // Create the password hash without borrowing self
            let password_hash = {
                use base64::{Engine as _, engine::general_purpose};
                let mut hasher = Sha256::new();
                hasher.update("testpass".as_bytes());
                general_purpose::STANDARD.encode(hasher.finalize())
            };

            Some(UserInfo {
                user_id: "user_123".to_string(),
                username: username.to_string(),
                display_name: "Test User".to_string(),
                email: Some("test@example.com".to_string()),
                password_hash,
                created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
                last_login: None,
                status: AccountStatus::Active,
                roles: vec!["user".to_string()],
                preferences: HashMap::new(),
            })
        } else {
            None
        }
    }

    /// Get permissions for user roles
    fn get_permissions_for_roles(&self, roles: &[String]) -> Vec<Permission> {
        let mut permissions = Vec::new();

        for role in roles {
            match role.as_str() {
                "user" => {
                    permissions.extend([
                        Permission::ReadWorld,
                        Permission::Chat,
                        Permission::ManageAvatar,
                        Permission::UseAi,
                    ]);
                }
                "moderator" => {
                    permissions.extend([
                        Permission::ReadWorld,
                        Permission::WriteWorld,
                        Permission::Chat,
                        Permission::ModerateChat,
                        Permission::ManageAvatar,
                        Permission::ManageAssets,
                        Permission::UseAi,
                    ]);
                }
                "admin" => {
                    permissions.extend([
                        Permission::ReadWorld,
                        Permission::WriteWorld,
                        Permission::AdminWorld,
                        Permission::Chat,
                        Permission::ModerateChat,
                        Permission::ManageAvatar,
                        Permission::ManageAssets,
                        Permission::UseAi,
                        Permission::AdminAi,
                        Permission::SystemAdmin,
                    ]);
                }
                _ => {
                    warn!("Unknown role: {}", role);
                }
            }
        }

        // Remove duplicates
        permissions.sort();
        permissions.dedup();
        permissions
    }

    /// Add a user to the cache (for testing/development)
    pub fn add_user_to_cache(&mut self, user_info: UserInfo) {
        self.user_cache.insert(user_info.username.clone(), user_info);
    }

    /// Start periodic cleanup of expired sessions
    pub async fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let cleanup_interval = self.config.session_timeout / 4; // Clean up 4 times per session timeout

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);

            loop {
                interval.tick().await;
                // Note: In a real implementation, we'd need to share the auth manager
                // across tasks properly, probably using Arc<Mutex<FinalverseAuth>>
                debug!("Periodic session cleanup would run here");
            }
        })
    }
}

impl Default for FinalverseAuth {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for authentication
pub mod utils {
    use super::*;
    use base64::{Engine as _, engine::general_purpose};

    /// Generate a secure random session ID
    pub fn generate_session_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Generate a secure random API key
    pub fn generate_api_key() -> String {
        let random_bytes: [u8; 32] = rand::random();
        general_purpose::STANDARD.encode(random_bytes)
    }

    /// Validate email format (basic validation)
    pub fn is_valid_email(email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() > 5
    }

    /// Validate username format
    pub fn is_valid_username(username: &str) -> bool {
        !username.is_empty() &&
            username.len() >= 3 &&
            username.len() <= 32 &&
            username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }

    /// Check password strength
    pub fn check_password_strength(password: &str) -> PasswordStrength {
        let length = password.len();
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        let criteria_met = [has_lowercase, has_uppercase, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        if length < 8 {
            PasswordStrength::Weak
        } else if length >= 12 && criteria_met >= 3 {
            PasswordStrength::Strong
        } else if length >= 8 && criteria_met >= 2 {
            PasswordStrength::Medium
        } else {
            PasswordStrength::Weak
        }
    }
}

/// Password strength enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum PasswordStrength {
    Weak,
    Medium,
    Strong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authentication_success() {
        let mut auth = FinalverseAuth::new();

        let request = AuthRequest {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            client_info: ClientInfo {
                client_name: "test".to_string(),
                client_version: "1.0.0".to_string(),
                platform: "test".to_string(),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("test-agent".to_string()),
            },
            two_factor_code: None,
        };

        let response = auth.authenticate(&request).await.unwrap();
        assert!(response.success);
        assert!(response.session_id.is_some());
        assert!(response.token.is_some());
    }

    #[tokio::test]
    async fn test_authentication_failure() {
        let mut auth = FinalverseAuth::new();

        let request = AuthRequest {
            username: "nonexistent".to_string(),
            password: "wrongpass".to_string(),
            client_info: ClientInfo {
                client_name: "test".to_string(),
                client_version: "1.0.0".to_string(),
                platform: "test".to_string(),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("test-agent".to_string()),
            },
            two_factor_code: None,
        };

        let response = auth.authenticate(&request).await.unwrap();
        assert!(!response.success);
        assert!(response.session_id.is_none());
        assert!(response.token.is_none());
    }

    #[test]
    fn test_token_validation() {
        let mut auth = FinalverseAuth::new();

        // Test invalid token
        let validation = auth.validate_token("invalid_token");
        assert!(!validation.valid);
        assert!(validation.session.is_none());
    }

    #[test]
    fn test_session_cleanup() {
        let mut auth = FinalverseAuth::new();
        let initial_count = auth.active_session_count();
        let cleaned = auth.cleanup_expired_sessions();
        assert_eq!(cleaned, 0);
        assert_eq!(auth.active_session_count(), initial_count);
    }

    #[test]
    fn test_password_utilities() {
        assert!(utils::is_valid_username("testuser"));
        assert!(!utils::is_valid_username(""));
        assert!(!utils::is_valid_username("us"));

        assert!(utils::is_valid_email("test@example.com"));
        assert!(!utils::is_valid_email("invalid"));

        assert_eq!(utils::check_password_strength("weak"), PasswordStrength::Weak);
        assert_eq!(utils::check_password_strength("StrongP@ss123"), PasswordStrength::Strong);
    }
}