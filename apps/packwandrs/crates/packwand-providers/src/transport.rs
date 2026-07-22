use std::io::Read;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
}

impl HttpRequest {
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: Vec::new(),
        }
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("GET {url} failed: {message}")]
pub struct TransportError {
    pub url: String,
    pub message: String,
    /// HTTP status code, when the failure was a non-2xx response rather than
    /// a connection/transport-level failure.
    pub status: Option<u16>,
    /// First ~500 characters of the response body, when available. Lets
    /// callers distinguish a real API error payload from a generic
    /// CDN/WAF block page (e.g. CloudFront's static "Request blocked" HTML),
    /// which returns the same status code but never reaches the actual API.
    pub body_snippet: Option<String>,
}

fn to_transport_error(url: &str, error: ureq::Error) -> TransportError {
    match error {
        ureq::Error::Status(code, response) => {
            let body_snippet = response
                .into_string()
                .ok()
                .map(|body| body.chars().take(500).collect());
            TransportError {
                url: url.to_owned(),
                message: format!("http status {code}"),
                status: Some(code),
                body_snippet,
            }
        }
        ureq::Error::Transport(inner) => TransportError {
            url: url.to_owned(),
            message: inner.to_string(),
            status: None,
            body_snippet: None,
        },
    }
}

pub trait Transport: Send + Sync {
    fn get(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError>;

    fn post_json(&self, request: HttpRequest, _body: &[u8]) -> Result<Vec<u8>, TransportError> {
        Err(TransportError {
            url: request.url,
            message: "transport does not support JSON POST requests".into(),
            status: None,
                body_snippet: None,
        })
    }
}

pub struct UreqTransport {
    agent: ureq::Agent,
    max_body_bytes: u64,
}

impl UreqTransport {
    pub fn new() -> Self {
        Self {
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(15))
                .timeout_read(Duration::from_secs(60))
                .user_agent(concat!("packwand/", env!("CARGO_PKG_VERSION")))
                .build(),
            max_body_bytes: 16 * 1024 * 1024,
        }
    }
}

impl Default for UreqTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for UreqTransport {
    fn get(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError> {
        let mut call = self.agent.get(&request.url);
        for (name, value) in &request.headers {
            call = call.set(name, value);
        }
        let response = call
            .call()
            .map_err(|error| to_transport_error(&request.url, error))?;
        let mut reader = response.into_reader().take(self.max_body_bytes + 1);
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .map_err(|error| TransportError {
                url: request.url.clone(),
                message: error.to_string(),
                status: None,
                body_snippet: None,
            })?;
        if bytes.len() as u64 > self.max_body_bytes {
            return Err(TransportError {
                url: request.url,
                message: format!("response exceeded the {} byte limit", self.max_body_bytes),
                status: None,
                body_snippet: None,
            });
        }
        Ok(bytes)
    }

    fn post_json(&self, request: HttpRequest, body: &[u8]) -> Result<Vec<u8>, TransportError> {
        let mut call = self
            .agent
            .post(&request.url)
            .set("Content-Type", "application/json");
        for (name, value) in &request.headers {
            call = call.set(name, value);
        }
        let response = call
            .send_bytes(body)
            .map_err(|error| to_transport_error(&request.url, error))?;
        let mut reader = response.into_reader().take(self.max_body_bytes + 1);
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .map_err(|error| TransportError {
                url: request.url.clone(),
                message: error.to_string(),
                status: None,
                body_snippet: None,
            })?;
        if bytes.len() as u64 > self.max_body_bytes {
            return Err(TransportError {
                url: request.url,
                message: format!("response exceeded the {} byte limit", self.max_body_bytes),
                status: None,
                body_snippet: None,
            });
        }
        Ok(bytes)
    }
}
