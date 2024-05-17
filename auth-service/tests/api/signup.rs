use crate::helpers::TestApp;

#[tokio::test]
async fn signup() {
    let app = TestApp::new().await;

    let response = app.get_signup().await;

    assert_eq!(response.status().as_u16(), 200);
}
