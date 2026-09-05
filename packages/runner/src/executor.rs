// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
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

use crate::{
    scheduler::task_needs_permit,
    types::{RunEntry, Task, TaskOutput},
};
use std::{path::Path, sync::Arc, time::Instant};
use tokio::sync::Semaphore;

/// Configuration for task execution.
#[derive(Clone, Default)]
pub struct TaskExecutorConfig {
    pub force: bool,
    pub cd: Option<std::path::PathBuf>,
    pub shell: Option<String>,
    pub timings: bool,
    pub continue_on_error: bool,
    pub dry_run: bool,
    pub skip_deps: bool,
}

/// Executes a single task.
pub async fn execute_task(
    task: &Task,
    all_tasks: &[Task],
    config: &TaskExecutorConfig,
    semaphore: Arc<Semaphore>,
) -> anyhow::Result<bool> {
    if !task_needs_permit(task) {
        return Ok(false);
    }

    let _permit = if task_needs_permit(task) {
        Some(semaphore.acquire().await.unwrap())
    } else {
        None
    };

    let start = Instant::now();

    if config.dry_run {
        println!("[dry-run] Would run task: {}", task.name);
        return Ok(true);
    }

    // Resolve the working directory
    let cwd = if let Some(ref dir) = task.dir {
        let base = task
            .config_root
            .as_deref()
            .unwrap_or_else(|| Path::new("."));
        base.join(dir)
    } else {
        config
            .cd
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
    };

    // Execute each run entry
    for entry in &task.command {
        match entry {
            RunEntry::Script(script) => {
                let (shell, shell_flag) = resolve_shell(
                    task.shell.as_deref().or(config.shell.as_deref()),
                );

                let mut cmd = tokio::process::Command::new(shell);
                cmd.arg(shell_flag);
                cmd.arg(script);
                cmd.current_dir(&cwd);

                // Set environment variables
                for (key, val) in &task.env {
                    cmd.env(key, val);
                }

                let status = cmd.status().await?;
                if !status.success() && !config.continue_on_error {
                    anyhow::bail!(
                        "Task '{}' failed with status: {}",
                        task.name,
                        status
                    );
                }
            }
            RunEntry::SingleTask { task, args, env } => {
                // Find and execute the sub-task
                if let Some(sub_task) =
                    all_tasks.iter().find(|t| t.name == *task)
                {
                    let mut sub = sub_task.clone();
                    sub.trailing_args = args.clone();
                    for (k, v) in env {
                        sub.env.insert(k.clone(), v.clone());
                    }
                    Box::pin(execute_task(
                        &sub,
                        all_tasks,
                        config,
                        semaphore.clone(),
                    ))
                    .await?;
                }
            }
            RunEntry::TaskGroup { tasks } => {
                for task_name in tasks {
                    if let Some(sub_task) =
                        all_tasks.iter().find(|t| t.name == *task_name)
                    {
                        Box::pin(execute_task(
                            sub_task,
                            all_tasks,
                            config,
                            semaphore.clone(),
                        ))
                        .await?;
                    }
                }
            }
        }
    }

    if config.timings {
        let elapsed = start.elapsed();
        println!("  {} completed in {:?}", task.name, elapsed);
    }

    Ok(true)
}

/// Resolve the shell executable and its flag for running a task script.
///
/// The flag must match the shell that will actually be used:
/// - `cmd.exe` (Windows default): `/C`
/// - PowerShell: `-Command`
/// - POSIX shells (sh, bash, zsh, fish, ...): `-c`
///
/// When no shell is configured, defaults to the platform's native shell
/// (`cmd` on Windows, `sh` elsewhere) so tasks work without a POSIX shell
/// installed on Windows.
fn resolve_shell<'a>(configured: Option<&'a str>) -> (&'a str, &'static str) {
    if let Some(shell) = configured {
        let name = shell.to_ascii_lowercase();
        let flag = if name.contains("cmd") {
            "/C"
        } else if name.contains("pwsh") || name.contains("powershell") {
            "-Command"
        } else {
            "-c"
        };
        // Return the user's shell string but with a matching flag.
        return (shell, flag);
    }

    #[cfg(windows)]
    {
        ("cmd", "/C")
    }
    #[cfg(not(windows))]
    {
        ("sh", "-c")
    }
}

/// Display task results in the terminal.
pub fn display_task_start(task: &Task, output: TaskOutput) {
    match output {
        TaskOutput::Quiet | TaskOutput::Silent => {}
        _ => {
            let prefix = format!("[{}]", task.name);
            println!(
                "{} {}",
                console::style(prefix).cyan().bold(),
                task.description
            );
        }
    }
}

pub fn display_task_finish(
    task: &Task,
    output: TaskOutput,
    duration: std::time::Duration,
) {
    match output {
        TaskOutput::Quiet | TaskOutput::Silent => {}
        _ => {
            let prefix = format!("[{}]", task.name);
            let completed = "completed in";
            println!(
                "{} {} {}",
                console::style(prefix).green().bold(),
                completed,
                console::style(format!("{duration:?}")).dim()
            );
        }
    }
}
