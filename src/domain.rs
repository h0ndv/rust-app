use contracts::Source;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SyncState {
    #[default]
    Idle,
    Running,
    Ok,
    Partial,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Overdue,
    Pending,
    Submitted,
    Graded,
    NoData,
    Unconfirmed,
    Informational,
}

/// Localization keys in declaration order, so `TASK_STATUS_KEY[status as usize]`
/// is valid.
pub const TASK_STATUS_KEY: [&str; 7] = [
    "status.overdue",
    "status.pending",
    "status.submitted",
    "status.graded",
    "status.no_data",
    "status.unconfirmed",
    "status.informational",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Announcement,
    Resource,
    File,
    Activity,
    Other,
}

/// Localization keys in declaration order, so `ITEM_KIND_KEY[kind as usize]`
/// is valid.
pub const ITEM_KIND_KEY: [&str; 5] = [
    "kinds.announcement",
    "kinds.resource",
    "kinds.file",
    "kinds.activity",
    "kinds.other",
];

/// Display labels in `Source` declaration order: Moodle, Ellucian, Banner.
pub const SOURCE_LABEL: [&str; 3] = ["Moodle", "Ellucian", "Banner"];

pub fn source_label(source: Source) -> &'static str {
    SOURCE_LABEL[source as usize]
}

pub fn task_status(value: Option<&str>) -> Option<TaskStatus> {
    let value = value.filter(|value| !value.is_empty())?;

    let status = match value {
        "atrasada" => TaskStatus::Overdue,
        "pendiente" => TaskStatus::Pending,
        "entregada" => TaskStatus::Submitted,
        "calificada" => TaskStatus::Graded,
        "sin_datos" => TaskStatus::NoData,
        "por_confirmar" => TaskStatus::Unconfirmed,
        "informativa" => TaskStatus::Informational,
        "overdue" => TaskStatus::Overdue,
        "pending" => TaskStatus::Pending,
        "submitted" => TaskStatus::Submitted,
        "graded" => TaskStatus::Graded,
        "no_data" => TaskStatus::NoData,
        "unconfirmed" => TaskStatus::Unconfirmed,
        "informational" => TaskStatus::Informational,
        _ => return None,
    };

    Some(status)
}

/// Unknown kinds collapse to `Other`, matching the React view; an absent kind
/// stays absent.
pub fn item_kind(value: Option<&str>) -> Option<ItemKind> {
    let value = value.filter(|value| !value.is_empty())?;

    let kind = match value {
        "anuncio" => ItemKind::Announcement,
        "recurso" => ItemKind::Resource,
        "archivo" => ItemKind::File,
        "announcement" => ItemKind::Announcement,
        "resource" => ItemKind::Resource,
        "file" => ItemKind::File,
        "activity" => ItemKind::Activity,
        _ => ItemKind::Other,
    };

    Some(kind)
}
