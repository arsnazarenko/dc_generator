use samsa::prelude::TcpConnection;
use std::{
    hash::{Hash, Hasher},
    time::Duration,
    vec,
};
use tokio::time;

mod args;
mod dc_metrics;

const CLIENT_ID: &str = "Data center metrics producer";
const CORRELATION_ID: i32 = 1;
// const DEFAULT_REPLICATION_FACTOR: i16 = 3;
// const DEFAULT_PARTITIONS_NUM: u32 = 3;
// const DEFAULT_TOPIC_NAME: &str = "dc_metrics";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::parse();

    match args.mode {
        args::Mode::Stdout => {
            stdout_mode(args.timeout, args.zones, args.servers_per_zone);
        }
        args::Mode::Kafka => {
            let topic = args.topic.unwrap();
            let partitions = args.partitions.unwrap();
            let replicas = args.replicas.unwrap();
            let brokers = args::parse_kafka_brokers(&args.address.unwrap())?;
            kafka_mode(
                brokers,
                &topic,
                partitions,
                replicas,
                args.zones,
                args.servers_per_zone,
                args.timeout,
            )
            .await
            .map_err(|e| std::io::Error::other(format!("Kafka client error: {}", e.to_string())))?;
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
    brokers: Vec<samsa::prelude::BrokerAddress>,
    topic: &str,
    partitions: usize,
    replicas: usize,
    zones: usize,
    servers_per_zone: usize,
    timeout: u64,
) -> samsa::prelude::Result<()> {
    let connection = TcpConnection::new_(brokers.clone()).await?;

    let _ = create_topics_manually(
        connection,
        CORRELATION_ID,
        CLIENT_ID,
        vec![((topic, replicas as i16), partitions as i32)]
            .into_iter()
            .collect(),
    )
    .await?;

    let producer =
        samsa::prelude::ProducerBuilder::<TcpConnection>::new(brokers.clone(), vec![topic.into()])
            .await?
            .required_acks(1)
            .clone()
            .build()
            .await;

    println!(
        "Producer connected to kafka: {:?}",
        brokers
            .iter()
            .map(|b| format!("{}:{}", b.host, b.port))
            .collect::<Vec<_>>()
    );

    let shared_producer = std::sync::Arc::new(producer);
    let shared_topic = std::sync::Arc::new(topic.to_string());

    let mut handles = Vec::with_capacity(zones);

    for zone_num in 0..zones {
        let zone_name = format!("zone-{}", (b'A' + zone_num as u8) as char);
        let producer = shared_producer.clone();
        let topic_cloned = shared_topic.clone();

        let handle = tokio::spawn(async move {
            let mut metrics_gen =
                dc_metrics::ServerMetricsGenerator::new(zone_name, servers_per_zone);
            let mut interval = time::interval(Duration::from_millis(timeout));
            loop {
                interval.tick().await;
                let metric = metrics_gen.next().unwrap();
                let message = samsa::prelude::ProduceMessage {
                    topic: topic_cloned.to_string(),
                    partition_id: get_partition(&metric.host_id, partitions) as i32,
                    key: Some(metric.host_id.into()),
                    value: Some(metric.message.into()),
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

fn get_partition<T: Hash>(key: &T, partitions_num: usize) -> usize {
    let mut hasher = std::hash::DefaultHasher::default();
    key.hash(&mut hasher);
    let h = hasher.finish() as usize;
    h % partitions_num
}

pub async fn create_topics_manually(
    mut conn: impl samsa::prelude::BrokerConnection,
    correlation_id: i32,
    client_id: &str,
    topics_with_partition_count: std::collections::HashMap<(&str, i16), i32>,
) -> samsa::prelude::Result<samsa::prelude::protocol::CreateTopicsResponse> {
    let mut create_topics =
        samsa::prelude::protocol::CreateTopicsRequest::new(correlation_id, client_id, 4000, false)?;

    for ((topic_name, replication_factor), num_partitions) in topics_with_partition_count {
        create_topics.add(topic_name, num_partitions, replication_factor);
    }

    conn.send_request(&create_topics).await?;

    let create_topics_response = conn.receive_response().await?;

    samsa::prelude::protocol::CreateTopicsResponse::try_from(create_topics_response.freeze())
}
