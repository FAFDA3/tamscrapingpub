FROM rust:1.56

WORKDIR /app

COPY . .

RUN cargo build --release

CMD ["./target/release/superscraper"]
