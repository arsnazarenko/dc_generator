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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::parse();

    match args.command {
        args::Commands::Stdout {
            dc_gen_params:
                args::GenParams {
                    rps,
                    threads,
                    servers_per_zone,
                    duration,
                },
        } => {
            stdout_mode(rps, threads, servers_per_zone, duration);
        }
        args::Commands::Kafka {
            brokers,
            topic,
            partitions,
            replicas,
            connections,
            dc_gen_params:
                args::GenParams {
                    rps,
                    threads,
                    servers_per_zone,
                    duration,
                },
        } => {
            kafka_mode(
                brokers,
                &topic,
                partitions,
                replicas,
                connections,
                threads,
                servers_per_zone,
                rps,
                duration,
            )
            .await
            .map_err(|e| std::io::Error::other(format!("Kafka client error: {}", e)))?;
        }
    }

    Ok(())
}

fn stdout_mode(rps: u64, workers: u8, servers_per_zone: u16, duration: Option<Duration>) {
    let interval_ms = match rps > 0 {
        true => (1000 / rps).max(1),
        false => 500,
    };

    let start_time = std::time::Instant::now();

    let gen_iterator = { 0..workers }
        .map(|worker_num| {
            let zone_name = format!("zone-{}", (b'A' + worker_num) as char);
            dc_metrics::ServerMetricsGenerator::new(zone_name, servers_per_zone as usize)
        })
        .cycle();

    for mut zone_gen in gen_iterator {
        if let Some(dur) = duration
            && start_time.elapsed() >= dur
        {
            break;
        }
        let metric = zone_gen.next().unwrap();
        println!("{}", metric.message);
        std::thread::sleep(Duration::from_millis(interval_ms));
    }
}

#[allow(clippy::too_many_arguments)]
async fn kafka_mode(
    brokers: Vec<samsa::prelude::BrokerAddress>,
    topic: &str,
    partitions: u16,
    replicas: u8,
    connections: u8,
    threads: u8,
    servers_per_zone: u16,
    rps: u64,
    duration: Option<Duration>,
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

    let interval_ms = if rps > 0 {
        1000_u64.checked_div(rps).map(|v| v.max(1)).unwrap()
    } else {
        500
    };

    let mut producers = Vec::with_capacity(connections as usize);
    for _ in 0..connections {
        let producer = samsa::prelude::ProducerBuilder::<TcpConnection>::new(
            brokers.clone(),
            vec![topic.into()],
        )
        .await?
        .required_acks(1)
        .clone()
        .build()
        .await;
        producers.push(std::sync::Arc::new(producer));
    }

    println!(
        "Created {} producer(s) connected to kafka: {:?}",
        connections,
        brokers
            .iter()
            .map(|b| format!("{}:{}", b.host, b.port))
            .collect::<Vec<_>>()
    );

    let shared_topic = std::sync::Arc::new(topic.to_string());

    let mut handles = Vec::with_capacity(threads as usize);

    for worker_num in 0..threads {
        let zone_name = format!("zone-{}", (b'A' + worker_num) as char);
        let producer = producers[(worker_num as usize) % producers.len()].clone();
        let topic_cloned = shared_topic.clone();

        let handle = tokio::spawn(async move {
            let mut metrics_gen =
                dc_metrics::ServerMetricsGenerator::new(zone_name, servers_per_zone as usize);
            let mut interval = time::interval(Duration::from_millis(interval_ms));
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

    if let Some(dur) = duration {
        tokio::time::sleep(dur).await;
        for handle in handles {
            handle.abort();
        }
    } else {
        for handle in handles {
            let _ = handle.await;
        }
    }
    Ok(())
}

fn get_partition<T: Hash>(key: &T, partitions_num: u16) -> u16 {
    let mut hasher = std::hash::DefaultHasher::default();
    key.hash(&mut hasher);
    let h = hasher.finish() as u16;
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
