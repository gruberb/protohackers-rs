FROM rust:latest as builder
RUN apt-get update && apt-get -y install ca-certificates cmake musl-tools libssl-dev && rm -rf /var/lib/apt/lists/*
COPY . .
RUN rustup default stable && rustup update
RUN rustup target add x86_64-unknown-linux-musl
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV RUST_LOG=info
RUN cargo build --target x86_64-unknown-linux-musl --release
FROM scratch
COPY --from=builder /target/x86_64-unknown-linux-musl/release/problem_01 .
EXPOSE 8080
CMD ["/problem_01"]
