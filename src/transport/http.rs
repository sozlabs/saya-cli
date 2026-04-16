use super::SayaTransport;

pub struct HttpTransport;

impl SayaTransport for HttpTransport {
    fn health(&self, base_url: &str) -> String {
        format!("health stub: would GET {base_url}/health")
    }

    fn chat(&self, base_url: &str, message: &str) -> String {
        format!("chat stub via {base_url}: {message}")
    }
}
