FROM rust:1.75.0 AS build-env

ARG BUILD_DATE
ARG VCS_REF
LABEL maintainer="Laurent <laurent@vromman.org>" \
    org.opencontainers.image.title="NavData Backend" \
    org.opencontainers.image.description="Endpoint to get information about airports or navaids" \
    org.opencontainers.image.authors="Laurent <laurent@vromman.org>" \
    org.opencontainers.image.vendor="Laurent Vromman" \
    org.opencontainers.image.documentation="https://github.com/leirn/navdata/README.md" \
    org.opencontainers.image.licenses="MIT" \
    org.opencontainers.image.version="0.1.0" \
    org.opencontainers.image.url="https://github.com/leirn/navdata/" \
    org.opencontainers.image.source="https://github.com/leirn/navdata/" \
    org.opencontainers.image.revision=$VCS_REF \
    org.opencontainers.image.created=$BUILD_DATE

WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc

ENV DATABASE_FOLDER=/data

VOLUME "/data"

ARG DEFAULT_IMAGE="/default.jpg"
ENV DEFAULT_IMAGE=${DEFAULT_IMAGE}

ENV HOST=0.0.0.0

ENV PORT=8080

ARG DATABASE_PATH=":memory:"
ENV DATABASE_PATH=${DATABASE_PATH}

ARG TOKEN_LIST=""
ENV TOKEN_LIST=${TOKEN_LIST}

ARG HTTPS="true"
ENV HTTPS=${HTTPS}

ARG RUST_LOG="warn"
ENV RUST_LOG=${RUST_LOG}

ARG RUST_BACKTRACE="0"
ENV RUST_BACKTRACE=${RUST_BACKTRACE}

EXPOSE 8080

COPY --from=build-env /app/target/release/api_backend /
COPY resources/default.jpg /
CMD ["./navdata"]