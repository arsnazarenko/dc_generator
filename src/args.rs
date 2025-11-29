use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "dc-generator")]
#[command(about = "Real-time data center metrics traffic generator")]
pub struct Args {
    /// Output mode
    #[arg(short, long)]
    pub mode: Mode,

    /// Kafka topic name (required when mode is kafka)
    #[arg(long, required_if_eq("mode", "kafka"))]
    pub topic: Option<String>,

    /// Kafka address (required when mode is kafka)
    #[arg(long, required_if_eq("mode", "kafka"))]
    pub address: Option<String>,

    /// Timeout between messages in milliseconds
    #[arg(short, long, default_value_t = 500)]
    pub timeout: u64,

    /// Number of zones in data center
    #[arg(long, default_value_t = 4)]
    pub zones: usize,

    /// Number of servers per zone
    #[arg(long, default_value_t = 100)]
    pub servers_per_zone: usize,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Mode {
    Stdout,
    Kafka,
}

pub fn parse() -> Args {
    Args::parse()
}

pub type KafkaBrokers = Vec<samsa::prelude::BrokerAddress>;

pub fn parse_kafka_brokers(address_str: &str) -> Result<KafkaBrokers, clap::Error> {
    let brokers = address_str
        .trim()
        .split(",")
        .map(|s| {
            let (host, port) = s.trim().split_once(":").ok_or_else(|| {
                clap::Error::raw(
                    clap::error::ErrorKind::InvalidValue,
                    "Kafka broker address must be in format: <HOST:PORT>",
                )
            })?;
            let port: u16 = port.parse().map_err(|_| {
                clap::Error::raw(clap::error::ErrorKind::InvalidValue, "Invalid port number")
            })?;
            let host = host.into();
            Ok(samsa::prelude::BrokerAddress { host, port })
        })
        .collect::<Result<Vec<_>, clap::Error>>()?;

    if brokers.is_empty() {
        Err(clap::Error::raw(
            clap::error::ErrorKind::InvalidValue,
            "Kafka brokers list must be in format: <HOST1:PORT,HOST2:PORT,...>",
        ))
    } else {
        Ok(brokers)
    }
}
