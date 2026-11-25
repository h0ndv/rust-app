use url::Url;

use crate::error::AppError;

pub const TRUSTED_HOSTS: [&str; 3] = [
    "aulasvirtuales.santotomas.cl",
    "experience.elluciancloud.com",
    "selferviceprd.santotomas.cl",
];

pub fn trusted_source_url(url: &Url) -> bool {
    url.scheme() == "https"
        && url
            .host_str()
            .is_some_and(|host| TRUSTED_HOSTS.contains(&host))
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
}

pub fn parse_trusted(value: &str) -> Result<Url, AppError> {
    let url =
        Url::parse(value).map_err(|error| AppError::Internal(format!("invalid URL: {error}")))?;
    if trusted_source_url(&url) {
        Ok(url)
    } else {
        Err(AppError::Forbidden)
    }
}

pub fn app_url(url: &Url, dev: bool) -> bool {
    match (url.scheme(), url.host_str()) {
        ("tauri", Some("localhost")) => true,
        ("http" | "https", Some("tauri.localhost")) => url.port().is_none(),
        ("http", _) => dev && url.port() == Some(1420),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_the_three_https_sources() {
        for host in TRUSTED_HOSTS {
            let url = Url::parse(&format!("https://{host}/course/view.php?id=1")).unwrap();
            assert!(trusted_source_url(&url), "{host} must be trusted");
        }
        for rejected in [
            "http://aulasvirtuales.santotomas.cl/x",
            "https://aulasvirtuales.santotomas.cl.evil.com/x",
            "https://evil.com/x",
            "https://user:pass@aulasvirtuales.santotomas.cl/x",
            "https://aulasvirtuales.santotomas.cl:8443/x",
            "file:///C:/Windows/System32/calc.exe",
            "javascript:alert(1)",
        ] {
            assert!(
                !trusted_source_url(&Url::parse(rejected).unwrap()),
                "{rejected} must be rejected"
            );
        }
    }

    #[test]
    fn normalizes_the_host_and_accepts_the_default_port() {
        assert!(trusted_source_url(
            &Url::parse("https://AULASVIRTUALES.SANTOTOMAS.CL/x").unwrap()
        ));
        assert!(trusted_source_url(
            &Url::parse("https://aulasvirtuales.santotomas.cl:443/x").unwrap()
        ));
    }

    #[test]
    fn parse_trusted_reports_rejected_input() {
        let url = parse_trusted("https://experience.elluciancloud.com/ustc/").unwrap();
        assert_eq!(url.host_str(), Some("experience.elluciancloud.com"));

        assert!(parse_trusted("https://evil.com/").is_err());
        assert!(parse_trusted("not a url").is_err());
    }

    #[test]
    fn rejects_without_echoing_the_url() {
        let error = parse_trusted("https://evil.com/?token=secret").unwrap_err();
        assert!(matches!(error, AppError::Forbidden));
        assert!(!error.to_string().contains("secret"));
    }

    #[test]
    fn main_window_stays_on_the_app_origin() {
        for allowed in [
            "http://tauri.localhost/index.html",
            "https://tauri.localhost/",
            "tauri://localhost/index.html",
        ] {
            assert!(app_url(&Url::parse(allowed).unwrap(), false), "{allowed}");
        }
        for rejected in [
            "https://evil.com/",
            "http://tauri.localhost.evil.com/",
            "http://tauri.localhost:444/",
            "tauri://evil.com/",
        ] {
            assert!(
                !app_url(&Url::parse(rejected).unwrap(), false),
                "{rejected}"
            );
        }
    }

    #[test]
    fn dev_origin_is_allowed_only_in_dev() {
        let local = Url::parse("http://localhost:1420/").unwrap();
        assert!(app_url(&local, true));
        assert!(!app_url(&local, false));

        let lan = Url::parse("http://192.168.1.10:1420/").unwrap();
        assert!(app_url(&lan, true));
        assert!(!app_url(&lan, false));
    }
}
