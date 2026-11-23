# Control de Estado y Máquina de Estados de Desarrollo

**Proyecto:** Mamori (守り)  
**Organización:** Mamori (Startup - Proyecto Personal)  
**Última Actualización:** 2026-11-22  

---

## 1. Directorio de Roles del Pipeline

| Rol | Agente / Función | Artefacto de Entrada | Artefacto de Salida |
| :--- | :--- | :--- | :--- |
| **1. PM / Ingesta** | Orquestador e Ingesta | Insumos crudos, entrevistas, notas en `contexto/` | Minutas, `product_charter.md` y actualización de `memoria/` |
| **2. Arquitecto SRS** | Elicitación y Requerimientos | Charter, Minutas y flujos de negocio | `docs/01_descubrimiento/flujos_de_proceso/` y `docs/02_requerimientos_srs/` (ISO 29148) |
| **3. Auditor QA** | Auditor de Consistencia | Documentos SRS (`docs/02/`) + Matriz de Certeza | `memoria/reportes_qa/` (`APROBADO` / `RECHAZADO`) |
| **4. Diseñador SDD** | Especificador Técnico | SRS Aprobado por QA | `specs/` (OpenAPI 3.0, Esquemas BD, Tablas de Verdad, Gherkin) |
| **5. Lead Rust Dev** | Ingeniero de Software Principal | **Artefactos liberados por SDD en `specs/`** | Código de producción en `crates/shared`, `crates/server`, `crates/client-desktop` y tests |

---

## 2. Estado Actual del Pipeline

- **Fase Actual:** `Fase 2: Modelado de Flujos y Requerimientos (Rol Activo: Arquitecto SRS)`
- **Bloqueo Activo:** Ninguno. Plantillas de flujos listas para recibir diagramas y reglas del fundador.
- **Próximo Hito:** Concluir el modelado de flujos de proceso para proceder a la auditoría QA y posterior especificación técnica ejecutable (SDD).
