# Contrato — Protocolo de Sincronización Terminal ⇄ Servidor

**Decisiones:** D-22 a D-25 · **Fuente de verdad del código:** `crates/shared/src/sync.rs`  
**Estado:** 🟢 Implementado por el servidor real (`crates/server`, D-29) y por el servidor simulado de pruebas. Sin autenticación en el POC: 🔴 **[PENDIENTE POR DEFINIR]** (DOM-07d).

---

## 1. Operaciones

| Operación | Endpoint | Request | Response |
| :--- | :--- | :--- | :--- |
| **Push** | `POST /api/v1/sync/push` | `PushRequest` | `PushResponse` |
| **Pull** | `POST /api/v1/sync/pull` | `PullRequest` | `PullResponse` |

## 2. DTOs

```jsonc
// ChangeRecord: un cambio de un registro completo
{
  "change_id": "0b8c…",            // UUID v4, único por cambio (idempotencia RN-SYNC-05)
  "origin_node": "5f1e…",          // node_id de la terminal que lo hizo
  "collection": "visits",          // units | parking_spots | residents | visits | packages
  "record_id": "v1",
  "op": "upsert",                  // upsert | delete
  "payload": { /* registro completo; null si op = delete */ },
  "created_at": "2026-11-24T15:02:11Z"
}

// PushRequest / PushResponse
{ "node_id": "5f1e…", "changes": [ /* ChangeRecord */ ] }
{ "accepted": [ "0b8c…" ], "server_seq": 42 }

// PullRequest / PullResponse
{ "node_id": "5f1e…", "since_seq": 37 }
{ "changes": [ { "seq": 38, "change": { /* ChangeRecord */ } } ], "server_seq": 42 }
```

## 3. Reglas del servidor

- Asigna `seq` creciente a cada cambio nuevo, en orden de llegada.
- Si el `change_id` ya existe, no lo duplica y lo devuelve en `accepted` (RN-SYNC-05).
- En Pull devuelve solo cambios con `seq > since_seq` y `origin_node ≠ node_id` (RN-SYNC-06).
- Respuestas HTTP: `200` con el DTO; `400` si la petición es inválida (p. ej. `origin_node` ≠ `node_id`); `500` ante fallo de base de datos. La terminal trata cualquier fallo de red o respuesta ≠ 200 como sincronización fallida y no altera su cola.
