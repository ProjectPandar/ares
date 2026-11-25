# syntax=docker/dockerfile:1.7
FROM node:22-bookworm-slim AS nodebin

FROM ubuntu:24.04 AS base

ARG RUST_VERSION=1.99.0
ARG WASM_BINDGEN_VERSION=0.2.121
ARG WASM_BINDGEN_SHA256=3039f38f65fe237b640cf06a140c919ca8d717ec5012146d145d3f27bb4d6b28
ARG NEXTEST_VERSION=0.9.152
ARG NEXTEST_SHA256=d039b43c3e0958441b6ad0a39c8884d018b3d2e8fb9928f209074b1100b5e17a
ARG BUN_CANARY_VERSION=1.4.5-canary.20261125.1
ARG BUN_CANARY_REVISION=1.4.6-canary.1+0966e99d8
ARG BUN_CANARY_SHA256=1adb5b5dd774fa7fa513e5f9fb49f9d18d5c221f32a201b544343ab218245051
ARG HK_VERSION=2.14.0
ARG HK_SHA256=230e89eaa3bf78aea23ac1584c8de9dd49141487f6067083003d8bdb38fa6c34

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    BUN_INSTALL=/usr/local/bun \
    PLAYWRIGHT_BROWSERS_PATH=/ms-playwright \
    PATH=/usr/local/cargo/bin:/usr/local/bun/bin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates curl git gcc g++ libc6-dev pkg-config python3 zstd jq xz-utils \
    && rm -rf /var/lib/apt/lists/*

# Node 22 binary only (no npm/yarn/corepack) for Playwright runner host & wrangler.
COPY --from=nodebin /usr/local/bin/node /usr/local/bin/node

# Rust toolchain (minimal profile + rustfmt, clippy, wasm32-unknown-unknown).
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal \
      --default-toolchain "${RUST_VERSION}" \
      --component rustfmt,clippy \
      --target wasm32-unknown-unknown \
    && rustc --version && cargo --version

# Prebuilt cargo-nextest (SHA-256 verified).
RUN curl -fsSL "https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-${NEXTEST_VERSION}/cargo-nextest-${NEXTEST_VERSION}-x86_64-unknown-linux-gnu.tar.gz" -o /tmp/nextest.tgz \
    && echo "${NEXTEST_SHA256}  /tmp/nextest.tgz" | sha256sum -c - \
    && tar -xzf /tmp/nextest.tgz -C /usr/local/cargo/bin cargo-nextest \
    && rm -f /tmp/nextest.tgz \
    && cargo nextest --version

# Prebuilt wasm-bindgen-cli locked to Cargo.lock (SHA-256 verified).
RUN curl -fsSL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${WASM_BINDGEN_VERSION}/wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl.tar.gz" -o /tmp/wbg.tgz \
    && echo "${WASM_BINDGEN_SHA256}  /tmp/wbg.tgz" | sha256sum -c - \
    && tar -xzf /tmp/wbg.tgz --strip-components=1 -C /usr/local/cargo/bin \
         "wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl/wasm-bindgen" \
         "wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl/wasm-bindgen-test-runner" \
    && rm -f /tmp/wbg.tgz \
    && wasm-bindgen --version

# Pinned immutable Bun canary (SHA-256 + revision verified).
RUN mkdir -p "${BUN_INSTALL}/bin" \
    && curl -fsSL "https://registry.npmjs.org/@oven/bun-linux-x64/-/bun-linux-x64-${BUN_CANARY_VERSION}.tgz" -o /tmp/bun.tgz \
    && echo "${BUN_CANARY_SHA256}  /tmp/bun.tgz" | sha256sum -c - \
    && tar -xzf /tmp/bun.tgz --strip-components=2 -C "${BUN_INSTALL}/bin" package/bin/bun \
    && ln -sf "${BUN_INSTALL}/bin/bun" "${BUN_INSTALL}/bin/bunx" \
    && rm -f /tmp/bun.tgz \
    && test "$(bun --revision)" = "${BUN_CANARY_REVISION}" \
    && bun --revision

# Pinned hk git-hooks runner (SHA-256 verified).
RUN curl -fsSL "https://github.com/jdx/hk/releases/download/v${HK_VERSION}/hk-x86_64-unknown-linux-gnu.tar.gz" -o /tmp/hk.tgz \
    && echo "${HK_SHA256}  /tmp/hk.tgz" | sha256sum -c - \
    && tar -xzf /tmp/hk.tgz -C /usr/local/bin hk \
    && rm -f /tmp/hk.tgz \
    && hk --version

RUN chmod -R a+rwX "${CARGO_HOME}" "${RUSTUP_HOME}"

FROM base AS browsers

ARG PLAYWRIGHT_VERSION=1.63.0

RUN mkdir -p "${PLAYWRIGHT_BROWSERS_PATH}" /tmp/pw \
    && bun add --cwd /tmp/pw "playwright@${PLAYWRIGHT_VERSION}" \
    && /tmp/pw/node_modules/.bin/playwright install --with-deps chromium webkit \
    && rm -rf /tmp/pw /var/lib/apt/lists/* \
    && chmod -R a+rX "${PLAYWRIGHT_BROWSERS_PATH}"
