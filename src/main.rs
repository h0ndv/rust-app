fn main() {
    #[cfg(target_arch = "wasm32")]
    dioxus::launch(rust_app_ui::App);
}
