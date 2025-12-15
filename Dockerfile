FROM rust:1.91-alpine AS builder
RUN apk add --no-cache musl-dev curl bzip2
WORKDIR /usr/src/covdir-report-action

# Download grcov
ARG GRCOV_VERSION=0.10.5
RUN curl -sL https://github.com/mozilla/grcov/releases/download/v${GRCOV_VERSION}/grcov-x86_64-unknown-linux-musl.tar.bz2 \
    | tar xjf - -C /usr/local/bin

COPY . .
RUN cargo install --path .

FROM scratch
COPY --from=builder /usr/local/bin/grcov /usr/local/bin/grcov
COPY --from=builder /usr/local/cargo/bin/covdir-report-action /usr/local/bin/covdir-report-action
ENTRYPOINT ["/usr/local/bin/covdir-report-action"]
