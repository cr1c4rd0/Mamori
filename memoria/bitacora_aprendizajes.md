# Bitácora Continua de Aprendizajes del Proyecto

**Proyecto:** Mamori  
**Organización:** Mamori (Startup - Proyecto Personal)  

---

### [2026-11-22] Aprendizaje 01 — Inicialización bajo Arnés v2.0 (4 Pilares Puros)
- **Contexto:** Creación del repositorio con separación estricta entre entrada (`contexto/`), memoria (`memoria/`), recetas nativas (`skills/` y `.agents/skills/`) e ingeniería/salida (`proyecto/`).

---

### [2026-11-22] Aprendizaje 02 — Definición del Núcleo Mamori: Seguridad Residencial & Clientes Desktop en Rust
- **Contexto:** El usuario confirmó que Mamori es su startup/proyecto personal (no consultoría externa para terceros). El alcance abarca documentación de ingeniería + desarrollo completo de software.

---

### [2026-11-22] Aprendizaje 03 — Aprobación de Monorepo Cargo Workspace y Stack Axum
- **Contexto:** Se consultó la conveniencia de separar en dos repositorios (cliente vs servidor) frente a un Monorepo.
- **Decisión Aprobada (D-05, D-06):** Mantener un **Monorepo con Cargo Workspace** unificado en Rust (`crates/shared`, `crates/server`, `crates/client-desktop`).

---

### [2026-11-22] Aprendizaje 04 — Protocolo de Traspaso SDD ➔ Lead Rust Engineer
- **Regla Establecida (D-07):** El Lead Rust Engineer opera estrictamente sobre los artefactos técnicos liberados y aprobados por el *Especificador Técnico (SDD)* en `specs/`.

---

### [2026-11-22] Aprendizaje 05 — Adopción de SLINT como Motor de GUI Nativo de Windows (D-10)
- **Contexto:** El fundador ratificó el requerimiento innegociable de que la terminal de portería sea una **aplicación nativa de Windows genuina**, sin tecnologías web ni navegadores embebidos (cero HTML/CSS/JavaScript/WebView2).
- **Decisión (D-10):** Se adopta **Slint (slint-rs)**, el framework nativo creado por los ingenieros de Qt para Rust:
  1. Compilación directa a binario máquina `.exe`.
  2. Renderizado acelerado por GPU (DirectX / software).
  3. Consumo mínimo de recursos (<12 MB RAM vs 150 MB de alternativas web).
  4. Latencia cero en captura de atajos de teclado (`F1`..`F5`) y hardware local (lectores de cédulas e impresoras térmicas).

---

### [2026-11-22] Aprendizaje 06 — Patrones Reactivos en Slint: Side Drawer y Option Lists Dinámicos (D-13, D-14)
- **Contexto:** En el registro de unidades, se requería una experiencia ágil para vigilantes: abrir un panel lateral deslizante (Side Drawer) que no tape por completo el contexto, capturar atajo `ESC` global para descartar, y permitir asociar múltiples parqueaderos privados mediante un flujo dinámico donde seleccionar una opción despliega automáticamente otro `ComboBox` para agregar adicionales, permitiendo editar o eliminar cualquier ítem previo.
- **Implementación Técnica:**
  - El drawer se implementa como un `Rectangle` flotante anclado a la derecha con un backdrop oscuro transparente que intercepta clics fuera del área.
  - La captura de `Key.Escape` en `FocusScope` permite un control de teclado natural.
  - El listado dinámico de parqueaderos se conecta mediante un `VecModel<SharedString>` en Rust consumido por un `for spot-val[idx] in root.selected-parking-list` en Slint, combinado con callbacks bidireccionales (`on_add_parking_selection`, `on_change_parking_selection`, `on_remove_parking_selection`).

---

### [2026-11-24] Aprendizaje 07 — Ventana maximizada al iniciar en Slint (backend winit, Windows)
- **Síntoma:** La terminal abría en 1200×780 (`preferred-width/height`) aunque se llamaba a `ui.window().set_maximized(true)`. `is_maximized()` devolvía `true`, pero el tamaño real no cambiaba.
- **Causa:** Slint 1.19 guarda la bandera de maximizado; si la llamada ocurre antes de que exista la ventana nativa, Windows nunca la recibe y las llamadas posteriores se descartan porque Slint "cree" que ya está maximizada.
- **Solución:** `ui.show()` y luego, con el bucle de eventos en marcha (`slint::Timer::single_shot`), alternar `set_maximized(false)` → `set_maximized(true)`. Verificado: 1920×1009 en monitor 1920×1080 (área útil sin barra de tareas). No es pantalla completa: conserva barra de título y barra de tareas.

---

### [2026-11-24] Aprendizaje 08 — Local-First con archivos JSON y servidor simulado distribuido (D-22 a D-26)
- **Diseño:** Cada acción del vigilante guarda el registro completo en `estado_local.json` y deja el cambio en `cola_envio.json` (Outbox). En las ventanas 03/10/15/22 h se hace Push de la cola y luego Pull de los cambios de otros nodos. El servidor simulado es un log append-only (`log_cambios.jsonl`) compartido por varias terminales con bloqueo de archivo del SO (`File::lock`, Rust ≥ 1.89).
- **Trampa evitada (orden de confirmación):** si la cola se vacía al confirmar el Push *antes* de aplicar el Pull, un cambio remoto del mismo registro ya no choca con nada y pisa la versión local, violando "gana la terminal" (D-24). Solución: confirmar (retirar de la cola) **después** de aplicar el Pull; la idempotencia por `change_id` del servidor hace seguro reenviar si algo falla a mitad. Cubierto por el test `dos_terminales_se_sincronizan_y_gana_la_terminal`.
- **Slint:** para refrescar listas desde un hilo de fondo se usa `slint::invoke_from_event_loop` + `ModelRc::as_any().downcast_ref::<VecModel<T>>()`, así las callbacks conservan su referencia al mismo modelo. `rotation-angle` está obsoleto: usar `transform-rotation` (+ `animation-tick()` para el giro del icono).
- **Prueba manual:** `MAMORI_SYNC_INTERVAL_SECS=10` acelera las ventanas; crear el archivo `SIN_CONEXION` en la carpeta del servidor simula caída de internet; dos instancias con distinto `MAMORI_DATA_DIR` simulan dos porterías.

---

### [2026-11-24] Aprendizaje 09 — Servidor real (Axum + PostgreSQL 17), sync manual F9 y foco de teclado en Slint (D-27 a D-29)
- **Entorno local:** PostgreSQL 17 instalado con `winget` como servicio `postgresql-x64-17`. Rol y base `mamori`; credenciales en `.env` (ignorado por git). El servidor aplica migraciones (`sqlx::migrate!`) y carga los datos iniciales si la base está vacía.
- **Orden de `seq` vs orden de commit:** con varios Push concurrentes, `bigserial` puede asignar `seq=10` a una transacción que confirma después de `seq=11`; un Pull intermedio avanzaría el cursor a 11 y perdería el 10 para siempre. Se evita serializando los Push con `pg_advisory_xact_lock` y devolviendo como cursor el último `seq` leído (no `MAX(seq)`).
- **Materialización tolerante:** cada cambio se aplica a las tablas relacionales dentro de un SAVEPOINT; si falla (p. ej. llave foránea), queda en `change_log.apply_error` y el Push no se rechaza, para que la terminal no quede reintentando para siempre.
- **Bug previo de foco (Slint):** tras el login el `LineEdit` de contraseña (con el foco) se destruye y ningún atajo `F1..F9` respondía hasta hacer clic. Solución: `changed is-authenticated => { key-handler.focus(); }`. Verificado con capturas: F4 cambia de módulo y F9 sincroniza justo después del login.
- **UX:** contra un servidor local la sync dura milisegundos; se mantiene "SINCRONIZANDO…" visible al menos 1 s para que el vigilante perciba la acción.
- **Prueba de integración:** `cargo test -p mamori-client -- --ignored` ejecuta dos terminales contra el servidor real y borra su registro de prueba al final (cambio `delete` por el protocolo).

---

### [2026-11-24] Aprendizaje 10 — CRUD de Unidades & Predios (D-30 a D-33)
- **Diseño:** las reglas viven en `crates/client-desktop/src/units_crud.rs` como funciones puras (`estado → Plan | CrudError`), probadas sin UI. `main.rs` solo lee el formulario, llama a la regla y guarda el `Plan` (torres → inmuebles → parqueaderos → residentes).
- **Eliminar = inactivar** (`is_active`), con `#[serde(default = "default_true")]` para que los archivos y cambios anteriores sigan siendo válidos. Las torres antiguas se completan al abrir con ids fijos (`t1..t4`), iguales en terminales y servidor (`ensure_seed_towers`).
- **Trampa evitada (orden de la cola):** compactar la cola con "quitar y agregar al final" reordenaba los cambios; un inmueble editado antes de sincronizar quedaba *después* de sus parqueaderos y el servidor rechazaba la llave foránea. Ahora la versión nueva reemplaza a la anterior en su misma posición (RN-SYNC-03).
- **Foco de teclado:** al cerrar cualquier panel lateral el foco vuelve al manejador global (`changed show-* => key-handler.focus()`); antes F1..F9 dejaban de responder tras cerrar un panel.
- **Pruebas automáticas de UI en Windows:** `mouse_event` inmediato tras `SetCursorPos` a veces no registra el clic en Slint (no hay hover previo); esperar ~150 ms entre mover y pulsar.
