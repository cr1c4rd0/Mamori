# Bitácora Continua de Aprendizajes del Proyecto

**Proyecto:** Mamori  
**Organización:** Mamori (Startup - Proyecto Personal)  

---

### [2026-11-22] Aprendizaje 01 — Inicialización bajo Arnés v2.0 (4 Pilares Puros)
- **Contexto:** Creación del repositorio con separación estricta entre entrada (`contexto/`), memoria (`memoria/`), recetas nativas (`skills/` y `.agents/skills/`) e ingeniería/salida (`proyecto/`).
- **Reglas Clave Activadas:**
  1. Semáforo de certeza técnica (`🟢 Real` / `🟡 Propuesta` / `🔴 Pendiente en ROJO`).
  2. Prohibición absoluta de citas a libros o autores académicos en entregables.

---

### [2026-11-22] Aprendizaje 02 — Definición del Núcleo Mamori: Seguridad Residencial & Clientes Desktop en Rust
- **Contexto:** El usuario confirmó que Mamori es su startup/proyecto personal (no consultoría externa para terceros). El alcance abarca documentación de ingeniería + desarrollo completo de software.
- **Dominio:** Plataforma de seguridad para conjuntos/unidades residenciales (residentes, visitantes, vehículos, paquetería).
- **Decisión Arquitectónica Clave:** Terminales de portería instalables para **Windows** desarrolladas en **Rust**.

---

### [2026-11-22] Aprendizaje 03 — Aprobación de Monorepo Cargo Workspace y Stack Axum
- **Contexto:** Se consultó la conveniencia de separar en dos repositorios (cliente vs servidor) frente a un Monorepo.
- **Decisión Aprobada (D-05, D-06):** Mantener un **Monorepo con Cargo Workspace** unificado en Rust:
  1. `crates/shared`: DTOs, modelos Serde, validaciones y enums compartidos entre cliente y servidor con cero desincronización.
  2. `crates/server`: Backend central con Rust (Axum + Tokio + SQLx + PostgreSQL).
  3. `crates/client-desktop`: Terminal de portería Windows (Tauri v2 + Rust).
- **Beneficio Principal:** Cero duplicidad de código, compilación atómica conjunta con `cargo check --workspace` y máxima agilidad para desarrollo guiado por especificaciones (SDD).
