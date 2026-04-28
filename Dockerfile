# Stage 1: builder
FROM rust:1.95-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY templates ./templates
COPY static ./static
COPY content ./content
RUN cargo build --release

# Stage 2: runtime
FROM debian:bookworm-slim
RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates \
	&& rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/blog /usr/local/bin/blog
COPY --from=builder /app/static ./static
COPY --from=builder /app/content ./content
ENV PORT=3000 APP_ENV=production
EXPOSE 3000
CMD [ "/usr/local/bin/blog" ]
