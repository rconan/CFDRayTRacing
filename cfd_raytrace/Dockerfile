FROM rust:1.61 as build
ADD Cargo.toml /
COPY ./src  /src
RUN cargo build --release --features s3 --bin main

FROM debian:buster-slim
RUN apt-get update -y
RUN apt-get install openssl -y 
COPY --from=build ./target/release/main /usr/bin/main
CMD ["/usr/bin/main"]