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
- **Decisión Aprobada (D-05, D-06):** Mantener un **Monorepo con Cargo Workspace** unificado en Rust (`crates/shared`, `crates/server`, `crates/client-desktop`).

---

### [2026-11-22] Aprendizaje 04 — Protocolo de Traspaso SDD ➔ Lead Rust Engineer
- **Contexto:** Definición del alcance del rol 5 (*Lead Rust Engineer*).
- **Regla Establecida (D-07):** El Lead Rust Engineer es un experto en arquitectura y desarrollo de software de producción. Opera estrictamente sobre los artefactos técnicos liberados y aprobados por el *Especificador Técnico (SDD)* en `specs/` (OpenAPI, esquemas BD, tablas de decisión, Gherkin). Esto garantiza que no haya código improvisado ni desalineado con los requerimientos.
