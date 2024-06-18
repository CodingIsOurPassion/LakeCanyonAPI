pub struct TestApp {
    pub socket_addr: std::net::SocketAddr,
}

impl TestApp {
    pub async fn spawn() -> Self {
        let api = lake_canyon_api::startup::Application::new();
        let listener = tokio::net::TcpListener::bind("127.0.0.0:0")
            .await
            .expect("Failed to bind random port!");

        let socket_addr = api.run(listener, false).await.expect("Failed to run API!");

        Self { socket_addr }
    }

    pub fn url(&self, path: &str) -> String {
        return format!(
            "http://{}:{}/{}",
            self.socket_addr.ip(),
            self.socket_addr.port(),
            path
        );
    }
}
