# Tahap 1: Kompilasi (Membangun aplikasi)
FROM rust:1.94-slim AS builder
WORKDIR /app
COPY . .
# Kita build dalam mode release agar ukurannya kecil dan performanya maksimal
RUN cargo build --release --bin sebuah-perpus

# Tahap 2: Menjalankan (Menyiapkan kontainer yang bersih)
FROM debian:bookworm-slim
WORKDIR /app

# Install dependencies yang sering dibutuhkan oleh Rust (seperti OpenSSL untuk database)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Salin hasil kompilasi dari Tahap 1
COPY --from=builder /app/target/release/sebuah-perpus /usr/local/bin/

# Buat folder untuk menyimpan file cover buku
RUN mkdir -p /app/storage

# Buka port 3000 sesuai dengan kode Axum kamu
EXPOSE 3000

# Jalankan aplikasinya
CMD ["sebuah-perpus"]