FROM rust:1.59 as builder

RUN USER=root cargo new --bin baste
WORKDIR ./baste
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/baste*
RUN cargo build --release


FROM debian:bullseye-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC

RUN mkdir -p ${APP}

COPY --from=builder /baste/target/release/baste ${APP}/baste

WORKDIR ${APP}

CMD ["./baste"]