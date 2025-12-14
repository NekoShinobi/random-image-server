FROM rust:1.92 AS base
RUN apt update && apt upgrade -y && apt install -y cmake && apt clean && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --version ^0.1
RUN cargo install sccache --version ^0.12
ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

FROM base AS planner
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef prepare --recipe-path recipe.json

FROM base AS builder
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo build --release

FROM ubuntu:24.04
WORKDIR /app

ENV USER=abc
ENV GROUP=$USER
ENV UID=1001
ENV GID=1001

RUN  groupadd -g $GID $GROUP \
    &&  useradd -u $UID -g $GROUP -M -p '' $USER


COPY --from=builder --chown=$USER:$GROUP /app/target/release/random-image-server /app/random-image-server

RUN mkdir /app/images && chown -R $USER:$GROUP /app

USER abc


CMD ["/app/random-image-server"]
