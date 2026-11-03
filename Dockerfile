FROM rust:1.99-alpine3.22 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations ./migrations

RUN apk add --no-cache musl-dev openssl-dev pkgconfig
RUN cargo build --release

FROM alpine:3.21

WORKDIR /app

RUN adduser -D utopia
RUN apk add --no-cache curl

COPY --from=builder /app/target/release/utopia /app/utopia
COPY migrations /app/migrations

USER utopia

EXPOSE 3000

ENTRYPOINT ["/app/utopia"]
