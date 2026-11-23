---
name: elicitacion_y_srs_iso29148
description: >-
  Metodología para el levantamiento de requerimientos y redacción del SRS bajo el
  estándar ISO/IEC/IEEE 29148, modelado en 4 perspectivas, arquitectura C4, sintaxis
  EARS/Gherkin y semáforo de certeza técnica con control visual en rojo.
---

# Skill 2: Levantamiento de Requerimientos y Redacción de SRS (ISO/IEC/IEEE 29148)

## Ubicación Canónica Única
Todo el trabajo de especificación de requerimientos vive en `proyecto/docs/02_requerimientos_srs/`:
- Versiones iterativas: `srs_v1.md`, `srs_v2.md`, etc.
- Versión certificada por QA: `srs_aprobado.md`.
- Casos de uso detallados: `casos_de_uso/`.

## Reglas de Redacción del SRS
1. **Modelado en 4 Perspectivas:**
   - **Contexto:** Frontera del sistema, actores y sistemas externos con los que interactúa.
   - **Interacción:** Casos de uso, diagramas de secuencia Mermaid y contratos de interfaz.
   - **Estructura:** Modelo de dominio, entidades (`classDiagram` / `erDiagram`) y Arquitectura C4 (C1 Contexto, C2 Contenedores, C3 Componentes).
   - **Comportamiento:** Máquinas de estados (`stateDiagram-v2`), reglas de negocio y manejo de excepciones.
2. **Estructura Atómica de cada Requerimiento Funcional (`RF-XXX`):**
   - **ID y Nombre**, **Estado de Certeza** (`🟢` / `🟡` / `🔴`), **Entregable Contractual Asociado** (`ENT-XX`), **Prioridad**, **Descripción en sintaxis EARS**, **Entradas / Procesamiento / Salidas**, **Restricciones Negativas (`RN-XX`)** y **Criterio de Aceptación Gherkin**.
3. **Control Visual en Rojo y Cero Citas Académicas:**
   - Todo elemento abierto o sin confirmar debe llevar `🔴 **[PENDIENTE POR DEFINIR]**` y estilo rojo en Mermaid (`fill:#ffebee,stroke:#d32f2f,stroke-width:2.5px,stroke-dasharray: 4 4`).
   - **Prohibido citar libros o autores académicos**; referenciar únicamente ISO/IEC/IEEE 29148, Modelo C4, OpenAPI 3.0 y JSON Schema 2020-12.