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

## Herramientas

Las tareas del proyecto están en `xtask`, en Rust:

```
cargo run -p xtask -- icono                 # ícono de la app (target/icono-cuadrado.png)
cargo run -p xtask -- icono --android DIR   # además lo reparte en los mipmap de Android
cargo run -p xtask -- env-render cuenta.json salida.env   # variables del servidor
cargo run -p xtask -- apk                   # APK firmado en target/ratio-android.apk
```

El APK también se compila en GitHub Actions: al empujar una etiqueta `v*` se publica como
release descargable, y desde la pestaña Actions se puede lanzar a mano.
