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

//! API Key plugin â€” CRUD for API keys with rate limit metadata.
//! /api-key/create, /api-key/list, /api-key/delete, /api-key/update.
//! Keys are hashed with SHA-256 before storage.

use crate::{AuthError, context::AuthState, plugin::AuthPlugin};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

/// An API key record stored in plugin_store namespace "apikey".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub id: String,
    pub user_id: String,
    pub name: String,
    /// SHA-256 hash of the raw key.
    pub key_hash: String,
    /// The raw key prefix (first 8 chars) for display.
    pub prefix: String,
    pub created_at: OffsetDateTime,
    pub expires_at: Option<OffsetDateTime>,
    pub rate_limit_max: Option<u32>,
    pub rate_limit_window_secs: Option<u64>,
    pub enabled: bool,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Hash an API key with SHA-256.
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a new API key (prefix + random).
pub fn generate_api_key() -> (String, String) {
    let raw = format!("mrs_{}", crate::utils::generate_token());
    let hash = hash_api_key(&raw);
    (raw, hash)
}

/// Verify an API key against stored hashes.
pub async fn verify_api_key(
    db: &dyn crate::database::DatabaseAdapter,
    key: &str,
) -> anyhow::Result<Option<ApiKeyRecord>> {
    let hash = hash_api_key(key);
    let entries = db.plugin_list("apikey").await?;
    for (_, val) in entries {
        let record: ApiKeyRecord = serde_json::from_value(val)?;
        if record.key_hash == hash && record.enabled {
            if let Some(exp) = &record.expires_at
                && *exp <= OffsetDateTime::now_utc()
            {
                continue;
            }
            return Ok(Some(record));
        }
    }
    Ok(None)
}

/// API Key plugin.
pub struct ApiKeyPlugin {
    state: Option<AuthState>,
}

impl ApiKeyPlugin {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl Default for ApiKeyPlugin {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    if let Some(v) = headers.get("authorization").and_then(|v| v.to_str().ok())
        && let Some(t) = v.strip_prefix("Bearer ")
    {
        return Some(t.to_string());
    }
    if let Some(v) = headers.get("cookie").and_then(|v| v.to_str().ok()) {
        for part in v.split(';') {
            let part = part.trim();
            if let Some(t) = part.strip_prefix("session=") {
                return Some(t.to_string());
            }
            if let Some(t) = part.strip_prefix("__montrs_session=") {
                return Some(t.to_string());
            }
        }
    }
    None
}

impl AuthPlugin for ApiKeyPlugin {
    fn name(&self) -> &'static str {
        "api_key"
    }

    fn on_build(&mut self, state: &AuthState) -> Result<(), AuthError> {
        self.state = Some(state.clone());
        Ok(())
    }

    fn router(&self) -> Router {
        let state = self.state.clone().expect("ApiKeyPlugin: state not set");
        Router::new()
            .route("/api-key/create", post(create_api_key))
            .route("/api-key/list", get(list_api_keys))
            .route("/api-key/delete", post(delete_api_key))
            .route("/api-key/update", post(update_api_key))
            .with_state(state)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub expires_in_secs: Option<i64>,
    pub rate_limit_max: Option<u32>,
    pub rate_limit_window_secs: Option<u64>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

async fn create_api_key(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    if req.name.is_empty() {
        return Err(AuthError::missing_field("name"));
    }

    let (raw, hash) = generate_api_key();
    let id = uuid::Uuid::new_v4().to_string();
    let record = ApiKeyRecord {
        id: id.clone(),
        user_id: session.user_id.clone(),
        name: req.name,
        key_hash: hash,
        prefix: raw[..8].to_string(),
        created_at: OffsetDateTime::now_utc(),
        expires_at: req
            .expires_in_secs
            .map(|s| OffsetDateTime::now_utc() + time::Duration::seconds(s)),
        rate_limit_max: req.rate_limit_max,
        rate_limit_window_secs: req.rate_limit_window_secs,
        enabled: true,
        metadata: req.metadata.unwrap_or_default(),
    };

    state
        .db
        .plugin_set("apikey", &id, serde_json::to_value(&record).unwrap())
        .await
        .map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;

    Ok(Json(json!({
        "id": id,
        "key": raw,
        "prefix": record.prefix,
        "createdAt": record.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
        "message": "Store this key securely. It will not be shown again.",
    })))
}

async fn list_api_keys(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    let entries = state.db.plugin_list("apikey").await.map_err(|e| {
        AuthError::new(
            crate::error::AuthErrorCode::InternalError,
            e.to_string(),
        )
    })?;

    let keys: Vec<Value> = entries
        .into_iter()
        .filter_map(|(_, v)| {
            let record: ApiKeyRecord = serde_json::from_value(v).ok()?;
            if record.user_id == session.user_id {
                Some(json!({
                    "id": record.id,
                    "name": record.name,
                    "prefix": record.prefix,
                    "createdAt": record.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                    "expiresAt": record.expires_at.map(|d| d.format(&time::format_description::well_known::Rfc3339).unwrap()),
                    "enabled": record.enabled,
                    "metadata": record.metadata,
                }))
            } else {
                None
            }
        })
        .collect();

    Ok(Json(json!({ "keys": keys })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteApiKeyRequest {
    pub id: String,
}

async fn delete_api_key(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeleteApiKeyRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    let entry =
        state
            .db
            .plugin_get("apikey", &req.id)
            .await?
            .ok_or_else(|| {
                AuthError::new(
                    crate::error::AuthErrorCode::InvalidToken,
                    "API key not found",
                )
            })?;
    let record: ApiKeyRecord = serde_json::from_value(entry).map_err(|_| {
        AuthError::new(
            crate::error::AuthErrorCode::InternalError,
            "Invalid API key record",
        )
    })?;

    if record.user_id != session.user_id {
        return Err(AuthError::forbidden());
    }

    state.db.plugin_delete("apikey", &req.id).await.ok();
    Ok(Json(json!({ "success": true, "deleted": req.id })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApiKeyRequest {
    pub id: String,
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub rate_limit_max: Option<u32>,
    pub rate_limit_window_secs: Option<u64>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

async fn update_api_key(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateApiKeyRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    let entry =
        state
            .db
            .plugin_get("apikey", &req.id)
            .await?
            .ok_or_else(|| {
                AuthError::new(
                    crate::error::AuthErrorCode::InvalidToken,
                    "API key not found",
                )
            })?;
    let mut record: ApiKeyRecord =
        serde_json::from_value(entry).map_err(|_| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                "Invalid API key record",
            )
        })?;

    if record.user_id != session.user_id {
        return Err(AuthError::forbidden());
    }

    if let Some(name) = req.name {
        record.name = name;
    }
    if let Some(enabled) = req.enabled {
        record.enabled = enabled;
    }
    if let Some(rl) = req.rate_limit_max {
        record.rate_limit_max = Some(rl);
    }
    if let Some(rlw) = req.rate_limit_window_secs {
        record.rate_limit_window_secs = Some(rlw);
    }
    if let Some(meta) = req.metadata {
        record.metadata = meta;
    }

    state
        .db
        .plugin_set("apikey", &req.id, serde_json::to_value(&record).unwrap())
        .await
        .map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;

    Ok(Json(json!({ "success": true, "id": req.id })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_api_key() {
        let hash = hash_api_key("mrs_test-key-123");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_key_generation() {
        let (raw, hash) = generate_api_key();
        assert!(raw.starts_with("mrs_"));
        assert_eq!(hash, hash_api_key(&raw));
        assert_eq!(raw.len(), 47); // mrs_ + 43 chars
    }
}
