---
name: exportacion_entregables
description: >-
  Compilación y exportación de entregables formales para el cliente en
  proyecto/entregables_cliente/: documentos Word (.docx) del SRS con branding corporativo
  y diagramas de alta resolución, y libros Excel (.xlsx) de checklists de sesiones.
---

# Skill 5: Exportación de Entregables Ejecutivos para el Cliente

## Ubicación Exclusiva de Salida
Todo archivo ofimático generado para entregar al cliente debe guardarse en `proyecto/entregables_cliente/`:
- **`word_srs/`**: Documentos `.docx` formales del SRS (`Especificacion_Requerimientos_SRS_[Cliente]_v[X].docx`).
- **`excel_checklists/`**: Libros `.xlsx` de control de sesiones, avance de requerimientos y matriz de insumos pendientes.
- **`presentaciones_y_diagramas/`**: Archivos `.pptx`, `.pdf` o `.vsdx`.

## Reglas de Calidad de Exportación
1. **Branding Oficial:** Utilizar los logos alojados en `proyecto/assets/branding/` (Equipo de Ingeniería Mamori y logo oficial del proyecto/cliente).
2. **Diagramas Renderizados:** Renderizar los bloques Mermaid a imágenes PNG/SVG de alta resolución en `proyecto/assets/img/` e incrustarlos en el `.docx` (nunca dejar código Mermaid crudo en un Word para el cliente).
3. **Higiene Visual:** Verificar tablas con encabezados estilizados, semáforo visual (`🟢`, `🟡`, `🔴`), cero rutas locales `file:///` y cero referencias bibliográficas académicas.