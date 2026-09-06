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

//! MCP (Model Context Protocol) OAuth plugin â€” thin wrappers similar to oauth_provider.
//! /.well-known/oauth-authorization-server, /mcp/authorize, /mcp/token, /mcp/register.

//TODO: Support the latest update of the MCP standard

use crate::{
    AuthError, context::AuthState, plugin::AuthPlugin, utils::generate_token,
};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use time::OffsetDateTime;

/// An MCP OAuth client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClient {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
    pub name: Option<String>,
    pub created_at: OffsetDateTime,
}

/// MCP OAuth plugin â€” authorization server for MCP clients.
pub struct McpPlugin {
    state: Option<AuthState>,
}

impl McpPlugin {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl Default for McpPlugin {
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

impl AuthPlugin for McpPlugin {
    fn name(&self) -> &'static str {
        "mcp"
    }

    fn on_build(&mut self, state: &AuthState) -> Result<(), AuthError> {
        self.state = Some(state.clone());
        Ok(())
    }

    fn router(&self) -> Router {
        let state = self.state.clone().expect("McpPlugin: state not set");
        Router::new()
            .route(
                "/.well-known/oauth-authorization-server",
                get(authorization_server_metadata),
            )
            .route("/mcp/authorize", get(mcp_authorize))
            .route("/mcp/token", post(mcp_token))
            .route("/mcp/register", post(mcp_register))
            .with_state(state)
    }
}

async fn authorization_server_metadata(
    State(state): State<AuthState>,
) -> Json<Value> {
    let base = state.config.base_url.trim_end_matches('/');
    Json(json!({
        "issuer": base,
        "authorization_endpoint": format!("{base}/api/auth/mcp/authorize"),
        "token_endpoint": format!("{base}/api/auth/mcp/token"),
        "registration_endpoint": format!("{base}/api/auth/mcp/register"),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256", "plain"],
        "token_endpoint_auth_methods_supported": ["client_secret_post", "none"],
    }))
}

#[derive(Debug, Deserialize)]
pub struct McpAuthorizeQuery {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

async fn mcp_authorize(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Query(q): Query<McpAuthorizeQuery>,
) -> Result<Json<Value>, AuthError> {
    let _client = state
        .db
        .plugin_get("mcp_client", &q.client_id)
        .await?
        .ok_or_else(|| {
            AuthError::new(
                crate::error::AuthErrorCode::OAuthError,
                "Unknown client",
            )
        })?;

    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    let code = generate_token();
    let code_data = json!({
        "client_id": q.client_id,
        "user_id": session.user_id,
        "redirect_uri": q.redirect_uri,
        "scope": q.scope.unwrap_or_else(|| "mcp".into()),
        "code_challenge": q.code_challenge,
    });

    let _ = crate::verification::create_verification(
        state.db.as_ref(),
        format!("mcp-code:{code}"),
        Some(serde_json::to_string(&code_data).unwrap_or_default()),
        300,
    )
    .await;

    Ok(Json(json!({
        "code": code,
        "state": q.state,
        "redirect_uri": q.redirect_uri,
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpTokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>,
}

async fn mcp_token(
    State(state): State<AuthState>,
    Json(req): Json<McpTokenRequest>,
) -> Result<Json<Value>, AuthError> {
    if req.grant_type != "authorization_code" {
        return Err(AuthError::new(
            crate::error::AuthErrorCode::OAuthError,
            format!("Unsupported grant_type: {}", req.grant_type),
        ));
    }

    let code = req.code.ok_or_else(|| AuthError::missing_field("code"))?;

    let rec = crate::verification::consume_verification(
        state.db.as_ref(),
        &format!("mcp-code:{code}"),
        &code,
    )
    .await
    .map_err(|_| AuthError::invalid_token());

    let rec = match rec {
        Ok(r) => r,
        Err(_) => {
            // Try listing verifications via consume_by_value with the code as value.
            // Our create_verification stored code_data as value under identifier mcp-code:{code}.
            // find_verification(identifier, value) needs both. Let's try finding by value.
            let all = state
                .db
                .find_verification(&format!("mcp-code:{code}"), &code)
                .await
                .ok()
                .flatten();
            if let Some(r) = all {
                let _ = state.db.delete_verification(&r.id).await;
                r
            } else {
                // Last resort: store code as both identifier suffix and use plugin store.
                return Err(AuthError::invalid_token());
            }
        }
    };

    let code_data: Value = serde_json::from_str(&rec.value).unwrap_or(json!({
        "user_id": rec.value
    }));
    let user_id = code_data
        .get("user_id")
        .and_then(|v| v.as_str())
        .unwrap_or(&rec.value);

    let access_token = generate_token();
    let refresh_token = generate_token();

    let _ = crate::verification::create_verification(
        state.db.as_ref(),
        format!("mcp-at:{access_token}"),
        Some(user_id.to_string()),
        3600,
    )
    .await;

    Ok(Json(json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": 3600,
        "refresh_token": refresh_token,
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpRegisterRequest {
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
    pub token_endpoint_auth_method: Option<String>,
}

async fn mcp_register(
    State(state): State<AuthState>,
    Json(req): Json<McpRegisterRequest>,
) -> Result<Json<Value>, AuthError> {
    if req.redirect_uris.is_empty() {
        return Err(AuthError::missing_field("redirect_uris"));
    }

    let client_id = generate_token();
    let client_secret = generate_token();
    let client = McpClient {
        client_id: client_id.clone(),
        client_secret: client_secret.clone(),
        redirect_uris: req.redirect_uris.clone(),
        name: req.client_name,
        created_at: OffsetDateTime::now_utc(),
    };

    state
        .db
        .plugin_set(
            "mcp_client",
            &client_id,
            serde_json::to_value(&client).unwrap(),
        )
        .await
        .map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;

    Ok(Json(json!({
        "client_id": client_id,
        "client_secret": client_secret,
        "redirect_uris": req.redirect_uris,
        "client_id_issued_at": client.created_at.unix_timestamp(),
    })))
}
