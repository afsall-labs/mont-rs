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

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};
use time::OffsetDateTime;

pub mod error_parser;
pub mod framework;
pub mod guides;
pub mod skills;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentSnapshot {
    pub project_name: String,
    pub timestamp: OffsetDateTime,
    pub framework_version: String,
    pub structure: Vec<FileEntry>,
    pub plates: Vec<PlateSummary>,
    pub routes: Vec<RouteSummary>,
    pub packages: Vec<PackageSummary>,
    pub agent_entry_point: Option<String>,
    pub documentation_snippets: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageSummary {
    pub name: String,
    pub path: String,
    pub invariants: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileEntry {
    pub path: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlateSummary {
    pub name: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RouteSummary {
    pub path: String,
    pub kind: String,
    pub description: String,
    pub input_validator: Option<serde_json::Value>,
    pub output_validator: Option<serde_json::Value>,
    pub params_validator: Option<serde_json::Value>,
    pub loader_output_validator: Option<serde_json::Value>,
    pub action_input_validator: Option<serde_json::Value>,
    pub action_output_validator: Option<serde_json::Value>,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorRecord {
    pub id: String,
    pub timestamp: OffsetDateTime,
    pub version: u32,
    pub status: ErrorStatus,
    pub detail: ProjectError,
    pub history: Vec<ErrorVersion>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ErrorStatus {
    Active,
    Resolved,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorVersion {
    pub version: u32,
    pub timestamp: OffsetDateTime,
    pub message: String,
    pub diff: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectError {
    pub package: Option<String>,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub code_context: String,
    pub level: String, // Error, Warning
    pub agent_metadata: Option<AgentErrorMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorTracking {
    pub errors: Vec<ConsolidatedError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsolidatedError {
    pub id: String,
    pub package: Option<String>,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub level: String,
    pub message: String,
    pub status: String,
    pub timestamp: OffsetDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentErrorMetadata {
    pub error_code: String,
    pub explanation: String,
    pub suggested_fixes: Vec<String>,
    pub rustc_error: Option<String>,
    pub documentation_refs: Vec<String>,
}

pub struct AgentManager {
    root_path: PathBuf,
}

impl AgentManager {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root.into(),
        }
    }

    pub fn agent_dir(&self) -> PathBuf {
        self.root_path.join(".agent")
    }

    pub fn errorfiles_dir(&self) -> PathBuf {
        self.agent_dir().join("errorfiles")
    }

    /// Scaffolds agent rules into .agent/rules/ and exports to IDE-specific formats.
    pub fn setup_ide_rules(&self) -> Result<String> {
        println!("Setting up agent rules in: {:?}", self.root_path);

        // 1. Write canonical rules to .agent/rules/
        let rules_dir = self.agent_dir().join("rules");
        if !rules_dir.exists() {
            println!("Creating .agent/rules directory: {:?}", rules_dir);
            fs::create_dir_all(&rules_dir)?;
        }

        fs::write(
            rules_dir.join("app-developer.md"),
            framework::APP_DEVELOPER_RULE,
        )?;
        fs::write(
            rules_dir.join("framework-contributor.md"),
            framework::FRAMEWORK_CONTRIBUTOR_RULE,
        )?;

        // 2. Export to Trae (.trae/rules/)
        self.export_rules_for_trae()?;

        // 3. Export to Cursor (.cursorrules)
        self.export_rules_for_cursor()?;

        // 4. Export to OpenCode (.opencode/rules/)
        self.export_rules_for_opencode()?;

        Ok("Successfully scaffolded .agent/rules/ and exported IDE \
            configurations"
            .to_string())
    }

    /// Exports rules to OpenCode IDE format (.opencode/rules/).
    pub fn export_rules_for_opencode(&self) -> Result<()> {
        let opencode_dir = self.root_path.join(".opencode").join("rules");
        if !opencode_dir.exists() {
            fs::create_dir_all(&opencode_dir)?;
        }

        let rules_dir = self.agent_dir().join("rules");
        for entry in fs::read_dir(&rules_dir)?.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
                let content = fs::read_to_string(entry.path())?;
                let name = entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                fs::write(opencode_dir.join(&name), content)?;
            }
        }
        Ok(())
    }

    /// Exports rules to Trae IDE format (.trae/rules/ with YAML frontmatter).
    pub fn export_rules_for_trae(&self) -> Result<()> {
        let trae_dir = self.root_path.join(".trae").join("rules");
        if !trae_dir.exists() {
            fs::create_dir_all(&trae_dir)?;
        }

        let rules_dir = self.agent_dir().join("rules");
        for entry in fs::read_dir(&rules_dir)?.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
                let content = fs::read_to_string(entry.path())?;
                let name = entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                let trae_content =
                    format!("---\nalwaysApply: false\n---\n{content}");
                fs::write(trae_dir.join(&name), trae_content)?;
            }
        }
        Ok(())
    }

    /// Exports rules to Cursor IDE format (.cursorrules single file).
    pub fn export_rules_for_cursor(&self) -> Result<()> {
        let cursorrules_path = self.root_path.join(".cursorrules");
        fs::write(&cursorrules_path, framework::APP_DEVELOPER_PROMPT)?;
        Ok(())
    }

    pub fn report_agent_error(
        &self,
        err: &dyn montrs_core::AgentError,
    ) -> Result<String> {
        let metadata = AgentErrorMetadata {
            error_code: err.error_code().to_string(),
            explanation: err.explanation(),
            suggested_fixes: err.suggested_fixes(),
            rustc_error: err.rustc_error(),
            documentation_refs: err.documentation_refs(),
        };

        self.report_project_error(ProjectError {
            package: None,
            file: "unknown".to_string(),
            line: 0,
            column: 0,
            message: err.to_string(),
            code_context: "".to_string(),
            level: "Error".to_string(),
            agent_metadata: Some(metadata),
        })
    }

    pub fn tracking_file(&self) -> PathBuf {
        self.errorfiles_dir().join("error_tracking.json")
    }

    pub fn ensure_dir(&self) -> Result<()> {
        let dir = self.agent_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        let error_dir = self.errorfiles_dir();
        if !error_dir.exists() {
            fs::create_dir_all(&error_dir)?;
        }
        let rules_dir = self.agent_dir().join("rules");
        if !rules_dir.exists() {
            fs::create_dir_all(&rules_dir)?;
        }
        Ok(())
    }

    pub fn write_snapshot(
        &self,
        snapshot: &AgentSnapshot,
        format: &str,
    ) -> Result<()> {
        self.ensure_dir()?;
        let content = match format {
            "yaml" => serde_yml::to_string(snapshot)?,
            "txt" => format!("{:#?}", snapshot),
            _ => serde_json::to_string_pretty(snapshot)?,
        };
        let ext = if format == "yaml" {
            "yaml"
        } else if format == "txt" {
            "txt"
        } else {
            "json"
        };
        fs::write(self.agent_dir().join(format!("agent.{}", ext)), content)?;
        Ok(())
    }

    pub fn write_error_record(&self, record: &ErrorRecord) -> Result<()> {
        self.ensure_dir()?;
        let error_group_dir = self.errorfiles_dir().join(&record.id);
        if !error_group_dir.exists() {
            fs::create_dir_all(&error_group_dir)?;
        }

        let content = serde_json::to_string_pretty(record)?;
        fs::write(
            error_group_dir.join(format!("v{}.json", record.version)),
            content,
        )?;

        // Update consolidated tracking
        self.update_consolidated_tracking(record)?;

        Ok(())
    }

    pub fn load_tracking(&self) -> Result<ErrorTracking> {
        let path = self.tracking_file();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            // Try new format first
            if let Ok(tracking) =
                serde_json::from_str::<ErrorTracking>(&content)
            {
                return Ok(tracking);
            }
            // Fallback to manual parsing if needed or try to adapt legacy
            if let Ok(value) =
                serde_json::from_str::<serde_json::Value>(&content)
                && let Some(active_issues) =
                    value.get("active_issues").and_then(|v| v.as_array())
            {
                let mut errors = Vec::new();
                for issue in active_issues {
                    if let (Some(id), Some(msg)) = (
                        issue.get("id").and_then(|v| v.as_str()),
                        issue.get("message").and_then(|v| v.as_str()),
                    ) {
                        let status = issue
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Pending")
                            .to_string();
                        errors.push(ConsolidatedError {
                            id: id.to_string(),
                            package: issue
                                .get("package")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            file: issue
                                .get("file")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            line: issue
                                .get("line")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0)
                                as u32,
                            column: issue
                                .get("column")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0)
                                as u32,
                            level: issue
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Error")
                                .to_string(),
                            message: msg.to_string(),
                            status,
                            timestamp: issue
                                .get("timestamp")
                                .and_then(|v| v.as_str())
                                .and_then(|s| {
                                    OffsetDateTime::parse(
                                        s,
                                        &time::format_description::well_known::Rfc3339,
                                    )
                                    .ok()
                                })
                                .unwrap_or_else(OffsetDateTime::now_utc),
                        });
                    }
                }
                if !errors.is_empty() {
                    return Ok(ErrorTracking { errors });
                }
            }
        }
        Ok(ErrorTracking { errors: Vec::new() })
    }

    pub fn save_tracking(&self, tracking: &ErrorTracking) -> Result<()> {
        self.ensure_dir()?;
        let content = serde_json::to_string_pretty(tracking)?;
        fs::write(self.tracking_file(), content)?;
        Ok(())
    }

    fn update_consolidated_tracking(&self, record: &ErrorRecord) -> Result<()> {
        let mut tracking = self
            .load_tracking()
            .unwrap_or(ErrorTracking { errors: Vec::new() });
        let status = match record.status {
            ErrorStatus::Active => "Pending",
            ErrorStatus::Resolved => "Fixed",
        }
        .to_string();

        let consolidated = ConsolidatedError {
            id: record.id.clone(),
            package: record.detail.package.clone(),
            file: record.detail.file.clone(),
            line: record.detail.line,
            column: record.detail.column,
            level: record.detail.level.clone(),
            message: record.detail.message.clone(),
            status,
            timestamp: record.timestamp,
        };

        if let Some(pos) =
            tracking.errors.iter().position(|e| e.id == record.id)
        {
            tracking.errors[pos] = consolidated;
        } else {
            tracking.errors.push(consolidated);
        }

        self.save_tracking(&tracking)
    }

    /// Reports a new error to the agent, creating or updating errorfile.json.
    pub fn report_project_error(
        &self,
        mut error: ProjectError,
    ) -> Result<String> {
        // Try to determine package from file path if not provided
        if error.package.is_none()
            && error.file != "unknown"
            && let Some(pkg) = self.determine_package(&error.file)
        {
            error.package = Some(pkg);
        }

        // Check if a similar active error already exists
        if let Ok(active_errors) = self.list_active_errors() {
            for existing in active_errors {
                if existing.detail.file == error.file
                    && existing.detail.line == error.line
                    && existing.detail.message == error.message
                {
                    return Ok(existing.id);
                }
            }
        }

        let id = uuid::Uuid::new_v4().to_string();
        let record = ErrorRecord {
            id: id.clone(),
            timestamp: OffsetDateTime::now_utc(),
            version: 1,
            status: ErrorStatus::Active,
            detail: error,
            history: Vec::new(),
        };
        self.write_error_record(&record)?;
        Ok(id)
    }

    pub fn list_active_errors(&self) -> Result<Vec<ErrorRecord>> {
        let mut active = Vec::new();
        let error_dir = self.errorfiles_dir();
        if !error_dir.exists() {
            return Ok(active);
        }

        for entry in fs::read_dir(error_dir)?.flatten() {
            if entry.path().is_dir() {
                for file in fs::read_dir(entry.path())?.flatten() {
                    if file.path().extension().and_then(|s| s.to_str())
                        == Some("json")
                        && let Ok(content) = fs::read_to_string(file.path())
                        && let Ok(record) =
                            serde_json::from_str::<ErrorRecord>(&content)
                        && let ErrorStatus::Active = record.status
                    {
                        active.push(record);
                    }
                }
            }
        }
        Ok(active)
    }

    pub fn report_error(&self, message: String) -> Result<()> {
        self.report_project_error(ProjectError {
            package: None,
            file: "unknown".to_string(),
            line: 0,
            column: 0,
            message,
            code_context: "".to_string(),
            level: "Error".to_string(),
            agent_metadata: None,
        })?;
        Ok(())
    }

    fn determine_package(&self, file_path: &str) -> Option<String> {
        let path = std::path::Path::new(file_path);
        // Look for "packages/NAME" or "apps/NAME"
        let components: Vec<_> = path.components().collect();
        for i in 0..components.len() {
            if let Some(c) = components[i].as_os_str().to_str()
                && (c == "packages" || c == "apps")
                && i + 1 < components.len()
            {
                return components[i + 1]
                    .as_os_str()
                    .to_str()
                    .map(|s| s.to_string());
            }
        }
        None
    }

    pub fn generate_diff(&self) -> Option<String> {
        // Try to use git diff if available
        std::process::Command::new("git")
            .args(["diff", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    let s = String::from_utf8_lossy(&output.stdout).to_string();
                    if s.is_empty() { None } else { Some(s) }
                } else {
                    None
                }
            })
    }

    pub fn auto_resolve_active_errors(
        &self,
        fix_message: String,
        diff: Option<String>,
    ) -> Result<()> {
        let walker = walkdir::WalkDir::new(self.errorfiles_dir()).into_iter();
        for entry in walker.filter_map(|e| e.ok()) {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json")
            {
                let content = fs::read_to_string(entry.path())?;
                if let Ok(record) =
                    serde_json::from_str::<ErrorRecord>(&content)
                    && let ErrorStatus::Active = record.status
                {
                    self.resolve_error(
                        &record.id,
                        fix_message.clone(),
                        diff.clone(),
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn resolve_error(
        &self,
        id: &str,
        fix_message: String,
        diff: Option<String>,
    ) -> Result<()> {
        let mut record = self.find_error(id)?;
        if let ErrorStatus::Active = record.status {
            record.status = ErrorStatus::Resolved;
            let history_version = record.version;
            record.version += 1;
            record.history.push(ErrorVersion {
                version: history_version,
                timestamp: OffsetDateTime::now_utc(),
                message: fix_message,
                diff,
            });
            self.write_error_record(&record)?;
        }
        Ok(())
    }

    fn find_error(&self, id: &str) -> Result<ErrorRecord> {
        let error_dir = self.errorfiles_dir();
        // Nested: <errorfiles>/<id>/v<N>.json
        let nested = error_dir.join(id);
        if nested.is_dir() {
            let mut latest: Option<ErrorRecord> = None;
            let mut highest_version: u32 = 0;
            for entry in fs::read_dir(&nested)?.flatten() {
                if entry.path().extension().and_then(|s| s.to_str())
                    == Some("json")
                {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Ok(record) =
                            serde_json::from_str::<ErrorRecord>(&content)
                        {
                            if record.version >= highest_version {
                                highest_version = record.version;
                                latest = Some(record);
                            }
                        }
                    }
                }
            }
            if let Some(record) = latest {
                return Ok(record);
            }
        }
        // Flat fallback: <errorfiles>/<id>.json
        let flat = error_dir.join(format!("{id}.json"));
        if flat.exists() {
            let content = fs::read_to_string(flat)?;
            return Ok(serde_json::from_str(&content)?);
        }
        anyhow::bail!("Error record not found: {}", id)
    }

    pub fn generate_tools_spec(&self) -> Result<serde_json::Value> {
        println!("Agent: Generating tools spec...");
        let mut tools = vec![
            serde_json::json!({
                "name": "montrs_build",
                "description": "Builds the MontRS project.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "release": { "type": "boolean", "description": "Build in release mode" }
                    }
                }
            }),
            serde_json::json!({
                "name": "montrs_spec",
                "description": "Generates a project specification.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "format": { "type": "string", "enum": ["json", "yaml", "txt"] }
                    }
                }
            }),
            serde_json::json!({
                "name": "montrs_fmt",
                "description": "Formats the project's Rust and view! code.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "check": { "type": "boolean", "description": "Check if files are formatted without modifying them" },
                        "path": { "type": "string", "description": "Path to format (default: .)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "montrs_sketch",
                "description": "Generates a single-file 'sketch' of a MontRS component.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Name of the sketch file" },
                        "kind": { "type": "string", "enum": ["plate", "route", "app"], "description": "Component type" }
                    },
                    "required": ["name"]
                }
            }),
            serde_json::json!({
                "name": "montrs_expand",
                "description": "Expands a sketch file into a full MontRS workspace structure.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Path to the sketch file" }
                    },
                    "required": ["path"]
                }
            }),
            serde_json::json!({
                "name": "montrs_run",
                "description": "Runs a custom task defined in montrs.toml.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "task": { "type": "string", "description": "Name of the task to run" }
                    },
                    "required": ["task"]
                }
            }),
            serde_json::json!({
                "name": "montrs_runner",
                "description": "Lists available custom tasks from montrs.toml.",
                "parameters": {
                    "type": "object",
                    "properties": {}
                }
            }),
        ];

        // Scan for package-specific tools/capabilities from READMEs and source comments
        let packages_dir = self.root_path.join("packages");
        if packages_dir.exists() {
            if let Ok(entries) = fs::read_dir(&packages_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let pkg_name = path
                            .file_name()
                            .map(|f| f.to_string_lossy())
                            .unwrap_or_default();

                        // 1. Scan README and docs/invariants.md for high-level capabilities
                        let readme_path = path.join("README.md");
                        let invariants_path =
                            path.join("docs").join("invariants.md");
                        let mut description = format!(
                            "Capability provided by package {}. Refer to its \
                             README for details.",
                            pkg_name
                        );

                        if invariants_path.exists() {
                            description = format!(
                                "Capability provided by package {}. MUST \
                                 follow invariants defined in agent.json \
                                 under packages['{}'].invariants",
                                pkg_name, pkg_name
                            );
                        } else if readme_path.exists()
                            && let Ok(content) =
                                fs::read_to_string(&readme_path)
                            && (content.contains("Agent Usage Guide")
                                || content.contains("Key Features")
                                || content.contains("Key Components")
                                || content.contains("Agent"))
                        {
                            // Keep default description or refine if needed
                        }

                        tools.push(serde_json::json!({
                            "name": format!("montrs_pkg_{}", pkg_name),
                            "description": description,
                            "parameters": { "type": "object", "properties": {} }
                        }));

                        // 2. Scan source for explicit @agent-tool markers
                        let src_dir = path.join("src");
                        if src_dir.exists() {
                            let walker =
                                walkdir::WalkDir::new(&src_dir).into_iter();
                            for entry in walker.filter_map(|e| e.ok()) {
                                if entry.path().is_file()
                                    && entry
                                        .path()
                                        .extension()
                                        .and_then(|s| s.to_str())
                                        == Some("rs")
                                    && let Ok(content) =
                                        fs::read_to_string(entry.path())
                                {
                                    for line in content.lines() {
                                        if line.contains("@agent-tool:")
                                            && let Some(tool_meta) = self
                                                .parse_agent_tool_marker(line)
                                        {
                                            let name = tool_meta["name"]
                                                .as_str()
                                                .unwrap_or_default();
                                            if !tools
                                                .iter()
                                                .any(|t| t["name"] == name)
                                            {
                                                tools.push(tool_meta);
                                            }
                                        }
                                        if line.contains("@agent-skill:")
                                            && let Some(skill_meta) = self
                                                .parse_agent_skill_marker(line)
                                        {
                                            let name = skill_meta["name"]
                                                .as_str()
                                                .unwrap_or_default();
                                            if !tools
                                                .iter()
                                                .any(|t| t["name"] == name)
                                            {
                                                tools.push(skill_meta);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // If packages_dir doesn't exist, we might be in a package itself or a flat structure
            // Just scan current src if it exists
            let src_dir = self.root_path.join("src");
            if src_dir.exists() {
                let walker = walkdir::WalkDir::new(&src_dir).into_iter();
                for entry in walker.filter_map(|e| e.ok()) {
                    if entry.path().is_file()
                        && entry.path().extension().and_then(|s| s.to_str())
                            == Some("rs")
                        && let Ok(content) = fs::read_to_string(entry.path())
                    {
                        for line in content.lines() {
                            if line.contains("@agent-tool:")
                                && let Some(tool_meta) =
                                    self.parse_agent_tool_marker(line)
                            {
                                let name = tool_meta["name"]
                                    .as_str()
                                    .unwrap_or_default();
                                if !tools.iter().any(|t| t["name"] == name) {
                                    tools.push(tool_meta);
                                }
                            }
                            if line.contains("@agent-skill:")
                                && let Some(skill_meta) =
                                    self.parse_agent_skill_marker(line)
                            {
                                let name = skill_meta["name"]
                                    .as_str()
                                    .unwrap_or_default();
                                if !tools.iter().any(|t| t["name"] == name) {
                                    tools.push(skill_meta);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Discover skills from skills/ directory
        let discovered_skills = skills::discover_skills(&self.root_path);
        let skill_tools = skills::skills_to_tools_json(&discovered_skills);
        tools.extend(skill_tools);

        Ok(serde_json::json!({ "tools": tools }))
    }

    fn parse_agent_tool_marker(&self, line: &str) -> Option<serde_json::Value> {
        // Expected format: @agent-tool: name="name" desc="description"
        let re = regex::Regex::new(
            r#"@agent-tool:\s+name="(?P<name>[^"]+)"\s+desc="(?P<desc>[^"]+)""#,
        )
        .ok()?;
        let caps = re.captures(line)?;

        Some(serde_json::json!({
            "name": caps.name("name")?.as_str(),
            "description": caps.name("desc")?.as_str(),
            "parameters": { "type": "object", "properties": {} }
        }))
    }

    fn parse_agent_skill_marker(
        &self,
        line: &str,
    ) -> Option<serde_json::Value> {
        // Expected format: @agent-skill: name="name" desc="description"
        let re = regex::Regex::new(
            r#"@agent-skill:\s+name="(?P<name>[^"]+)"\s+desc="(?P<desc>[^"]+)""#,
        )
        .ok()?;
        let caps = re.captures(line)?;

        Some(serde_json::json!({
            "name": format!("skill_{}", caps.name("name")?.as_str().replace('-', "_")),
            "description": caps.name("desc")?.as_str(),
            "parameters": { "type": "object", "properties": {} }
        }))
    }

    fn get_file_description(&self, path: &std::path::Path) -> Option<String> {
        let content = fs::read_to_string(path).ok()?;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("//!") || line.starts_with("///") {
                let desc = line
                    .trim_start_matches("//!")
                    .trim_start_matches("///")
                    .trim();
                if !desc.is_empty() {
                    return Some(desc.to_string());
                }
            }
        }
        None
    }

    fn discover_plates_heuristically(
        &self,
    ) -> (Vec<PlateSummary>, Vec<RouteSummary>) {
        let mut plates = Vec::new();
        let mut routes = Vec::new();

        // Scan root src, packages/*/src, and templates/*/src
        let mut scan_dirs = vec![self.root_path.join("src")];

        for dir_name in &["packages", "templates"] {
            let base_dir = self.root_path.join(dir_name);
            if base_dir.exists()
                && let Ok(entries) = fs::read_dir(base_dir)
            {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let src = entry.path().join("src");
                        if src.exists() {
                            println!("Agent: Scanning for plates in {:?}", src);
                            scan_dirs.push(src);
                        }
                        let tests = entry.path().join("tests");
                        if tests.exists() {
                            println!(
                                "Agent: Scanning for plates in {:?}",
                                tests
                            );
                            scan_dirs.push(tests);
                        }
                    }
                }
            }
        }

        let plate_re =
            regex::Regex::new(r"impl\s+Plate(?:<[^>]+>)?\s+for\s+(\w+)")
                .unwrap();
        let route_re =
            regex::Regex::new(r"impl\s+Route(?:<[^>]+>)?\s+for\s+(\w+)")
                .unwrap();

        for src_dir in scan_dirs {
            if !src_dir.exists() {
                continue;
            }
            let walker = walkdir::WalkDir::new(&src_dir).into_iter();
            for entry in walker.filter_map(|e| e.ok()) {
                if entry.path().is_file()
                    && entry.path().extension().and_then(|s| s.to_str())
                        == Some("rs")
                    && let Ok(content) = fs::read_to_string(entry.path())
                {
                    println!("Agent: Checking file {:?}", entry.path());
                    // Discover Plates
                    for caps in plate_re.captures_iter(&content) {
                        let name = caps[1].to_string();
                        println!("Agent: Found plate implementation: {}", name);
                        if !plates.iter().any(|m: &PlateSummary| m.name == name)
                        {
                            plates.push(PlateSummary {
                                name,
                                description: self
                                    .get_file_description(entry.path())
                                    .unwrap_or_else(|| {
                                        "Discovered plate".to_string()
                                    }),
                                dependencies: Vec::new(),
                                metadata: HashMap::new(),
                            });
                        }
                    }

                    // Discover Routes
                    for caps in route_re.captures_iter(&content) {
                        let name = caps[1].to_string();
                        println!("Agent: Found route implementation: {}", name);
                        routes.push(RouteSummary {
                            path: format!("(impl) {}", name),
                            kind: "Route".to_string(),
                            description: format!(
                                "Heuristically discovered Route: {}",
                                name
                            ),
                            input_validator: None,
                            output_validator: None,
                            params_validator: None,
                            loader_output_validator: None,
                            action_input_validator: None,
                            action_output_validator: None,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }

        (plates, routes)
    }

    pub fn write_tools_spec(&self) -> Result<()> {
        let tools = self.generate_tools_spec()?;
        let content = serde_json::to_string_pretty(&tools)?;
        let path = self.agent_dir().join("tools.json");
        fs::write(path, content)?;
        Ok(())
    }

    /// Generates a comprehensive snapshot of the codebase.
    pub fn generate_snapshot(
        &self,
        project_name: &str,
    ) -> Result<AgentSnapshot> {
        self.generate_snapshot_with_spec(project_name, None)
    }

    /// Generates a snapshot of the framework itself using embedded data.
    /// Useful for global reference or when not in a MontRS project.
    pub fn generate_framework_snapshot(&self) -> AgentSnapshot {
        let mut documentation_snippets = HashMap::new();
        documentation_snippets.insert(
            "architecture".to_string(),
            guides::ARCHITECTURE_GUIDE.to_string(),
        );
        documentation_snippets.insert(
            "debugging".to_string(),
            guides::DEBUGGING_GUIDE.to_string(),
        );
        documentation_snippets.insert(
            "agent/index".to_string(),
            framework::AGENT_INDEX.to_string(),
        );
        documentation_snippets.insert(
            "workflows/fixing-errors".to_string(),
            framework::FIXING_ERRORS_WORKFLOW.to_string(),
        );
        documentation_snippets.insert(
            "workflows/adding-features".to_string(),
            framework::ADDING_FEATURES_WORKFLOW.to_string(),
        );
        documentation_snippets.insert(
            "skills/guide".to_string(),
            framework::SKILLS_GUIDE.to_string(),
        );
        documentation_snippets.insert(
            "skills/database-setup".to_string(),
            framework::SKILL_DATABASE_SETUP.to_string(),
        );
        documentation_snippets.insert(
            "skills/testing".to_string(),
            framework::SKILL_TESTING.to_string(),
        );
        documentation_snippets.insert(
            "skills/deployment".to_string(),
            framework::SKILL_DEPLOYMENT.to_string(),
        );
        documentation_snippets.insert(
            "contributor/prdoc".to_string(),
            framework::PRDOC_GUIDE.to_string(),
        );
        documentation_snippets.insert(
            "contributor/prdoc-template".to_string(),
            framework::PRDOC_TEMPLATE.to_string(),
        );

        let framework_invariants = framework::get_framework_invariants();
        let mut packages = Vec::new();
        for (name, invariants) in framework_invariants {
            packages.push(PackageSummary {
                name: name.to_string(),
                path: format!("packages/{}", name),
                invariants: Some(invariants.to_string()),
                description: Some(format!(
                    "MontRS {} package (embedded reference)",
                    name
                )),
            });
        }

        AgentSnapshot {
            project_name: "MontRS Framework".to_string(),
            timestamp: OffsetDateTime::now_utc(),
            framework_version: "0.1.0".to_string(),
            structure: Vec::new(),
            plates: Vec::new(),
            routes: Vec::new(),
            packages,
            agent_entry_point: Some(framework::AGENT_INDEX.to_string()),
            documentation_snippets,
        }
    }

    pub fn generate_snapshot_with_spec(
        &self,
        project_name: &str,
        spec: Option<montrs_core::AppSpecExport>,
    ) -> Result<AgentSnapshot> {
        let mut structure = Vec::new();
        let walker = ignore::WalkBuilder::new(&self.root_path)
            .hidden(false)
            .git_ignore(true)
            .add_custom_ignore_filename(".agentignore")
            .filter_entry(|entry| {
                let name = entry.file_name().to_string_lossy();
                name != ".git" && name != "target" && name != ".agent"
            })
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_file() {
                let relative_path = path
                    .strip_prefix(&self.root_path)?
                    .to_string_lossy()
                    .into_owned();
                let description = if path.extension().and_then(|s| s.to_str())
                    == Some("rs")
                {
                    self.get_file_description(path)
                } else {
                    None
                };

                structure.push(FileEntry {
                    path: relative_path,
                    description,
                });
            }
        }

        let mut documentation_snippets = HashMap::new();
        documentation_snippets.insert(
            "architecture".to_string(),
            guides::ARCHITECTURE_GUIDE.to_string(),
        );
        documentation_snippets.insert(
            "debugging".to_string(),
            guides::DEBUGGING_GUIDE.to_string(),
        );

        // Add embedded workflows and prompts
        documentation_snippets.insert(
            "workflows/fixing-errors".to_string(),
            framework::FIXING_ERRORS_WORKFLOW.to_string(),
        );
        documentation_snippets.insert(
            "workflows/adding-features".to_string(),
            framework::ADDING_FEATURES_WORKFLOW.to_string(),
        );
        documentation_snippets.insert(
            "skills/guide".to_string(),
            framework::SKILLS_GUIDE.to_string(),
        );
        documentation_snippets.insert(
            "skills/database-setup".to_string(),
            framework::SKILL_DATABASE_SETUP.to_string(),
        );
        documentation_snippets.insert(
            "skills/testing".to_string(),
            framework::SKILL_TESTING.to_string(),
        );
        documentation_snippets.insert(
            "skills/deployment".to_string(),
            framework::SKILL_DEPLOYMENT.to_string(),
        );
        documentation_snippets.insert(
            "contributor/prdoc".to_string(),
            framework::PRDOC_GUIDE.to_string(),
        );
        documentation_snippets.insert(
            "contributor/prdoc-template".to_string(),
            framework::PRDOC_TEMPLATE.to_string(),
        );
        documentation_snippets.insert(
            "agent/index".to_string(),
            framework::AGENT_INDEX.to_string(),
        );
        documentation_snippets.insert(
            "agent/app-developer-prompt".to_string(),
            framework::APP_DEVELOPER_PROMPT.to_string(),
        );
        documentation_snippets.insert(
            "agent/framework-contributor-prompt".to_string(),
            framework::FRAMEWORK_CONTRIBUTOR_PROMPT.to_string(),
        );

        // Scan global docs directory
        let docs_dir = self.root_path.join("docs");
        if docs_dir.exists() {
            let walker = walkdir::WalkDir::new(&docs_dir).into_iter();
            for entry in walker.filter_map(|e| e.ok()) {
                if entry.path().is_file()
                    && entry.path().extension().and_then(|s| s.to_str())
                        == Some("md")
                    && let Ok(relative_path) =
                        entry.path().strip_prefix(&docs_dir)
                {
                    let key = relative_path.to_string_lossy().to_string();
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        documentation_snippets
                            .insert(format!("docs/{}", key), content);
                    }
                }
            }
        }

        // Discover packages and their invariants/internal docs
        let mut packages = Vec::new();
        let packages_dir = self.root_path.join("packages");

        // Use embedded framework invariants as a base/fallback
        let framework_invariants = framework::get_framework_invariants();

        if packages_dir.exists()
            && let Ok(entries) = fs::read_dir(&packages_dir)
        {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let relative_path = entry
                        .path()
                        .strip_prefix(&self.root_path)?
                        .to_string_lossy()
                        .to_string();

                    let pkg_docs_dir = entry.path().join("docs");
                    let invariants_path = pkg_docs_dir.join("invariants.md");
                    let invariants = if invariants_path.exists() {
                        fs::read_to_string(invariants_path).ok()
                    } else {
                        // Fallback to embedded framework invariant if it matches a core package
                        framework_invariants
                            .get(name.as_str())
                            .map(|s| s.to_string())
                    };

                    // Also scan for other package-specific docs
                    if pkg_docs_dir.exists() {
                        let walker =
                            walkdir::WalkDir::new(&pkg_docs_dir).into_iter();
                        for doc_entry in walker.filter_map(|e| e.ok()) {
                            if doc_entry.path().is_file()
                                && doc_entry
                                    .path()
                                    .extension()
                                    .and_then(|s| s.to_str())
                                    == Some("md")
                                && let Ok(rel_doc_path) =
                                    doc_entry.path().strip_prefix(&pkg_docs_dir)
                            {
                                let key =
                                    rel_doc_path.to_string_lossy().to_string();
                                if let Ok(content) =
                                    fs::read_to_string(doc_entry.path())
                                {
                                    documentation_snippets.insert(
                                        format!(
                                            "packages/{}/docs/{}",
                                            name, key
                                        ),
                                        content,
                                    );
                                }
                            }
                        }
                    }

                    packages.push(PackageSummary {
                        name,
                        path: relative_path,
                        invariants,
                        description: None,
                    });
                }
            }
        }

        // Check for Agent Entry Point
        let entry_point_path =
            self.root_path.join("docs").join("agent").join("index.md");
        let agent_entry_point = if entry_point_path.exists() {
            Some(fs::read_to_string(entry_point_path)?)
        } else {
            // Fallback to embedded framework entry point
            Some(framework::AGENT_INDEX.to_string())
        };

        let (plates, routes) = if let Some(s) = spec {
            let mut plates = Vec::new();
            let mut routes = Vec::new();

            for plate_spec in s.plates {
                plates.push(PlateSummary {
                    name: plate_spec.name,
                    description: plate_spec.description,
                    dependencies: plate_spec.dependencies,
                    metadata: plate_spec.metadata,
                });
            }

            for (path, meta) in s.router.routes {
                routes.push(RouteSummary {
                    path: path.clone(),
                    kind: "Route".to_string(),
                    description: meta.loader_description.clone(),
                    input_validator: None,
                    output_validator: None,
                    params_validator: None,
                    loader_output_validator: None,
                    action_input_validator: None,
                    action_output_validator: None,
                    metadata: HashMap::new(),
                });
            }
            (plates, routes)
        } else {
            self.discover_plates_heuristically()
        };

        Ok(AgentSnapshot {
            project_name: project_name.to_string(),
            timestamp: OffsetDateTime::now_utc(),
            framework_version: "0.1.0".to_string(),
            structure,
            plates,
            routes,
            packages,
            agent_entry_point,
            documentation_snippets,
        })
    }

    pub fn check_invariants(
        &self,
        snapshot: &AgentSnapshot,
    ) -> Result<Vec<String>> {
        let mut violations = Vec::new();

        // 1. Check Plate Dependencies
        // For each plate, all its dependencies must exist in the snapshot
        for plate in &snapshot.plates {
            for dep in &plate.dependencies {
                if !snapshot.plates.iter().any(|p| &p.name == dep) {
                    violations.push(format!(
                        "Plate '{}' depends on missing plate '{}'.",
                        plate.name, dep
                    ));
                }
            }
        }

        // 2. Check for circular dependencies (simple depth-limited check)
        for plate in &snapshot.plates {
            let mut visited = std::collections::HashSet::new();
            let mut stack = vec![(&plate.name, 0)];

            while let Some((current_name, depth)) = stack.pop() {
                if depth > 10 {
                    // Limit depth to avoid infinite loops in complex cycles
                    violations.push(format!(
                        "Potential deep dependency cycle involving plate '{}'.",
                        plate.name
                    ));
                    break;
                }

                if visited.contains(current_name) {
                    violations.push(format!(
                        "Circular dependency detected involving plate '{}'.",
                        current_name
                    ));
                    break;
                }
                visited.insert(current_name);

                if let Some(p) =
                    snapshot.plates.iter().find(|p| &p.name == current_name)
                {
                    for dep in &p.dependencies {
                        stack.push((dep, depth + 1));
                    }
                }
            }
        }

        // 3. Check for package invariants and documentation
        for pkg in &snapshot.packages {
            if pkg.invariants.is_none() {
                violations.push(format!(
                    "Package '{}' is missing invariants. All framework \
                     packages must define architectural rules.",
                    pkg.name
                ));
            }
        }

        // 4. Check for unified entry point
        if snapshot.agent_entry_point.is_none() {
            violations.push(
                "Project is missing a unified agent entry point \
                 (docs/agent/index.md)."
                    .to_string(),
            );
        }

        Ok(violations)
    }

    /// Runs health diagnostics on the project or a specific package.
    pub fn run_doctor(&self, package: Option<&str>) -> Result<Vec<String>> {
        let mut reports = Vec::new();

        // 1. Check Cargo.toml presence and validity
        let cargo_toml = self.root_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            reports.push(
                "❌ Root Cargo.toml not found. Is this a Rust project?"
                    .to_string(),
            );
        } else {
            reports.push("✅ Root Cargo.toml found.".to_string());
        }

        // 2. Check for .agent directory
        if !self.agent_dir().exists() {
            reports.push(
                "⚠️ .agent directory not found. Run 'agent check' to \
                 initialize."
                    .to_string(),
            );
        } else {
            reports.push("✅ .agent directory exists.".to_string());
        }

        // 3. Check for error tracking file
        if !self.tracking_file().exists() {
            reports.push(
                "ℹ️ No error tracking file found. No errors currently tracked."
                    .to_string(),
            );
        } else {
            let tracking = self.load_tracking()?;
            let active_count = tracking
                .errors
                .iter()
                .filter(|e| e.status == "Active")
                .count();
            if active_count > 0 {
                reports.push(format!(
                    "⚠️ Found {} active errors in tracking.",
                    active_count
                ));
            } else {
                reports.push("✅ All tracked errors are resolved.".to_string());
            }
        }

        // 4. Package specific checks
        if let Some(pkg_name) = package {
            let pkg_path = self.root_path.join("packages").join(pkg_name);
            if !pkg_path.exists() {
                reports.push(format!(
                    "❌ Package '{}' not found in packages/ directory.",
                    pkg_name
                ));
            } else {
                reports.push(format!(
                    "✅ Package '{}' directory exists.",
                    pkg_name
                ));
                let pkg_cargo = pkg_path.join("Cargo.toml");
                if !pkg_cargo.exists() {
                    reports.push(format!(
                        "❌ Package '{}' is missing Cargo.toml.",
                        pkg_name
                    ));
                }
            }
        }

        // 5. Check toolchain (basic)
        let rustc = std::process::Command::new("rustc")
            .arg("--version")
            .output();
        match rustc {
            Ok(output) if output.status.success() => {
                let version =
                    String::from_utf8_lossy(&output.stdout).trim().to_string();
                reports
                    .push(format!("✅ Rust toolchain available: {}", version));
            }
            _ => reports.push("❌ rustc not found in PATH.".to_string()),
        }

        Ok(reports)
    }
}
