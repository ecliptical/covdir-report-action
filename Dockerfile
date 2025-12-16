FROM rust:1.92-alpine AS builder
RUN apk add --no-cache musl-dev curl bzip2
RUN rustup component add llvm-tools
# Find and copy llvm tools to a known location
RUN mkdir -p /llvm-tools && \
    find /usr/local/rustup -name 'llvm-profdata' -exec cp {} /llvm-tools/ \; && \
    find /usr/local/rustup -name 'llvm-cov' -exec cp {} /llvm-tools/ \;
WORKDIR /usr/src/covdir-report-action

# Download grcov for the current architecture
ARG GRCOV_VERSION=0.10.5
RUN curl -sL "https://github.com/mozilla/grcov/releases/download/v${GRCOV_VERSION}/grcov-$(uname -m)-unknown-linux-musl.tar.bz2" \
    | tar xjf - -C /usr/local/bin

COPY . .
RUN cargo install --path .

FROM scratch
COPY --from=builder /usr/local/bin/grcov /usr/local/bin/grcov
COPY --from=builder /usr/local/cargo/bin/covdir-report-action /usr/local/bin/covdir-report-action
# Copy LLVM tools required by grcov --llvm flag
COPY --from=builder /llvm-tools/llvm-profdata /usr/local/bin/llvm-profdata
COPY --from=builder /llvm-tools/llvm-cov /usr/local/bin/llvm-cov
# Copy musl libc and libgcc_s required by LLVM tools
COPY --from=builder /lib/ld-musl-*.so.1 /lib/
COPY --from=builder /usr/lib/libgcc_s.so.1 /usr/lib/
# Default LLVM tools path (can be overridden)
ENV LLVM_PATH=/usr/local/bin
ENTRYPOINT ["/usr/local/bin/covdir-report-action"]
