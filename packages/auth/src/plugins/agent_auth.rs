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

//! Agent Auth plugin â€” register and manage agent tokens for programmatic access.
//! /agent/register, /agent/token, /agent/capability â€” store agents in plugin_store.

use crate::{AuthError, context::AuthState, plugin::AuthPlugin};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use time::OffsetDateTime;

/// A registered agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub token_hash: String,
    pub capabilities: Vec<String>,
    pub enabled: bool,
    pub created_at: OffsetDateTime,
    pub last_used_at: Option<OffsetDateTime>,
    pub metadata: HashMap<String, String>,
}

/// Agent Auth plugin â€” manage agents for programmatic access.
pub struct AgentAuthPlugin {
    state: Option<AuthState>,
}

impl AgentAuthPlugin {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl Default for AgentAuthPlugin {
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

impl AuthPlugin for AgentAuthPlugin {
    fn name(&self) -> &'static str {
        "agent_auth"
    }

    fn on_build(&mut self, state: &AuthState) -> Result<(), AuthError> {
        self.state = Some(state.clone());
        Ok(())
    }

    fn router(&self) -> Router {
        let state = self.state.clone().expect("AgentAuthPlugin: state not set");
        Router::new()
            .route("/agent/register", post(register_agent))
            .route("/agent/token", post(get_token))
            .route("/agent/capability", post(check_capability))
            .route("/agent/list", get(list_agents))
            .route("/agent/revoke", post(revoke_agent))
            .with_state(state)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterAgentRequest {
    pub name: String,
    pub capabilities: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
}

async fn register_agent(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<RegisterAgentRequest>,
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

    let id = uuid::Uuid::new_v4().to_string();
    let raw_token = crate::utils::generate_token();
    let token_hash = sha256_hex(&raw_token);

    let agent = Agent {
        id: id.clone(),
        name: req.name,
        user_id: session.user_id.clone(),
        token_hash,
        capabilities: req.capabilities.unwrap_or_else(|| vec!["*".into()]),
        enabled: true,
        created_at: OffsetDateTime::now_utc(),
        last_used_at: None,
        metadata: req.metadata.unwrap_or_default(),
    };

    state
        .db
        .plugin_set("agent", &id, serde_json::to_value(&agent).unwrap())
        .await
        .map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;

    Ok(Json(json!({
        "agentId": id,
        "token": raw_token,
        "message": "Store this token securely. It will not be shown again.",
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTokenRequest {
    pub agent_id: String,
}

async fn get_token(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<GetTokenRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    let entry = state
        .db
        .plugin_get("agent", &req.agent_id)
        .await?
        .ok_or_else(|| {
            AuthError::new(
                crate::error::AuthErrorCode::InvalidToken,
                "Agent not found",
            )
        })?;
    let agent: Agent = serde_json::from_value(entry).map_err(|_| {
        AuthError::new(
            crate::error::AuthErrorCode::InternalError,
            "Invalid agent record",
        )
    })?;

    if agent.user_id != session.user_id {
        return Err(AuthError::forbidden());
    }

    // Generate a new token.
    let raw_token = crate::utils::generate_token();
    let new_hash = sha256_hex(&raw_token);

    let mut updated = agent;
    updated.token_hash = new_hash;
    updated.last_used_at = None;

    state
        .db
        .plugin_set(
            "agent",
            &req.agent_id,
            serde_json::to_value(&updated).unwrap(),
        )
        .await
        .ok();

    Ok(Json(json!({
        "token": raw_token,
        "agentId": req.agent_id,
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityRequest {
    pub capability: String,
    pub agent_token: Option<String>,
}

async fn check_capability(
    State(state): State<AuthState>,
    Json(req): Json<CapabilityRequest>,
) -> Result<Json<Value>, AuthError> {
    // Authenticate via agent token or session.
    let agent = if let Some(at) = &req.agent_token {
        let hash = sha256_hex(at);
        let entries = state.db.plugin_list("agent").await.map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;
        let mut found = None;
        for (_, val) in entries {
            if let Ok(a) = serde_json::from_value::<Agent>(val)
                && a.token_hash == hash
                && a.enabled
            {
                found = Some(a);
                break;
            }
        }
        found.ok_or_else(AuthError::invalid_token)?
    } else {
        return Err(AuthError::missing_field("agentToken"));
    };

    let has_capability = agent
        .capabilities
        .iter()
        .any(|c| c == "*" || c == &req.capability);

    // Update last_used_at.
    let mut updated = agent;
    updated.last_used_at = Some(OffsetDateTime::now_utc());
    state
        .db
        .plugin_set(
            "agent",
            &updated.id,
            serde_json::to_value(&updated).unwrap(),
        )
        .await
        .ok();

    Ok(Json(json!({
        "hasCapability": has_capability,
        "capability": req.capability,
        "agentId": updated.id,
    })))
}

async fn list_agents(
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

    let entries = state.db.plugin_list("agent").await.map_err(|e| {
        AuthError::new(
            crate::error::AuthErrorCode::InternalError,
            e.to_string(),
        )
    })?;

    let agents: Vec<Value> = entries
        .into_iter()
        .filter_map(|(_, v)| {
            let a: Agent = serde_json::from_value(v).ok()?;
            if a.user_id == session.user_id {
                Some(json!({
                    "id": a.id,
                    "name": a.name,
                    "capabilities": a.capabilities,
                    "enabled": a.enabled,
                    "createdAt": a.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                    "lastUsedAt": a.last_used_at.map(|d| d.format(&time::format_description::well_known::Rfc3339).unwrap()),
                    "metadata": a.metadata,
                }))
            } else {
                None
            }
        })
        .collect();

    Ok(Json(json!({ "agents": agents })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevokeAgentRequest {
    pub agent_id: String,
}

async fn revoke_agent(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<RevokeAgentRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    let entry = state
        .db
        .plugin_get("agent", &req.agent_id)
        .await?
        .ok_or_else(|| {
            AuthError::new(
                crate::error::AuthErrorCode::InvalidToken,
                "Agent not found",
            )
        })?;
    let agent: Agent = serde_json::from_value(entry).map_err(|_| {
        AuthError::new(
            crate::error::AuthErrorCode::InternalError,
            "Invalid agent record",
        )
    })?;

    if agent.user_id != session.user_id {
        return Err(AuthError::forbidden());
    }

    state.db.plugin_delete("agent", &req.agent_id).await.ok();

    Ok(Json(json!({ "success": true, "revoked": req.agent_id })))
}

fn sha256_hex(input: &str) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
