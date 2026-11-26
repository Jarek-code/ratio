# Ratio

Aplicación de recetas de café y coctelería en la que la proporción de cada bebida se ve
dentro del vaso. Cada receta se dibuja con las cantidades reales de sus ingredientes, y al
ajustarlas el dibujo cambia con ellas: la altura de cada capa, el color y la graduación
salen del cálculo, no de una imagen.

El repositorio es un espacio de trabajo de Rust con cinco piezas: el cliente, el backend,
el motor de proporciones, los tipos que ambos comparten y las herramientas de compilación.
El cliente y el servidor usan el mismo motor, así que una bebida se calcula igual en la
pantalla que en la base de datos.

## Qué hace

- **Biblioteca** con el catálogo completo, búsqueda y filtros por familia y por pestaña de
  café o coctelería.
- **Ficha de receta** con el vaso dibujado por capas, ingredientes, pasos, equipo, fotos y
  la graduación resultante.
- **Crear y ajustar** recetas: se elige el vaso, se agregan ingredientes del catálogo y
  cada capa se ajusta con su deslizador. La vista previa se actualiza mientras se edita.
- **Historial de versiones** de una bebida, con la receta original y sus derivadas.
- **Equivalencias** de los ingredientes en mililitros, onzas o partes, y **lista de compra**
  a partir de las recetas elegidas.
- **Preparación guiada** paso a paso, con temporizador.
- **La Barra**: muro con recetas publicadas, comentarios, respuestas, valoraciones,
  seguimiento entre cuentas y reportes de contenido.
- **Actividad** con las novedades de las recetas propias, y **moderación** con la cola de
  reportes y el historial de decisiones, reservada a las cuentas con rol de moderación.
- **Cuenta** con alias, declaración de mayoría de edad y perfil público.
- **Ajustes** de unidades, tema claro u oscuro, idioma (español, inglés y mandarín
  simplificado) y repaso de la bienvenida.
- **Sin conexión**: el recetario y los guardados se conservan en el teléfono, y los cambios
  hechos sin red quedan en una cola que se envía al recuperar la conexión.
- **Notificaciones** push cuando alguien responde a un comentario, valora una receta, publica
  una versión derivada o cuando moderación revisa un reporte.

## Cómo está hecho

| Pieza | Tecnología |
| --- | --- |
| Cliente | Dioxus 0.7, compilado a Android, iOS y escritorio |
| Servidor | Dioxus fullstack sobre Axum, sin interfaz |
| Base de datos | PostgreSQL en Supabase, con SQLx |
| Autenticación | Supabase Auth |
| Fotos | Supabase Storage |
| Push | Firebase Cloud Messaging en Android y APNs en iOS |
| Servidor desplegado | Render |
| APK | Gradle con el proyecto que genera `dx`, firmado con `apksigner` |

El motor de proporciones no depende de ninguna de esas piezas: recibe una lista de
ingredientes con su cantidad y devuelve la geometría del vaso, el reparto por capas, la
graduación y el dibujo en SVG.

## Estructura

```
ratio/
├── apps/movil/            cliente: pantallas, cuenta, copia local, fotos y push
├── crates/proporciones/   motor: geometría, capas, graduación, unidades y SVG
├── crates/api/            tipos compartidos y server functions
├── crates/servidor/       backend: registra las server functions y sirve /salud
├── xtask/                 herramientas del proyecto (ícono, variables, APK)
├── migrations/            esquema y cambios de la base, en orden
├── docs/                  esquema de la base en Graphviz
├── .github/workflows/     compilación del APK y publicación del release
├── Dockerfile             imagen del backend
└── docker-compose.yml     PostgreSQL para desarrollo local
```

## Requisitos

- Rust estable, con los objetivos de Android que se indican más abajo.
- Docker, para la base de datos local.
- Para el cliente móvil: Java 17, el SDK de Android y el NDK 30.
- El CLI de Dioxus: `cargo install dioxus-cli --version 0.7.10 --locked`.

## Puesta en marcha

### 1. Base de datos

```sh
docker compose up -d
```

Levanta PostgreSQL 16 en el puerto 5432 con la base `ratio`, el usuario `ratio` y la
contraseña `ratio`, que es lo que espera `DATABASE_URL` en `.env.example`.

### 2. Variables del servidor

```sh
cp .env.example .env
```

El servidor lee ese archivo al arrancar. Las obligatorias son `DATABASE_URL` y las de
Supabase; las de notificaciones se pueden dejar vacías, porque el servidor funciona sin
push.

| Variable | Para qué |
| --- | --- |
| `IP`, `PORT` | Dirección en la que escucha el servidor |
| `RUST_LOG` | Nivel de registro |
| `DATABASE_URL` | Cadena de conexión de PostgreSQL |
| `MIGRACIONES_AL_ARRANCAR` | Aplica `migrations/` al iniciar; en producción se deja en `false` |
| `SUPABASE_URL`, `SUPABASE_ANON_KEY` | Proyecto de Supabase: autenticación, fotos y consultas |
| `FCM_PROJECT_ID`, `FCM_CLIENT_EMAIL`, `FCM_PRIVATE_KEY` | Cuenta de servicio de Firebase para Android |
| `APNS_KEY_P8`, `APNS_KEY_ID`, `APNS_TEAM_ID`, `APNS_SANDBOX` | Clave de APNs para iOS |

`FCM_PRIVATE_KEY` lleva la clave completa con los saltos de línea escapados (`\n`).
`cargo run -p xtask -- env-render` arma ese bloque a partir del JSON de la cuenta de
servicio, que se descarga de la consola de Firebase.

### 3. Servidor

```sh
cargo run -p servidor
```

Queda escuchando en `http://127.0.0.1:8080`, con `GET /salud` para comprobar que responde.
Si falta configuración el servidor arranca igual y esa ruta informa qué falta, en lugar de
terminar sin dejar diagnóstico.

### 4. Cliente

```sh
rustup target add aarch64-linux-android
cd apps/movil
dx serve --platform android
```

`dx serve` compila el cliente, arma el proyecto de Gradle y lo instala en el teléfono o en
el emulador conectado. Para trabajar solo en la interfaz se puede usar
`dx serve --platform desktop`.

El cliente apunta al servidor desplegado. Para usar el servidor local se compila con la
variable `RATIO_SERVIDOR`:

```sh
RATIO_SERVIDOR=http://192.168.1.10:8080 dx build --platform android --release
```

La dirección tiene que ser alcanzable desde el teléfono, así que se usa la IP de la
computadora en la red local: `127.0.0.1` apunta al propio teléfono.

## Base de datos

El esquema vive en `migrations/`, con un archivo por cambio y en orden de fecha. Se aplica
con `MIGRACIONES_AL_ARRANCAR=true` o con `sqlx migrate run`.

`docs/esquema.dot` describe las tablas y sus relaciones; el dibujo se genera con:

```sh
dot -Tsvg docs/esquema.dot -o docs/esquema.svg
```

Las consultas se validan al compilar con `SQLX_OFFLINE=true` y el directorio `.sqlx/`, que
se regenera con `cargo sqlx prepare` cuando cambia alguna consulta.

## Despliegue

El backend se despliega en Render con el `Dockerfile` del repositorio. Antes de publicar
hay que tener la base de Supabase con las migraciones aplicadas y las variables del
servidor cargadas en el panel.

La aplicación publicada usa `https://ratio-689g.onrender.com`. Si el servicio cambia de
dirección, se ajusta en `apps/movil/src/main.rs` o se compila el cliente con
`RATIO_SERVIDOR`.

## Aplicación Android

```sh
cargo run -p xtask -- apk
```

El comando compila las dos arquitecturas (`arm64-v8a` y `armeabi-v7a`), reparte el ícono
en las densidades del lanzador, arma el APK de release, lo firma y lo deja en
`target/ratio-android.apk`. Necesita los dos objetivos instalados:

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi
```

`.parches/manganis` es una copia del paquete de recursos de Dioxus sin la restricción de 64
bits, declarada en `.cargo/config.toml`. Sin ese parche el cliente no compila para
`armeabi-v7a`, que es la arquitectura de los teléfonos de 32 bits. Se puede retirar cuando
esa corrección llegue a una versión 0.7 de Dioxus.

La clave de firma se crea la primera vez en `target/ratio-release.jks`. Para usar otra se
indican `RATIO_KEYSTORE`, `RATIO_KEYSTORE_PASSWORD` y `RATIO_KEY_ALIAS`.

El APK se instala en teléfonos desde Android 7.0 (API 24). Si cambia la clave de firma hay
que desinstalar la versión anterior antes de instalar la nueva.

En iOS el cliente se compila con `dx build --platform ios`. El envío de notificaciones por
APNs requiere una cuenta de Apple Developer de pago; el permiso está descrito y apagado en
`Dioxus.toml`.

### Publicación

El flujo de `.github/workflows/android.yml` compila el APK en GitHub Actions y lo publica
como release descargable al empujar una etiqueta:

```sh
git tag v0.2.1
git push origin v0.2.1
```

El número de versión sale de `apps/movil/Cargo.toml`, y el `versionCode` de Android se
calcula a partir de él. Para firmar con la clave del proyecto se crean tres secretos en el
repositorio: `ANDROID_KEYSTORE_B64` (la clave en base64), `ANDROID_KEYSTORE_PASSWORD` y
`ANDROID_KEY_ALIAS`. Sin ellos el APK se firma con una clave nueva en cada compilación.

## Herramientas

Las tareas del proyecto están en `xtask`:

```sh
cargo run -p xtask -- icono                              # ícono cuadrado de la app
cargo run -p xtask -- icono --android <res>              # y sus densidades en Android
cargo run -p xtask -- env-render <cuenta.json> <salida>  # variables del servidor
cargo run -p xtask -- apk                                # APK firmado
```

## Documentación de las tecnologías

- [Dioxus](https://dioxuslabs.com/learn/0.7/)
- [SQLx](https://docs.rs/sqlx)
- [Supabase](https://supabase.com/docs)
- [Firebase Cloud Messaging](https://firebase.google.com/docs/cloud-messaging)
- [Render](https://render.com/docs)
