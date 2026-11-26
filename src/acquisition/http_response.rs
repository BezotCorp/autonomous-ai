use std::collections::BTreeMap;

/// Response of an HTTP exchange as printed by a command-line client with
/// its headers (`HTTP/x status`, headers, blank line, body).
#[derive(Debug)]
pub(crate) struct HttpResponse {
    status: u16,
    headers: BTreeMap<String, String>,
    body: String,
}

impl HttpResponse {
    pub(crate) fn parse(output: &str) -> Result<Self, String> {
        let (head, body) = output
            .split_once("\r\n\r\n")
            .or_else(|| output.split_once("\n\n"))
            .unwrap_or((output, ""));

        let mut lines = head.lines();
        let status_line = lines.next().ok_or("Empty HTTP response")?;

        let status = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|status| status.parse().ok())
            .ok_or_else(|| format!("Invalid HTTP status line: {status_line}"))?;

        let headers = lines
            .filter_map(|line| line.split_once(':'))
            .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_owned()))
            .collect();

        Ok(Self {
            status,
            headers,
            body: body.to_owned(),
        })
    }

    pub(crate) fn status(&self) -> u16 {
        self.status
    }

    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }

    pub(crate) fn body(&self) -> &str {
        &self.body
    }

    pub(crate) fn into_body(self) -> String {
        self.body
    }

    /// Target of the `rel="next"` link of a paginated response.
    pub(crate) fn next_page(&self) -> Option<&str> {
        self.header("link")?.split(',').find_map(|link| {
            let (target, relation) = link.split_once(';')?;

            relation
                .contains("rel=\"next\"")
                .then(|| target.trim().trim_start_matches('<').trim_end_matches('>'))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::HttpResponse;

    #[test]
    fn parses_status_headers_and_links() {
        let response = HttpResponse::parse(
            "HTTP/2.0 200 OK\r\nEtag: W/\"x\"\r\nLink: <https://h/p?page=2>; rel=\"next\", <https://h/p?page=3>; rel=\"last\"\r\n\r\n[1]",
        )
        .unwrap();

        assert_eq!(response.status(), 200);
        assert_eq!(response.header("etag"), Some("W/\"x\""));
        assert_eq!(response.next_page(), Some("https://h/p?page=2"));
        assert_eq!(response.body(), "[1]");
    }
}
