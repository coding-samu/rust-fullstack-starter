# syntax=docker/dockerfile:1.7
FROM rust:1.81 as base
WORKDIR /app
RUN rustup target add wasm32-unknown-unknown && \
    cargo install cargo-watch --version 8.1.1 --locked
COPY . .

FROM base as dev
CMD ["bash", "-lc", "cargo watch -x run"]

# Production build stage (placeholder for future static build pipeline)
# FROM rust:1.86 as build
# WORKDIR /app
# COPY Cargo.toml ./
# COPY Cargo.loc[k] ./
# COPY src ./src
# RUN cargo build --release
#
# FROM nginx:alpine as runtime
# COPY --from=build /app/target/site /usr/share/nginx/html
# EXPOSE 80
