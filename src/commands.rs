use crate::transport::SayaTransport;

pub fn run_version() -> String {
    format!("saya-cli {}", env!("CARGO_PKG_VERSION"))
}

pub fn run_health(transport: &dyn SayaTransport, base_url: &str) -> String {
    transport.health(base_url)
}

pub fn run_chat(transport: &dyn SayaTransport, base_url: &str, message: &str) -> String {
    transport.chat(base_url, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::SayaTransport;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockTransport {
        calls: Mutex<Vec<String>>,
    }

    impl SayaTransport for MockTransport {
        fn health(&self, base_url: &str) -> String {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push(format!("health:{base_url}"));
            "ok".to_string()
        }

        fn chat(&self, base_url: &str, message: &str) -> String {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push(format!("chat:{base_url}:{message}"));
            "done".to_string()
        }
    }

    #[test]
    fn health_routes_through_transport() {
        let transport = MockTransport::default();
        let result = run_health(&transport, "http://127.0.0.1:3010");
        assert_eq!(result, "ok");
        let calls = transport.calls.lock().expect("lock poisoned");
        assert_eq!(calls.as_slice(), ["health:http://127.0.0.1:3010"]);
    }

    #[test]
    fn chat_routes_through_transport() {
        let transport = MockTransport::default();
        let result = run_chat(&transport, "http://127.0.0.1:3010", "hello");
        assert_eq!(result, "done");
        let calls = transport.calls.lock().expect("lock poisoned");
        assert_eq!(calls.as_slice(), ["chat:http://127.0.0.1:3010:hello"]);
    }
}
