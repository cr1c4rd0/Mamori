---
name: ingesta_y_minutas
description: >-
  Protocolo obligatorio para organizar archivos sueltos en la raíz, convertir documentos
  de contexto/DOCUMENTOS y notas de contexto/notas a Markdown estructurado, generar
  minutas formales y actualizar la memoria viva del proyecto.
---

# Skill 1: Ingesta de Contexto, Generación de Minutas y Memoria Activa

## Cuándo Activar
- Cada vez que el usuario deposite archivos en `contexto/DOCUMENTOS/`, `contexto/notas/`, `contexto/videos/` o deje archivos sueltos (`.mp4`, `.docx`, `.pdf`, `.xlsx`, `.md` de Meet) en la raíz del proyecto.

## Procedimiento Paso a Paso
1. **Saneamiento de Raíz (Auto-Organización):**
   - Si hay grabaciones `.mp4` en la raíz ➔ mover a `contexto/videos/`.
   - Si hay notas o transcripciones de llamadas en la raíz ➔ mover a `contexto/notas/`.
   - Si hay documentos del cliente (`.docx`, `.pdf`, `.xlsx`, `.pptx`) en la raíz ➔ mover a `contexto/DOCUMENTOS/`.
2. **Extracción y Conversión Íntegra:**
   - Si es el contrato, propuesta comercial u Orden de Trabajo ➔ transcribir a `proyecto/docs/00_contrato_y_alcance/` y poblar `proyecto/docs/00_contrato_y_alcance/contrato.md` con la lista numerada de Entregables (`ENT-01..ENT-N`).
   - Si es una nota/transcripción de reunión ➔ generar minuta formal en `proyecto/docs/01_descubrimiento/minutas/Minuta_YYYY-MM-DD_[Tema].md` incluyendo: Asistentes, Objetivo, Hallazgos AS-IS/TO-BE, Acuerdos Cerrados (`D-XX`) y Puntos Abiertos (`DA-XX`).
   - Si es un flujograma o documento funcional ➔ convertir a Markdown con diagramas Mermaid en `proyecto/docs/01_descubrimiento/flujos_as_is_to_be/`.
3. **Actualización Autónoma de Memoria (`/memoria`):**
   - Registrar decisiones cerradas (`D-XX`) y abiertas (`DA-XX`) en `memoria/acuerdos_y_decisiones_clave.md`.
   - Actualizar el semáforo (`🟢` / `🟡` / `🔴`) en `memoria/matriz_certeza_real_vs_propuesta.md`.
   - Añadir nuevos términos del cliente en `memoria/glosario_terminos_negocio.md`.
   - Registrar el aprendizaje en `memoria/bitacora_aprendizajes.md` y actualizar `memoria/estado_flujo_agentes.md`.