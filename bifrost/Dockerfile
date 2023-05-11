FROM rust:latest

WORKDIR /bifrost

COPY Cargo.toml .

COPY src/ ./src/

COPY templates/ ./templates/

RUN cargo build --release

EXPOSE 8080

CMD ["./target/release/bifrost"]
