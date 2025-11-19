use rand::Rng;
use serde::Serialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Serialize)]
struct MetricData {
    event_id: String,
    host_id: String,
    zone: String,
    timestamp: i64,
    metric: String,
    value: f64,
    unit: String,
    tags: serde_json::Value,
}

#[derive(Clone)]
enum Status {
    Normal,
    Overloaded,
}

struct HostState {
    last_values: HashMap<String, f64>,
    status: Status,
    failure_until: Option<i64>,
}

pub struct ServerMetric {
    pub message: String,
    pub host_id: String,
}

pub struct ServerMetricsGenerator {
    zone: String,
    servers: usize,
    metrics: Vec<String>,
    hosts: Vec<HostState>,
}

impl ServerMetricsGenerator {
    pub fn new(zone: String, servers: usize) -> Self {
        let metrics = vec![
            "CPU_USAGE".to_string(),
            "MEM_USAGE".to_string(),
            "DISK_IO_READ".to_string(),
            "DISK_IO_WRITE".to_string(),
            "NET_IN".to_string(),
            "NET_OUT".to_string(),
            "CPU_TEMP".to_string(),
        ];
        let hosts = (0..servers)
            .map(|_| HostState {
                last_values: HashMap::new(),
                status: Status::Normal,
                failure_until: None,
            })
            .collect();
        Self {
            zone,
            servers,
            metrics,
            hosts,
        }
    }
}

impl Iterator for ServerMetricsGenerator {
    type Item = ServerMetric;

    fn next(&mut self) -> Option<Self::Item> {
        let mut rng = rand::thread_rng();
        let host_index = rng.gen_range(0..self.servers);
        let metric_index = rng.gen_range(0..self.metrics.len());
        let server_in_zone = host_index + 1;
        let zone = self.zone.clone();
        let servers_per_rack = 30;
        let rack = ((server_in_zone - 1) / servers_per_rack) + 1;
        let host_id = format!("srv-{:02}-rack-{:02}", server_in_zone, rack);
        let metric = &self.metrics[metric_index];
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let host_state = &mut self.hosts[host_index];

        // Check failure
        let is_failed = if let Some(until) = host_state.failure_until {
            if timestamp < until {
                true
            } else {
                host_state.failure_until = None;
                host_state.status = Status::Normal;
                false
            }
        } else {
            false
        };

        if is_failed {
            return self.next();
        }

        let initial_value = match metric.as_str() {
            "CPU_USAGE" | "MEM_USAGE" => 50.0,
            "DISK_IO_READ" | "DISK_IO_WRITE" | "NET_IN" | "NET_OUT" => 100.0,
            "CPU_TEMP" => 60.0,
            _ => 0.0,
        };

        let prev = host_state
            .last_values
            .get(metric)
            .cloned()
            .unwrap_or(initial_value);
        let mut new_value = {
            let change = rng.gen_range(-0.1..=0.1);
            let mut val = prev * (1.0 + change);
            if matches!(host_state.status, Status::Overloaded) {
                val *= 1.3;
            }
            if rng.gen_bool(0.05) {
                host_state.status = Status::Overloaded;
                val *= rng.gen_range(1.2..1.5);
            }
            if rng.gen_bool(0.01) {
                host_state.failure_until = Some(timestamp + rng.gen_range(10000..30000));
            }
            val
        };

        // Clamp values
        new_value = new_value.max(0.0);
        match metric.as_str() {
            "CPU_USAGE" | "MEM_USAGE" => new_value = new_value.min(100.0),
            "CPU_TEMP" => new_value = new_value.min(100.0).max(20.0),
            _ => {}
        }

        host_state.last_values.insert(metric.clone(), new_value);

        let unit = match metric.as_str() {
            "CPU_USAGE" | "MEM_USAGE" => "%",
            "DISK_IO_READ" | "DISK_IO_WRITE" | "NET_IN" | "NET_OUT" => "MB/s",
            "CPU_TEMP" => "°C",
            _ => "",
        };

        let event_id = Uuid::new_v4().to_string();
        let data = MetricData {
            event_id,
            host_id: host_id.clone(),
            zone,
            timestamp,
            metric: metric.clone(),
            value: new_value,
            unit: unit.to_string(),
            tags: serde_json::json!({}),
        };
        let message = serde_json::to_string(&data).unwrap();
        Some(ServerMetric { message, host_id })
    }
}
