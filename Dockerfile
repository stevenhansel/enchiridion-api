# Stage 1
FROM lukemathwalker/cargo-chef:latest-rust-1.60.0 as chef

WORKDIR /app
RUN apt update && apt install lld clang -y

# Stage 2
FROM chef as planner
COPY . .

# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

# Stage 3 
FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json

# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json

# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
ENV SQLX_OFFLINE true

# Build our project
RUN cargo build --release --bin enchiridion_api

# Stage 4 
FROM debian:bullseye-slim AS runtime

WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/enchiridion_api enchiridion_api

ENTRYPOINT ["./api"]