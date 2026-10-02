use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::{Account, Measurement, NewMeasurement};
use crate::error::AppError;
use crate::repository::{GroupRepository, MeasurementRepository};

/// Recording and reading a group's environmental measurements. Any member —
/// including a capture-station (`Service`) account sampling automatically — may
/// record; the readings are shaped into per-metric series for the charts.
pub struct MeasurementService {
    groups: Arc<dyn GroupRepository>,
    measurements: Arc<dyn MeasurementRepository>,
}

/// One chartable series: a metric, its unit, and its points over time.
#[derive(Serialize)]
pub struct Series {
    pub metric: String,
    pub unit: Option<String>,
    pub points: Vec<Point>,
}

#[derive(Serialize)]
pub struct Point {
    pub t: String, // RFC3339 timestamp
    pub v: f64,
}

impl MeasurementService {
    pub fn new(
        groups: Arc<dyn GroupRepository>,
        measurements: Arc<dyn MeasurementRepository>,
    ) -> Self {
        Self {
            groups,
            measurements,
        }
    }

    /// Record a reading. Requires the caller to be a member of the group.
    pub async fn record(
        &self,
        recorder: &Account,
        group_username: &str,
        input: NewMeasurement,
    ) -> Result<(), AppError> {
        self.groups
            .role_of(group_username, recorder.id)
            .await?
            .ok_or_else(|| AppError::Forbidden("not a member of this group".to_string()))?;
        if input.metric.trim().is_empty() {
            return Err(AppError::BadRequest("metric is required".to_string()));
        }

        let measurement = Measurement {
            id: Uuid::new_v4(),
            group_username: group_username.to_string(),
            metric: input.metric.trim().to_string(),
            value: input.value,
            unit: input.unit.filter(|u| !u.trim().is_empty()),
            recorded_at: input.recorded_at.unwrap_or_else(Utc::now),
            recorded_by: Some(recorder.id),
        };
        self.measurements.insert(&measurement).await?;
        Ok(())
    }

    /// Group a group's readings into per-metric series, ready for Chart.js. The
    /// unit is taken from the first reading of each metric. An empty result is
    /// the signal the UI uses to render *no* chart section.
    pub async fn series(&self, group_username: &str) -> Result<Vec<Series>, AppError> {
        let rows = self.measurements.for_group(group_username).await?;
        // BTreeMap keeps metrics in a stable, alphabetical order.
        let mut by_metric: BTreeMap<String, Series> = BTreeMap::new();
        for m in rows {
            let entry = by_metric.entry(m.metric.clone()).or_insert_with(|| Series {
                metric: m.metric.clone(),
                unit: m.unit.clone(),
                points: Vec::new(),
            });
            entry.points.push(Point {
                t: m.recorded_at.to_rfc3339(),
                v: m.value,
            });
        }
        Ok(by_metric.into_values().collect())
    }
}
