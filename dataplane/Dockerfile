# FROM archlinux as builder

# RUN pacman -Syu --noconfirm
# RUN pacman -S base-devel protobuf rustup --noconfirm

# RUN rustup default stable
# RUN rustup install nightly
# RUN rustup component add rust-src --toolchain nightly
# RUN rustup target add x86_64-unknown-linux-musl
# RUN --mount=type=cache,target=/root/.cargo/registry \
#     cargo install bpf-linker

# WORKDIR /workspace

# COPY . .
# RUN --mount=type=cache,target=/workspace/target/ \
#     --mount=type=cache,target=/root/.cargo/registry \
#     cargo xtask build-ebpf
# RUN --mount=type=cache,target=/workspace/target/ \
#     --mount=type=cache,target=/root/.cargo/registry \
#     RUSTFLAGS=-Ctarget-feature=+crt-static cargo build --target=x86_64-unknown-linux-musl
# RUN --mount=type=cache,target=/workspace/target/ \
#     cp /workspace/target/x86_64-unknown-linux-musl/debug/loader /workspace/dataplane
# RUN cp $(which rust-gdb) /workspace/rust-gdb

# FROM ubuntu:22.04

# LABEL org.opencontainers.image.source=https://github.com/kubernetes-sigs/blixt
# LABEL org.opencontainers.image.licenses=GPL-2.0-only,BSD-2-Clause

# WORKDIR /opt/blixt/

# COPY --from=builder /workspace/dataplane /opt/blixt/dataplane

# COPY LICENSE.GPL-2.0 /opt/blixt/LICENSE.GPL-2.0
# COPY LICENSE.BSD-2-Clause /opt/blixt/LICENSE.BSD-2-Clause

# COPY --from=builder /workspace/rust-gdb /usr/local/bin/rust-gdb

# RUN apt-get update && \
#     apt-get install -y net-tools procps cgdb && \
#     rm -rf /var/lib/apt/lists/*

# ENTRYPOINT ["/opt/blixt/dataplane"]


# 使用 Ubuntu 作为基础镜像
FROM ubuntu:22.04

WORKDIR /workspace

RUN apt-get update && \
    apt-get install -y curl build-essential protobuf-compiler libprotobuf-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . $HOME/.cargo/env && \
    rustup default stable && \
    rustup install nightly && \
    rustup component add rust-src --toolchain nightly && \
    rustup target add x86_64-unknown-linux-gnu

RUN . $HOME/.cargo/env && \
    cargo install bpf-linker

COPY . .

RUN . $HOME/.cargo/env && \
    cargo xtask build-ebpf && \
    RUSTFLAGS="-Ctarget-feature=+crt-static --cfg tokio_unstable" cargo build --target=x86_64-unknown-linux-gnu

RUN mkdir -p /opt/blixt

RUN cp target/x86_64-unknown-linux-gnu/debug/loader /opt/blixt/dataplane

WORKDIR /opt/blixt/

COPY LICENSE.GPL-2.0 /opt/blixt/LICENSE.GPL-2.0
COPY LICENSE.BSD-2-Clause /opt/blixt/LICENSE.BSD-2-Clause

RUN apt-get update && \
    apt-get install -y net-tools procps && \
    rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["/opt/blixt/dataplane"]
