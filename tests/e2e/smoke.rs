//! Windows end-to-end smoke test for the release app.
//!
//! This is a real WebDriver harness, but it is ignored by default because it
//! needs external tooling and a prepared app profile:
//!
//! 1. `cargo install tauri-driver --locked`
//! 2. Microsoft Edge Driver matching the installed WebView2/Edge, on `PATH`
//!    (`msedgedriver.exe`, or pass `--native-driver` to `tauri-driver`).
//! 3. A built app (`cargo tauri build` or `cargo tauri build --debug
//!    --no-bundle`). Set `RUST_APP_E2E_BINARY` to point at it; the default is
//!    `target/release/rust-app.exe`.
//! 4. A disposable Windows account or VM, or a copy of both the app database
//!    and the WebView2 profile. Launching the release app uses the real
//!    `%APPDATA%\com.santotomas.rustapp` SQLite database and the real WebView2
//!    profile, so the run **may write** settings, session state, window state
//!    and sync rows. Setting `APPDATA`/`LOCALAPPDATA` for the driver process is
//!    not enough: Tauri resolves the Windows known folders. Do not point this
//!    at the daily profile. If a disposable account is not possible, back up
//!    `%APPDATA%\com.santotomas.rustapp` and
//!    `%LOCALAPPDATA%\com.santotomas.rustapp`, run the test, then restore both.
//! 5. An app profile that already completed onboarding, or the app starts on
//!    the onboarding screen and the sidebar never appears.
//!
//! Run it with:
//! `cargo test -p rust-app-ui --test e2e -- --ignored --nocapture`
#![cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]

use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use thirtyfour::prelude::*;

const WEBDRIVER_PORT: u16 = 4444;
const NAV_PATHS: [&str; 9] = [
    "/",
    "/ramos",
    "/tareas",
    "/recursos",
    "/notas",
    "/horario",
    "/novedades",
    "/buscar",
    "/ajustes",
];

/// Kills `tauri-driver` even when an assertion panics.
struct DriverProcess(Child);

impl DriverProcess {
    fn start() -> Self {
        let binary = std::env::var("TAURI_DRIVER").unwrap_or_else(|_| "tauri-driver".into());
        let child = Command::new(&binary)
            .args(["--port", &WEBDRIVER_PORT.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap_or_else(|error| {
                panic!(
                    "could not start {binary}: {error}. Install it with `cargo install tauri-driver --locked`."
                )
            });

        Self(child)
    }
}

impl Drop for DriverProcess {
    fn drop(&mut self) {
        // `msedgedriver.exe` and the app are children of tauri-driver, so a
        // plain kill would leave them running after a failed assertion.
        let pid = self.0.id().to_string();
        let _ = Command::new("taskkill")
            .args(["/PID", &pid, "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn application_path() -> PathBuf {
    let path = std::env::var("RUST_APP_E2E_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target/release/rust-app.exe"));
    let path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .expect("current directory")
            .join(path)
    };

    assert!(
        path.is_file(),
        "built app not found at {}. Run `cargo tauri build` or set RUST_APP_E2E_BINARY.",
        path.display()
    );

    path
}

fn wait_for_driver() {
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", WEBDRIVER_PORT)).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    panic!("tauri-driver did not listen on 127.0.0.1:{WEBDRIVER_PORT}");
}

fn webdriver_port_is_busy() -> bool {
    TcpStream::connect(("127.0.0.1", WEBDRIVER_PORT)).is_ok()
}

#[tokio::test]
#[ignore = "requires tauri-driver and a built app"]
async fn sidebar_has_nine_items_and_grades_route_renders() {
    assert!(
        !webdriver_port_is_busy(),
        "something is already listening on 127.0.0.1:{WEBDRIVER_PORT}; stop the existing \
         tauri-driver or WebDriver server before running this test"
    );

    let _tauri_driver = DriverProcess::start();
    wait_for_driver();

    let application = application_path();
    let mut capabilities = Capabilities::new();
    capabilities
        .set("browserName", "wry")
        .expect("browserName capability");
    capabilities
        .set(
            "tauri:options",
            serde_json::json!({ "application": application.to_string_lossy() }),
        )
        .expect("tauri:options capability");

    let driver = WebDriver::new(format!("http://127.0.0.1:{WEBDRIVER_PORT}"), capabilities)
        .await
        .expect("webdriver session");

    let title = driver.title().await.expect("window title");
    assert_eq!(title, "rust-app", "unexpected window title");

    let sidebar_deadline = Instant::now() + Duration::from_secs(30);
    let links = loop {
        let links = driver
            .find_all(By::Css("aside nav a"))
            .await
            .expect("sidebar links");
        if links.len() == 9 {
            break links;
        }
        assert!(
            Instant::now() < sidebar_deadline,
            "the sidebar did not appear; the app may be on onboarding. Complete onboarding \
             in this profile first."
        );
        tokio::time::sleep(Duration::from_millis(200)).await;
    };

    let mut paths = Vec::new();
    for link in &links {
        if let Some(href) = link.attr("href").await.expect("link href") {
            paths.push(href);
        }
    }
    for expected in NAV_PATHS {
        assert!(
            paths.iter().any(|href| href.ends_with(expected)),
            "sidebar is missing {expected}; links were {paths:?}"
        );
    }

    driver
        .find(By::Css("aside nav a[href='/notas']"))
        .await
        .expect("grades link")
        .click()
        .await
        .expect("click the grades link");

    let route_deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let url = driver.current_url().await.expect("current url");
        if url.path().ends_with("/notas") {
            break;
        }
        assert!(
            Instant::now() < route_deadline,
            "the router did not navigate to /notas, url is {url}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let heading_deadline = Instant::now() + Duration::from_secs(15);
    let heading = loop {
        let headings = driver
            .find_all(By::Css("main h2"))
            .await
            .expect("screen headings");
        let mut found = None;
        for heading in headings {
            let text = heading.text().await.expect("heading text");
            if text == "Grades" || text == "Notas" {
                found = Some(text);
                break;
            }
        }
        if let Some(text) = found {
            break text;
        }
        assert!(
            Instant::now() < heading_deadline,
            "the grades heading never rendered"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    };
    assert!(
        heading == "Grades" || heading == "Notas",
        "unexpected grades heading {heading:?}"
    );

    driver.quit().await.expect("close the webdriver session");
}
