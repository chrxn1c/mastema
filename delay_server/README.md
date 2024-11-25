# Delay server

A tool for some applications to interact with some foreign app via `TcpStream` or row socket. 

## Running

> Note: You have to be in workspace root

### Native way

- `$ cargo run --bin delay_server`

### Docker Compose (logs could be missing)

- `$ docker compose -f <path_to_docker-compose.yml> up -d --build`