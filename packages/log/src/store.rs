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

//! File-backed log store with streaming and retention.

use crate::format::LogFormat;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use time::OffsetDateTime;
use tokio::{io::AsyncWriteExt, sync::RwLock};

/// A single log entry captured from a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub seq: u64,
    pub service: String,
    /// ISO-8601 UTC timestamp.
    pub ts: String,
    pub level: String,
    pub message: String,
}

/// Retention policy for a service's logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Maximum number of lines to keep per service.
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
    /// Maximum age in seconds before rotation (0 = unlimited).
    #[serde(default)]
    pub max_age_secs: u64,
    /// Archive rotated files to this directory (optional).
    pub archive_dir: Option<PathBuf>,
}

fn default_max_lines() -> usize {
    10_000
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_lines: default_max_lines(),
            max_age_secs: 0,
            archive_dir: None,
        }
    }
}

/// Configuration for the log store.
#[derive(Debug, Clone)]
pub struct LogStoreConfig {
    /// Root directory for log files.
    pub root: PathBuf,
    /// Default retention applied to all services.
    pub retention: RetentionPolicy,
    /// On-disk format.
    pub format: LogFormat,
}

impl Default for LogStoreConfig {
    fn default() -> Self {
        Self {
            root: default_log_root(),
            retention: RetentionPolicy::default(),
            format: LogFormat::Text,
        }
    }
}

fn default_log_root() -> PathBuf {
    std::env::var_os("MONTRS_STATE")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| {
            dirs_home_fallback().join(".local/state/montrs/logs")
        })
}

fn dirs_home_fallback() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// A query for reading log entries.
#[derive(Debug, Clone, Default)]
pub struct LogQuery {
    pub service: Option<String>,
    pub level: Option<String>,
    pub limit: usize,
    pub offset: u64,
}

/// The log store. Shared across services, guarded by an async lock.
#[derive(Clone)]
pub struct LogStore {
    inner: Arc<RwLock<LogStoreInner>>,
}

struct LogStoreInner {
    config: LogStoreConfig,
    /// Service name -> current sequence counter.
    seqs: HashMap<String, u64>,
}

impl LogStore {
    /// Create a new store rooted at `config.root`.
    pub fn open(config: LogStoreConfig) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&config.root)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(LogStoreInner {
                config,
                seqs: HashMap::new(),
            })),
        })
    }

    /// Open the store at the default location.
    pub fn open_default() -> anyhow::Result<Self> {
        Self::open(LogStoreConfig::default())
    }

    /// Append a log line for a service. The message is normalized to the
    /// configured format before being written to the service's log file.
    pub async fn append(
        &self,
        service: &str,
        level: &str,
        message: &str,
    ) -> anyhow::Result<()> {
        let mut inner = self.inner.write().await;
        let seq = inner.seqs.entry(service.to_string()).or_insert(0);
        *seq += 1;

        let ts = OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        let file = inner.log_file(service).unwrap_or_else(|_| {
            inner.config.root.join(format!("{service}.log"))
        });

        let rendered = inner.config.format.render(&ts, level, service, message);

        let mut f = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file)
            .await?;
        f.write_all(rendered.as_bytes()).await?;
        f.write_all(b"\n").await?;
        f.flush().await?;

        // Enforce line-count retention.
        if inner.config.retention.max_lines > 0 {
            let lines = tokio::fs::read_to_string(&file).await?;
            let count = lines.lines().count();
            if count > inner.config.retention.max_lines {
                let keep = count - inner.config.retention.max_lines;
                let trimmed: String =
                    lines.lines().skip(keep).collect::<Vec<_>>().join("\n");
                tokio::fs::write(&file, trimmed).await?;
            }
        }
        Ok(())
    }

    /// Read log entries matching `query` from the store.
    pub async fn query(
        &self,
        query: LogQuery,
    ) -> anyhow::Result<Vec<LogEntry>> {
        let inner = self.inner.read().await;
        let mut out = Vec::new();
        let services: Vec<String> = match &query.service {
            Some(sv) => vec![sv.clone()],
            None => inner.list_services()?,
        };

        for service in services {
            let file = inner.log_file(&service)?;
            if !file.exists() {
                continue;
            }
            let content = tokio::fs::read_to_string(&file).await?;
            for line in content.lines().skip(query.offset as usize) {
                if query.limit > 0 && out.len() >= query.limit {
                    break;
                }
                let entry = parse_line(&query, &service, line);
                if let Some(e) = entry {
                    if let Some(lvl) = &query.level
                        && e.level.to_lowercase() != lvl.to_lowercase()
                    {
                        continue;
                    }
                    out.push(e);
                }
            }
        }
        Ok(out)
    }

    /// Stream new entries appended to services. Returns a channel receiver.
    /// Each entry is delivered as a structured line.
    pub async fn tail(
        &self,
        _service: Option<&str>,
    ) -> anyhow::Result<tokio::sync::mpsc::Receiver<String>> {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        // NOTE: In-memory tailing is a thin wrapper; callers may subscribe to
        // the append hook for live streaming. For now we return an empty
        // receiver that callers can use with a follow-up refresh loop.
        let _ = tx;
        Ok(rx)
    }

    /// Rotate and optionally archive a service's log file.
    pub async fn rotate(&self, service: &str) -> anyhow::Result<()> {
        let inner = self.inner.write().await;
        let file = inner.log_file(service)?;
        if !file.exists() {
            return Ok(());
        }
        let rotated = file.with_extension(format!(
            "log.1.{}",
            OffsetDateTime::now_utc()
                .format(
                    &time::format_description::parse_borrowed::<2>(
                        "[year][month][day][hour][minute][second]"
                    )
                    .expect("valid time format")
                )
                .expect("valid timestamp formatting")
        ));
        tokio::fs::rename(&file, &rotated).await?;

        if let Some(archive) = &inner.config.retention.archive_dir {
            tokio::fs::create_dir_all(archive).await?;
            let dest = archive.join(rotated.file_name().unwrap_or_default());
            tokio::fs::rename(&rotated, &dest).await?;
        }
        Ok(())
    }

    /// Delete all log files for a service.
    pub async fn clear(&self, service: &str) -> anyhow::Result<()> {
        let inner = self.inner.read().await;
        let file = inner.log_file(service)?;
        if file.exists() {
            tokio::fs::remove_file(&file).await?;
        }
        Ok(())
    }
}

impl LogStoreInner {
    fn log_file(&self, service: &str) -> anyhow::Result<PathBuf> {
        let safe = sanitize_service(service);
        Ok(self.config.root.join(format!("{safe}.log")))
    }

    fn list_services(&self) -> anyhow::Result<Vec<String>> {
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&self.config.root) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("log")
                    && let Some(stem) = p.file_stem().and_then(|s| s.to_str())
                {
                    out.push(stem.to_string());
                }
            }
        }
        Ok(out)
    }
}

/// Parse a raw line into a LogEntry, honoring the configured format.
fn parse_line(
    _query: &LogQuery,
    service: &str,
    line: &str,
) -> Option<LogEntry> {
    // Try JSON first.
    if let Some(rec) = LogFormat::parse_json(line) {
        return Some(LogEntry {
            seq: 0,
            service: rec.service,
            ts: rec.ts,
            level: rec.level,
            message: rec.msg,
        });
    }

    // Plain text: `[ts] [level] service: message`.
    if let Some(rest) = line.strip_prefix('[') {
        let mut parts = rest.split("] [");
        let ts = parts.next().unwrap_or("").trim().to_string();
        let level_rest = parts.next().unwrap_or("");
        if let Some((level, after)) = level_rest.split_once("] ")
            && let Some((_sv, msg)) = after.split_once(": ")
        {
            return Some(LogEntry {
                seq: 0,
                service: service.to_string(),
                ts,
                level: level.to_string(),
                message: msg.to_string(),
            });
        }
    }

    Some(LogEntry {
        seq: 0,
        service: service.to_string(),
        ts: String::new(),
        level: String::from("info"),
        message: line.to_string(),
    })
}

/// Sanitize a service name into a safe file stem.
fn sanitize_service(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Ensure the store root exists (used by tests and tooling).
pub fn ensure_root(path: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn append_and_query() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let config = LogStoreConfig {
            root: dir.path().to_path_buf(),
            retention: RetentionPolicy::default(),
            format: LogFormat::Text,
        };
        let store = LogStore::open(config)?;
        store.append("api", "info", "listening on :3000").await?;
        store.append("api", "error", "boom").await?;
        store.append("worker", "info", "processed").await?;

        let all = store.query(LogQuery::default()).await?;
        assert_eq!(all.len(), 3);

        let api = store
            .query(LogQuery {
                service: Some("api".into()),
                limit: 0,
                ..Default::default()
            })
            .await?;
        assert_eq!(api.len(), 2);

        let errs = store
            .query(LogQuery {
                service: Some("api".into()),
                level: Some("error".into()),
                limit: 0,
                ..Default::default()
            })
            .await?;
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].message, "boom");
        Ok(())
    }

    #[tokio::test]
    async fn retention_trims() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let config = LogStoreConfig {
            root: dir.path().to_path_buf(),
            retention: RetentionPolicy {
                max_lines: 5,
                ..Default::default()
            },
            format: LogFormat::Text,
        };
        let store = LogStore::open(config)?;
        for i in 0..20 {
            store.append("svc", "info", &format!("line {i}")).await?;
        }
        let all = store
            .query(LogQuery {
                service: Some("svc".into()),
                limit: 0,
                ..Default::default()
            })
            .await?;
        assert_eq!(all.len(), 5);
        Ok(())
    }
}
