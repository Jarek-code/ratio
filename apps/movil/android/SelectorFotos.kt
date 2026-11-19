package com.ratio.app

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import java.io.File

/**
 * Actividad auxiliar que abre el selector de fotos del sistema y deja la foto elegida en la
 * carpeta privada de la app.
 *
 * Existe porque Dioxus no expone el callback de resultados de actividad en Android: en vez de
 * esperar el resultado dentro de la app, esta actividad lo recibe, escribe los bytes en
 * `filesDir/Ratio/` y se cierra. Rust (`selector_android.rs`) la lanza con `startActivity` y
 * espera el archivo.
 *
 * Archivos que escribe, en `filesDir/Ratio/`:
 *
 * - `foto-pendiente`          bytes de la foto elegida.
 * - `foto-pendiente-tipo`     tipo MIME que declaró el selector.
 * - `foto-pendiente-estado`   `ok` si hay foto, `vacio` si la persona canceló.
 */
class SelectorFotos : Activity() {
    companion object {
        private const val CARPETA = "Ratio"
        private const val ARCHIVO_BYTES = "foto-pendiente"
        private const val ARCHIVO_TIPO = "foto-pendiente-tipo"
        private const val ARCHIVO_ESTADO = "foto-pendiente-estado"
        private const val PETICION = 4821

        /** Tipos que acepta el bucket de Storage, para no dejar elegir uno que se rechace. */
        private val TIPOS = arrayOf("image/jpeg", "image/png", "image/webp")
    }

    override fun onCreate(estado: Bundle?) {
        super.onCreate(estado)

        // Al girar la pantalla la actividad se recrea: no volver a abrir el selector
        if (estado != null) return

        limpiar()

        val intencion = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "image/*"
            putExtra(Intent.EXTRA_MIME_TYPES, TIPOS)
        }

        startActivityForResult(intencion, PETICION)
    }

    override fun onActivityResult(peticion: Int, resultado: Int, datos: Intent?) {
        super.onActivityResult(peticion, resultado, datos)

        val uri: Uri? = datos?.data
        val elegida = peticion == PETICION && resultado == RESULT_OK && uri != null

        if (elegida && uri != null) {
            try {
                // Copiar la foto a la carpeta privada: Rust no puede leer el content:// directamente
                contentResolver.openInputStream(uri)?.use { entrada ->
                    archivo(ARCHIVO_BYTES).outputStream().use { salida -> entrada.copyTo(salida) }
                }
                archivo(ARCHIVO_TIPO).writeText(contentResolver.getType(uri) ?: "image/jpeg")
                archivo(ARCHIVO_ESTADO).writeText("ok")
            } catch (causa: Exception) {
                limpiar()
                archivo(ARCHIVO_ESTADO).writeText("vacio")
            }
        } else {
            // Canceló o el sistema no devolvió nada: se avisa para que Rust no espere de más
            limpiar()
            archivo(ARCHIVO_ESTADO).writeText("vacio")
        }

        finish()
    }

    /** Archivo dentro de `filesDir/Ratio`, la misma carpeta que usa `sesion::carpeta_datos`. */
    private fun archivo(nombre: String): File {
        val carpeta = File(filesDir, CARPETA)
        carpeta.mkdirs()

        return File(carpeta, nombre)
    }

    /** Borra los restos de una elección anterior para no entregar una foto vieja. */
    private fun limpiar() {
        archivo(ARCHIVO_BYTES).delete()
        archivo(ARCHIVO_TIPO).delete()
        archivo(ARCHIVO_ESTADO).delete()
    }
}
