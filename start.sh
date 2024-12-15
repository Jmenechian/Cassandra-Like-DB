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
  if [ "$i" -eq 2 ]; then
    COMMAND="bash -c \"apt-get update && apt-get install -y \
                   iputils-ping libglvnd0 libglvnd-dev libgl1-mesa-glx \
                   libgl1-mesa-dri libglu1-mesa mesa-utils libx11-dev \
                   x11-apps libxkbcommon-x11-0 && \
                   rustup target add x86_64-apple-darwin && \
                   cargo run --bin server 10.0.0.2\""
  else
    # Comando para los demás nodos
    COMMAND="bash -c \"apt-get update && apt-get install -y \
                   iputils-ping libglvnd0 libglvnd-dev libgl1-mesa-glx \
                   libgl1-mesa-dri libglu1-mesa mesa-utils libx11-dev \
                   x11-apps libxkbcommon-x11-0 && \
                   rustup target add x86_64-apple-darwin && \
                   cargo run --bin server ${IP_ADDRESS} 10.0.0.2\"
                   "
  fi

  # Exportar las variables de entorno
  export NODE_NAME=$NODE_NAME
  export IP_ADDRESS=$IP_ADDRESS
  export NODE_NUMBER=$NODE_NUMBER
  export COMMAND=$COMMAND

  # Usar envsubst para sustituir las variables en la plantilla y agregarlas al archivo
  envsubst < "$TEMPLATE_FILE" >> "$TMP_COMPOSE_FILE"



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

# Ejecutar docker-compose para levantar los contenedores
if docker-compose -f "$TMP_COMPOSE_FILE" up -d; then
  echo "Todos los contenedores se levantaron correctamente."
else
  echo "Hubo un error al levantar los contenedores."
  exit 1
fi

# Abrir una terminal para cada contenedor solo si está en ejecución
for i in $(seq 2 $((NUM_NODOS + 1))); do
  sleep 10
  NODE_NAME="node$i"
  CONTAINER_NAME=$(docker ps --filter "name=$NODE_NAME" --format "{{.Names}}")

  if [ -n "$CONTAINER_NAME" ]; then
      echo "Abriendo terminal para $CONTAINER_NAME"
      # Detectar el sistema operativo
      OS=$(uname)

      if [ "$OS" == "Darwin" ]; then
          # Para macOS
          osascript -e "
          tell application \"Terminal\"
              activate
              do script \"arch -x86_64 zsh\"
              delay 1 -- Espera 1 segundo para asegurarte de que el comando anterior se ejecute
              do script \"docker logs -f $CONTAINER_NAME\" in front window
              delay 2
          end tell
          "
      elif [ "$OS" == "Linux" ]; then
          # Para Linux (suponiendo que usas gnome-terminal)
          gnome-terminal -- bash -c "docker exec -it $CONTAINER_NAME bash -c '/app/ping_check.sh $NUM_NODOS'; exec bash"
      else
          echo "Sistema operativo no soportado"
      fi
  else
      echo "El contenedor $NODE_NAME no está en ejecución."
  fi
done

# Limpiar archivo temporal
#rm -f "$TMP_COMPOSE_FILE"
#-c '/app/ping_check.sh $NUM_NODOS'
echo "Todos los nodos han sido creados."