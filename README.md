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

- para correr servidor -> `cargo run --bin server <n>`
- para correr cliente -> `cargo run --bin client <n>`
- para correr ui -> `cargo run --bin gui <n>`

### Como testear
```cargo test```

##

### Como cliente por terminal, se puede: 

-  Crear tablas de "Estado de los vuelos"
``` sql
CREATE TABLE origin  ( airport STR,
                        destination STR, 
                        date DATETIME,
                        flight STR,
                        status STR, 
                        timestamp TIMESTAMP, 
                        PRIMARY KEY (airport, date, flight) )
WITH CLUSTERING ORDER BY (timestamp ASC)
```

``` sql
CREATE TABLE destination (  airport STR,
                            origin STR, 
                            date DATETIME,
                            flight STR,
                            status STR, 
                            timestamp TIMESTAMP, 
                            PRIMARY KEY (airport, date, flight) )
WITH CLUSTERING ORDER BY (timestamp ASC)
```
- Intertar nuevos vuelos:
``` sql
INSERT INTO origin (airport, destination, date, flight, status)
VALUES (MEX, AEP, 24-12-24, 123, scheduled)
``` 

- Ver valores de vuelos:
``` sql
SELECT *
FROM origin
WHERE date = 24-12-24
  AND airport = MEX
``` 

- Actualizar vuelos:
``` sql
UPDATE origin
SET status = boarding
WHERE flight = 101
  AND airport = MEX
  AND date = 24-12-24
```

- Borrar vuelos:
``` sql
DELETE
FROM origin
WHERE flight = 101
  AND airport = MEX
  AND date = 24-12-24
```
