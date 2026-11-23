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
