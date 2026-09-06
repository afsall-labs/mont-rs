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

use montrs_agent::{AgentManager, AgentSnapshot, PlateSummary};
use std::collections::HashMap;
use tempfile::tempdir;
use time::OffsetDateTime;

#[test]
fn test_invariant_dependencies() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let manager = AgentManager::new(root);

    let mut snapshot = AgentSnapshot {
        project_name: "test".to_string(),
        timestamp: OffsetDateTime::now_utc(),
        framework_version: "0.1.0".to_string(),
        structure: Vec::new(),
        plates: vec![PlateSummary {
            name: "PlateA".to_string(),
            description: "A".to_string(),
            dependencies: vec!["PlateB".to_string()],
            metadata: HashMap::new(),
        }],
        routes: Vec::new(),
        packages: vec![montrs_agent::PackageSummary {
            name: "test-pkg".to_string(),
            path: "packages/test-pkg".to_string(),
            invariants: Some("Must be fast".to_string()),
            description: None,
        }],
        agent_entry_point: Some("docs/agent/index.md".to_string()),
        documentation_snippets: HashMap::new(),
    };

    // Case 1: Missing dependency
    let violations = manager.check_invariants(&snapshot).unwrap();
    assert_eq!(violations.len(), 1);
    assert!(violations[0].contains("depends on missing plate 'PlateB'"));

    // Case 2: Dependency exists
    snapshot.plates.push(PlateSummary {
        name: "PlateB".to_string(),
        description: "B".to_string(),
        dependencies: Vec::new(),
        metadata: HashMap::new(),
    });
    let violations = manager.check_invariants(&snapshot).unwrap();
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_invariant_cycles() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let manager = AgentManager::new(root);

    let snapshot = AgentSnapshot {
        project_name: "test".to_string(),
        timestamp: OffsetDateTime::now_utc(),
        framework_version: "0.1.0".to_string(),
        structure: Vec::new(),
        plates: vec![
            PlateSummary {
                name: "PlateA".to_string(),
                description: "A".to_string(),
                dependencies: vec!["PlateB".to_string()],
                metadata: HashMap::new(),
            },
            PlateSummary {
                name: "PlateB".to_string(),
                description: "B".to_string(),
                dependencies: vec!["PlateA".to_string()],
                metadata: HashMap::new(),
            },
        ],
        routes: Vec::new(),
        packages: vec![montrs_agent::PackageSummary {
            name: "test-pkg".to_string(),
            path: "packages/test-pkg".to_string(),
            invariants: Some("Must be fast".to_string()),
            description: None,
        }],
        agent_entry_point: Some("docs/agent/index.md".to_string()),
        documentation_snippets: HashMap::new(),
    };

    let violations = manager.check_invariants(&snapshot).unwrap();
    assert!(
        violations
            .iter()
            .any(|v| v.contains("Circular dependency detected"))
    );
}
