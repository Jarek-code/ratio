# Ratio

Recetas de café y coctelería donde la proporción se ve dentro del vaso. Es un workspace de
Rust con el cliente móvil, el backend y el motor de proporciones.

## Carpetas

```
ratio/
├── apps/movil/            cliente móvil (Dioxus): interfaz y pantallas
├── crates/proporciones/   motor de las tazas: geometría, capas, graduación, unidades y SVG
├── crates/api/            tipos que viajan entre cliente y servidor, y server functions
├── crates/servidor/       backend Dioxus sin interfaz; registra las server functions
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

### Base de datos

El backend abre un pool de SQLx al arrancar y `/salud` comprueba la conexión: `200` si la base
contesta, `503` si no (el servicio sigue en pie para que el diagnóstico se lea en el log).

```
cp .env.example .env     # y pon tu DATABASE_URL
cargo run -p servidor
curl http://127.0.0.1:8080/salud
```

En Render, `DATABASE_URL` va como variable de entorno del servicio (nunca en el repo) y usa la
cadena del **session pooler** de Supabase: Render no tiene salida IPv6 y la conexión directa
`db.<proyecto>.supabase.co` solo resuelve a IPv6.

El cliente se compila desde `apps/movil` con la CLI de Dioxus:

```
dx serve --platform android
dx bundle --platform android --release
```
