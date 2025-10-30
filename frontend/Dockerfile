# syntax=docker/dockerfile:1.7
FROM rust:1.81 as base
WORKDIR /app
RUN rustup target add wasm32-unknown-unknown && cargo install cargo-leptos cargo-watch
COPY . .

FROM base as dev
CMD ["bash", "-lc", "cargo watch -x run"]

FROM rust:1.81 as build
WORKDIR /app
RUN rustup target add wasm32-unknown-unknown && cargo install cargo-leptos
COPY Cargo.toml Cargo.lock .
COPY src ./src
RUN cargo leptos build --release

FROM nginx:alpine as runtime
COPY --from=build /app/target/site /usr/share/nginx/html
EXPOSE 80
