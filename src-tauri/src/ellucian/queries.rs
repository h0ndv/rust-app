use serde::Serialize;
use serde_json::Value;

pub const ETHOS_URL: &str = "https://experience.elluciancloud.com/ustc/api/ethos-query";

/// Identifiers of the "Clases" extension of Ellucian Experience. The
/// endpoint does not take GraphQL: it takes this descriptor plus `queryId`.
pub const EXTENSION_ID: &str = "6f147f85-1a0f-4d88-884c-ada96a44bdb9";
pub const CARD_ID: &str = "001G000000oShyJIAS|Ellucian-LAC|Clases|Clases";
pub const MANIFEST_TYPE: &str = "Clases";

pub const PERIODS_QUERY: &str = "student-academic-periods";
pub const CLASSES_QUERY: &str = "section-registrations-by-academic-period";
pub const SCHEDULE_QUERY: &str = "instructional-events-by-section";
pub const GRADES_QUERY: &str = "student-transcript-grades";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EthosQuery<'a> {
    pub extension_id: &'a str,
    pub card_id: &'a str,
    pub manifest_type: &'a str,
    pub query_id: &'a str,
    pub properties: Value,
}

impl<'a> EthosQuery<'a> {
    pub fn new(query_id: &'a str, properties: Value) -> Self {
        Self {
            extension_id: EXTENSION_ID,
            card_id: CARD_ID,
            manifest_type: MANIFEST_TYPE,
            query_id,
            properties,
        }
    }
}
