# Esquema de Almacenamiento Local — POC en Archivos JSON

**Decisión:** D-22 · **Reglas:** `specs/reglas_negocio/RN_sincronizacion_local_first.md`  
**Estado:** 🟡 Propuesta de diseño para el POC (motor definitivo pendiente: JSON vs SQLite, DOM-07a)

---

## 1. Ubicación

| Nodo | Carpeta por defecto | Variable para cambiarla |
| :--- | :--- | :--- |
| Terminal de portería | `%APPDATA%\Mamori\terminal\` | `MAMORI_DATA_DIR` |
| Servidor simulado | `%APPDATA%\Mamori\servidor_simulado\` | `MAMORI_SERVER_DIR` |

Para simular dos terminales en el mismo equipo se lanzan dos instancias con distinto `MAMORI_DATA_DIR` y el mismo `MAMORI_SERVER_DIR`.

---

## 2. Archivos de la terminal

| Archivo | Contenido | Tipo Rust |
| :--- | :--- | :--- |
| `estado_local.json` | Estado completo: `units`, `parking_spots`, `residents`, `visits`, `packages`. | `mamori_shared::sync::DataSnapshot` |
| `cola_envio.json` | Cambios locales pendientes de confirmación (Outbox). | `Vec<mamori_shared::sync::ChangeRecord>` |
| `sync_meta.json` | `node_id`, `server_seq` (cursor), `last_success_at`, `last_attempt_at`, `last_error`. | `client-desktop::store::SyncMeta` |

Si `estado_local.json` no existe, se crea a partir de `specs/mocks_dummy_data/seed_estado_local.json` (embebido en el `.exe`), sin generar cambios en la cola.

## 3. Archivos del servidor simulado

| Archivo | Contenido |
| :--- | :--- |
| `log_cambios.jsonl` | Un `SequencedChange` por línea, con `seq` creciente. Es la fuente de verdad del servidor. |
| `estado_central.json` | `DataSnapshot` materializado aplicando el log; lo que verán otros roles. |
| `.lock` | Bloqueo de archivo del sistema operativo para que varias terminales no escriban a la vez. |
| `SIN_CONEXION` | **Opcional.** Si existe, el servidor responde "no disponible" (simula caída de internet). |

---

## 4. Registros (colecciones)

| Colección | Clave | Campos |
| :--- | :--- | :--- |
| `units` | `id` | `tower`, `unit_number`, `unit_type`, `intercom?`, `mobile?`, `landline?`, `parking[]` (códigos) |
| `parking_spots` | `code` | `spot_type` (`Carro` / `Moto`), `assigned_unit_id?` (vacío = disponible) |
| `residents` | `id` | `full_name`, `document?`, `units[]` (etiquetas `Torre - Inmueble`), `role`, `mobile?`, `email?` |
| `visits` | `id` | `visitor_name`, `document`, `destination`, `access_type`, `entry_at` (UTC), `exit_at?` (UTC) |
| `packages` | `id` | `destination`, `recipient`, `carrier`, `description`, `received_at` (UTC), `delivered_at?` (UTC) |

Una visita está **activa** mientras `exit_at` sea vacío; un paquete está **pendiente** mientras `delivered_at` sea vacío (RN-SYNC-N06).

---

## 5. Escritura atómica (RN-SYNC-N04)

1. Serializar a `<archivo>.tmp`.
2. `fsync` del archivo temporal.
3. Renombrar `<archivo>.tmp` → `<archivo>` (reemplazo atómico en NTFS).

---

## 6. Servidor con el que sincroniza la terminal (D-29)

| Variable | Efecto | Valor por defecto |
| :--- | :--- | :--- |
| `MAMORI_SERVER_URL` | URL del servidor real (Axum + PostgreSQL). | `http://127.0.0.1:8080` |
| `MAMORI_SERVER=sim` | Usa el servidor simulado en archivos (`MAMORI_SERVER_DIR`) en lugar del real. | — (usa el real) |
| `MAMORI_SYNC_INTERVAL_SECS` | **Solo desarrollo.** Sincroniza cada N segundos en lugar de las ventanas fijas. | — |
