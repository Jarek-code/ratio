# Selector de fotos de Android

Dioxus 0.7 no expone el callback de resultados de actividad (`onActivityResult`) a Rust, y en
Android no usa un WebView, así que el `<input type="file">` no abre nada. La foto se elige con
una actividad auxiliar propia que escribe los bytes en la carpeta privada de la app; el lado
de Rust (`apps/movil/src/selector_android.rs`) la lanza por JNI y espera el archivo.

## Qué hay que hacer en el proyecto Android

El proyecto Gradle lo genera `dx` en cada compilación, así que estos dos archivos se copian
después de generarlo (o se mantienen en tu copia local del proyecto Android):

1. Copia `SelectorFotos.kt` en el paquete de la app:

   ```
   android/app/src/main/java/com/ratio/app/SelectorFotos.kt
   ```

   Si tu `applicationId` o tu paquete no es `com.ratio.app`, cambia la primera línea del
   archivo **y** la constante `CLASE` de `src/selector_android.rs` para que coincidan.

2. Declara la actividad en `android/app/src/main/AndroidManifest.xml`, dentro de
   `<application>`:

   ```xml
   <activity
       android:name=".SelectorFotos"
       android:exported="false"
       android:theme="@android:style/Theme.Translucent.NoTitleBar" />
   ```

3. Compila y prueba en el teléfono:

   ```
   dx serve --platform android
   ```

## Cómo se comunican

| Paso | Quién | Qué hace |
| --- | --- | --- |
| 1 | Rust | Borra los archivos de la elección anterior |
| 2 | Rust | Lanza `com.ratio.app.SelectorFotos` con `startActivity` por JNI |
| 3 | Kotlin | Abre `ACTION_OPEN_DOCUMENT` con los tipos que acepta el bucket |
| 4 | Kotlin | Copia la foto a `filesDir/Ratio/foto-pendiente` y su tipo a `foto-pendiente-tipo` |
| 5 | Kotlin | Escribe `ok` o `vacio` en `foto-pendiente-estado` y se cierra |
| 6 | Rust | Revisa la carpeta cada 250 ms, hasta 180 s, y devuelve los bytes |

Si la actividad no está en el APK, la app avisa con el mensaje «No se encontró la actividad del
selector», que es lo que se ve hasta que se copie el archivo en el proyecto Android.
