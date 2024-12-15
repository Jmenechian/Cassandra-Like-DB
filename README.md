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
INSERT INTO origin (airport, destination, date, flight, status) VALUES (AEP, MEX, 25-12-24, 123, scheduled)
``` 

- Ver valores de vuelos:
``` sql
SELECT * FROM origin WHERE date = 25-12-24 AND airport = AEP
``` 

- Actualizar vuelos:
``` sql
UPDATE origin SET status = boarding WHERE flight = 101 AND airport = MEX AND date = 24-12-24
```

- Borrar vuelos:
``` sql
DELETE FROM origin WHERE flight = 101 AND airport = MEX AND date = 24-12-24
```


keys [417, 983, 1440, 1977, 2042, 2807, 2842, 2872, 3383, 4230, 4264, 4405, 4627, 4631, 4659, 4852, 5077, 5218, 5321, 5368, 5420, 5858, 6008, 6024, 7319, 7468, 7577]

keys [2, 9, 9, 6, 3, 2, 4, 4, 5, 8, 1, 7, 6, 3, 1, 3, 5, 5, 4, 8, 8, 9, 6, 7, 2, 7, 1]


Tokens: [4264, 7577, 4659] => 1
Tokens: [2807, 417, 7319] => 2
Tokens: [2042, 4631, 4852] => 3
Tokens: [2872, 5321, 2842] => 4
Tokens: [5218, 3383, 5077] => 5
Tokens: [1977, 6008, 4627] => 6
Tokens: [4405, 6024, 7468] => 7
Tokens: [4230, 5420, 5368] => 8

Tokens: [5858, 1440, 983] => 9