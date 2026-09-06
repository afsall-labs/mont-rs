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
use console::style;
use montrs_fmt::{FormatterSettings, format_source};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub async fn run(
    settings: FormatterSettings,
    check: bool,
    path: String,
    verbose: bool,
) -> Result<()> {
    let input_path = PathBuf::from(path);

    let mut exit_code = 0;
    let mut files_checked = 0;
    let mut files_formatted = 0;

    if input_path.is_file() {
        if format_one_file(&input_path, &settings, check, verbose)? {
            exit_code = 1;
            files_formatted += 1;
        }
        files_checked += 1;
    } else {
        for entry in WalkDir::new(&input_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            if format_one_file(entry.path(), &settings, check, verbose)? {
                exit_code = 1;
                files_formatted += 1;
            }
            files_checked += 1;
        }
    }

    if check {
        if exit_code == 0 {
            println!("{}", style("All files are correctly formatted.").green());
        } else {
            println!(
                "{}",
                style(format!("{} files need formatting.", files_formatted))
                    .red()
            );
            anyhow::bail!("Formatting check failed");
        }
    } else {
        if verbose {
            println!(
                "Checked {} files, formatted {} files.",
                files_checked, files_formatted
            );
        }
    }

    Ok(())
}

fn format_one_file(
    path: &Path,
    settings: &FormatterSettings,
    check: bool,
    verbose: bool,
) -> anyhow::Result<bool> {
    let original = std::fs::read_to_string(path)?;
    let formatted = match format_source(&original, settings) {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "{} {}: {}",
                style("Error").red().bold(),
                path.display(),
                e
            );
            return Ok(false);
        }
    };

    if original != formatted {
        if check {
            println!(
                "{} {} is not formatted",
                style("✘").red(),
                path.display()
            );
            return Ok(true);
        } else {
            std::fs::write(path, formatted)?;
            if verbose {
                println!("{} Formatted {}", style("✓").green(), path.display());
            }
        }
    }

    Ok(false)
}
