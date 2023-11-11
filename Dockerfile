# syntax=docker/dockerfile:1
FROM ubuntu:23.10

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    bc \
    bison \
    build-essential \
    bzip2 \
    clang \
    cpio \
    curl \
    flex \
    gawk \
    git \
    grep \
    kmod \
    libelf-dev \
    libssl-dev \
    lld \
    llvm \
    make \
    rsync \
    sudo \
    wget \
    && apt-get clean \
    && rm -rf /root/.cache \
    && rm -rf /var/lib/apt/lists/*

# Set user password
RUN echo 'ubuntu:ubuntu' | chpasswd

# Install Rust and required targets/components
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.71.0

RUN set -eux; \
    url="https://static.rust-lang.org/rustup/archive/1.26.0/x86_64-unknown-linux-gnu/rustup-init"; \
    wget "$url"; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host x86_64-unknown-linux-gnu; \
    rm rustup-init; \
    chown -R dev.dev $RUSTUP_HOME $CARGO_HOME && chmod -R 0777 $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    rustup component add rust-src rustfmt clippy; \
    cargo install --locked --version 0.65.1 bindgen-cli

# Useful to check whether or not a script is running inside a container
ENV INSIDE_DOCKER_CONTAINER "1"

# Switch to non-root user
USER ubuntu
WORKDIR /home/ubuntu

# Open a shell by default
CMD ["/bin/bash"]
