---
name: sdd_contratos_y_gherkin
description: >-
  Derivación de especificaciones técnicas ejecutables (Spec-Driven Development) a partir
  de un SRS aprobado: contratos OpenAPI 3.0 y JSON Schema Draft 2020-12, tablas de verdad,
  pseudocódigo determinista, DDLs de base de datos, mocks y escenarios Gherkin.
---

# Skill 4: Generación de Especificaciones Ejecutables (SDD)

## Precondición Estricta
Solo se ejecuta cuando `memoria/reportes_qa/reporte_qa_srs.md` certifica `ESTADO: APROBADO` y existe `proyecto/docs/02_requerimientos_srs/srs_aprobado.md`.

## Entregables Canónicos en `proyecto/specs/`
1. **`specs_tecnicas.md`**: Documento maestro que consolida la arquitectura ejecutable y el índice de contratos.
2. **`contratos_api/`**: Esquemas formales OpenAPI 3.0.3 / JSON Schema Draft 2020-12 con tipos estrictos, validaciones regex, códigos de error HTTP y `additionalProperties: false`.
3. **`reglas_negocio/`**: Tablas de verdad deterministas (combinaciones exhaustivas de entrada ➔ salida) y pseudocódigo de validación junto con las restricciones negativas (`RN-XX`).
4. **`esquemas_bd/`**: Scripts DDL (SQL / BigQuery / NoSQL) con llaves primarias, foráneas, particionamiento e índices.
5. **`criterios_gherkin/`**: Archivos `.feature` con escenarios BDD automatizables (casos felices, casos borde y casos de rechazo/escalamiento HITL).
6. **`mocks_dummy_data/`**: Payloads JSON sintéticos realistas para pruebas desacopladas.