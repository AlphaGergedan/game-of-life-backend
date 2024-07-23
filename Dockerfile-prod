FROM rust:1.79.0 AS builder
ARG RELEASE
WORKDIR /usr/src/game-of-life-backend
COPY . .
RUN if [ "${RELEASE}" = "true" ]; then cargo build --release; else cargo build; fi

FROM debian:bookworm-slim AS runner
RUN addgroup --gid 1000 gameoflife && adduser --uid 1000 --gid 1000 --gecos "" --disabled-password gameoflife
USER gameoflife
WORKDIR /usr/local/bin
ARG BACKEND=/usr/src/game-of-life-backend/target/release/backend
EXPOSE 8080
COPY --from=builder ${BACKEND} backend
CMD ["./backend"]
