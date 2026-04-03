FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy pre-built server binary
COPY target/release/server ./server

# Copy pre-built frontend assets
COPY target/site ./target/site

# Copy migrations for embedded sqlx migrations
COPY migrations ./migrations

ENV DATABASE_URL=sqlite:///app/data/data.db?mode=rwc
ENV RUST_LOG=info

EXPOSE 8080

# Create data directory for SQLite
RUN mkdir -p /app/data

COPY start.sh ./start.sh
RUN chmod +x ./start.sh

CMD ["./start.sh"]
