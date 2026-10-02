-- Environmental measurements attached to a group's study: a numeric time series
-- (temperature, water pH, counts...). These drive the group's charts. Recorded
-- by members — including capture-station (Service) accounts that sample the
-- outside world automatically.
CREATE TABLE measurements (
    id             UUID PRIMARY KEY,
    group_username TEXT NOT NULL REFERENCES groups(username) ON DELETE CASCADE,
    metric         TEXT NOT NULL,                 -- "température", "pH"...
    value          DOUBLE PRECISION NOT NULL,
    unit           TEXT,                          -- "°C", "cm"... (may be empty)
    recorded_at    TIMESTAMPTZ NOT NULL,          -- when the reading was taken
    recorded_by    UUID REFERENCES accounts(id) ON DELETE SET NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX measurements_group_metric_idx
    ON measurements (group_username, metric, recorded_at);
