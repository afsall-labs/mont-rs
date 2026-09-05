// Ø¨ÙØ³Ù’Ù…Ù Ø§Ù„Ù„ÙŽÙ‘Ù‡Ù Ø§Ù„Ø±ÙŽÙ‘Ø­Ù’Ù…ÙŽÙ†Ù Ø§Ù„Ø±ÙŽÙ‘Ø­ÙÙŠÙ…
// This file is part of montrs.
// Copyright (C) 2026-Present Afsall Inc.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// Alternatively, this file is available under the MIT License:
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Database adapter trait for the auth system.
//!
//! Backend-agnostic: use the in-memory adapter for development, or
//! implement this trait for PostgreSQL/MySQL/SQLite via montrs-orm.

use crate::entities::{DefaultAccount, DefaultSession, DefaultUser};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use time::OffsetDateTime;

/// A complete user record.
#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: String,
    pub email: String,
    pub email_verified: bool,
    pub password_hash: Option<String>,
    pub name: Option<String>,
    pub image: Option<String>,
    pub username: Option<String>,
    pub phone_number: Option<String>,
    pub phone_verified: bool,
    pub role: Option<String>,
    pub banned: bool,
    pub ban_reason: Option<String>,
    pub two_factor_enabled: bool,
    pub is_anonymous: bool,
    pub last_login_method: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<&DefaultUser> for UserRecord {
    fn from(u: &DefaultUser) -> Self {
        Self {
            id: u.id.clone(),
            email: u.email.clone(),
            email_verified: u.email_verified,
            password_hash: u.password_hash.clone(),
            name: u.name.clone(),
            image: u.image.clone(),
            username: u.username.clone(),
            phone_number: u.phone_number.clone(),
            phone_verified: u.phone_verified,
            role: u.role.clone(),
            banned: u.banned,
            ban_reason: u.ban_reason.clone(),
            two_factor_enabled: u.two_factor_enabled,
            is_anonymous: u.is_anonymous,
            last_login_method: u.last_login_method.clone(),
            metadata: u.metadata.clone(),
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

/// A session record.
#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub impersonated_by: Option<String>,
    pub active_organization_id: Option<String>,
}

impl From<&DefaultSession> for SessionRecord {
    fn from(s: &DefaultSession) -> Self {
        Self {
            id: s.id.clone(),
            user_id: s.user_id.clone(),
            token: s.token.clone(),
            expires_at: s.expires_at,
            created_at: s.created_at,
            ip_address: s.ip_address.clone(),
            user_agent: s.user_agent.clone(),
            impersonated_by: s.impersonated_by.clone(),
            active_organization_id: s.active_organization_id.clone(),
        }
    }
}

/// An account record (OAuth / credential link).
#[derive(Debug, Clone)]
pub struct AccountRecord {
    pub id: String,
    pub user_id: String,
    pub provider_id: String,
    pub provider_account_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub access_token_expires_at: Option<OffsetDateTime>,
    pub password: Option<String>,
}

impl From<&DefaultAccount> for AccountRecord {
    fn from(a: &DefaultAccount) -> Self {
        Self {
            id: a.id.clone(),
            user_id: a.user_id.clone(),
            provider_id: a.provider_id.clone(),
            provider_account_id: a.provider_account_id.clone(),
            access_token: a.access_token.clone(),
            refresh_token: a.refresh_token.clone(),
            id_token: a.id_token.clone(),
            access_token_expires_at: a.access_token_expires_at,
            password: a.password.clone(),
        }
    }
}

/// Verification token / OTP record.
#[derive(Debug, Clone)]
pub struct VerificationRecord {
    pub id: String,
    pub identifier: String,
    pub value: String,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

/// Field updates for a user.
#[derive(Debug, Clone, Default)]
pub struct UserUpdate {
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub password_hash: Option<String>,
    pub name: Option<String>,
    pub image: Option<String>,
    pub username: Option<String>,
    pub phone_number: Option<String>,
    pub phone_verified: Option<bool>,
    pub role: Option<String>,
    pub banned: Option<bool>,
    pub ban_reason: Option<String>,
    pub two_factor_enabled: Option<bool>,
    pub is_anonymous: Option<bool>,
    pub last_login_method: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// The database adapter trait. All methods are async.
#[async_trait]
pub trait DatabaseAdapter: Send + Sync + 'static {
    // â”€â”€ Users â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    async fn create_user(&self, user: &DefaultUser) -> anyhow::Result<()>;
    async fn find_user_by_email(
        &self,
        email: &str,
    ) -> anyhow::Result<Option<UserRecord>>;
    async fn find_user_by_id(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<UserRecord>>;
    async fn find_user_by_username(
        &self,
        username: &str,
    ) -> anyhow::Result<Option<UserRecord>>;
    async fn find_user_by_phone(
        &self,
        phone: &str,
    ) -> anyhow::Result<Option<UserRecord>>;
    async fn list_users(
        &self,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<Vec<UserRecord>>;
    async fn update_user(
        &self,
        id: &str,
        updates: UserUpdate,
    ) -> anyhow::Result<()>;
    async fn delete_user(&self, id: &str) -> anyhow::Result<()>;

    // â”€â”€ Sessions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    async fn create_session(
        &self,
        session: &DefaultSession,
    ) -> anyhow::Result<()>;
    async fn find_session(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<SessionRecord>>;
    async fn find_session_by_token(
        &self,
        token: &str,
    ) -> anyhow::Result<Option<SessionRecord>>;
    async fn list_sessions(
        &self,
        user_id: &str,
    ) -> anyhow::Result<Vec<SessionRecord>>;
    async fn delete_session(&self, id: &str) -> anyhow::Result<()>;
    async fn delete_user_sessions(&self, user_id: &str) -> anyhow::Result<()>;

    // â”€â”€ Accounts â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    async fn create_account(
        &self,
        account: &DefaultAccount,
    ) -> anyhow::Result<()>;
    async fn find_account(
        &self,
        provider_id: &str,
        provider_account_id: &str,
    ) -> anyhow::Result<Option<AccountRecord>>;
    async fn list_accounts(
        &self,
        user_id: &str,
    ) -> anyhow::Result<Vec<AccountRecord>>;
    async fn delete_account(&self, id: &str) -> anyhow::Result<()>;

    // â”€â”€ Verification â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    async fn create_verification(
        &self,
        record: &VerificationRecord,
    ) -> anyhow::Result<()>;
    async fn find_verification(
        &self,
        identifier: &str,
        value: &str,
    ) -> anyhow::Result<Option<VerificationRecord>>;
    async fn find_verification_by_value(
        &self,
        value: &str,
    ) -> anyhow::Result<Option<VerificationRecord>>;
    async fn delete_verification(&self, id: &str) -> anyhow::Result<()>;
    async fn delete_verifications_for(
        &self,
        identifier: &str,
    ) -> anyhow::Result<()>;

    // â”€â”€ Generic plugin KV (orgs, api keys, etc. can use until dedicated tables) â”€â”€
    async fn plugin_set(
        &self,
        namespace: &str,
        key: &str,
        value: serde_json::Value,
    ) -> anyhow::Result<()>;
    async fn plugin_get(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Option<serde_json::Value>>;
    async fn plugin_delete(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<()>;
    async fn plugin_list(
        &self,
        namespace: &str,
    ) -> anyhow::Result<Vec<(String, serde_json::Value)>>;
}

/// An in-memory database adapter for development and testing.
#[derive(Default)]
pub struct MemoryDatabaseAdapter {
    users: Mutex<Vec<DefaultUser>>,
    sessions: Mutex<Vec<DefaultSession>>,
    accounts: Mutex<Vec<DefaultAccount>>,
    verifications: Mutex<Vec<VerificationRecord>>,
    plugin_store: Mutex<HashMap<String, HashMap<String, serde_json::Value>>>,
}

impl MemoryDatabaseAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[async_trait]
impl DatabaseAdapter for MemoryDatabaseAdapter {
    async fn create_user(&self, user: &DefaultUser) -> anyhow::Result<()> {
        self.users.lock().unwrap().push(user.clone());
        Ok(())
    }

    async fn find_user_by_email(
        &self,
        email: &str,
    ) -> anyhow::Result<Option<UserRecord>> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.email.eq_ignore_ascii_case(email))
            .map(UserRecord::from))
    }

    async fn find_user_by_id(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<UserRecord>> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.id == id)
            .map(UserRecord::from))
    }

    async fn find_user_by_username(
        &self,
        username: &str,
    ) -> anyhow::Result<Option<UserRecord>> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| {
                u.username
                    .as_ref()
                    .is_some_and(|n| n.eq_ignore_ascii_case(username))
            })
            .map(UserRecord::from))
    }

    async fn find_user_by_phone(
        &self,
        phone: &str,
    ) -> anyhow::Result<Option<UserRecord>> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.phone_number.as_deref() == Some(phone))
            .map(UserRecord::from))
    }

    async fn list_users(
        &self,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<Vec<UserRecord>> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .skip(offset)
            .take(if limit == 0 { usize::MAX } else { limit })
            .map(UserRecord::from)
            .collect())
    }

    async fn update_user(
        &self,
        id: &str,
        updates: UserUpdate,
    ) -> anyhow::Result<()> {
        let mut users = self.users.lock().unwrap();
        if let Some(u) = users.iter_mut().find(|u| u.id == id) {
            if let Some(v) = updates.email {
                u.email = v;
            }
            if let Some(v) = updates.email_verified {
                u.email_verified = v;
            }
            if let Some(v) = updates.password_hash {
                u.password_hash = Some(v);
            }
            if let Some(v) = updates.name {
                u.name = Some(v);
            }
            if let Some(v) = updates.image {
                u.image = Some(v);
            }
            if let Some(v) = updates.username {
                u.username = Some(v);
            }
            if let Some(v) = updates.phone_number {
                u.phone_number = Some(v);
            }
            if let Some(v) = updates.phone_verified {
                u.phone_verified = v;
            }
            if let Some(v) = updates.role {
                u.role = Some(v);
            }
            if let Some(v) = updates.banned {
                u.banned = v;
            }
            if let Some(v) = updates.ban_reason {
                u.ban_reason = Some(v);
            }
            if let Some(v) = updates.two_factor_enabled {
                u.two_factor_enabled = v;
            }
            if let Some(v) = updates.is_anonymous {
                u.is_anonymous = v;
            }
            if let Some(v) = updates.last_login_method {
                u.last_login_method = Some(v);
            }
            if let Some(v) = updates.metadata {
                u.metadata = v;
            }
            u.updated_at = OffsetDateTime::now_utc();
        }
        Ok(())
    }

    async fn delete_user(&self, id: &str) -> anyhow::Result<()> {
        self.users.lock().unwrap().retain(|u| u.id != id);
        Ok(())
    }

    async fn create_session(
        &self,
        session: &DefaultSession,
    ) -> anyhow::Result<()> {
        self.sessions.lock().unwrap().push(session.clone());
        Ok(())
    }

    async fn find_session(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<SessionRecord>> {
        Ok(self
            .sessions
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.id == id)
            .map(SessionRecord::from))
    }

    async fn find_session_by_token(
        &self,
        token: &str,
    ) -> anyhow::Result<Option<SessionRecord>> {
        Ok(self
            .sessions
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.token == token || s.id == token)
            .map(SessionRecord::from))
    }

    async fn list_sessions(
        &self,
        user_id: &str,
    ) -> anyhow::Result<Vec<SessionRecord>> {
        Ok(self
            .sessions
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.user_id == user_id)
            .map(SessionRecord::from)
            .collect())
    }

    async fn delete_session(&self, id: &str) -> anyhow::Result<()> {
        self.sessions
            .lock()
            .unwrap()
            .retain(|s| s.id != id && s.token != id);
        Ok(())
    }

    async fn delete_user_sessions(&self, user_id: &str) -> anyhow::Result<()> {
        self.sessions
            .lock()
            .unwrap()
            .retain(|s| s.user_id != user_id);
        Ok(())
    }

    async fn create_account(
        &self,
        account: &DefaultAccount,
    ) -> anyhow::Result<()> {
        self.accounts.lock().unwrap().push(account.clone());
        Ok(())
    }

    async fn find_account(
        &self,
        provider_id: &str,
        provider_account_id: &str,
    ) -> anyhow::Result<Option<AccountRecord>> {
        Ok(self
            .accounts
            .lock()
            .unwrap()
            .iter()
            .find(|a| {
                a.provider_id == provider_id
                    && a.provider_account_id == provider_account_id
            })
            .map(AccountRecord::from))
    }

    async fn list_accounts(
        &self,
        user_id: &str,
    ) -> anyhow::Result<Vec<AccountRecord>> {
        Ok(self
            .accounts
            .lock()
            .unwrap()
            .iter()
            .filter(|a| a.user_id == user_id)
            .map(AccountRecord::from)
            .collect())
    }

    async fn delete_account(&self, id: &str) -> anyhow::Result<()> {
        self.accounts.lock().unwrap().retain(|a| a.id != id);
        Ok(())
    }

    async fn create_verification(
        &self,
        record: &VerificationRecord,
    ) -> anyhow::Result<()> {
        self.verifications.lock().unwrap().push(record.clone());
        Ok(())
    }

    async fn find_verification(
        &self,
        identifier: &str,
        value: &str,
    ) -> anyhow::Result<Option<VerificationRecord>> {
        Ok(self
            .verifications
            .lock()
            .unwrap()
            .iter()
            .find(|v| v.identifier == identifier && v.value == value)
            .cloned())
    }

    async fn find_verification_by_value(
        &self,
        value: &str,
    ) -> anyhow::Result<Option<VerificationRecord>> {
        Ok(self
            .verifications
            .lock()
            .unwrap()
            .iter()
            .find(|v| v.value == value)
            .cloned())
    }

    async fn delete_verification(&self, id: &str) -> anyhow::Result<()> {
        self.verifications.lock().unwrap().retain(|v| v.id != id);
        Ok(())
    }

    async fn delete_verifications_for(
        &self,
        identifier: &str,
    ) -> anyhow::Result<()> {
        self.verifications
            .lock()
            .unwrap()
            .retain(|v| v.identifier != identifier);
        Ok(())
    }

    async fn plugin_set(
        &self,
        namespace: &str,
        key: &str,
        value: serde_json::Value,
    ) -> anyhow::Result<()> {
        self.plugin_store
            .lock()
            .unwrap()
            .entry(namespace.to_string())
            .or_default()
            .insert(key.to_string(), value);
        Ok(())
    }

    async fn plugin_get(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Option<serde_json::Value>> {
        Ok(self
            .plugin_store
            .lock()
            .unwrap()
            .get(namespace)
            .and_then(|m| m.get(key).cloned()))
    }

    async fn plugin_delete(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<()> {
        if let Some(m) = self.plugin_store.lock().unwrap().get_mut(namespace) {
            m.remove(key);
        }
        Ok(())
    }

    async fn plugin_list(
        &self,
        namespace: &str,
    ) -> anyhow::Result<Vec<(String, serde_json::Value)>> {
        Ok(self
            .plugin_store
            .lock()
            .unwrap()
            .get(namespace)
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default())
    }
}
