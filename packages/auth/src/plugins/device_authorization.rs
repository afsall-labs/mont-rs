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

//! Device Authorization plugin â€” RFC 8628 Device Flow.
//! /device/code, /device/token, /device/approve, /device/deny.
//! Uses plugin_store namespace "device".

use crate::{
    AuthError, context::AuthState, plugin::AuthPlugin, utils::generate_token,
};
use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use time::OffsetDateTime;

/// A device authorization request (RFC 8628).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRequest {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_at: OffsetDateTime,
    pub interval: u64,
    pub status: DeviceStatus,
    pub user_id: Option<String>,
    pub client_id: Option<String>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

/// Device Authorization plugin (RFC 8628).
pub struct DeviceAuthorizationPlugin {
    state: Option<AuthState>,
}

impl DeviceAuthorizationPlugin {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl Default for DeviceAuthorizationPlugin {
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

impl AuthPlugin for DeviceAuthorizationPlugin {
    fn name(&self) -> &'static str {
        "device_authorization"
    }

    fn on_build(&mut self, state: &AuthState) -> Result<(), AuthError> {
        self.state = Some(state.clone());
        Ok(())
    }

    fn router(&self) -> Router {
        let state = self
            .state
            .clone()
            .expect("DeviceAuthorizationPlugin: state not set");
        Router::new()
            .route("/device/code", post(device_code))
            .route("/device/token", post(device_token))
            .route("/device/approve", post(device_approve))
            .route("/device/deny", post(device_deny))
            .with_state(state)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeRequest {
    pub client_id: Option<String>,
    pub scopes: Option<Vec<String>>,
}

async fn device_code(
    State(state): State<AuthState>,
    Json(req): Json<DeviceCodeRequest>,
) -> Result<Json<Value>, AuthError> {
    let device_code = generate_token();
    let user_code = generate_user_code();
    let base_url = state.config.base_url.trim_end_matches('/').to_string();

    let device_req = DeviceRequest {
        device_code: device_code.clone(),
        user_code: user_code.clone(),
        verification_uri: format!("{base_url}/api/auth/device/approve"),
        verification_uri_complete: format!(
            "{base_url}/api/auth/device/approve?code={user_code}"
        ),
        expires_at: OffsetDateTime::now_utc() + time::Duration::seconds(600),
        interval: 5,
        status: DeviceStatus::Pending,
        user_id: None,
        client_id: req.client_id,
        scopes: req.scopes.unwrap_or_default(),
    };

    state
        .db
        .plugin_set(
            "device",
            &device_code,
            serde_json::to_value(&device_req).unwrap(),
        )
        .await
        .map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;

    Ok(Json(json!({
        "device_code": device_code,
        "user_code": user_code,
        "verification_uri": device_req.verification_uri,
        "verification_uri_complete": device_req.verification_uri_complete,
        "expires_in": 600,
        "interval": 5,
    })))
}

fn generate_user_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    (0..8)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTokenRequest {
    pub device_code: String,
    pub grant_type: Option<String>,
}

async fn device_token(
    State(state): State<AuthState>,
    Json(req): Json<DeviceTokenRequest>,
) -> Result<Json<Value>, AuthError> {
    let entry = state
        .db
        .plugin_get("device", &req.device_code)
        .await?
        .ok_or_else(|| {
            AuthError::new(
                crate::error::AuthErrorCode::InvalidToken,
                "Invalid device code",
            )
        })?;
    let device_req: DeviceRequest =
        serde_json::from_value(entry).map_err(|_| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                "Invalid device record",
            )
        })?;

    if device_req.expires_at <= OffsetDateTime::now_utc() {
        state
            .db
            .plugin_delete("device", &req.device_code)
            .await
            .ok();
        return Err(AuthError::invalid_token());
    }

    match device_req.status {
        DeviceStatus::Pending => {
            return Err(AuthError::new(
                crate::error::AuthErrorCode::InvalidToken,
                "authorization_pending",
            ));
        }
        DeviceStatus::Denied => {
            state
                .db
                .plugin_delete("device", &req.device_code)
                .await
                .ok();
            return Err(AuthError::new(
                crate::error::AuthErrorCode::Forbidden,
                "access_denied",
            ));
        }
        DeviceStatus::Expired => {
            state
                .db
                .plugin_delete("device", &req.device_code)
                .await
                .ok();
            return Err(AuthError::invalid_token());
        }
        DeviceStatus::Approved => {}
    }

    let user_id = device_req
        .user_id
        .clone()
        .ok_or_else(AuthError::user_not_found)?;
    let session = state
        .session
        .create(&user_id, state.session_expires_secs())
        .await
        .map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;

    state
        .db
        .plugin_delete("device", &req.device_code)
        .await
        .ok();

    Ok(Json(json!({
        "access_token": session.token,
        "token_type": "Bearer",
        "expires_in": state.session_expires_secs(),
        "session_id": session.id,
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceApproveRequest {
    pub code: Option<String>,
    pub device_code: Option<String>,
}

async fn device_approve(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeviceApproveRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    // Find the device request by user_code or device_code.
    let entries = state.db.plugin_list("device").await.map_err(|e| {
        AuthError::new(
            crate::error::AuthErrorCode::InternalError,
            e.to_string(),
        )
    })?;

    let mut found_key = None;
    for (key, val) in &entries {
        if let Ok(dr) = serde_json::from_value::<DeviceRequest>(val.clone())
            && dr.status == DeviceStatus::Pending
        {
            if let Some(code) = &req.code
                && dr.user_code == *code
            {
                found_key = Some(key.clone());
                break;
            }
            if let Some(dc) = &req.device_code
                && dr.device_code == *dc
            {
                found_key = Some(key.clone());
                break;
            }
        }
    }

    let key = found_key.ok_or_else(|| {
        AuthError::new(
            crate::error::AuthErrorCode::InvalidToken,
            "No pending device request found",
        )
    })?;

    let mut device_req: DeviceRequest = serde_json::from_value(
        entries
            .into_iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v)
            .unwrap(),
    )
    .unwrap();

    device_req.status = DeviceStatus::Approved;
    device_req.user_id = Some(session.user_id.clone());

    state
        .db
        .plugin_set("device", &key, serde_json::to_value(&device_req).unwrap())
        .await
        .ok();

    Ok(Json(json!({ "success": true, "approved": true })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDenyRequest {
    pub code: Option<String>,
    pub device_code: Option<String>,
}

async fn device_deny(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeviceDenyRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let _session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    let entries = state.db.plugin_list("device").await.map_err(|e| {
        AuthError::new(
            crate::error::AuthErrorCode::InternalError,
            e.to_string(),
        )
    })?;

    let mut found_key = None;
    for (key, val) in &entries {
        if let Ok(dr) = serde_json::from_value::<DeviceRequest>(val.clone())
            && dr.status == DeviceStatus::Pending
        {
            if let Some(code) = &req.code
                && dr.user_code == *code
            {
                found_key = Some(key.clone());
                break;
            }
            if let Some(dc) = &req.device_code
                && dr.device_code == *dc
            {
                found_key = Some(key.clone());
                break;
            }
        }
    }

    let key = found_key.ok_or_else(|| {
        AuthError::new(
            crate::error::AuthErrorCode::InvalidToken,
            "No pending device request found",
        )
    })?;

    let mut device_req: DeviceRequest = serde_json::from_value(
        entries
            .into_iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v)
            .unwrap(),
    )
    .unwrap();

    device_req.status = DeviceStatus::Denied;

    state
        .db
        .plugin_set("device", &key, serde_json::to_value(&device_req).unwrap())
        .await
        .ok();

    Ok(Json(json!({ "success": true, "denied": true })))
}
