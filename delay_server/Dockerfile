FROM rust:1-slim-bookworm

WORKDIR /usr/delay_server

COPY . .

RUN cargo build

CMD ["cargo", "run", "delayserver"]