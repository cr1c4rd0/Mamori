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
| **DOM-03: Cliente Portería** | GUI Windows 100% Nativa en Rust | 🟢 `[CONFIRMADO / REAL]` | **SLINT** (cero tecnologías web, compilación directa a `.exe`, renderizado GPU/DirectX). | Confirmado por fundador (D-04, D-10). |
| **DOM-04: Servidor Central** | Backend en Rust con Axum + PostgreSQL | 🟢 `[CONFIRMADO / REAL]` | Backend asíncrono con Tokio, WebSockets y SQLx unificado con el cliente. | Confirmado por fundador (D-05, D-06). |
| **DOM-05: Arquitectura Repo** | Monorepo con Cargo Workspace | 🟢 `[CONFIRMADO / REAL]` | Un solo repositorio con `crates/shared`, `crates/server`, `crates/client-desktop`. | Confirmado por fundador (D-05). |
| **DOM-06: Estructura Predial** | Configuración de Torres e Inmuebles | 🟢 `[CONFIRMADO / REAL]` | `FLP-01`: Torres, Inmuebles, Citófono/Celular/Fijo y Parqueaderos privados. | Confirmado por fundador (D-09). |
| **DOM-07: Resiliencia de Red** | Operación offline (Local-First) | 🟢 `[CONFIRMADO / REAL]` | Obligatoria (D-22). POC: archivos JSON locales + cola de envío (Outbox). Sync a las 03/10/15/22 h (D-23). Gana la terminal (D-24). | Confirmado por fundador (D-22 a D-26). |
| **DOM-07a: Almacenamiento POC** | Archivos JSON vs SQLite | 🟡 `[PROPUESTA DE DISEÑO]` | POC en JSON con escritura atómica. Migración a SQLite cuando el volumen lo exija; solo cambia `store.rs`. | Decidir motor definitivo para el MVP. |
| **DOM-07b: Servidor de Sync** | Axum + PostgreSQL local | 🟢 `[CONFIRMADO / REAL]` | Endpoints push/pull reales sobre PostgreSQL 17 (D-27, D-29). El servidor simulado se conserva para pruebas. | Confirmado por fundador. |
| **DOM-07c: Política de reintentos** | Reintento / sync al encender / contador | 🔴 `[PENDIENTE POR DEFINIR]` | POC: sin reintentos, sin sync al encender, sin contador. Sync manual ya definida (D-28). | DA-04, DA-05, DA-07. |
| **DOM-07d: Autenticación de terminales** | Quién puede llamar a `/api/v1/sync/*` | 🔴 `[PENDIENTE POR DEFINIR]` | POC local sin autenticación ni TLS (solo escucha en `127.0.0.1`). | Definir token por terminal / mTLS antes de exponer el servidor en red. |
| **DOM-08: Notificaciones** | Avisos de visitas y encomiendas | 🔴 `[PENDIENTE POR DEFINIR]` | Canales por evaluar (WhatsApp Cloud API, Push notification). | Definir canal prioritario para el MVP (DA-02). |
