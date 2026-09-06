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

//! Passkey plugin â€” WebAuthn credential storage scaffold.
//! Registration/auth options as JSON placeholders; full webauthn-rs is TODO.
//! Implements storage + list/delete fully.

use crate::{AuthError, context::AuthState, plugin::AuthPlugin};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use time::OffsetDateTime;

/// A stored passkey credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyCredential {
    pub id: String,
    pub user_id: String,
    /// Credential ID (base64url-encoded raw CBOR).
    pub credential_id: String,
    /// Public key (raw COSE key bytes, base64url-encoded).
    pub public_key: String,
    /// Sign count for replay detection.
    pub sign_count: u32,
    pub created_at: OffsetDateTime,
    pub device_name: Option<String>,
}

/// Passkey plugin â€” WebAuthn credential management.
pub struct PasskeyPlugin {
    state: Option<AuthState>,
}

impl PasskeyPlugin {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl Default for PasskeyPlugin {
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

impl AuthPlugin for PasskeyPlugin {
    fn name(&self) -> &'static str {
        "passkey"
    }

    fn on_build(&mut self, state: &AuthState) -> Result<(), AuthError> {
        self.state = Some(state.clone());
        Ok(())
    }

    fn router(&self) -> Router {
        let state = self.state.clone().expect("PasskeyPlugin: state not set");
        Router::new()
            .route("/passkey/register-options", post(register_options))
            .route("/passkey/register", post(register_credential))
            .route("/passkey/auth-options", post(auth_options))
            .route("/passkey/authenticate", post(authenticate))
            .route("/passkey/list", get(list_credentials))
            .route("/passkey/delete", post(delete_credential))
            .with_state(state)
    }
}

async fn register_options(
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

    // TODO: Use webauthn-rs to generate proper registration options.
    // For now, return a placeholder that describes the expected structure.
    Ok(Json(json!({
        "challenge": "TODO-generate-real-challenge",
        "rp": {
            "name": "MontRS",
            "id": url::Url::parse(&state.config.base_url)
                .ok()
                .and_then(|u| u.host_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "localhost".into()),
        },
        "user": {
            "id": session.user_id,
            "name": session.user_id,
            "displayName": session.user_id,
        },
        "pubKeyCredParams": [
            {"type": "public-key", "alg": -7},
            {"type": "public-key", "alg": -257}
        ],
        "timeout": 60000,
        "attestation": "none",
        "note": "Full webauthn-rs integration is TODO. Submit the registration response as JSON to /passkey/register.",
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterCredentialRequest {
    pub credential_id: String,
    pub public_key: String,
    pub sign_count: Option<u32>,
    pub device_name: Option<String>,
}

async fn register_credential(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<RegisterCredentialRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    if req.credential_id.is_empty() || req.public_key.is_empty() {
        return Err(AuthError::missing_field("credentialId or publicKey"));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let credential = PasskeyCredential {
        id: id.clone(),
        user_id: session.user_id.clone(),
        credential_id: req.credential_id,
        public_key: req.public_key,
        sign_count: req.sign_count.unwrap_or(0),
        created_at: OffsetDateTime::now_utc(),
        device_name: req.device_name,
    };

    state
        .db
        .plugin_set("passkey", &id, serde_json::to_value(&credential).unwrap())
        .await
        .map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;

    Ok(Json(json!({ "success": true, "id": id })))
}

async fn auth_options(
    State(state): State<AuthState>,
) -> Result<Json<Value>, AuthError> {
    // TODO: Use webauthn-rs to generate assertion options.
    Ok(Json(json!({
        "challenge": "TODO-generate-real-challenge",
        "timeout": 60000,
        "rpId": url::Url::parse(&state.config.base_url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "localhost".into()),
        "allowCredentials": [],
        "userVerification": "preferred",
        "note": "Full webauthn-rs integration is TODO. Submit the assertion response as JSON to /passkey/authenticate.",
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    pub credential_id: String,
    pub signature: Option<String>,
    pub authenticator_data: Option<String>,
    pub client_data_json: Option<String>,
}

async fn authenticate(
    State(state): State<AuthState>,
    Json(req): Json<AuthenticateRequest>,
) -> Result<Json<Value>, AuthError> {
    // Look up the credential.
    let entries = state.db.plugin_list("passkey").await.map_err(|e| {
        AuthError::new(
            crate::error::AuthErrorCode::InternalError,
            e.to_string(),
        )
    })?;

    let mut found = None;
    for (_, val) in entries {
        if let Ok(cred) = serde_json::from_value::<PasskeyCredential>(val)
            && cred.credential_id == req.credential_id
        {
            found = Some(cred);
            break;
        }
    }

    let credential = found.ok_or_else(|| {
        AuthError::new(
            crate::error::AuthErrorCode::InvalidToken,
            "Passkey not found",
        )
    })?;

    // TODO: Verify signature with webauthn-rs.
    // For now, accept any signature.
    if req.signature.is_none() {
        return Err(AuthError::missing_field("signature"));
    }

    // Create a session for the user.
    let session = state
        .session
        .create(&credential.user_id, state.session_expires_secs())
        .await
        .map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;

    let user = state
        .db
        .find_user_by_id(&credential.user_id)
        .await?
        .ok_or_else(AuthError::user_not_found)?;
    let profile: crate::entities::UserProfile = (&user).into();

    Ok(Json(json!({
        "user": profile,
        "session": crate::session::session_json(&session),
        "token": session.token,
    })))
}

async fn list_credentials(
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

    let entries = state.db.plugin_list("passkey").await.map_err(|e| {
        AuthError::new(
            crate::error::AuthErrorCode::InternalError,
            e.to_string(),
        )
    })?;

    let credentials: Vec<Value> = entries
        .into_iter()
        .filter_map(|(_, v)| {
            let cred: PasskeyCredential = serde_json::from_value(v).ok()?;
            if cred.user_id == session.user_id {
                Some(json!({
                    "id": cred.id,
                    "credentialId": cred.credential_id,
                    "deviceName": cred.device_name,
                    "createdAt": cred.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                }))
            } else {
                None
            }
        })
        .collect();

    Ok(Json(json!({ "credentials": credentials })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCredentialRequest {
    pub id: String,
}

async fn delete_credential(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeleteCredentialRequest>,
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
            .plugin_get("passkey", &req.id)
            .await?
            .ok_or_else(|| {
                AuthError::new(
                    crate::error::AuthErrorCode::InvalidToken,
                    "Passkey not found",
                )
            })?;
    let cred: PasskeyCredential =
        serde_json::from_value(entry).map_err(|_| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                "Invalid passkey record",
            )
        })?;

    if cred.user_id != session.user_id {
        return Err(AuthError::forbidden());
    }

    state.db.plugin_delete("passkey", &req.id).await.ok();
    Ok(Json(json!({ "success": true, "deleted": req.id })))
}
