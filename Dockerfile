FROM rust:1.94-alpine AS builder

WORKDIR /build

COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc:latest

WORKDIR /app

COPY --from=builder /build/target/release/kubemal .

ENTRYPOINT [ "./kubemal" ]
