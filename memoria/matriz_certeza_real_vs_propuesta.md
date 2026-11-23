# Matriz de Certeza Técnica: Real vs. Propuesta de Diseño vs. Pendiente

**Proyecto:** Mamori  
**Dominio:** Seguridad y Control de Acceso Residencial  
**Última Actualización:** 2026-11-22  

---

## Convención del Semáforo de Madurez
- **🟢 `[CONFIRMADO / REAL]`**: Definición acordada formalmente con el fundador de Mamori.
- **🟡 `[PROPUESTA DE DISEÑO — SUJETA A PULIMIENTO]`**: Especificación preliminar propuesta por el equipo de ingeniería como acelerador técnico.
- **🔴 `[PENDIENTE POR DEFINIR]`**: Proceso, integración o regla pendiente de decisión.

---

## Matriz de Control por Dominio

| ID Dominio / Componente | Elemento Técnico / Funcional | Estado Actual | Detalle de lo Definido / Propuesto | Insumo / Decisión Requerida para pasar a 🟢 100% Real |
| :--- | :--- | :---: | :--- | :--- |
| **DOM-01: Modelo de Proyecto** | Proyecto personal / Startup propia | 🟢 `[CONFIRMADO / REAL]` | Desarrollo simultáneo de especificación formal y código fuente funcional. | Confirmado por fundador (D-02). |
| **DOM-02: Dominio de Negocio** | Gestión de seguridad residencial | 🟢 `[CONFIRMADO / REAL]` | Ingreso/salida personas, vehículos, residentes y paquetería. | Confirmado por fundador (D-03). |
| **DOM-03: Cliente Portería** | Terminal Windows instalable en Rust | 🟢 `[CONFIRMADO / REAL]` | App nativa con Tauri v2 + Rust para guardias de portería. | Confirmado por fundador (D-04). |
| **DOM-04: Servidor Central** | Backend en Rust con Axum + PostgreSQL | 🟢 `[CONFIRMADO / REAL]` | Backend asíncrono con Tokio, WebSockets y SQLx unificado con el cliente. | Confirmado por fundador (D-05, D-06). |
| **DOM-05: Arquitectura Repo** | Monorepo con Cargo Workspace | 🟢 `[CONFIRMADO / REAL]` | Un solo repositorio con `crates/shared`, `crates/server`, `crates/client-desktop`. | Confirmado por fundador (D-05). |
| **DOM-06: Resiliencia de Red** | Operación offline (Local-First) | 🟡 `[PROPUESTA DE DISEÑO]` | SQLite local en la terminal Windows con cola de sincronización (Outbox Pattern) hacia el servidor central. | Confirmar si la operación offline es obligatoria para el MVP (DA-01). |
| **DOM-07: Hardware de Portería** | Lectores USB, impresoras de tiquetes | 🟡 `[PROPUESTA DE DISEÑO]` | Soporte para lector de código de barras USB (cédulas) e impresora térmica ESC/POS. | Confirmar hardware físico a soportar en v1.0 (DA-04). |
| **DOM-08: Notificaciones** | Avisos de visitas y encomiendas | 🔴 `[PENDIENTE POR DEFINIR]` | Canales por evaluar (WhatsApp Cloud API, Push notification). | Definir canal prioritario para el MVP (DA-03). |
