# Directrices Operativas del Asistente (GEMINI.md)

**Proyecto:** Mamori (守り)  
**Organización:** Mamori (Startup - Proyecto Personal)  
**Dominio:** Seguridad y Control de Acceso para Unidades Residenciales  
**Stack Principal:** Monorepo Cargo Workspace (Rust) | Axum (Server) | Tauri v2 (Client Windows) | PostgreSQL | ISO 29148  

---

## 1. Arquitectura del Repositorio
- **`contexto/`:** Notas crudas, observaciones de campo en porterías, benchmarking.
- **`memoria/`:** Persistencia viva: decisiones (`D-XX`), decisiones abiertas (`DA-XX`), matriz de certeza, bitácora y glosario.
- **`docs/`:** Documentación viva: Visión (`00_vision`), Descubrimiento (`01`), SRS ISO 29148 (`02`), Arquitectura C4 (`03`), QA (`04`).
- **`specs/`:** Especificaciones ejecutables SDD: Contratos API, Esquemas BD, Reglas de negocio y BDD Gherkin.
- **`crates/`:** Código fuente Rust: `shared` (modelos comunes), `server` (backend central Axum), `client-desktop` (terminal Windows).
- **`infra/`:** Docker Compose para desarrollo local (Postgres) y scripts de despliegue.

---

## 2. Reglas Clave de Ejecución
1. **Cero Suposiciones:** Si falta un dato o regla operativa, marcarlo con `🔴 **[PENDIENTE POR DEFINIR]**` y preguntar al fundador.
2. **Semáforo de Certeza Técnica:** Mantener sincronizada `memoria/matriz_certeza_real_vs_propuesta.md`.
3. **Spec-to-Code:** La especificación en `specs/` guía directamente el código en `crates/`.
