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

## Acceso

El formulario de la app habla con las server functions `entrar`, `registrar` y `sesion`, que
reenvían la contraseña a **Supabase Auth** y reciben un token. El servidor no guarda ni hashea
contraseñas: el hash vive en `auth.users.encrypted_password`, y del token solo se pide la cuenta
para leer su perfil en `usuario` (alias, rol y mayoría de edad). La app guarda el token en
memoria y lo reenvía en cada petición que lo necesite, como las sugerencias del día.

Para que funcione hacen falta dos variables de entorno en el servidor:

```
SUPABASE_URL=https://<proyecto>.supabase.co
SUPABASE_ANON_KEY=<clave pública: anon o publishable (sb_publishable_…), Ajustes → API>
```

Si el proyecto exige confirmar el correo (Auth → Providers → Email), el alta crea la cuenta pero
no inicia la sesión hasta que la persona confirme; la app lo avisa con ese mensaje.

En Render, `DATABASE_URL` va como variable de entorno del servicio (nunca en el repo) y usa la
cadena del **session pooler** de Supabase: Render no tiene salida IPv6 y la conexión directa
`db.<proyecto>.supabase.co` solo resuelve a IPv6.

El cliente se compila desde `apps/movil` con la CLI de Dioxus:

```
dx serve --platform android
dx bundle --platform android --release
```
