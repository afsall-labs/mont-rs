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

//! Session management â€” create, validate, refresh, and revoke sessions.

use crate::{
    database::{DatabaseAdapter, SessionRecord, UserRecord},
    entities::DefaultSession,
};
use std::sync::Arc;
use time::OffsetDateTime;
use tower_http::cors::CorsLayer;

/// Manages session lifecycle.
#[derive(Clone)]
pub struct SessionManager {
    secret: String,
    adapter: Arc<dyn DatabaseAdapter>,
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new(secret: String, adapter: Arc<dyn DatabaseAdapter>) -> Self {
        Self { secret, adapter }
    }

    /// Create a new session for a user.
    pub async fn create(
        &self,
        user_id: &str,
        expires_in_secs: u64,
    ) -> anyhow::Result<DefaultSession> {
        let session = DefaultSession::new(user_id, expires_in_secs);
        self.adapter.create_session(&session).await?;
        Ok(session)
    }

    /// Validate a session token. Returns the session if valid and not expired.
    pub async fn validate(
        &self,
        token: &str,
    ) -> anyhow::Result<Option<SessionRecord>> {
        let Some(session) = self.adapter.find_session_by_token(token).await?
        else {
            return Ok(None);
        };
        if session.expires_at <= OffsetDateTime::now_utc() {
            let _ = self.adapter.delete_session(&session.id).await;
            return Ok(None);
        }
        Ok(Some(session))
    }

    /// Get the user associated with a valid session.
    pub async fn get_user(
        &self,
        token: &str,
    ) -> anyhow::Result<Option<UserRecord>> {
        let Some(session) = self.validate(token).await? else {
            return Ok(None);
        };
        self.adapter.find_user_by_id(&session.user_id).await
    }

    /// Revoke a session.
    pub async fn revoke(&self, token: &str) -> anyhow::Result<()> {
        self.adapter.delete_session(token).await
    }

    /// Revoke all sessions for a user.
    pub async fn revoke_all(&self, user_id: &str) -> anyhow::Result<()> {
        self.adapter.delete_user_sessions(user_id).await
    }

    /// List sessions for a user.
    pub async fn list(
        &self,
        user_id: &str,
    ) -> anyhow::Result<Vec<SessionRecord>> {
        self.adapter.list_sessions(user_id).await
    }

    /// CORS layer placeholder for middleware chain stability.
    pub fn middleware(&self) -> CorsLayer {
        CorsLayer::permissive()
    }

    /// The signing secret.
    pub fn secret(&self) -> &str {
        &self.secret
    }

    /// Underlying adapter.
    pub fn adapter(&self) -> &Arc<dyn DatabaseAdapter> {
        &self.adapter
    }
}

/// JSON helper for session responses.
pub fn session_json(session: &DefaultSession) -> serde_json::Value {
    serde_json::json!({
        "id": session.id,
        "userId": session.user_id,
        "token": session.token,
        "expiresAt": session.expires_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
        "createdAt": session.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
    })
}
