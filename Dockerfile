FROM rust:1.94 AS base
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

RUN apt update && apt install -y xz-utils && apt clean && rm -rf /var/lib/apt/lists/*

ARG S6_OVERLAY_VERSION="3.2.1.0"
ARG S6_OVERLAY_ARCH="x86_64"

WORKDIR /app
RUN mkdir /images
RUN groupadd -g 991 abc &&  useradd -u 991 -g abc -M -p '' abc


COPY --from=builder /app/target/release/random-image-server /app/random-image-server


ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz /tmp
RUN tar -C / -Jxpf /tmp/s6-overlay-noarch.tar.xz && rm /tmp/s6-overlay-noarch.tar.xz
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-${S6_OVERLAY_ARCH}.tar.xz /tmp
RUN tar -C / -Jxpf /tmp/s6-overlay-${S6_OVERLAY_ARCH}.tar.xz && rm /tmp/s6-overlay-${S6_OVERLAY_ARCH}.tar.xz
ADD --chmod=755 "https://raw.githubusercontent.com/linuxserver/docker-mods/mod-scripts/lsiown.v1" "/usr/bin/lsiown"

COPY root/ /

ENTRYPOINT ["/init"]
