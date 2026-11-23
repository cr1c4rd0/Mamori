# Control de Estado y Máquina de Estados de Desarrollo

**Proyecto:** Mamori (守り)  
**Organización:** Mamori (Startup - Proyecto Personal)  
**Última Actualización:** 2026-11-22  

---

## 1. Directorio de Fases del Flujo de Producto

| Fase | Enfoque | Artefactos Principales | Estado |
| :---: | :--- | :--- | :---: |
| **Fase 0** | **Visión y Descubrimiento** | `docs/00_vision_y_roadmap/product_charter.md`, flujos AS-IS / TO-BE de portería. | 🟢 **COMPLETADA** |
| **Fase 1** | **Estructura y Workspace Base** | Monorepo Cargo Workspace, `crates/shared`, `server`, `client-desktop`, `infra/`. | 🟢 **COMPLETADA** |
| **Fase 2** | **Especificación SRS (ISO 29148)** | Redacción de requerimientos funcionales y no funcionales para Residentes, Accesos, Vehículos y Paquetería. | 🟡 **EN PROGRESO** |
| **Fase 3** | **Especificaciones Técnicas (SDD)** | Contratos OpenAPI, esquemas SQL en `specs/` y escenarios BDD Gherkin. | 🔴 PENDIENTE |
| **Fase 4** | **Implementación Backend & Shared** | Endpoints de Axum, migraciones SQLx y modelos en `crates/shared`. | 🔴 PENDIENTE |
| **Fase 5** | **Implementación Terminal Windows** | Interfaz de portería en Tauri v2 con atajos de teclado y resiliencia offline. | 🔴 PENDIENTE |

---

## 2. Registro Histórico de Transiciones

| Iteración | Fecha | Fase Origen | Fase Destino | Descripción del Hito |
| :---: | :---: | :---: | :---: | :--- |
| **#00** | 2026-11-22 | - | Fase 0 | Inicialización con arnés base. |
| **#01** | 2026-11-22 | Fase 0 | Fase 1 | Pivote estratégico a Startup/Proyecto Personal: Adopción de Monorepo Cargo Workspace (Rust), estructuración de `crates/`, `docs/`, `specs/`, `infra/` y Product Charter de Mamori. |
