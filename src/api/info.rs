pub async fn route_info() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "routes": ["/register", "/login", "/user_profile", "/submit", "/retrieve"],
    }))
}
