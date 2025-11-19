# DC Generator

A real-time traffic generator for Kafka that simulates server metrics in a data center environment. It generates metrics such as CPU usage, memory usage, disk I/O, network traffic, and CPU temperature for multiple servers across different zones.

## Features

- Generates realistic server metrics with random variations and occasional overloads/failures
- Configurable number of zones and servers per zone
- Asynchronous I/O

## Build and Run

### Local Build

2. Clone the repository and navigate to the project directory
3. Build the project:
   ```bash
   cargo build --release
   ```

### Local Run

Run the generator in stdout mode:
```bash
./target/release/dc-generator stdout [OPTIONS]
```

Run the generator in Kafka mode (requires a running Kafka instance):
```bash
./target/release/dc-generator kafka [OPTIONS]
```

### Docker Build and Run

1. Ensure you have Docker and Docker Compose installed
2. Build and run with Docker Compose:
   ```bash
   docker-compose up --build
   ```

This will start Kafka, Kafka UI, and the DC generator automatically.

## Command Line Flags

### Stdout Command

Outputs generated metrics to stdout.

- `-t, --timeout <TIMEOUT>`: Timeout between messages in milliseconds (default: 500)
- `--zones <ZONES>`: Number of zones in data center (default: 4)
- `--servers-per-zone <SERVERS_PER_ZONE>`: Number of servers per zone (default: 10)

Example:
```bash
dc-generator stdout --timeout 1000 --zones 2 --servers-per-zone 5
```

### Kafka Command

Sends generated metrics to a Kafka topic.

- `-t, --topic <TOPIC>`: Kafka topic name (default: "dc_metrics")
- `-a, --address <ADDRESS>`: Kafka host (default: "127.0.0.1:9092")
- `--timeout <TIMEOUT>`: Timeout between messages in milliseconds (default: 500)
- `--zones <ZONES>`: Number of zones in data center (default: 4)
- `--servers-per-zone <SERVERS_PER_ZONE>`: Number of servers per zone (default: 10)

Example:
```bash
dc-generator kafka --topic my_metrics --address localhost:9092 --timeout 200 --zones 3 --servers-per-zone 8
```

## Dependencies

- samsa: Kafka client library
- clap: Command line argument parser
- tokio: Asynchronous runtime
- uuid: UUID generation
- serde/serde_json: JSON serialization
- rand: Random number generation
