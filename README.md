# Ratio

Recetas de café y coctelería donde la proporción se ve dentro del vaso. Es un workspace de
Rust con el cliente móvil, el backend y el motor de proporciones.

## Carpetas

```
ratio/
├── apps/movil/            cliente móvil (Dioxus): interfaz y pantallas
├── crates/proporciones/   motor de las tazas: geometría, capas, graduación y unidades
├── crates/api/            tipos que viajan entre cliente y servidor, y server functions
├── crates/servidor/       backend Dioxus sin interfaz; registra las server functions
├── migraciones/           SQL de la base de datos
├── Dockerfile             imagen del backend
├── docker-compose.yml     PostgreSQL para desarrollo local
└── .env.example           variables del servidor
```

`apps/movil` y `crates/servidor` usan el mismo `crates/api` y el mismo
`crates/proporciones`: la proporción se calcula igual en los dos lados.

## Uso

```
cargo run -p servidor    # backend en IP:PORT
```

El cliente se compila desde `apps/movil` con la CLI de Dioxus:

```
dx serve --platform android
dx bundle --platform android --release
```
