package dev.dioxus.main

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import com.google.firebase.FirebaseApp
import com.google.firebase.FirebaseOptions
import com.google.firebase.messaging.FirebaseMessaging
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import java.io.File

typealias BuildConfig = com.ratio.app.BuildConfig

/// Canal único de notificaciones de Actividad.
private const val CANAL_ACTIVIDAD = "ratio_actividad"

/// Archivo donde se deja el token de FCM para que Rust lo lea y lo registre.
private const val ARCHIVO_TOKEN = "push-token.txt"

/// Archivo donde se deja la ruta que debe abrir la app al tocar una notificación.
private const val ARCHIVO_RUTA = "abrir-ruta.txt"

/// Extra con la ruta que abre la notificación que pinta la propia app.
private const val EXTRA_RUTA = "ratio_route"

/// Carpeta privada compartida con Rust (`sesion::carpeta_datos`).
private const val CARPETA = "Ratio"

/// Etiqueta de los mensajes de diagnóstico.
private const val ETIQUETA = "RatioPush"

/**
 * Actividad principal: arranca la vista de Dioxus y prepara Firebase para push.
 *
 * Firebase se inicializa aquí, con los identificadores públicos del proyecto, porque el
 * plugin de Gradle `google-services` no está disponible en el proyecto que genera `dx`.
 */
class MainActivity : WryActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        inicializarFirebase(this)
        crearCanal(this)
        pedirPermisoNotificaciones()
        pedirTokenActual(this)
        guardarRuta(intent)
    }

    /**
     * La app ya estaba abierta y la persona tocó una notificación: se guarda la ruta para
     * que el lado Rust navegue a Actividad.
     */
    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        guardarRuta(intent)
    }

    /**
     * Deja la ruta pedida por la notificación. Acepta tanto el extra propio de las
     * notificaciones que pinta la app como los datos que FCM añade al abrir la app.
     */
    private fun guardarRuta(intent: Intent?) {
        if (intent == null) {
            return
        }

        val ruta = intent.getStringExtra(EXTRA_RUTA)
            ?: if (intent.getStringExtra("tipo") != null) "/actividad" else return

        // Consumir los extras para no repetir la navegación si la Activity se reutiliza
        intent.removeExtra(EXTRA_RUTA)
        intent.removeExtra("tipo")
        intent.removeExtra("receta_id")

        try {
            val carpeta = File(filesDir, CARPETA)
            carpeta.mkdirs()
            File(carpeta, ARCHIVO_RUTA).writeText(ruta)
        } catch (causa: Exception) {
            Log.w(ETIQUETA, "no se pudo guardar la ruta de la notificación", causa)
        }
    }

    /**
     * Android 13+ necesita permiso explícito para mostrar notificaciones. Si se niega, el
     * token igual se registra: los datos llegan, solo no se pinta el aviso.
     */
    private fun pedirPermisoNotificaciones() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            return
        }
        val concedido = ContextCompat.checkSelfPermission(
            this,
            Manifest.permission.POST_NOTIFICATIONS,
        ) == PackageManager.PERMISSION_GRANTED
        if (!concedido) {
            requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 1001)
        }
    }
}

/**
 * Recibe el token cuando FCM lo crea o lo rota y los mensajes mientras la app está abierta.
 */
class PushService : FirebaseMessagingService() {
    override fun onNewToken(token: String) {
        guardarToken(this, token)
    }

    override fun onMessageReceived(mensaje: RemoteMessage) {
        // En segundo plano el sistema pinta solo las notificaciones con bloque `notification`.
        // Aquí se cubre el primer plano y los mensajes que solo traen datos.
        val titulo = mensaje.notification?.title ?: "Ratio"
        val cuerpo = mensaje.notification?.body ?: "Tienes una novedad en tu actividad."
        mostrarNotificacion(this, titulo, cuerpo)
    }
}

/**
 * Inicializa Firebase con los valores públicos de `google-services.json`.
 *
 * Si el proyecto cambia, actualiza estas cuatro constantes junto con ese archivo:
 * `mobilesdk_app_id`, `api_key`, `project_id` y `project_number`.
 */
private fun inicializarFirebase(contexto: Context) {
    try {
        if (FirebaseApp.getApps(contexto).isNotEmpty()) {
            return
        }
        val opciones = FirebaseOptions.Builder()
            .setApplicationId("1:950023410890:android:29f84b2fdc81d5345b9033")
            .setApiKey("AIzaSyAs4R4HPpF0Kr03yORo2HwoTEcC89sgqBA")
            .setProjectId("ratio-820b6")
            .setGcmSenderId("950023410890")
            .build()
        FirebaseApp.initializeApp(contexto, opciones)
    } catch (causa: Exception) {
        Log.w(ETIQUETA, "no se pudo inicializar Firebase", causa)
    }
}

/**
 * Crea el canal de notificaciones en Android 8+ (sin él, el sistema descarta los avisos).
 */
private fun crearCanal(contexto: Context) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
        return
    }
    val canal = NotificationChannel(
        CANAL_ACTIVIDAD,
        "Actividad",
        NotificationManager.IMPORTANCE_DEFAULT,
    ).apply {
        description = "Respuestas, valoraciones, versiones y moderación"
    }
    contexto.getSystemService(NotificationManager::class.java)?.createNotificationChannel(canal)
}

/**
 * Pide a FCM el token actual. Si aún no está listo, `onNewToken` lo entregará después.
 */
private fun pedirTokenActual(contexto: Context) {
    try {
        FirebaseMessaging.getInstance().token.addOnCompleteListener { tarea ->
            if (tarea.isSuccessful) {
                guardarToken(contexto, tarea.result)
            } else {
                Log.w(ETIQUETA, "FCM no entregó el token", tarea.exception)
            }
        }
    } catch (causa: Exception) {
        Log.w(ETIQUETA, "FCM no está disponible", causa)
    }
}

/**
 * Deja el token en la carpeta privada de la app. Rust lo lee y lo registra en el servidor.
 *
 * Es un archivo y no una llamada JNI a propósito: el servicio de FCM puede correr con la
 * app cerrada, cuando el runtime de Rust no está disponible.
 */
private fun guardarToken(contexto: Context, token: String) {
    try {
        val carpeta = File(contexto.filesDir, CARPETA)
        carpeta.mkdirs()
        File(carpeta, ARCHIVO_TOKEN).writeText(token)
    } catch (causa: Exception) {
        Log.w(ETIQUETA, "no se pudo guardar el token push", causa)
    }
}

/**
 * Pinta la notificación que llega con la app abierta.
 */
private fun mostrarNotificacion(contexto: Context, titulo: String, cuerpo: String) {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
        ContextCompat.checkSelfPermission(
            contexto,
            Manifest.permission.POST_NOTIFICATIONS,
        ) != PackageManager.PERMISSION_GRANTED
    ) {
        return
    }

    val abrir = Intent(contexto, MainActivity::class.java).apply {
        flags = Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP
        putExtra(EXTRA_RUTA, "/actividad")
    }
    val pendiente = PendingIntent.getActivity(
        contexto,
        0,
        abrir,
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
    )

    val aviso = NotificationCompat.Builder(contexto, CANAL_ACTIVIDAD)
        .setSmallIcon(contexto.applicationInfo.icon)
        .setContentTitle(titulo)
        .setContentText(cuerpo)
        .setContentIntent(pendiente)
        .setAutoCancel(true)
        .setPriority(NotificationCompat.PRIORITY_DEFAULT)
        .build()

    try {
        NotificationManagerCompat.from(contexto)
            .notify(System.currentTimeMillis().toInt(), aviso)
    } catch (causa: SecurityException) {
        Log.w(ETIQUETA, "sin permiso para mostrar la notificación", causa)
    }
}
