# ============================================================
#  Etapa 1️⃣ : Builder con cache de dependencias
# ============================================================
FROM rustlang/rust:nightly-bookworm-slim AS builder

WORKDIR /usr/src/app

# ✅ Instalar dependencias del sistema necesarias
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    xz-utils \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# ✅ Instalar sqlx-cli (solo PostgreSQL)
RUN cargo install sqlx-cli --no-default-features --features postgres

# ------------------------------------------------------------
# 🧩 Fase 1: Cache de dependencias
# ------------------------------------------------------------
# Copiamos solo los manifiestos para compilar dependencias
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Compilamos dependencias sin el código real (cachea crates)
RUN cargo build --release

# ------------------------------------------------------------
# 🧩 Fase 2: Copiar el proyecto real y FORZAR recompilación
# ------------------------------------------------------------
COPY . .

# Habilitar modo offline de SQLx
ENV SQLX_OFFLINE=true

# ⚠️ CRÍTICO: Eliminar el binario dummy y forzar recompilación completa
RUN rm -rf target/release/main target/release/deps/main-* && \
    touch src/main.rs && \
    cargo build --release

# ✅ Verificar que el binario contiene el código correcto
RUN strings target/release/main | grep -q "Sistema Bancario\|Logging configurado" || \
    (echo "❌ ERROR: El binario no contiene el código esperado" && exit 1)

# ============================================================
#  Etapa 2️⃣ : Imagen final (Runtime)
# ============================================================
FROM debian:bookworm-slim AS final

WORKDIR /app

# ✅ Instalar dependencias de runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    netcat-traditional \
    postgresql-client \
    dos2unix \
    && rm -rf /var/lib/apt/lists/*

# ------------------------------------------------------------
# 🧩 Copiar artefactos desde el builder
# ------------------------------------------------------------
# 1. El binario principal (ajusta el nombre si no es "main")
COPY --from=builder /usr/src/app/target/release/main ./main

# 2. sqlx-cli
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

# 3. Scripts y migraciones
COPY --from=builder /usr/src/app/init-services.sh ./init-services.sh
COPY --from=builder /usr/src/app/migrations ./migrations
COPY entrypoint.sh ./entrypoint.sh

# Dar permisos y convertir fin de línea
RUN chmod +x ./entrypoint.sh ./init-services.sh \
    && dos2unix ./entrypoint.sh ./init-services.sh

# Puerto expuesto
EXPOSE 5000

ENTRYPOINT ["./entrypoint.sh"]