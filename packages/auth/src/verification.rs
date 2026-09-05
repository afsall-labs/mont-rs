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

//! Shared verification token store helpers (OTP, magic link, email verify).

use crate::{
    database::{DatabaseAdapter, VerificationRecord},
    utils::generate_token,
};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// Create a verification record and return its raw token value.
pub async fn create_verification(
    db: &dyn DatabaseAdapter,
    identifier: impl Into<String>,
    value: Option<String>,
    expires_in_secs: i64,
) -> anyhow::Result<VerificationRecord> {
    let token = value.unwrap_or_else(generate_token);
    let record = VerificationRecord {
        id: Uuid::new_v4().to_string(),
        identifier: identifier.into(),
        value: token,
        expires_at: OffsetDateTime::now_utc()
            + Duration::seconds(expires_in_secs),
        created_at: OffsetDateTime::now_utc(),
    };
    db.create_verification(&record).await?;
    Ok(record)
}

/// Consume a verification by identifier + value. Deletes on success.
pub async fn consume_verification(
    db: &dyn DatabaseAdapter,
    identifier: &str,
    value: &str,
) -> anyhow::Result<VerificationRecord> {
    let Some(rec) = db.find_verification(identifier, value).await? else {
        return Err(crate::AuthError::invalid_token().into());
    };
    if rec.expires_at <= OffsetDateTime::now_utc() {
        let _ = db.delete_verification(&rec.id).await;
        return Err(crate::AuthError::invalid_token().into());
    }
    db.delete_verification(&rec.id).await?;
    Ok(rec)
}

/// Consume a verification by token value only (looks up identifier).
pub async fn consume_verification_by_value(
    db: &dyn DatabaseAdapter,
    value: &str,
) -> anyhow::Result<VerificationRecord> {
    let Some(rec) = db.find_verification_by_value(value).await? else {
        return Err(crate::AuthError::invalid_token().into());
    };
    if rec.expires_at <= OffsetDateTime::now_utc() {
        let _ = db.delete_verification(&rec.id).await;
        return Err(crate::AuthError::invalid_token().into());
    }
    db.delete_verification(&rec.id).await?;
    Ok(rec)
}

/// Create a numeric OTP verification (for email-otp / phone).
pub async fn create_otp(
    db: &dyn DatabaseAdapter,
    identifier: impl Into<String>,
    length: usize,
    expires_in_secs: i64,
) -> anyhow::Result<VerificationRecord> {
    let otp = crate::utils::generate_otp(length);
    create_verification(db, identifier, Some(otp), expires_in_secs).await
}
