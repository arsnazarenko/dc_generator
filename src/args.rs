use std::{num::NonZeroU64, time::Duration};

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dc-generator")]
#[command(about = "Real-time traffic generator for Kafka")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args)]
pub struct GenParams {
    /// Number of messages per second
    #[arg(short, long, default_value_t = 10)]
    pub rps: u64,

    /// Number of parallel workers
    #[arg(short, long, default_value_t = 4)]
    pub threads: u8,

    /// Number of servers per zone
    #[arg(short, long, default_value_t = 80)]
    pub servers_per_zone: u16,

    /// Duration of load(1s, 100s, 1m, 10m)
    #[arg(short, value_parser = parse_duration)]
    pub duration: Option<Duration>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Output messages to stdout
    Stdout {
        // parameters for generator
        #[clap(flatten)]
        dc_gen_params: GenParams,
    },
    /// Send messages to Kafka
    Kafka {
        // Kafka address (required when mode is kafka)
        #[arg(long, value_parser = parse_kafka_brokers)]
        brokers: KafkaBrokers,

        // Kafka topic
        #[arg(long, default_value = "dc_metrics")]
        topic: String,

        // Number of topic partitions
        #[arg(short, long, default_value_t = 3)]
        partitions: u16,

        // Number of topic replicas
        #[arg(short, long, default_value_t = 3)]
        replicas: u8,

        // Number of Kafka producer connections
        #[arg(short, long, default_value_t = 1)]
        connections: u8,

        // parameters for generator
        #[clap(flatten)]
        dc_gen_params: GenParams,
    },
}

pub fn parse() -> CliArgs {
    CliArgs::parse()
}

pub fn parse_duration(duration_str: &str) -> Result<std::time::Duration, clap::Error> {
    let duration_str = duration_str.trim().to_lowercase();
    if duration_str.len() < 2 {
        return Err(clap::Error::raw(
            clap::error::ErrorKind::InvalidValue,
            "Duration value len must be in format: N[s|m]",
        ));
    }
    let (value_str, suffix) = duration_str.split_at(duration_str.len() - 1);
    let value: NonZeroU64 = value_str.parse().map_err(|_| {
        clap::Error::raw(
            clap::error::ErrorKind::InvalidValue,
            "Failed to parse duration value. Duration value must be positive number",
        )
    })?;
    let duration = match suffix {
        "s" => Duration::from_secs(value.into()),
        "m" => Duration::from_mins(value.into()),
        _ => {
            return Err(clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                "Invalid duration value. duration parameter can be in format: <NUMBER[s|m]>",
            ));
        }
    };
    Ok(duration)
}

type KafkaBrokers = Vec<samsa::prelude::BrokerAddress>;

fn parse_kafka_brokers(address_str: &str) -> Result<KafkaBrokers, clap::Error> {
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
