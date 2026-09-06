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

//! Invariant tests for montrs-tool.

use montrs_tool::*;
use std::path::Path;

#[test]
fn test_tool_request_parse() {
    let req = ToolRequest::parse("rust@1.84.0");
    assert_eq!(req.name, "rust");
    assert_eq!(req.version, Some("1.84.0".to_string()));
}

#[test]
fn test_tool_request_parse_no_version() {
    let req = ToolRequest::parse("node");
    assert_eq!(req.name, "node");
    assert_eq!(req.version, None);
}

#[test]
fn test_tool_request_parse_latest() {
    let req = ToolRequest::parse("cargo@latest");
    assert_eq!(req.name, "cargo");
    assert_eq!(req.version, Some("latest".to_string()));
}

#[test]
fn test_tool_manager_new() {
    let tm = ToolManager::new();
    let install = tm.install_dir.to_string_lossy().replace('\\', "/");
    let shims = tm.shims_dir.to_string_lossy().replace('\\', "/");
    assert!(install.contains("montrs/installs"));
    assert!(shims.contains("montrs/shims"));
}

#[test]
fn test_tool_manager_lookup() {
    let tm = ToolManager::new();
    let tool = tm.lookup("rust");
    assert!(tool.is_some());
    let tool = tm.lookup("nonexistent");
    assert!(tool.is_none());
}

#[test]
fn test_backend_types() {
    assert_eq!(BackendType::Core.as_str(), "core");
    assert_eq!(BackendType::GitHub.as_str(), "github");
    assert_eq!(BackendType::Cargo.as_str(), "cargo");
    assert_eq!(BackendType::Ubi.as_str(), "ubi");
}

#[test]
fn test_tool_version_construct() {
    let tv = ToolVersion {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        backend: BackendType::Core,
        url: None,
        checksum: None,
        install_path: Path::new("/tmp").to_path_buf(),
        bins: vec!["test".to_string()],
    };
    assert_eq!(tv.name, "test");
    assert_eq!(tv.version, "1.0.0");
    assert_eq!(tv.bins, vec!["test"]);
}

#[test]
fn test_create_backend() {
    let backend = create_backend("test", "core", None, None).unwrap();
    assert_eq!(backend.name(), "test");
    assert_eq!(backend.backend_type(), BackendType::Core);
}

#[test]
fn test_create_backend_github() {
    let backend =
        create_backend("ripgrep", "github:BurntSushi/ripgrep", None, None)
            .unwrap();
    assert_eq!(backend.backend_type(), BackendType::GitHub);
}

#[test]
fn test_create_backend_default_github() {
    let backend =
        create_backend("unknown-tool", "unknown", None, None).unwrap();
    assert_eq!(backend.backend_type(), BackendType::GitHub);
}

#[test]
fn test_tool_error_display() {
    let err = ToolError::NotFound("test".to_string());
    assert!(err.to_string().contains("not found"));
    let err = ToolError::AlreadyInstalled("test".to_string());
    assert!(err.to_string().contains("Already installed"));
    let err = ToolError::NotInstalled("test".to_string());
    assert!(err.to_string().contains("Not installed"));
}

#[tokio::test]
async fn test_sha256_digest_missing_file() {
    let result = sha256_digest(Path::new("/nonexistent/file")).await;
    assert!(result.is_err());
}
