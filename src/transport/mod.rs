pub mod http;

pub trait SayaTransport {
    fn health(&self, base_url: &str) -> String;
    fn chat(&self, base_url: &str, message: &str) -> String;
}
