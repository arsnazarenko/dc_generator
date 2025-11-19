use clap::{Parser, Subcommand};
use samsa::prelude::TcpConnection;
use std::time::Duration;
use tokio::time;

mod dc_metrics;

#[derive(Parser)]
#[command(name = "dc-generator-native")]
#[command(about = "Real-time traffic generator for Kafka (native Rust)")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Output messages to stdout
    Stdout {
        /// Timeout between messages in milliseconds
        #[arg(short, long, default_value_t = 500)]
        timeout: u64,

        /// Number of zones in data center
        #[arg(long, default_value_t = 4)]
        zones: usize,

        /// Number of servers per zone
        #[arg(long, default_value_t = 10)]
        servers_per_zone: usize,
    },
    /// Send messages to Kafka
    Kafka {
        /// Kafka topic name
        #[arg(short, long, default_value = "dc_metrics")]
        topic: String,

        /// Kafka address
        #[arg(short, long, default_value = "127.0.0.1:9092")]
        address: String,

        /// Timeout between messages in milliseconds
        #[arg(short, long, default_value_t = 500)]
        timeout: u64,

        /// Number of zones in data center
        #[arg(long, default_value_t = 4)]
        zones: usize,

        /// Number of servers per zone
        #[arg(long, default_value_t = 10)]
        servers_per_zone: usize,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Stdout {
            timeout,
            zones,
            servers_per_zone,
        } => {
            stdout_mode(timeout, zones, servers_per_zone);
        }
        Commands::Kafka {
            topic,
            address,
            timeout,
            zones,
            servers_per_zone,
        } => {
            kafka_mode(timeout, &topic, &address, zones, servers_per_zone)
                .await
                .map_err(|e| {
                    std::io::Error::other(format!("Kafka client error: {}", e.to_string()))
                })?;
        }
    }

    Ok(())
}

fn stdout_mode(timeout: u64, zones: usize, servers_per_zone: usize) {
    let gen_iterator = (0..zones)
        .into_iter()
        .map(|zone_num| {
            let zone_name = format!("zone-{}", (b'A' + zone_num as u8) as char);
            dc_metrics::ServerMetricsGenerator::new(zone_name, servers_per_zone)
        })
        .cycle();

    for mut zone_gen in gen_iterator {
        let metric = zone_gen.next().unwrap();
        println!("{}", metric.message);
        std::thread::sleep(Duration::from_millis(timeout));
    }
}

async fn kafka_mode(
    timeout: u64,
    topic: &str,
    address: &str,
    zones: usize,
    servers_per_zone: usize,
) -> samsa::prelude::Result<()> {
    let (host, port_str) = address
        .split_once(':')
        .expect("Address of kafka must be in format {host:port}");

    let host: String = host.to_string();
    let port: u16 = port_str.parse().unwrap();
    let bootstrap_addrs = vec![samsa::prelude::BrokerAddress {
        host: host.into(),
        port,
    }];
    let producer = samsa::prelude::ProducerBuilder::<TcpConnection>::new(
        bootstrap_addrs,
        vec![topic.to_string()],
    )
    .await?
    .build()
    .await;

    println!("Producer connected to kafka on address {}", address);
    let producer = std::sync::Arc::new(producer);
    let shared_topic = std::sync::Arc::new(topic.to_string());
    let mut handles = vec![];

    for zone_num in 0..zones {
        let zone = format!("zone-{}", (b'A' + zone_num as u8) as char);
        let producer = producer.clone();
        let topic_cloned = shared_topic.clone();

        let handle = tokio::spawn(async move {
            let mut metrics_gen = dc_metrics::ServerMetricsGenerator::new(zone, servers_per_zone);
            let mut interval = time::interval(Duration::from_millis(timeout));
            loop {
                interval.tick().await;
                let metric = metrics_gen.next().unwrap();
                let message = samsa::prelude::ProduceMessage {
                    topic: topic_cloned.to_string(),
                    partition_id: 0,
                    key: Some(metric.message.into()),
                    value: Some(metric.host_id.into()),
                    headers: vec![],
                };
                producer.produce(message).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
    Ok(())
}
