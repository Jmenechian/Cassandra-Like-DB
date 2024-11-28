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
```

``` sql
CREATE TABLE destination (airport STR, origin STR, date DATETIME, flight STR, status STR, timestamp TIMESTAMP, PRIMARY KEY (airport, date, flight)) WITH CLUSTERING ORDER BY (timestamp ASC)
```
- Intertar nuevos vuelos:
``` sql
INSERT INTO origin (airport, destination, date, flight, status) VALUES (MEX, AEP, 24-12-24, 123, scheduled)
``` 

- Ver valores de vuelos:
``` sql
SELECT * FROM origin WHERE date = 24-12-24 AND airport = MEX
``` 

- Actualizar vuelos:
``` sql
UPDATE origin SET status = boarding WHERE flight = 101 AND airport = MEX AND date = 24-12-24
```

- Borrar vuelos:
``` sql
DELETE FROM origin WHERE flight = 101 AND airport = MEX AND date = 24-12-24
```
