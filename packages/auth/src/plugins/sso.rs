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

//! SSO plugin â€” OIDC IdP config store + SAML config storage + ACS stub.
//! /sso/register, /sso/providers, /sign-in/sso.

use crate::{
    AuthError,
    context::AuthState,
    entities::{DefaultUser, UserProfile},
    plugin::AuthPlugin,
};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use time::OffsetDateTime;

/// An SSO identity provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoProvider {
    pub id: String,
    pub name: String,
    pub provider_type: String, // "oidc" or "saml"
    pub issuer: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub authorization_url: Option<String>,
    pub token_url: Option<String>,
    pub userinfo_url: Option<String>,
    pub jwks_url: Option<String>,
    /// SAML-specific fields.
    pub saml_entry_point: Option<String>,
    pub saml_cert: Option<String>,
    pub saml_acs_url: Option<String>,
    pub domains: Vec<String>,
    pub enabled: bool,
    pub created_at: OffsetDateTime,
}

/// SSO plugin.
pub struct SsoPlugin {
    state: Option<AuthState>,
}

impl SsoPlugin {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl Default for SsoPlugin {
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

impl AuthPlugin for SsoPlugin {
    fn name(&self) -> &'static str {
        "sso"
    }

    fn on_build(&mut self, state: &AuthState) -> Result<(), AuthError> {
        self.state = Some(state.clone());
        Ok(())
    }

    fn router(&self) -> Router {
        let state = self.state.clone().expect("SsoPlugin: state not set");
        Router::new()
            .route("/sso/register", post(register_provider))
            .route("/sso/providers", get(list_providers))
            .route("/sign-in/sso", post(sign_in_sso))
            .route("/sso/saml/acs", post(saml_acs))
            .with_state(state)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterSsoRequest {
    pub name: String,
    pub provider_type: String,
    pub issuer: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub authorization_url: Option<String>,
    pub token_url: Option<String>,
    pub userinfo_url: Option<String>,
    pub jwks_url: Option<String>,
    pub saml_entry_point: Option<String>,
    pub saml_cert: Option<String>,
    pub domains: Option<Vec<String>>,
}

async fn register_provider(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<RegisterSsoRequest>,
) -> Result<Json<Value>, AuthError> {
    // Require admin session.
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let user = state
        .session
        .get_user(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;
    if user.role.as_deref() != Some("admin") {
        return Err(AuthError::forbidden());
    }

    if req.name.is_empty() {
        return Err(AuthError::missing_field("name"));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let provider = SsoProvider {
        id: id.clone(),
        name: req.name,
        provider_type: req.provider_type,
        issuer: req.issuer,
        client_id: req.client_id,
        client_secret: req.client_secret,
        authorization_url: req.authorization_url,
        token_url: req.token_url,
        userinfo_url: req.userinfo_url,
        jwks_url: req.jwks_url,
        saml_entry_point: req.saml_entry_point,
        saml_cert: req.saml_cert,
        saml_acs_url: Some(format!(
            "{}/api/auth/sso/saml/acs",
            state.config.base_url.trim_end_matches('/')
        )),
        domains: req.domains.unwrap_or_default(),
        enabled: true,
        created_at: OffsetDateTime::now_utc(),
    };

    state
        .db
        .plugin_set("sso", &id, serde_json::to_value(&provider).unwrap())
        .await
        .map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;

    Ok(Json(json!({ "provider": provider })))
}

async fn list_providers(
    State(state): State<AuthState>,
) -> Result<Json<Value>, AuthError> {
    let entries = state.db.plugin_list("sso").await.map_err(|e| {
        AuthError::new(
            crate::error::AuthErrorCode::InternalError,
            e.to_string(),
        )
    })?;

    let providers: Vec<Value> = entries
        .into_iter()
        .filter_map(|(_, v)| {
            let p: SsoProvider = serde_json::from_value(v).ok()?;
            // Don't expose secrets.
            Some(json!({
                "id": p.id,
                "name": p.name,
                "providerType": p.provider_type,
                "domains": p.domains,
                "enabled": p.enabled,
            }))
        })
        .collect();

    Ok(Json(json!({ "providers": providers })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInSsoRequest {
    pub provider_id: Option<String>,
    pub email: Option<String>,
    pub domain: Option<String>,
}

async fn sign_in_sso(
    State(state): State<AuthState>,
    Json(req): Json<SignInSsoRequest>,
) -> Result<Json<Value>, AuthError> {
    // Resolve provider by id or domain.
    let provider = if let Some(id) = &req.provider_id {
        let entry = state.db.plugin_get("sso", id).await?.ok_or_else(|| {
            AuthError::new(
                crate::error::AuthErrorCode::ProviderNotConfigured,
                "SSO provider not found",
            )
        })?;
        serde_json::from_value::<SsoProvider>(entry).map_err(|_| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                "Invalid SSO provider",
            )
        })?
    } else if let Some(domain) = &req.domain {
        let entries = state.db.plugin_list("sso").await.map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;
        let mut found = None;
        for (_, v) in entries {
            if let Ok(p) = serde_json::from_value::<SsoProvider>(v)
                && p.domains.iter().any(|d| d == domain)
                && p.enabled
            {
                found = Some(p);
                break;
            }
        }
        found.ok_or_else(|| {
            AuthError::new(
                crate::error::AuthErrorCode::ProviderNotConfigured,
                "No SSO provider for domain",
            )
        })?
    } else if let Some(email) = &req.email {
        let domain = email.split('@').nth(1).unwrap_or("");
        let entries = state.db.plugin_list("sso").await.map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;
        let mut found = None;
        for (_, v) in entries {
            if let Ok(p) = serde_json::from_value::<SsoProvider>(v)
                && p.domains.iter().any(|d| d == domain)
                && p.enabled
            {
                found = Some(p);
                break;
            }
        }
        found.ok_or_else(|| {
            AuthError::new(
                crate::error::AuthErrorCode::ProviderNotConfigured,
                "No SSO provider for email domain",
            )
        })?
    } else {
        return Err(AuthError::missing_field("providerId, email, or domain"));
    };

    match provider.provider_type.as_str() {
        "oidc" => {
            let auth_url =
                provider.authorization_url.as_ref().ok_or_else(|| {
                    AuthError::new(
                        crate::error::AuthErrorCode::ProviderNotConfigured,
                        "Missing authorization_url",
                    )
                })?;
            let client_id = provider.client_id.as_ref().ok_or_else(|| {
                AuthError::new(
                    crate::error::AuthErrorCode::ProviderNotConfigured,
                    "Missing client_id",
                )
            })?;
            let state_param = crate::utils::generate_token();
            let redirect = format!(
                "{}/api/auth/sso/callback/{}",
                state.config.base_url.trim_end_matches('/'),
                provider.id
            );
            let url = format!(
                "{auth_url}?client_id={client_id}&redirect_uri={}&\
                 response_type=code&scope=openid+email+profile&\
                 state={state_param}",
                urlencoding_encode(&redirect),
            );

            let _ = crate::verification::create_verification(
                state.db.as_ref(),
                format!("sso-state:{}", provider.id),
                Some(state_param.clone()),
                600,
            )
            .await;

            Ok(Json(json!({
                "url": url,
                "state": state_param,
                "providerId": provider.id,
            })))
        }
        "saml" => {
            let entry_point =
                provider.saml_entry_point.as_ref().ok_or_else(|| {
                    AuthError::new(
                        crate::error::AuthErrorCode::ProviderNotConfigured,
                        "Missing SAML entry point",
                    )
                })?;
            Ok(Json(json!({
                "url": entry_point,
                "providerId": provider.id,
                "providerType": "saml",
                "acsUrl": provider.saml_acs_url,
            })))
        }
        _ => Err(AuthError::new(
            crate::error::AuthErrorCode::ProviderNotConfigured,
            format!("Unknown provider type: {}", provider.provider_type),
        )),
    }
}

/// SAML Assertion Consumer Service stub.
async fn saml_acs(
    State(state): State<AuthState>,
    body: String,
) -> Result<Json<Value>, AuthError> {
    // TODO: Full SAML response parsing and signature verification.
    // For now, extract a NameID-like email from the raw body if present.
    let email = body
        .lines()
        .find_map(|line| {
            if line.contains("@") && line.contains("NameID") {
                // Crude extraction.
                let start = line.find('>').map(|i| i + 1)?;
                let end = line.rfind('<')?;
                Some(line[start..end].to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| format!("saml-{}@sso.local", uuid::Uuid::new_v4()));

    let user = match state.db.find_user_by_email(&email).await? {
        Some(u) => u,
        None => {
            let mut nu = DefaultUser::new(&email, None);
            nu.email_verified = true;
            state.db.create_user(&nu).await.map_err(|e| {
                AuthError::new(
                    crate::error::AuthErrorCode::InternalError,
                    e.to_string(),
                )
            })?;
            state.db.find_user_by_email(&email).await?.ok_or_else(|| {
                AuthError::new(
                    crate::error::AuthErrorCode::InternalError,
                    "Failed to create user",
                )
            })?
        }
    };

    state
        .db
        .update_user(
            &user.id,
            crate::database::UserUpdate {
                last_login_method: Some("sso-saml".into()),
                ..Default::default()
            },
        )
        .await?;

    let session = state
        .session
        .create(&user.id, state.session_expires_secs())
        .await
        .map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::InternalError,
                e.to_string(),
            )
        })?;

    let profile: UserProfile = (&user).into();
    Ok(Json(json!({
        "user": profile,
        "session": crate::session::session_json(&session),
        "token": session.token,
    })))
}

fn urlencoding_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
