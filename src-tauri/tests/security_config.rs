use serde_json::Value;

const CONFIG: &str = include_str!("../tauri.conf.json");

fn config() -> Value {
    serde_json::from_str(CONFIG).unwrap()
}

#[test]
fn main_window_is_created_by_the_app() {
    assert_eq!(
        config()["app"]["windows"][0]["create"].as_bool(),
        Some(false),
        "the config window would skip the navigation guard"
    );
    assert_eq!(
        config()["app"]["windows"][0]["label"].as_str(),
        Some("main")
    );
}

#[test]
fn production_csp_locks_the_webview_down() {
    let csp = &config()["app"]["security"]["csp"];
    assert_eq!(csp["default-src"].as_str(), Some("'self'"));
    assert_eq!(
        csp["script-src"].as_str(),
        Some("'self' 'wasm-unsafe-eval'")
    );
    assert_eq!(csp["style-src"].as_str(), Some("'self' 'unsafe-inline'"));
    assert_eq!(csp["img-src"].as_str(), Some("'self' data:"));
    assert_eq!(csp["font-src"].as_str(), Some("'self'"));
    assert_eq!(
        csp["connect-src"].as_str(),
        Some("'self' ipc: http://ipc.localhost")
    );
    assert_eq!(csp["object-src"].as_str(), Some("'none'"));
    assert_eq!(csp["base-uri"].as_str(), Some("'self'"));
    assert_eq!(csp["form-action"].as_str(), Some("'none'"));
    assert_eq!(csp["frame-ancestors"].as_str(), Some("'none'"));
}

#[test]
fn dev_csp_allows_vite_hmr() {
    let csp = &config()["app"]["security"]["devCsp"];
    assert_eq!(
        csp["connect-src"].as_str(),
        Some("'self' ipc: http://ipc.localhost ws://localhost:1420")
    );
    assert_eq!(
        csp["script-src"].as_str(),
        Some("'self' 'unsafe-inline' 'unsafe-eval'")
    );
    assert_eq!(csp["default-src"].as_str(), Some("'self'"));
}
