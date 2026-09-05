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

//! Tiny deterministic code highlighter. Pure string processing, so it produces
//! identical output during SSR and hydration.

const RUST_KEYWORDS: &[&str] = &[
    "as",
    "async",
    "await",
    "const",
    "derive",
    "dyn",
    "else",
    "enum",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "return",
    "self",
    "struct",
    "trait",
    "true",
    "false",
    "type",
    "use",
    "view",
    "view_route",
    "where",
    "while",
];

/// HTML-escape a raw string.
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Strip the SPDX license header block from a source file, returning the
/// implementation. Used so copied snippets (blocks/components) show only the
/// code, like shadcn/ui does.
pub fn strip_license(source: &str) -> &str {
    const END_MARKER: &str = "// SOFTWARE.\n\n";
    source
        .rfind(END_MARKER)
        .map(|idx| &source[idx + END_MARKER.len()..])
        .unwrap_or(source)
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn wrap(class: &str, inner: &str, out: &mut String) {
    out.push_str("<span class=\"");
    out.push_str(class);
    out.push_str("\">");
    out.push_str(inner);
    out.push_str("</span>");
}

/// Syntax-highlight a Rust snippet into HTML. Handles `//` comments,
/// `"strings"`, `#[...]` attributes, numbers, and a keyword list.
pub fn highlight_rust(source: &str) -> String {
    let mut out = String::with_capacity(source.len() + 128);
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    let n = chars.len();

    while i < n {
        let c = chars[i];

        // Line comments
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            let start = i;
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            let (bs, be) = (src_idx(&chars, start), src_idx(&chars, i));
            wrap("token-comment", &escape_html(&source[bs..be]), &mut out);
            continue;
        }

        // String literals (double-quoted; handles basic escapes)
        if c == '"' {
            let start = i;
            i += 1;
            while i < n {
                if chars[i] == '\\' && i + 1 < n {
                    i += 2;
                    continue;
                }
                if chars[i] == '"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let (bs, be) = (src_idx(&chars, start), src_idx(&chars, i));
            wrap("token-string", &escape_html(&source[bs..be]), &mut out);
            continue;
        }

        // Attributes: `#[...]` — highlight the brackets, leave the rest plain
        if c == '#' && i + 1 < n && chars[i + 1] == '[' {
            out.push_str("<span class=\"token-keyword\">#[</span>");
            i += 2;
            let mut depth = 1;
            while i < n && depth > 0 {
                if chars[i] == '[' {
                    depth += 1;
                } else if chars[i] == ']' {
                    depth -= 1;
                }
                i += 1;
            }
            out.push_str("<span class=\"token-keyword\">]</span>");
            continue;
        }

        // Identifiers / keywords
        if is_ident_start(c) {
            let start = i;
            while i < n && is_ident_continue(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let escaped = escape_html(&word);
            if RUST_KEYWORDS.contains(&word.as_str()) {
                wrap("token-keyword", &escaped, &mut out);
            } else if word
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_uppercase())
            {
                wrap("token-type", &escaped, &mut out);
            } else {
                out.push_str(&escaped);
            }
            continue;
        }

        // Numbers
        if c.is_ascii_digit() {
            let start = i;
            while i < n
                && (chars[i].is_ascii_digit()
                    || chars[i] == '_'
                    || chars[i] == '.'
                    || chars[i] == 'x'
                    || chars[i] == 'e')
            {
                i += 1;
            }
            let (bs, be) = (src_idx(&chars, start), src_idx(&chars, i));
            wrap("token-number", &escape_html(&source[bs..be]), &mut out);
            continue;
        }

        out.push(c);
        i += 1;
    }

    out
}

/// Escape + highlight a terminal-ish snippet (keeps dashes and brackets plain).
#[allow(dead_code)]
pub fn highlight_terminal(source: &str) -> String {
    escape_html(source)
}

/// Convert a char index in the Vec back to a byte index in the original str.
fn src_idx(chars: &[char], char_idx: usize) -> usize {
    chars.iter().take(char_idx).map(|c| c.len_utf8()).sum()
}
