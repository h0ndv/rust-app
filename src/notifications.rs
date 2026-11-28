use contracts::{DashboardDto, SettingsDto};

use crate::api::Api;
use crate::i18n::{t, tn};

pub fn should_notify(settings: &SettingsDto, dashboard: &DashboardDto) -> Option<(String, String)> {
    if !settings.notifications {
        return None;
    }

    let overdue = i64::try_from(dashboard.overdue.len()).ok()?;
    let due_soon = i64::try_from(dashboard.due_soon.len()).ok()?;

    let mut parts = Vec::new();
    if overdue > 0 {
        parts.push(tn("notifications.overdue", overdue));
    }
    if due_soon > 0 {
        parts.push(tn("notifications.dueSoon", due_soon));
    }

    if parts.is_empty() {
        return None;
    }

    Some((t("notifications.title"), parts.join(" · ")))
}

pub async fn ensure_notification_permission(api: &Api) -> bool {
    if api.notification_permission().await.unwrap_or(false) {
        return true;
    }

    api.request_notification_permission().await.unwrap_or(false)
}
