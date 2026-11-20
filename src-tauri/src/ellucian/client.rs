use std::sync::Arc;

use serde_json::json;

use crate::auth::{SessionManager, Source};
use crate::ellucian::models::{AcademicPeriod, EllucianClass, InstructionalEvent};
use crate::ellucian::parser;
use crate::ellucian::queries::{
    CLASSES_QUERY, ETHOS_URL, EthosQuery, PERIODS_QUERY, SCHEDULE_QUERY,
};
use crate::error::AppError;
use crate::http::HttpClient;

pub struct EllucianClient {
    http: HttpClient,
    session: Arc<SessionManager>,
}

impl EllucianClient {
    pub fn new(http: HttpClient, session: Arc<SessionManager>) -> Self {
        Self { http, session }
    }

    async fn query(
        &self,
        query_id: &str,
        properties: serde_json::Value,
    ) -> Result<String, AppError> {
        let cookies = self.session.cookie_header(Source::Ellucian).await?;
        let body = EthosQuery::new(query_id, properties);

        let response = self.http.post_json(ETHOS_URL, &cookies, &body).await?;
        let text = crate::http::read_text(response).await?;

        if text.trim_start().starts_with('<') {
            return Err(AppError::Unauthorized);
        }
        Ok(text)
    }

    pub async fn fetch_periods(&self) -> Result<Vec<AcademicPeriod>, AppError> {
        let body = self.query(PERIODS_QUERY, json!({})).await?;
        parser::parse_periods_response(&body)
    }

    pub async fn fetch_classes_for(&self, period_id: &str) -> Result<Vec<EllucianClass>, AppError> {
        let body = self
            .query(CLASSES_QUERY, json!({ "academicPeriodId": period_id }))
            .await?;
        parser::parse_classes_response(&body)
    }

    pub async fn fetch_classes(&self) -> Result<Vec<EllucianClass>, AppError> {
        let periods = self.fetch_periods().await?;
        let current = parser::current_period(&periods, chrono::Utc::now().timestamp())
            .ok_or(AppError::NotFound)?;
        self.fetch_classes_for(&current.id).await
    }

    /// The schedule is requested per section, so it first resolves the classes
    /// of the term and then queries each one.
    pub async fn fetch_schedule(&self) -> Result<Vec<InstructionalEvent>, AppError> {
        let classes = self.fetch_classes().await?;
        self.fetch_schedule_for(&classes).await
    }

    pub async fn fetch_schedule_for(
        &self,
        classes: &[EllucianClass],
    ) -> Result<Vec<InstructionalEvent>, AppError> {
        let mut out = Vec::new();

        for class in classes {
            let body = self
                .query(SCHEDULE_QUERY, json!({ "sectionId": class.section_id }))
                .await?;
            let mut events = parser::parse_schedule_response(&body)?;
            for event in &mut events {
                event.section_id = Some(class.section_id.clone());
                event.title = Some(class.title.clone());
                event.term = Some(class.term.clone());
            }
            out.extend(events);
        }

        Ok(out)
    }
}
