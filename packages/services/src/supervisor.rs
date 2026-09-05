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

//! Service supervisor â€” the background process manager.
//!
//! Manages the lifecycle of services: start, stop, restart, signal,
//! ready checks, retry, hooks, and state persistence.

use crate::{
    config::ServiceConfig,
    hooks::{HookRunner, LifecycleEvent},
    ready,
    service::{Service, ServiceStatus},
    service_id::ServiceId,
    state::StateFile,
};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use time::OffsetDateTime;
use tokio::{
    process::Command,
    sync::{RwLock, Semaphore},
    time::{Duration, sleep},
};
use tracing::{error, info, warn};

/// The supervisor manages all services.
#[derive(Clone)]
pub struct Supervisor {
    inner: Arc<RwLock<SupervisorInner>>,
    /// Maximum concurrent service starts.
    concurrency: Arc<Semaphore>,
}

struct SupervisorInner {
    services: HashMap<ServiceId, Service>,
    state_file: StateFile,
    configs: HashMap<ServiceId, ServiceConfig>,
    #[allow(dead_code)]
    data_dir: PathBuf,
}

impl Supervisor {
    /// Create a new supervisor with the given service configs.
    pub fn new(
        configs: HashMap<String, ServiceConfig>,
        data_dir: PathBuf,
    ) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let state_file =
            StateFile::open(data_dir.join("montrs-services.toml"))?;

        let services: HashMap<ServiceId, Service> = configs
            .iter()
            .map(|(name, cfg)| {
                let id = ServiceId::from_name(name.clone());
                let mut svc = Service::new(id.clone(), cfg.clone());
                if let Some(st) = state_file.get(&id.to_string())
                    && !st.enabled
                {
                    svc.status = ServiceStatus::Stopped;
                }
                (id, svc)
            })
            .collect();

        let configs = configs
            .into_iter()
            .map(|(name, cfg)| (ServiceId::from_name(name), cfg))
            .collect();

        let supervisor = Self {
            inner: Arc::new(RwLock::new(SupervisorInner {
                services,
                state_file,
                configs,
                data_dir,
            })),
            concurrency: Arc::new(Semaphore::new(4)),
        };

        // Start the background retry loop.
        let s = supervisor.clone();
        tokio::spawn(async move {
            s.retry_loop().await;
        });

        Ok(supervisor)
    }

    /// Create a supervisor from a TOML services map.
    pub fn from_toml_map(
        map: &HashMap<String, toml::Value>,
        data_dir: PathBuf,
    ) -> anyhow::Result<Self> {
        let configs = ServiceConfig::from_toml_map(map)?;
        Self::new(configs, data_dir)
    }

    /// Get the service list with current status.
    pub async fn list(&self) -> Vec<(ServiceId, ServiceStatus, Option<u32>)> {
        let inner = self.inner.read().await;
        inner
            .services
            .values()
            .map(|s| (s.id.clone(), s.status, s.pid))
            .collect()
    }

    /// Get a single service's status.
    pub async fn status(&self, id: &ServiceId) -> Option<ServiceStatus> {
        let inner = self.inner.read().await;
        inner.services.get(id).map(|s| s.status)
    }

    /// Start a service by ID.
    pub async fn start(&self, id: &ServiceId) -> anyhow::Result<()> {
        let _permit = self.concurrency.acquire().await?;

        // Check if already running.
        {
            let inner = self.inner.read().await;
            if let Some(svc) = inner.services.get(id)
                && svc.status.is_active()
            {
                return Err(
                    crate::ServicesError::AlreadyRunning(id.clone()).into()
                );
            }
        }

        let config = {
            let inner = self.inner.read().await;
            inner
                .configs
                .get(id)
                .cloned()
                .ok_or_else(|| crate::ServicesError::NotFound(id.clone()))?
        };

        info!("starting service: {id}");

        // Start dependencies first (non-recursive to avoid Send issues).
        for dep in &config.depends {
            let dep_id = ServiceId::from_name(dep.clone());
            let inner = self.inner.write().await;
            if let Some(dep_svc) = inner.services.get(&dep_id)
                && !dep_svc.status.is_active()
            {
                drop(inner);
                Box::pin(self.start(&dep_id)).await?;
                continue;
            }
            drop(inner);
        }

        // Spawn the process.
        let cmd = config.run.clone().ok_or_else(|| {
            anyhow::anyhow!("service {id} has no `run` command")
        })?;

        let mut command = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(&cmd);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(&cmd);
            c
        };

        if let Some(dir) = &config.dir {
            command.current_dir(dir);
        }
        for (k, v) in &config.env {
            command.env(k, v);
        }

        let child = command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let pid = child
            .id()
            .ok_or_else(|| anyhow::anyhow!("failed to get PID"))?;

        // Update service state.
        {
            let mut inner = self.inner.write().await;
            if let Some(svc) = inner.services.get_mut(id) {
                svc.mark_started(child);
                svc.pid = Some(pid);
            }
            let st = inner.state_file.get_mut(&id.to_string());
            st.pid = Some(pid);
            st.last_start = Some(
                OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            );
            st.start_count += 1;
            st.was_running = true;
            let _ = inner.state_file.save();
        }

        // Run ready checks in a spawned task.
        let ready_checks = config.ready.clone();
        let ready_delay = config.ready_delay;
        let svc_id = id.clone();
        let supervisor = self.clone();
        tokio::spawn(async move {
            let out_matcher = |_: &str| false;
            if let Err(e) = ready::wait_ready(
                &svc_id,
                &ready_checks,
                ready_delay,
                out_matcher,
            )
            .await
            {
                warn!("service {svc_id}: ready check failed: {e}");
                let mut inner = supervisor.inner.write().await;
                if let Some(svc) = inner.services.get_mut(&svc_id) {
                    svc.mark_failed();
                }
                return;
            }
            let mut inner = supervisor.inner.write().await;
            if let Some(svc) = inner.services.get_mut(&svc_id) {
                svc.mark_ready();
                HookRunner::run_if_present(
                    &svc_id.to_string(),
                    &svc.config.hooks,
                    LifecycleEvent::Ready,
                )
                .await;
            }
            let st = inner.state_file.get_mut(&svc_id.to_string());
            st.was_running = true;
            let _ = inner.state_file.save();
            info!("service {svc_id} is ready");
        });

        Ok(())
    }

    /// Stop a service by ID.
    pub async fn stop(&self, id: &ServiceId) -> anyhow::Result<()> {
        info!("stopping service: {id}");

        let mut inner = self.inner.write().await;
        let svc = inner
            .services
            .get_mut(id)
            .ok_or_else(|| crate::ServicesError::NotFound(id.clone()))?;

        if !svc.status.is_active() {
            return Err(crate::ServicesError::NotRunning(id.clone()).into());
        }

        svc.mark_stopping();
        svc.keep_alive
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // Try graceful shutdown.
        #[allow(unused_variables)]
        let pid = svc.pid;
        #[cfg(unix)]
        if let Some(pid) = pid {
            use nix::{
                sys::signal::{Signal, kill},
                unistd::Pid,
            };
            let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
        }

        // Wait for graceful shutdown.
        let graceful = tokio::time::timeout(Duration::from_secs(10), async {
            #[cfg(unix)]
            {
                if let Some(pid) = pid {
                    // Wait until the process terminates (SIGTERM sent above).
                    loop {
                        // Check if process is still alive by sending signal 0.
                        use nix::{
                            sys::signal::{Signal, kill},
                            unistd::Pid,
                        };
                        if kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
                            .is_err()
                        {
                            // Process no longer exists.
                            break;
                        }
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }
            #[cfg(not(unix))]
            {
                // On non-Unix, just wait.
                sleep(Duration::from_secs(1)).await;
            }
        })
        .await;

        if graceful.is_err() {
            #[cfg(unix)]
            {
                use nix::{
                    sys::signal::{Signal, kill},
                    unistd::Pid,
                };
                if let Some(pid) = pid {
                    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGKILL);
                }
            }
        }

        svc.mark_stopped();

        HookRunner::run_if_present(
            &id.to_string(),
            &svc.config.hooks,
            LifecycleEvent::Stop,
        )
        .await;

        let st = inner.state_file.get_mut(&id.to_string());
        st.pid = None;
        st.last_stop = Some(
            OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
        );
        st.was_running = false;
        let _ = inner.state_file.save();

        Ok(())
    }

    /// Restart a service.
    pub async fn restart(&self, id: &ServiceId) -> anyhow::Result<()> {
        self.stop(id).await?;
        sleep(Duration::from_millis(500)).await;
        self.start(id).await
    }

    /// Start all enabled services.
    pub async fn start_all(&self) -> anyhow::Result<()> {
        let ids: Vec<ServiceId> = {
            let inner = self.inner.read().await;
            inner.services.keys().cloned().collect()
        };
        let sorted = self.topological_sort(&ids).await;
        for id in sorted {
            if let Err(e) = self.start(&id).await {
                warn!("failed to start {id}: {e}");
            }
        }
        Ok(())
    }

    /// Stop all running services.
    pub async fn stop_all(&self) -> anyhow::Result<()> {
        let ids: Vec<ServiceId> = {
            let inner = self.inner.read().await;
            inner
                .services
                .values()
                .filter(|s| s.status.is_active())
                .map(|s| s.id.clone())
                .collect()
        };
        let sorted = self.topological_sort(&ids).await;
        for id in sorted.into_iter().rev() {
            if let Err(e) = self.stop(&id).await {
                warn!("failed to stop {id}: {e}");
            }
        }
        Ok(())
    }

    /// Background retry loop: periodically checks for failed services
    /// and restarts them with retry logic.
    async fn retry_loop(self) {
        loop {
            sleep(Duration::from_secs(2)).await;

            let to_retry: Vec<(ServiceId, ServiceConfig)> = {
                let inner = self.inner.read().await;
                inner
                    .services
                    .iter()
                    .filter(|(_, svc)| svc.status == ServiceStatus::Failed)
                    .filter_map(|(id, svc)| {
                        let config = inner.configs.get(id)?;
                        if svc.retry.should_retry(&config.retry) {
                            Some((id.clone(), config.clone()))
                        } else {
                            None
                        }
                    })
                    .collect()
            };

            for (id, config) in to_retry {
                // Update retry state.
                {
                    let mut inner = self.inner.write().await;
                    if let Some(svc) = inner.services.get_mut(&id) {
                        svc.retry.record_failure(&config.retry).await;

                        if !svc.retry.should_retry(&config.retry) {
                            error!("service {id}: exhausted retries");
                            let st = inner.state_file.get_mut(&id.to_string());
                            st.fail_count += 1;
                            st.was_running = false;
                            let _ = inner.state_file.save();
                            continue;
                        }

                        warn!(
                            "service {id}: restarting (attempt {}/{})",
                            svc.retry.attempts, config.retry.count
                        );
                    }
                }

                // Fire retry hook.
                {
                    let inner = self.inner.read().await;
                    if let Some(svc) = inner.services.get(&id) {
                        HookRunner::run_if_present(
                            &id.to_string(),
                            &svc.config.hooks,
                            LifecycleEvent::Retry,
                        )
                        .await;
                    }
                }

                // Restart by spawning a new task.
                let s = self.clone();
                let id2 = id.clone();
                tokio::spawn(async move {
                    if let Err(e) = s.start(&id2).await {
                        error!("service {id2}: restart failed: {e}");
                    }
                });
            }
        }
    }

    /// Topological sort of services by dependency.
    async fn topological_sort(&self, ids: &[ServiceId]) -> Vec<ServiceId> {
        let inner = self.inner.read().await;
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();

        fn visit(
            id: &ServiceId,
            services: &HashMap<ServiceId, Service>,
            visited: &mut std::collections::HashSet<ServiceId>,
            sorted: &mut Vec<ServiceId>,
        ) {
            if visited.contains(id) {
                return;
            }
            visited.insert(id.clone());
            if let Some(svc) = services.get(id) {
                for dep in &svc.config.depends {
                    let dep_id = ServiceId::from_name(dep.clone());
                    visit(&dep_id, services, visited, sorted);
                }
            }
            sorted.push(id.clone());
        }

        for id in ids {
            visit(id, &inner.services, &mut visited, &mut sorted);
        }
        sorted
    }
}

/// Get the default supervisor data directory.
pub fn default_data_dir() -> PathBuf {
    crate::state::state_root()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_supervisor_create() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let mut configs = HashMap::new();
        configs.insert(
            "test".to_string(),
            ServiceConfig {
                run: Some("echo hello".to_string()),
                ..Default::default()
            },
        );
        let supervisor = Supervisor::new(configs, dir.path().to_path_buf())?;
        let list = supervisor.list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0.to_string(), "test");
        Ok(())
    }

    #[tokio::test]
    async fn test_start_stop() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let mut configs = HashMap::new();
        configs.insert(
            "quick".to_string(),
            ServiceConfig {
                run: Some("echo started && sleep 10".to_string()),
                ..Default::default()
            },
        );
        let supervisor = Supervisor::new(configs, dir.path().to_path_buf())?;
        let id = ServiceId::from_name("quick");
        supervisor.start(&id).await?;
        sleep(Duration::from_millis(200)).await;
        let status = supervisor.status(&id).await;
        assert!(
            status == Some(ServiceStatus::Starting)
                || status == Some(ServiceStatus::Running)
        );
        supervisor.stop(&id).await?;
        let status = supervisor.status(&id).await;
        assert_eq!(status, Some(ServiceStatus::Stopped));
        Ok(())
    }
}
