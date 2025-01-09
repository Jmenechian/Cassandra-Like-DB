FROM rust:1.81

WORKDIR /

COPY ./Cargo.toml ./Cargo.toml

COPY src ./src