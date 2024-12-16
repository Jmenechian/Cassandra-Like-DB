# Usar la imagen base de Rust
FROM rust:latest

# Establecer el directorio de trabajo dentro del contenedor
WORKDIR /app

# Instalar dependencias necesarias
RUN apt-get update && \
    apt-get install -y iputils-ping && \
    rustup target add x86_64-apple-darwin


# Copiar los archivos del proyecto al contenedor (opcional)
COPY . /app

# Mantener el contenedor activo
CMD ["bash", "-c", "tail -f /dev/null"]
