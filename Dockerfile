FROM rust:1.90 as codebase-viewer-builder
WORKDIR /build
COPY ./codebase_viewer /build/codebase_viewer
WORKDIR /build/codebase_viewer
RUN cargo build --release

FROM rust:1.90 as agent-builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=codebase-viewer-builder /build/codebase_viewer/target/release/codebase_viewer /usr/local/bin/codebase_viewer
COPY --from=agent-builder /app/target/release/ai_code_agent /usr/local/bin/ai_code_agent

ENTRYPOINT ["ai_code_agent", "--codebase-viewer-path", "/usr/local/bin/codebase_viewer"]
