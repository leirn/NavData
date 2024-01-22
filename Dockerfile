FROM rust:1.75.0 AS build-env

ARG BUILD_DATE
ARG VCS_REF
LABEL maintainer="Laurent <laurent@vromman.org>" \
    org.opencontainers.image.title="NavData REST API" \
    org.opencontainers.image.description="REST API to get information about airports or navaids" \
    org.opencontainers.image.authors="Laurent <laurent@vromman.org>" \
    org.opencontainers.image.vendor="Laurent Vromman" \
    org.opencontainers.image.documentation="https://github.com/leirn/navdata/README.md" \
    org.opencontainers.image.licenses="MIT" \
    org.opencontainers.image.version="0.1.5" \
    org.opencontainers.image.url="https://github.com/leirn/navdata/" \
    org.opencontainers.image.source="https://github.com/leirn/navdata/" \
    org.opencontainers.image.revision=$VCS_REF \
    org.opencontainers.image.created=$BUILD_DATE

WORKDIR /app
COPY . /app
RUN apt-get update && apt-get -y install sqlite3
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:lastest

ENV DATABASE_FOLDER=/data

VOLUME "/data"
VOLUME "/config"

ARG RUST_LOG="warn"
ENV RUST_LOG=${RUST_LOG}

ARG RUST_BACKTRACE="0"
ENV RUST_BACKTRACE=${RUST_BACKTRACE}

EXPOSE 8080

COPY --from=build-env /app/target/release/nav_data /
COPY --from=build-env /usr/lib/x86_64-linux-gnu/libsqlite3.so.0.8.6 /usr/lib/x86_64-linux-gnu/libsqlite3.so.0
CMD ["./nav_data --config /config/config.yaml"]