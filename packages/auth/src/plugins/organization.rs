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

//! Organization plugin â€” multi-tenant orgs, members, invites, roles.
//! Store orgs/members/invites as JSON in plugin_store namespace "org".

use crate::{
    AuthError,
    context::AuthState,
    plugin::AuthPlugin,
    plugins::access::{Authorization, Statement, authorize},
};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use time::OffsetDateTime;

/// An organization record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: OffsetDateTime,
    pub created_by: String,
}

/// A membership record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: OffsetDateTime,
}

/// An invite record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    pub id: String,
    pub organization_id: String,
    pub email: String,
    pub role: String,
    pub inviter_id: String,
    pub status: String,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

/// Organization plugin.
pub struct OrganizationPlugin {
    state: Option<AuthState>,
}

impl OrganizationPlugin {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl Default for OrganizationPlugin {
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

/// Default role statements for org roles.
fn org_role_statements(role: &str) -> Vec<Statement> {
    match role {
        "owner" | "admin" => vec![Statement {
            effect: "allow".into(),
            actions: vec!["*".into()],
            resources: vec!["org:*".into()],
        }],
        "member" => vec![Statement {
            effect: "allow".into(),
            actions: vec!["org:read".into(), "org:list".into()],
            resources: vec!["org:*".into()],
        }],
        _ => vec![],
    }
}

impl AuthPlugin for OrganizationPlugin {
    fn name(&self) -> &'static str {
        "organization"
    }

    fn on_build(&mut self, state: &AuthState) -> Result<(), AuthError> {
        self.state = Some(state.clone());
        Ok(())
    }

    fn router(&self) -> Router {
        let state = self
            .state
            .clone()
            .expect("OrganizationPlugin: state not set");
        Router::new()
            .route("/organization/create", post(create_org))
            .route("/organization/list", get(list_orgs))
            .route("/organization/update", post(update_org))
            .route("/organization/delete", post(delete_org))
            .route("/organization/invite-member", post(invite_member))
            .route("/organization/accept-invite", post(accept_invite))
            .route("/organization/set-active", post(set_active))
            .route("/organization/list-members", get(list_members))
            .route("/organization/remove-member", post(remove_member))
            .route("/organization/update-member-role", post(update_member_role))
            .with_state(state)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrgRequest {
    pub name: String,
    pub slug: Option<String>,
    pub logo: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

async fn create_org(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateOrgRequest>,
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
    let slug = req.slug.unwrap_or_else(|| {
        req.name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect()
    });

    let org = Organization {
        id: id.clone(),
        name: req.name,
        slug,
        logo: req.logo,
        metadata: req.metadata.unwrap_or_default(),
        created_at: OffsetDateTime::now_utc(),
        created_by: session.user_id.clone(),
    };

    state
        .db
        .plugin_set("org", &id, serde_json::to_value(&org).unwrap())
        .await
        .map_err(|e| {
            AuthError::new(
                crate::error::AuthErrorCode::OrganizationError,
                e.to_string(),
            )
        })?;

    // Add creator as owner member.
    let member = Member {
        id: uuid::Uuid::new_v4().to_string(),
        organization_id: id.clone(),
        user_id: session.user_id.clone(),
        role: "owner".into(),
        created_at: OffsetDateTime::now_utc(),
    };
    state
        .db
        .plugin_set(
            "org_member",
            &format!("{}:{}", id, session.user_id),
            serde_json::to_value(&member).unwrap(),
        )
        .await
        .ok();

    Ok(Json(json!({ "organization": org })))
}

async fn list_orgs(
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

    let members = state.db.plugin_list("org_member").await.map_err(|e| {
        AuthError::new(
            crate::error::AuthErrorCode::InternalError,
            e.to_string(),
        )
    })?;

    let mut org_ids = Vec::new();
    for (_, val) in members {
        if let Ok(m) = serde_json::from_value::<Member>(val)
            && m.user_id == session.user_id
        {
            org_ids.push(m.organization_id);
        }
    }

    let mut orgs = Vec::new();
    for oid in org_ids {
        if let Ok(Some(val)) = state.db.plugin_get("org", &oid).await
            && let Ok(org) = serde_json::from_value::<Organization>(val)
        {
            orgs.push(org);
        }
    }

    Ok(Json(json!({ "organizations": orgs })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrgRequest {
    pub organization_id: String,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub logo: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

async fn update_org(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateOrgRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    require_org_role(&state, &session.user_id, &req.organization_id, "admin")
        .await?;

    let entry = state
        .db
        .plugin_get("org", &req.organization_id)
        .await?
        .ok_or_else(|| {
            AuthError::new(
                crate::error::AuthErrorCode::OrganizationError,
                "Organization not found",
            )
        })?;
    let mut org: Organization =
        serde_json::from_value(entry).map_err(|_| {
            AuthError::new(
                crate::error::AuthErrorCode::OrganizationError,
                "Invalid org",
            )
        })?;

    if let Some(name) = req.name {
        org.name = name;
    }
    if let Some(slug) = req.slug {
        org.slug = slug;
    }
    if let Some(logo) = req.logo {
        org.logo = Some(logo);
    }
    if let Some(meta) = req.metadata {
        org.metadata = meta;
    }

    state
        .db
        .plugin_set(
            "org",
            &req.organization_id,
            serde_json::to_value(&org).unwrap(),
        )
        .await
        .ok();

    Ok(Json(json!({ "organization": org })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteOrgRequest {
    pub organization_id: String,
}

async fn delete_org(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeleteOrgRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    require_org_role(&state, &session.user_id, &req.organization_id, "owner")
        .await?;

    state
        .db
        .plugin_delete("org", &req.organization_id)
        .await
        .ok();

    // Clean up members.
    let members = state.db.plugin_list("org_member").await.unwrap_or_default();
    for (key, val) in members {
        if let Ok(m) = serde_json::from_value::<Member>(val)
            && m.organization_id == req.organization_id
        {
            let _ = state.db.plugin_delete("org_member", &key).await;
        }
    }

    Ok(Json(
        json!({ "success": true, "deleted": req.organization_id }),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteMemberRequest {
    pub organization_id: String,
    pub email: String,
    pub role: Option<String>,
}

async fn invite_member(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<InviteMemberRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    require_org_role(&state, &session.user_id, &req.organization_id, "admin")
        .await?;

    let id = uuid::Uuid::new_v4().to_string();
    let invite = Invite {
        id: id.clone(),
        organization_id: req.organization_id.clone(),
        email: req.email.clone(),
        role: req.role.unwrap_or_else(|| "member".into()),
        inviter_id: session.user_id.clone(),
        status: "pending".into(),
        expires_at: OffsetDateTime::now_utc() + time::Duration::days(7),
        created_at: OffsetDateTime::now_utc(),
    };

    state
        .db
        .plugin_set("org_invite", &id, serde_json::to_value(&invite).unwrap())
        .await
        .ok();

    let _ = state
        .email
        .send(crate::email::EmailMessage {
            to: req.email,
            subject: "You've been invited to an organization".into(),
            body_text: format!(
                "You've been invited. Accept with invite id: {id}"
            ),
            body_html: None,
        })
        .await;

    Ok(Json(json!({ "invite": invite })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptInviteRequest {
    pub invite_id: String,
}

async fn accept_invite(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<AcceptInviteRequest>,
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
        .plugin_get("org_invite", &req.invite_id)
        .await?
        .ok_or_else(|| {
            AuthError::new(
                crate::error::AuthErrorCode::OrganizationError,
                "Invite not found",
            )
        })?;
    let mut invite: Invite = serde_json::from_value(entry).map_err(|_| {
        AuthError::new(
            crate::error::AuthErrorCode::OrganizationError,
            "Invalid invite",
        )
    })?;

    if invite.status != "pending"
        || invite.expires_at <= OffsetDateTime::now_utc()
    {
        return Err(AuthError::new(
            crate::error::AuthErrorCode::OrganizationError,
            "Invite expired or already used",
        ));
    }

    invite.status = "accepted".into();
    state
        .db
        .plugin_set(
            "org_invite",
            &req.invite_id,
            serde_json::to_value(&invite).unwrap(),
        )
        .await
        .ok();

    let member = Member {
        id: uuid::Uuid::new_v4().to_string(),
        organization_id: invite.organization_id.clone(),
        user_id: session.user_id.clone(),
        role: invite.role.clone(),
        created_at: OffsetDateTime::now_utc(),
    };
    state
        .db
        .plugin_set(
            "org_member",
            &format!("{}:{}", invite.organization_id, session.user_id),
            serde_json::to_value(&member).unwrap(),
        )
        .await
        .ok();

    Ok(Json(json!({ "success": true, "member": member })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetActiveRequest {
    pub organization_id: String,
}

async fn set_active(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<SetActiveRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    // Verify membership.
    let key = format!("{}:{}", req.organization_id, session.user_id);
    let _ =
        state
            .db
            .plugin_get("org_member", &key)
            .await?
            .ok_or_else(|| {
                AuthError::new(
                    crate::error::AuthErrorCode::Forbidden,
                    "Not a member of this organization",
                )
            })?;

    // Store active org on session via plugin store.
    state
        .db
        .plugin_set(
            "active_org",
            &session.id,
            json!({ "organizationId": req.organization_id }),
        )
        .await
        .ok();

    Ok(Json(
        json!({ "success": true, "activeOrganizationId": req.organization_id }),
    ))
}

async fn list_members(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(q): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    let org_id = q
        .get("organizationId")
        .or_else(|| q.get("organization_id"))
        .ok_or_else(|| AuthError::missing_field("organizationId"))?;

    require_org_role(&state, &session.user_id, org_id, "member").await?;

    let members = state.db.plugin_list("org_member").await.map_err(|e| {
        AuthError::new(
            crate::error::AuthErrorCode::InternalError,
            e.to_string(),
        )
    })?;

    let list: Vec<Value> = members
        .into_iter()
        .filter_map(|(_, val)| {
            let m: Member = serde_json::from_value(val).ok()?;
            if m.organization_id == *org_id {
                Some(json!({
                    "id": m.id,
                    "userId": m.user_id,
                    "role": m.role,
                    "createdAt": m.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                }))
            } else {
                None
            }
        })
        .collect();

    Ok(Json(json!({ "members": list })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMemberRequest {
    pub organization_id: String,
    pub user_id: String,
}

async fn remove_member(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<RemoveMemberRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    require_org_role(&state, &session.user_id, &req.organization_id, "admin")
        .await?;

    let key = format!("{}:{}", req.organization_id, req.user_id);
    state.db.plugin_delete("org_member", &key).await.ok();

    Ok(Json(json!({ "success": true, "removed": req.user_id })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemberRoleRequest {
    pub organization_id: String,
    pub user_id: String,
    pub role: String,
}

async fn update_member_role(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> Result<Json<Value>, AuthError> {
    let token =
        extract_token(&headers).ok_or_else(AuthError::invalid_session)?;
    let session = state
        .session
        .validate(&token)
        .await?
        .ok_or_else(AuthError::invalid_session)?;

    require_org_role(&state, &session.user_id, &req.organization_id, "admin")
        .await?;

    let key = format!("{}:{}", req.organization_id, req.user_id);
    let entry =
        state
            .db
            .plugin_get("org_member", &key)
            .await?
            .ok_or_else(|| {
                AuthError::new(
                    crate::error::AuthErrorCode::OrganizationError,
                    "Member not found",
                )
            })?;
    let mut member: Member = serde_json::from_value(entry).map_err(|_| {
        AuthError::new(
            crate::error::AuthErrorCode::OrganizationError,
            "Invalid member",
        )
    })?;

    member.role = req.role;
    state
        .db
        .plugin_set("org_member", &key, serde_json::to_value(&member).unwrap())
        .await
        .ok();

    Ok(Json(json!({ "success": true, "member": member })))
}

/// Require that the user has at least the given role level in the org.
async fn require_org_role(
    state: &AuthState,
    user_id: &str,
    org_id: &str,
    min_role: &str,
) -> Result<(), AuthError> {
    let key = format!("{org_id}:{user_id}");
    let entry = state
        .db
        .plugin_get("org_member", &key)
        .await?
        .ok_or_else(AuthError::forbidden)?;
    let member: Member =
        serde_json::from_value(entry).map_err(|_| AuthError::forbidden())?;

    let rank = |r: &str| match r {
        "owner" => 3,
        "admin" => 2,
        "member" => 1,
        _ => 0,
    };

    if rank(&member.role) < rank(min_role) {
        return Err(AuthError::forbidden());
    }

    // Also run access control for extra safety.
    let statements = org_role_statements(&member.role);
    if authorize(&statements, "org:read", "org:*") == Authorization::Denied
        && min_role != "member"
    {
        return Err(AuthError::forbidden());
    }

    Ok(())
}
