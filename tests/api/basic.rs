#[tokio::test]
async fn index_returns_200_ok() {
    let app = crate::helpers::TestApp::spawn().await;
    let client = reqwest::Client::new();
    let response = client
        .get(app.url(""))
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success());
}
