#!/bin/bash

# Verificar el número de nodos a través de la variable de entorno
if [ -z "$1" ]; then
  echo "Error: La variable NUM_NODOS no está definida."
  exit 1
fi

# Realizar el ping a cada uno de los nodos (de 10.0.0.2 a 10.0.0.(NUM_NODOS+1))
for i in $(seq 2 $(($1 + 1))); do
  IP_ADDRESS="10.0.0.$i"
  echo "Realizando ping a $IP_ADDRESS..."

  # Ejecutar el ping y esperar respuesta (un máximo de 4 intentos)
  ping -c 4 "$IP_ADDRESS" > /dev/null 2>&1

  if [ $? -eq 0 ]; then
    echo "El nodo $IP_ADDRESS está alcanzable."
  else
    echo "Error: El nodo $IP_ADDRESS no responde."
  fi
done
