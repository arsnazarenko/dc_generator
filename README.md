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
Kafka UI available on http://localhost:8080

## Command Line Flags

### Stdout Command

Outputs generated metrics to stdout.

- `-t, --timeout <TIMEOUT>`: Timeout between messages in milliseconds (default: 500)
- `--zones <ZONES>`: Number of zones in data center (default: 4)
- `--servers-per-zone <SERVERS_PER_ZONE>`: Number of servers per zone (default: 80)

Example:
```bash
dc-generator stdout --timeout 1000 --zones 2 --servers-per-zone 5
```

### Kafka Command

Sends generated metrics to a Kafka topic.

- `--brokers <BROKERS>`: Kafka broker addresses (comma separated)
- `--topic <TOPIC>`: Kafka topic name (default: dc_metrics)
- `--partitions <PARTITIONS>`: Number of topic partitions (default: 3)
- `--replicas <REPLICAS>`: Number of topic replicas (default: 3)
- `-t, --timeout <TIMEOUT>`: Timeout between messages in milliseconds (default: 500)
- `--zones <ZONES>`: Number of zones in data center (default: 4)
- `--servers-per-zone <SERVERS_PER_ZONE>`: Number of servers per zone (default: 80)

Example:
```bash
dc-generator kafka --brokers localhost:9092 --topic my_metrics --timeout 200 --zones 3 --servers-per-zone 8
```

Example of generated metrics:
```
{"event_id":"6f52f735-fb69-44e6-b053-e04c81808f9b","host_id":"srv-26-rack-01","zone":"zone-A","timestamp":1763663600372,"metric":"CPU_TEMP","value":61.046022925919566,"unit":"°C","tags":{}}
{"event_id":"2698ff31-d554-40b0-9633-d0c9837e966d","host_id":"srv-33-rack-02","zone":"zone-B","timestamp":1763663601372,"metric":"DISK_IO_WRITE","value":97.47286075704702,"unit":"MB/s","tags":{}}
{"event_id":"94f0f218-7929-4512-9c30-94a70c313e2f","host_id":"srv-34-rack-02","zone":"zone-C","timestamp":1763663602373,"metric":"MEM_USAGE","value":50.13302370656123,"unit":"%","tags":{}}
{"event_id":"537f288d-3dbe-46b4-9039-32806f12948b","host_id":"srv-42-rack-02","zone":"zone-D","timestamp":1763663603373,"metric":"NET_IN","value":91.54716135431559,"unit":"MB/s","tags":{}}
{"event_id":"af621930-8eba-4bb3-a6a6-52209b95e70f","host_id":"srv-47-rack-02","zone":"zone-E","timestamp":1763663604373,"metric":"DISK_IO_READ","value":90.4204838633165,"unit":"MB/s","tags":{}}
{"event_id":"8de4abec-09f4-46f4-a8c1-72e353934695","host_id":"srv-03-rack-01","zone":"zone-F","timestamp":1763663605373,"metric":"MEM_USAGE","value":47.820221492638424,"unit":"%","tags":{}}
{"event_id":"e1e2d752-6582-4dcc-af32-1be83a8a08aa","host_id":"srv-28-rack-01","zone":"zone-G","timestamp":1763663606374,"metric":"DISK_IO_WRITE","value":90.74315572862865,"unit":"MB/s","tags":{}}
{"event_id":"6c67ca42-cd9b-4fda-8079-fa7100a03b08","host_id":"srv-35-rack-02","zone":"zone-H","timestamp":1763663607374,"metric":"DISK_IO_WRITE","value":109.53819511295102,"unit":"MB/s","tags":{}}
{"event_id":"f801ad7e-0ae7-49aa-b9c4-b73fad3d1079","host_id":"srv-14-rack-01","zone":"zone-A","timestamp":1763663608374,"metric":"NET_OUT","value":97.6912128345963,"unit":"MB/s","tags":{}}
```
