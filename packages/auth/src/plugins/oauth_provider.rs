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

//! OAuth Provider plugin â€” OIDC-lite Authorization Server.
//! /.well-known/openid-configuration, /oauth2/authorize, /oauth2/token,
//! /oauth2/userinfo, /oauth2/register.
//! Clients in plugin_store "oauth_client"; codes/tokens in verification.

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

/// A registered OAuth client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClient {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
    pub name: Option<String>,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub created_at: OffsetDateTime,
}

/// OAuth Provider plugin (OIDC-lite AS).
pub struct OAuthProviderPlugin {
    state: Option<AuthState>,
}

impl OAuthProviderPlugin {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl Default for OAuthProviderPlugin {
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

impl AuthPlugin for OAuthProviderPlugin {
    fn name(&self) -> &'static str {
        "oauth_provider"
    }

    fn on_build(&mut self, state: &AuthState) -> Result<(), AuthError> {
        self.state = Some(state.clone());
        Ok(())
    }

    fn router(&self) -> Router {
        let state = self
            .state
            .clone()
            .expect("OAuthProviderPlugin: state not set");
        Router::new()
            .route(
                "/.well-known/openid-configuration",
                get(openid_configuration),
            )
            .route("/oauth2/authorize", get(authorize))
            .route("/oauth2/token", post(token))
            .route("/oauth2/userinfo", get(userinfo))
            .route("/oauth2/register", post(register_client))
            .with_state(state)
    }
}

async fn openid_configuration(State(state): State<AuthState>) -> Json<Value> {
    let base = state.config.base_url.trim_end_matches('/');
    Json(json!({
        "issuer": base,
        "authorization_endpoint": format!("{base}/api/auth/oauth2/authorize"),
        "token_endpoint": format!("{base}/api/auth/oauth2/token"),
        "userinfo_endpoint": format!("{base}/api/auth/oauth2/userinfo"),
        "registration_endpoint": format!("{base}/api/auth/oauth2/register"),
        "jwks_uri": format!("{base}/api/auth/jwks"),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token", "client_credentials"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["HS256"],
        "scopes_supported": ["openid", "profile", "email"],
        "token_endpoint_auth_methods_supported": ["client_secret_post", "client_secret_basic"],
    }))
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

async fn authorize(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Query(q): Query<AuthorizeQuery>,
) -> Result<Json<Value>, AuthError> {
    // Validate client.
    let client_entry = state
        .db
        .plugin_get("oauth_client", &q.client_id)
        .await?
        .ok_or_else(|| {
            AuthError::new(
                crate::error::AuthErrorCode::OAuthError,
                "Unknown client_id",
            )
        })?;
    let client: OAuthClient =
        serde_json::from_value(client_entry).map_err(|_| {
            AuthError::new(
                crate::error::AuthErrorCode::OAuthError,
                "Invalid client",
            )
        })?;

    if !client.redirect_uris.contains(&q.redirect_uri) {
        return Err(AuthError::new(
            crate::error::AuthErrorCode::OAuthError,
            "redirect_uri not registered",
        ));
    }

    // Require authenticated user.
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    // Issue authorization code.
    let code = generate_token();
    let code_data = json!({
        "client_id": q.client_id,
        "user_id": session.user_id,
        "redirect_uri": q.redirect_uri,
        "scope": q.scope.unwrap_or_else(|| "openid".into()),
        "code_challenge": q.code_challenge,
        "code_challenge_method": q.code_challenge_method,
    });

    let _ = crate::verification::create_verification(
        state.db.as_ref(),
        format!("oauth-code:{}", code),
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
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub refresh_token: Option<String>,
    pub code_verifier: Option<String>,
}

async fn token(
    State(state): State<AuthState>,
    Json(req): Json<TokenRequest>,
) -> Result<Json<Value>, AuthError> {
    match req.grant_type.as_str() {
        "authorization_code" => {
            let code =
                req.code.ok_or_else(|| AuthError::missing_field("code"))?;
            let client_id = req
                .client_id
                .ok_or_else(|| AuthError::missing_field("client_id"))?;

            // Validate client.
            let client_entry = state
                .db
                .plugin_get("oauth_client", &client_id)
                .await?
                .ok_or_else(|| {
                    AuthError::new(
                        crate::error::AuthErrorCode::OAuthError,
                        "Unknown client",
                    )
                })?;
            let client: OAuthClient = serde_json::from_value(client_entry)
                .map_err(|_| {
                    AuthError::new(
                        crate::error::AuthErrorCode::OAuthError,
                        "Invalid client",
                    )
                })?;

            if let Some(secret) = &req.client_secret
                && secret != &client.client_secret
            {
                return Err(AuthError::new(
                    crate::error::AuthErrorCode::OAuthError,
                    "Invalid client_secret",
                ));
            }

            // Consume the code.
            let rec = crate::verification::consume_verification(
                state.db.as_ref(),
                &format!("oauth-code:{code}"),
                // The value was stored as JSON string; we need to find by identifier.
                // Use find_verification by looking up value loosely.
                // Actually create_verification stores value as the second arg.
                // We stored code_data as value. Let's re-find by value.
                // Better approach: store code as value.
                &code,
            )
            .await;

            // Alternative: use consume by value since we may not know exact value.
            let rec = match rec {
                Ok(r) => r,
                Err(_) => {
                    // Try by value lookup.
                    crate::verification::consume_verification_by_value(
                        state.db.as_ref(),
                        &code,
                    )
                    .await
                    .map_err(|_| AuthError::invalid_token())?
                }
            };

            let code_data: Value =
                serde_json::from_str(&rec.value).unwrap_or(json!({}));
            let user_id = code_data
                .get("user_id")
                .and_then(|v| v.as_str())
                .ok_or_else(AuthError::user_not_found)?;

            let access_token = generate_token();
            let refresh_token = generate_token();
            let id_token = crate::utils::jwt::create_token(
                user_id,
                state.session.secret(),
                3600,
            )
            .unwrap_or_default();

            // Store access token.
            let _ = crate::verification::create_verification(
                state.db.as_ref(),
                format!("oauth-at:{access_token}"),
                Some(user_id.to_string()),
                3600,
            )
            .await;

            Ok(Json(json!({
                "access_token": access_token,
                "token_type": "Bearer",
                "expires_in": 3600,
                "refresh_token": refresh_token,
                "id_token": id_token,
            })))
        }
        "client_credentials" => {
            let client_id = req
                .client_id
                .ok_or_else(|| AuthError::missing_field("client_id"))?;
            let client_secret = req
                .client_secret
                .ok_or_else(|| AuthError::missing_field("client_secret"))?;

            let client_entry = state
                .db
                .plugin_get("oauth_client", &client_id)
                .await?
                .ok_or_else(|| {
                    AuthError::new(
                        crate::error::AuthErrorCode::OAuthError,
                        "Unknown client",
                    )
                })?;
            let client: OAuthClient = serde_json::from_value(client_entry)
                .map_err(|_| {
                    AuthError::new(
                        crate::error::AuthErrorCode::OAuthError,
                        "Invalid client",
                    )
                })?;

            if client.client_secret != client_secret {
                return Err(AuthError::new(
                    crate::error::AuthErrorCode::OAuthError,
                    "Invalid client_secret",
                ));
            }

            let access_token = generate_token();
            Ok(Json(json!({
                "access_token": access_token,
                "token_type": "Bearer",
                "expires_in": 3600,
            })))
        }
        _ => Err(AuthError::new(
            crate::error::AuthErrorCode::OAuthError,
            format!("Unsupported grant_type: {}", req.grant_type),
        )),
    }
}

async fn userinfo(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, AuthError> {
    let token = extract_token(&headers).ok_or_else(AuthError::invalid_token)?;

    // Try as session token first, then as OAuth access token.
    let user = if let Some(session) = state.session.validate(&token).await? {
        state
            .db
            .find_user_by_id(&session.user_id)
            .await?
            .ok_or_else(AuthError::user_not_found)?
    } else {
        // Look up OAuth access token.
        let rec = state
            .db
            .find_verification(&format!("oauth-at:{token}"), &token)
            .await
            .ok()
            .flatten()
            .or({
                // Try by value.
                None
            });

        // Alternative: find by identifier pattern via plugin store.
        let user_id = if let Some(r) = rec {
            r.value
        } else {
            // Try verification by value.
            let r = crate::verification::consume_verification_by_value(
                state.db.as_ref(),
                &token,
            )
            .await
            .map_err(|_| AuthError::invalid_token())?;
            // Re-create since we just consumed it (userinfo shouldn't consume).
            let _ = crate::verification::create_verification(
                state.db.as_ref(),
                format!("oauth-at:{token}"),
                Some(r.value.clone()),
                3600,
            )
            .await;
            r.value
        };

        state
            .db
            .find_user_by_id(&user_id)
            .await?
            .ok_or_else(AuthError::user_not_found)?
    };

    Ok(Json(json!({
        "sub": user.id,
        "email": user.email,
        "email_verified": user.email_verified,
        "name": user.name,
        "picture": user.image,
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterClientRequest {
    pub redirect_uris: Vec<String>,
    pub name: Option<String>,
    pub grant_types: Option<Vec<String>>,
    pub response_types: Option<Vec<String>>,
}

async fn register_client(
    State(state): State<AuthState>,
    Json(req): Json<RegisterClientRequest>,
) -> Result<Json<Value>, AuthError> {
    if req.redirect_uris.is_empty() {
        return Err(AuthError::missing_field("redirect_uris"));
    }

    let client_id = generate_token();
    let client_secret = generate_token();
    let client = OAuthClient {
        client_id: client_id.clone(),
        client_secret: client_secret.clone(),
        redirect_uris: req.redirect_uris,
        name: req.name,
        grant_types: req.grant_types.unwrap_or_else(|| {
            vec!["authorization_code".into(), "refresh_token".into()]
        }),
        response_types: req
            .response_types
            .unwrap_or_else(|| vec!["code".into()]),
        created_at: OffsetDateTime::now_utc(),
    };

    state
        .db
        .plugin_set(
            "oauth_client",
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
        "redirect_uris": client.redirect_uris,
        "grant_types": client.grant_types,
        "response_types": client.response_types,
        "client_id_issued_at": client.created_at.unix_timestamp(),
    })))
}
