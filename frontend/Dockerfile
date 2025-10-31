# syntax=docker/dockerfile:1.7
FROM rust:1.81 as base
WORKDIR /app
RUN rustup target add wasm32-unknown-unknown && \
    cargo install cargo-leptos --version 0.2.28 --locked && \
    cargo install cargo-watch --version 8.1.1 --locked
COPY . .

FROM base as dev
CMD ["bash", "-lc", "cargo watch -x run"]

FROM rust:1.81 as build
WORKDIR /app
RUN rustup target add wasm32-unknown-unknown && cargo install cargo-leptos --version 0.2.28 --locked
# Copy Cargo.toml and optionally Cargo.lock (may not exist in fresh workspace)
COPY Cargo.toml ./
COPY Cargo.loc[k] ./
COPY src ./src
# Generate lock file if missing and build
RUN cargo leptos build --release

FROM nginx:alpine as runtime
COPY --from=build /app/target/site /usr/share/nginx/html
EXPOSE 80