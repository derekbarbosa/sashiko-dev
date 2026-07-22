// Copyright 2026 The Sashiko Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Helpers for fetching and parsing patches from lore.kernel.org.
//!
//! Provides detection of lore URLs and message-IDs, HTTP fetching of
//! raw mbox content, and conversion into [`ReviewInput`] for the local
//! review pipeline.

use anyhow::{Result, bail};

use crate::worker::{PatchInput, ReviewInput};

/// Characters that must be percent-encoded in a URL path segment when
/// embedding a message-ID.  Mirrors the set used in `api.rs` for
/// `fetch_and_inject_thread`.
const PATH_SEGMENT_ENCODE: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
    .add(b'/')
    .add(b'\\')
    .add(b'?')
    .add(b'#')
    .add(b' ')
    .add(b'%');

/// Returns `true` when `input` looks like a lore.kernel.org URL.
///
/// Only matches when the hostname is exactly `lore.kernel.org` (not a
/// substring of another domain).
pub fn is_lore_url(input: &str) -> bool {
    input.starts_with("https://lore.kernel.org") || input.starts_with("http://lore.kernel.org")
}

/// Returns `true` when `input` looks like an RFC 5322 message-ID
/// rather than a git ref, file path, or commit range.
///
/// Kernel message-IDs almost always have a dot in the local part
/// (e.g. `20231025202513.12358-1-tony.luck@intel.com`).  Requiring a
/// dot after `@` distinguishes them from bare `user@branch` patterns
/// that should be treated as git refs.
pub fn is_message_id(input: &str) -> bool {
    if let Some(at_pos) = input.find('@') {
        let domain = &input[at_pos + 1..];
        // Must have '@', domain must contain '.', and the input must
        // not look like a path or range.
        domain.contains('.')
            && !input.contains('/')
            && !input.contains('\\')
            && !input.contains("..")
    } else {
        false
    }
}

/// Extracts the message-ID component from a lore.kernel.org URL.
///
/// Handles patterns such as:
/// - `https://lore.kernel.org/all/<msgid>/`
/// - `https://lore.kernel.org/all/<msgid>`
/// - `https://lore.kernel.org/<list>/<msgid>/raw`
pub fn extract_message_id_from_lore_url(url: &str) -> Option<String> {
    let stripped = url
        .strip_prefix("https://lore.kernel.org")
        .or_else(|| url.strip_prefix("http://lore.kernel.org"))?;

    let path = stripped.trim_start_matches('/');
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // Walk segments in reverse, skipping known suffixes like "raw" or
    // "t.mbox.gz", and return the first segment that looks like a
    // message-ID (contains '@').
    for seg in segments.iter().rev() {
        if *seg == "raw" || *seg == "t.mbox.gz" {
            continue;
        }
        let cleaned = seg.trim_start_matches('<').trim_end_matches('>');
        if cleaned.contains('@') {
            return Some(cleaned.to_string());
        }
    }
    None
}

/// Maximum compressed download size (10 MiB, same as the daemon).
const MAX_MBOX_DOWNLOAD: usize = 10 * 1024 * 1024;

/// Maximum decompressed mbox size (50 MiB, same as the daemon).
const MAX_MBOX_DECOMPRESSED: u64 = 50 * 1024 * 1024;

/// Fetches the raw mbox for a single message from lore.kernel.org.
///
/// Uses the `/t.mbox.gz` endpoint (gzipped thread archive) rather than
/// `/raw` because lore's Anubis bot protection blocks `/raw` requests
/// but allows `/t.mbox.gz`.  The full thread is downloaded and
/// decompressed, then the individual message matching `message_id` is
/// extracted.
///
/// The message-ID is percent-encoded to handle IDs that contain
/// path-significant characters such as `/`.
pub async fn fetch_mbox_from_lore(message_id: &str) -> Result<String> {
    let clean_id = message_id
        .trim_start_matches('<')
        .trim_end_matches('>');
    let encoded_id =
        percent_encoding::utf8_percent_encode(&clean_id, PATH_SEGMENT_ENCODE).to_string();
    let url = format!("https://lore.kernel.org/all/{}/t.mbox.gz", encoded_id);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let resp = client.get(&url).send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        bail!("message not found on lore.kernel.org: {}", clean_id);
    }
    if !resp.status().is_success() {
        bail!("lore.kernel.org returned HTTP {}", resp.status());
    }

    let bytes = resp.bytes().await?;
    if bytes.len() > MAX_MBOX_DOWNLOAD {
        bail!(
            "compressed mbox too large ({} bytes, limit {} bytes)",
            bytes.len(),
            MAX_MBOX_DOWNLOAD
        );
    }

    // Decompress the gzipped mbox.  Mirrors the approach used by
    // fetch_and_inject_thread in api.rs.
    let raw = tokio::task::spawn_blocking(move || -> Result<String> {
        use std::io::Read;
        let decoder = flate2::read::GzDecoder::new(&bytes[..]);
        let mut limited = decoder.take(MAX_MBOX_DECOMPRESSED);
        let mut raw_bytes = Vec::new();
        limited.read_to_end(&mut raw_bytes)?;

        if raw_bytes.len() as u64 == MAX_MBOX_DECOMPRESSED {
            let mut probe = [0u8; 1];
            if limited.into_inner().read(&mut probe).unwrap_or(0) > 0 {
                bail!(
                    "decompressed mbox exceeds {} byte limit",
                    MAX_MBOX_DECOMPRESSED
                );
            }
        }
        String::from_utf8(raw_bytes).map_err(|e| anyhow::anyhow!("invalid UTF-8 in mbox: {}", e))
    })
    .await??;

    // The thread mbox contains all messages in the thread.  Extract
    // only the message that matches our target message-ID.
    extract_message_from_mbox(&raw, &clean_id)
}

/// Splits a concatenated mbox into individual messages and returns the
/// one whose `Message-ID` header matches `target_id`.
fn extract_message_from_mbox(mbox: &str, target_id: &str) -> Result<String> {
    // mboxrd format: messages are separated by "From " lines at the
    // start of a line.  Split on that boundary.
    let mut messages: Vec<&str> = Vec::new();
    let mut start = 0;

    for (i, _) in mbox.match_indices("\nFrom ") {
        // The boundary is at i+1 (the 'F' after the newline).
        if start < i + 1 {
            messages.push(&mbox[start..i + 1]);
        }
        start = i + 1;
    }
    if start < mbox.len() {
        messages.push(&mbox[start..]);
    }

    // If the mbox starts with "From " (no preceding newline), the
    // first split might miss it.  Handle a single-message mbox.
    if messages.is_empty() && !mbox.is_empty() {
        messages.push(mbox);
    }

    let target_lower = target_id.to_lowercase();
    for msg in &messages {
        // Look for Message-ID header (case-insensitive).
        for line in msg.lines() {
            if line.is_empty() {
                // End of headers.
                break;
            }
            if let Some(rest) = line
                .strip_prefix("Message-ID:")
                .or_else(|| line.strip_prefix("Message-Id:"))
                .or_else(|| line.strip_prefix("message-id:"))
            {
                let id = rest
                    .trim()
                    .trim_start_matches('<')
                    .trim_end_matches('>')
                    .to_lowercase();
                if id == target_lower {
                    return Ok(msg.to_string());
                }
            }
        }
    }

    bail!(
        "message {} not found in thread mbox ({} messages in thread)",
        target_id,
        messages.len()
    )
}

/// Parses a raw mbox email into a [`ReviewInput`] suitable for the
/// local review pipeline.
///
/// The returned [`PatchInput`] entries have `commit_id: None` because
/// mbox-sourced patches do not originate from a local git history.
/// The caller must use `current_tree: false` when passing the result
/// to the review worker so that patches are applied via `git am`.
pub fn build_review_input_from_mbox(raw_mbox: &str) -> Result<ReviewInput> {
    let (metadata, patch) = crate::patch::parse_email(raw_mbox.as_bytes())?;

    let patch = patch.ok_or_else(|| {
        anyhow::anyhow!(
            "no patch content found in message {}. \
             The message may be a reply or cover letter without a diff.",
            metadata.message_id
        )
    })?;

    let subject = metadata.subject.clone();
    let patch_input = PatchInput {
        index: metadata.index as i64,
        diff: patch.diff,
        subject: Some(metadata.subject),
        author: Some(metadata.author),
        date: Some(metadata.date),
        message_id: Some(metadata.message_id),
        commit_id: None,
    };

    Ok(ReviewInput {
        id: 0,
        subject,
        patches: vec![patch_input],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_lore_url_positive() {
        assert!(is_lore_url(
            "https://lore.kernel.org/all/20231025202513.12358-1-tony.luck@intel.com/"
        ));
        assert!(is_lore_url("http://lore.kernel.org/linux-mm/abc@def.com/"));
        assert!(is_lore_url("https://lore.kernel.org/all/abc@def/raw"));
    }

    #[test]
    fn test_is_lore_url_negative() {
        assert!(!is_lore_url("HEAD"));
        assert!(!is_lore_url("HEAD~3..HEAD"));
        assert!(!is_lore_url("abc1234"));
        assert!(!is_lore_url("tony.luck@intel.com"));
        assert!(!is_lore_url("https://example.com/foo"));
        assert!(!is_lore_url("patches.mbox"));
        // Subdomains and embedded strings must not match.
        assert!(!is_lore_url("https://not-lore.kernel.org/all/foo@bar/"));
        assert!(!is_lore_url(
            "https://evil.com/redirect?url=lore.kernel.org"
        ));
    }

    #[test]
    fn test_is_message_id_positive() {
        assert!(is_message_id("tony.luck@intel.com"));
        assert!(is_message_id("20231025202513.12358-1-tony.luck@intel.com"));
        assert!(is_message_id("abc@def.org"));
    }

    #[test]
    fn test_is_message_id_negative() {
        assert!(!is_message_id("HEAD"));
        assert!(!is_message_id("abc1234"));
        assert!(!is_message_id("HEAD~3..HEAD"));
        assert!(!is_message_id("path/to/file@somewhere"));
        assert!(!is_message_id("path\\to\\file@somewhere"));
        assert!(!is_message_id("a@b..c"));
        // Git-style refs with @ but no dot in domain should NOT match.
        assert!(!is_message_id("user@branch"));
        assert!(!is_message_id("git@github"));
    }

    #[test]
    fn test_extract_message_id_with_trailing_slash() {
        let url = "https://lore.kernel.org/all/\
                    20231025202513.12358-1-tony.luck@intel.com/";
        assert_eq!(
            extract_message_id_from_lore_url(url),
            Some("20231025202513.12358-1-tony.luck@intel.com".into())
        );
    }

    #[test]
    fn test_extract_message_id_without_trailing_slash() {
        let url = "https://lore.kernel.org/all/\
                    20231025202513.12358-1-tony.luck@intel.com";
        assert_eq!(
            extract_message_id_from_lore_url(url),
            Some("20231025202513.12358-1-tony.luck@intel.com".into())
        );
    }

    #[test]
    fn test_extract_message_id_with_raw_suffix() {
        let url = "https://lore.kernel.org/all/\
                    20231025202513.12358-1-tony.luck@intel.com/raw";
        assert_eq!(
            extract_message_id_from_lore_url(url),
            Some("20231025202513.12358-1-tony.luck@intel.com".into())
        );
    }

    #[test]
    fn test_extract_message_id_list_path() {
        let url = "https://lore.kernel.org/linux-mm/abc@def.com/";
        assert_eq!(
            extract_message_id_from_lore_url(url),
            Some("abc@def.com".into())
        );
    }

    #[test]
    fn test_extract_message_id_angle_brackets() {
        let url = "https://lore.kernel.org/all/<abc@def.com>/";
        assert_eq!(
            extract_message_id_from_lore_url(url),
            Some("abc@def.com".into())
        );
    }

    #[test]
    fn test_extract_message_id_no_at_sign() {
        // A URL without a message-ID segment returns None.
        let url = "https://lore.kernel.org/all/";
        assert_eq!(extract_message_id_from_lore_url(url), None);
    }

    #[test]
    fn test_extract_message_id_not_lore() {
        let url = "https://example.com/all/abc@def/";
        assert_eq!(extract_message_id_from_lore_url(url), None);
    }

    #[test]
    fn test_build_review_input_from_mbox_valid() {
        // Minimal valid mbox with a diff.
        let mbox = "\
From: Test Author <test@example.com>\r\n\
Subject: [PATCH] test: add a simple test\r\n\
Date: Mon, 1 Jan 2024 00:00:00 +0000\r\n\
Message-ID: <test-msg@example.com>\r\n\
\r\n\
This is the commit message.\r\n\
\r\n\
---\r\n\
 test.c | 1 +\r\n\
 1 file changed, 1 insertion(+)\r\n\
\r\n\
diff --git a/test.c b/test.c\r\n\
index 0000000..1111111 100644\r\n\
--- a/test.c\r\n\
+++ b/test.c\r\n\
@@ -0,0 +1 @@\r\n\
+// new line\r\n\
-- \r\n\
2.34.1\r\n";

        let result = build_review_input_from_mbox(mbox);
        assert!(result.is_ok(), "parse failed: {:?}", result.err());

        let ri = result.unwrap();
        assert_eq!(ri.id, 0);
        assert!(!ri.patches.is_empty());

        let p = &ri.patches[0];
        assert!(p.commit_id.is_none());
        assert!(p.subject.is_some());
        assert!(p.author.is_some());
        assert!(!p.diff.is_empty());
    }

    #[test]
    fn test_build_review_input_from_mbox_no_diff() {
        // A plain reply with no diff should produce an error.
        let mbox = "\
From: Reply Author <reply@example.com>\r\n\
Subject: Re: [PATCH] something\r\n\
Date: Mon, 1 Jan 2024 00:00:00 +0000\r\n\
Message-ID: <reply-msg@example.com>\r\n\
\r\n\
Looks good to me!\r\n";

        let result = build_review_input_from_mbox(mbox);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("no patch content"),
            "unexpected error: {}",
            err
        );
    }
}
