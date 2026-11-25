use std::{io::Read, process::Command};

use crate::{
    documentation_status::DocumentationStatus, package_documentation::PackageDocumentation,
};

pub(crate) struct DocumentationProvider;

impl DocumentationProvider {
    const MAX_COMPRESSED_SIZE: usize = 32 * 1024 * 1024;
    const MAX_DECOMPRESSED_SIZE: u64 = 128 * 1024 * 1024;

    pub(crate) fn acquire(package: &str, version: &semver::Version) -> PackageDocumentation {
        let source = format!("https://docs.rs/crate/{package}/{version}/json.zst",);

        let mut documentation = PackageDocumentation::new(package, version, source);

        if package.is_empty()
            || !package
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            documentation.fail(
                DocumentationStatus::Unsupported,
                "Invalid registry package name",
            );

            return documentation;
        }

        let response = Command::new("curl")
            .args([
                "--silent",
                "--show-error",
                "--location",
                "--proto",
                "=https",
                "--proto-redir",
                "=https",
                "--connect-timeout",
                "10",
                "--max-time",
                "60",
                "--max-filesize",
                &Self::MAX_COMPRESSED_SIZE.to_string(),
                "--write-out",
                "\\n%{http_code}",
                documentation.source(),
            ])
            .output();

        let response = match response {
            Ok(response) => response,
            Err(error) => {
                documentation.fail(DocumentationStatus::Unavailable, error.to_string());

                return documentation;
            }
        };

        if !response.status.success() {
            documentation.fail(
                DocumentationStatus::Unavailable,
                String::from_utf8_lossy(&response.stderr).trim(),
            );

            return documentation;
        }

        let Some(separator) = response.stdout.iter().rposition(|byte| *byte == b'\n') else {
            documentation.fail(
                DocumentationStatus::Unavailable,
                "Missing HTTP response status",
            );

            return documentation;
        };

        let body = &response.stdout[..separator];
        let status = &response.stdout[separator + 1..];

        match status {
            b"200" => {}
            b"404" | b"410" => {
                documentation.fail(
                    DocumentationStatus::NotFound,
                    "Documentation is not available for this version",
                );

                return documentation;
            }
            status => {
                documentation.fail(
                    DocumentationStatus::Unavailable,
                    format!("HTTP {}", String::from_utf8_lossy(status),),
                );

                return documentation;
            }
        }

        let decoder = match zstd::stream::read::Decoder::new(body) {
            Ok(decoder) => decoder,
            Err(error) => {
                documentation.fail(DocumentationStatus::Unsupported, error.to_string());

                return documentation;
            }
        };

        let mut decoded = Vec::new();

        if let Err(error) = decoder
            .take(Self::MAX_DECOMPRESSED_SIZE + 1)
            .read_to_end(&mut decoded)
        {
            documentation.fail(DocumentationStatus::Unsupported, error.to_string());

            return documentation;
        }

        if decoded.len() as u64 > Self::MAX_DECOMPRESSED_SIZE {
            documentation.fail(
                DocumentationStatus::Unsupported,
                "Documentation exceeds the decompression budget",
            );

            return documentation;
        }

        match serde_json::from_slice(&decoded) {
            Ok(document) => documentation.integrate(document),
            Err(error) => documentation.fail(DocumentationStatus::Unsupported, error.to_string()),
        }

        documentation
    }
}
