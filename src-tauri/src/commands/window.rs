use tauri::Theme;

fn map_theme(theme: &str) -> Result<Option<Theme>, String> {
    match theme {
        "system" => Ok(None),
        "light" => Ok(Some(Theme::Light)),
        "dark" => Ok(Some(Theme::Dark)),
        other => Err(format!("unknown theme: {other}")),
    }
}

#[tauri::command]
pub fn set_window_theme(window: tauri::WebviewWindow, theme: String) -> Result<(), String> {
    window
        .set_theme(map_theme(&theme)?)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::map_theme;
    use tauri::Theme;

    #[test]
    fn maps_the_three_preferences() {
        assert_eq!(map_theme("system"), Ok(None));
        assert_eq!(map_theme("light"), Ok(Some(Theme::Light)));
        assert_eq!(map_theme("dark"), Ok(Some(Theme::Dark)));
    }

    #[test]
    fn rejects_unknown_values() {
        assert!(map_theme("blue").is_err());
    }
}
