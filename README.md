# Taller de Programacion {Grupo: Ferris}

## Integrantes
- Benito, Agustin
- Menechian, Javier
- La Rosa, Martina
- Levi, Dolores

## Como usar 

A continuacion se detallan los pasos para compilar y ejecutar el programa.

### Compilacion
```cargo build```

### Como correr

- Para orrer el Servidor -> `cargo run --bin server <n>`
- Para correr el Cliente -> `cargo run --bin client <n>`
- Para correr la UI -> `cargo run --bin gui <n>`
- Para correr el Simulador de vuelos -> `cargo run --bin simulator`

_Siento el `<n>` el numero de nodo._

### Como testear
```cargo test```

##

### Como cliente por terminal, se puede: 

-  Crear tablas de "Estado de los vuelos"
``` sql
CREATE TABLE origin  (airport STR, destination STR, date DATETIME, flight STR, status STR, timestamp TIMESTAMP, PRIMARY KEY (airport, date, flight)) WITH CLUSTERING ORDER BY (timestamp ASC)
CREATE TABLE destination (airport STR, origin STR, date DATETIME, flight STR, status STR, timestamp TIMESTAMP, PRIMARY KEY (airport, date, flight)) WITH CLUSTERING ORDER BY (timestamp ASC)
```

- Intertar nuevos vuelos:
``` sql
INSERT INTO origin (airport, destination, date, flight, status) VALUES (AEP, MEX, 25-12-24, 123, scheduled)
INSERT INTO destination (airport, origin, date, flight, status) VALUES (AEP, MEX, 25-12-24, 123, scheduled)
``` 

- Ver valores de vuelos:
``` sql
SELECT * FROM origin WHERE date = 25-12-24 AND airport = AEP
``` 


MEX,AEP,24-12-24,2,scheduled,

- Actualizar vuelos:
``` sql
UPDATE origin SET status = boarding WHERE flight = 101 AND airport = MEX AND date = 24-12-24
```

- Borrar vuelos:
``` sql
DELETE FROM origin WHERE flight = 101 AND airport = MEX AND date = 24-12-24
```

### Docker
(Se usa sólo para server).

#### Crear el Docker Compose
``` bash
./start.sh n
```
Siendo n la cantidad de nodos que se desean levantar.

#### Iniciar nodos
```bash
docker compose up
```

#### Cliente:
``` bash
cargo run --bin client 127.0.0.1 800n
```

#### GUI:
``` bash
cargo run --bin gui 127.0.0.1 800n
```

#### Simulador:
``` bash
cargo run --bin simulator 127.0.0.1 800n
```

#### Agregar un nuevo servidor:
```bash
docker run -d \
  --name node7 \
  --network network_net \
  --ip 10.0.0.7\
  -p 8007:8000 \
  -v $(pwd):/app \
  -v $(pwd)/data_8007/tables:/app/src/tables \
  -v $(pwd)/data_8007/logger:/app/src/loggerstuff/logs \
  -v $(pwd)/data_8007/hinted_handoff:/app/src/tables/logs \
  -v $(pwd)/target:/app/target \
  -w /app \
  rust:1.81 \
  bash -c 'apt-get update && apt-get install -y iputils-ping && cargo run --bin server 10.0.0.7'
```

##### Otro servidor de ejemplo
```bash
docker run -d \
  --name node8 \
  --network network_net \
  --ip 10.0.0.8\
  -p 8008:8000 \
  -v $(pwd):/app \
  -v $(pwd)/data_8008/tables:/app/src/tables \
  -v $(pwd)/data_8008/logger:/app/src/loggerstuff/logs \
  -v $(pwd)/data_8008/hinted_handoff:/app/src/tables/logs \
  -v $(pwd)/target:/app/target \
  -w /app \
  rust:1.81 \
  bash -c 'apt-get update && apt-get install -y iputils-ping && cargo run --bin server 10.0.0.8'
```


#### Para matar un nodo:
```bash
docker kill --signal=SIGINT container_name
```

