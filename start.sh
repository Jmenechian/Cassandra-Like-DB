#!/bin/bash

# Asegúrate de recibir el número de nodos como argumento
if [ -z "$1" ]; then
  echo "Uso: $0 <número_de_nodos>"
  exit 1
fi

NUM_NODOS=$1

# Validar que el número de nodos sea válido
if ! [[ "$NUM_NODOS" =~ ^[0-9]+$ ]] || [ "$NUM_NODOS" -le 0 ]; then
  echo "Error: El número de nodos debe ser un número entero positivo."
  exit 1
fi

TEMPLATE_FILE="docker-compose.template.yml"
TMP_COMPOSE_FILE="docker-compose.yml"

NETWORK_NAME="network_net"
SUBNET="10.0.0.0/24"

echo "Verificando la red network_net..."
if ! docker network inspect network_net >/dev/null 2>&1; then
  echo "Creando la red network_net con el subnet 10.0.0.0/24..."
  docker network create --driver bridge --subnet=10.0.0.0/24 network_net || {
    echo "Error: No se pudo crear la red. Verifica los permisos de Docker."
    exit 1
  }
else
  echo "La red $NETWORK_NAME ya existe. Usándola."
fi

# Crear el archivo docker-compose.yml vacío con la cabecera
echo "version: '3.8'" > "$TMP_COMPOSE_FILE"
echo "services:" >> "$TMP_COMPOSE_FILE"

# Crear nodos dinámicamente
for i in $(seq 2 $((NUM_NODOS + 1))); do
  IP_ADDRESS="10.0.0.$((i))"  # La IP empieza desde 10.0.0.2
  NODE_NAME="node$i"
  NODE_NUMBER=$((8000 + i))

  echo "Creando nodo $NODE_NAME con IP $IP_ADDRESS"

  # Verificar si ya existe un contenedor con el mismo nombre
  EXISTING_CONTAINER=$(docker ps -a --filter "name=$NODE_NAME" --format "{{.Names}}")
  
  if [ -n "$EXISTING_CONTAINER" ]; then
    echo "El contenedor $NODE_NAME ya existe. Eliminando contenedor..."
    docker stop "$EXISTING_CONTAINER" >/dev/null 2>&1
    docker rm "$EXISTING_CONTAINER" >/dev/null 2>&1
  fi
  # Comando para el primer nodo
  COMMAND="bash -c \"apt-get update && apt-get install -y \
          iputils-ping && \
          cargo run --bin server ${IP_ADDRESS}\"
          "

  # Exportar las variables de entorno
  export NODE_NAME=$NODE_NAME
  export IP_ADDRESS=$IP_ADDRESS
  export NODE_NUMBER=$NODE_NUMBER
  export COMMAND=$COMMAND

  if [ "$i" -eq 2 ]; then
  cat <<EOL >> "$TMP_COMPOSE_FILE"
  builder:
    container_name: builder
    build:
      context: .
      dockerfile: Dockerfile
    image: my_build_image
    healthcheck:
      test: ["CMD", "sh", "-c", "test -f /target/release/server"]  # Check if the binary is created
      interval: 10s
      timeout: 15s
      retries: 5
    command: bash -c "cargo build --release && sleep infinity" 
EOL
  fi 


  # Usar envsubst para sustituir las variables en la plantilla y agregarlas al archivo
  envsubst < "$TEMPLATE_FILE" >> "$TMP_COMPOSE_FILE"

  if [ "$i" -eq 2 ]; then
    cat <<EOL >> "$TMP_COMPOSE_FILE"
    healthcheck:
      test: ["CMD", "sh", "-c", "nc -z localhost 8002"] # Check if TCP port 8002 is open
      interval: 10s
      timeout: 15s
      retries: 100
    depends_on:
      builder:
        condition: service_healthy
EOL
  else 
    cat <<EOL >> "$TMP_COMPOSE_FILE"
    depends_on:
      node2:
        condition: service_healthy
      builder:
        condition: service_healthy
EOL
  fi

  # Limpiar las variables de entorno después de usarlas
  unset NODE_NAME
  unset IP_ADDRESS
  unset NODE_NUMBER
  unset COMMAND
done

# Añadir el bloque de red al final
cat <<EOF >> "$TMP_COMPOSE_FILE"
networks:
  network_net:
    external: true
EOF