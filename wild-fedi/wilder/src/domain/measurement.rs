use chrono::{DateTime, Utc};
use uuid::Uuid;

/// One numeric reading in a group's environmental time series.
#[derive(Debug, Clone)]
pub struct Measurement {
    pub id: Uuid,
    pub group_username: String,
    pub metric: String,
    pub value: f64,
    pub unit: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub recorded_by: Option<Uuid>,
}

/// Input for recording a reading. `recorded_at` defaults to now when omitted.
#[derive(Debug, Clone)]
pub struct NewMeasurement {
    pub metric: String,
    pub value: f64,
    pub unit: Option<String>,
    pub recorded_at: Option<DateTime<Utc>>,
}
