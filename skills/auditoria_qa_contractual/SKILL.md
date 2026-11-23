---
name: auditoria_qa_contractual
description: >-
  Checklist hermético de 7 filtros para auditar cualquier versión del SRS contra
  el contrato del proyecto y la matriz de certeza antes de aprobar el paso a SDD
  o exportación al cliente.
---

# Skill 3: Auditoría de Calidad y Cumplimiento Contractual del SRS (7 Filtros de Rechazo)

## Propósito
Auditar implacablemente `proyecto/docs/02_requerimientos_srs/srs_v[X].md` contra `proyecto/docs/00_contrato_y_alcance/contrato.md` y `memoria/matriz_certeza_real_vs_propuesta.md`, emitiendo el dictamen en `memoria/reportes_qa/reporte_qa_srs.md`.

## Los 7 Filtros Obligatorios (Causal de `ESTADO: RECHAZADO` si falla alguno)
1. **Trazabilidad Contractual 1 a 1:** Todos los entregables (`ENT-01..ENT-N`) de `contrato.md` deben estar cubiertos sin entregables huérfanos.
2. **Control Estricto de Alcance (Cero Scope Creep):** No incluir módulos o integraciones fuera de la fase contratada ni violar decisiones en firme (`D-XX`).
3. **Pureza Contextual y Cero Alucinaciones:** Ninguna decisión abierta (`DA-XX`) puede presentarse como cerrada sin respaldo en minutas. Cero rutas locales (`file:///...`) o nombres de otros clientes.
4. **Calidad ISO/IEC/IEEE 29148 y 4 Perspectivas:** Requerimientos atómicos, verificables, con restricciones negativas (`RN-XX`), diagramas Mermaid con sintaxis segura (`["..."]`) y modelos C4 completos.
5. **Cero Referencias a Libros o Autores Académicos:** Cualquier mención a autores o libros de texto (ej. Sommerville) en lugar de estándares industriales causa rechazo inmediato.
6. **Control Visual en ROJO:** Todo ítem abierto o pendiente debe tener `🔴 **[PENDIENTE POR DEFINIR]**` en texto/tablas y estilo rojo punteado en diagramas Mermaid.
7. **Taxonomía Real vs. Propuesta vs. Pendiente:** Cada sección debe declarar su etiqueta (`🟢` / `🟡` / `🔴`) y especificar qué insumo del cliente se requiere para que cada `🟡 Propuesta` pase a `🟢 Real`.