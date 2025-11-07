# syntax=docker/dockerfile:1.7
FROM rust:1.88 as base
WORKDIR /app
RUN cargo install cargo-watch --version 8.4.0
COPY . .

FROM base as dev
CMD ["/usr/local/cargo/bin/cargo", "watch", "-x", "run"]

FROM rust:1.88 as build
WORKDIR /app
# Copy Cargo.toml and optionally Cargo.lock (may not exist in fresh workspace)
COPY Cargo.toml ./
COPY Cargo.loc[k] ./
COPY src ./src
# Generate lock file if missing and build
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 as runtime
WORKDIR /app
COPY --from=build /app/target/release/backend /app/backend
EXPOSE 3000
USER 65532:65532
ENV RUST_LOG=info
CMD ["/app/backend"]
