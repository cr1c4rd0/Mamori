# Matriz Inmutable de Acuerdos y Decisiones Clave

**Proyecto:** Mamori  
**Organización:** Mamori (Startup - Proyecto Personal)  

---

## 1. Decisiones en Firme (`D-XX` — 🟢 Confirmadas)

| ID | Fecha | Stakeholder / Fuente | Descripción del Acuerdo / Decisión | Impacto en SRS / Arquitectura |
| :---: | :---: | :--- | :--- | :--- |
| **D-01** | 2026-11-22 | Gobernanza / Kickoff | Inicialización formal del proyecto **Mamori**. | Establece la estructura base de ingeniería y gobernanza. |
| **D-02** | 2026-11-22 | Fundador (Usuario) | **Cambio de Paradigma: Proyecto Personal / Startup Propia**. No es un proyecto de consultoría para "servi". Se construirá tanto la **documentación de ingeniería** (SRS, C4, SDD) como el **software funcional** (código, BD, infra). | Reemplazo del concepto de "contrato comercial" por Product Charter y creación de carpetas de código fuente. |
| **D-03** | 2026-11-22 | Fundador (Usuario) | **Dominio de Negocio:** Sistema integral de seguridad y control de acceso para **Unidades Residenciales** (conjuntos cerrados / condominios). Módulos clave: Gestión de residentes, control de acceso de personas (visitantes/contratistas), control vehicular, y gestión y notificación de paquetería/encomiendas. | Define el núcleo funcional del SRS (ISO 29148) y las entidades del modelo de datos. |
| **D-04** | 2026-11-22 | Fundador (Usuario) | **Arquitectura de Clientes (Terminales de Portería):** Aplicación de escritorio **instalable nativa para Windows**, desarrollada en **Rust** (Tauri v2 para motor nativo + UI optimizada para guardias con atajos de teclado). | Requiere arquitectura cliente de escritorio con acceso a hardware local (lectores, impresoras, scanners). |
| **D-05** | 2026-11-22 | Fundador (Usuario) | **Arquitectura de Repositorio: Monorepo con Cargo Workspace (Rust)**. No se separará en múltiples repositorios. Todo el ecosistema (cliente desktop, servidor central y crate compartido) convive en el mismo repo para compartir DTOs y garantizar cero desincronización de contratos. | Adopción de `Cargo.toml` raíz con `crates/shared`, `crates/server` y `crates/client-desktop`. |
| **D-06** | 2026-11-22 | Fundador (Usuario) | **Stack del Servidor Central:** **Rust con Axum + Tokio + SQLx + PostgreSQL**. Se confirma la unificación de stack con el cliente para máxima eficiencia, concurrencia y modelos compartidos. | Define la arquitectura C4 (C2: Contenedores y C3: Componentes). |

---

## 2. Decisiones Abiertas (`DA-XX` — 🔴 Pendientes por Definir)

| ID | Fecha Apertura | Dominio / Tema | Pregunta o Definición Pendiente | Responsable | Estado |
| :---: | :---: | :--- | :--- | :---: | :---: |
| **DA-01** | 2026-11-22 | Resiliencia de Red | ¿Las terminales de portería deben soportar **operación offline (Local-First)** con SQLite local si se cae el internet, sincronizando en diferido con el servidor central? | Fundador / Requerimientos | 🔴 ABIERTA (Propuesta: Sí, obligatorio) |
| **DA-02** | 2026-11-22 | Framework UI Cliente | Para la interfaz webview de Tauri v2: ¿Preferir **React + Tailwind** o **Svelte + Tailwind** para la UI de los vigilantes? | Fundador / UI | 🔴 ABIERTA |
| **DA-03** | 2026-11-22 | Notificación a Residentes | ¿Cuál será el canal principal de notificación para el MVP cuando llegue visita o paquete? (WhatsApp API, Notificación Push a app móvil, Telegram o SMS). | Fundador / Requerimientos | 🔴 ABIERTA |
| **DA-04** | 2026-11-22 | Integración de Hardware | ¿Qué dispositivos físicos de portería se integrarán en el MVP? (Lector de código de barras/cédulas 2D, impresora térmica de tiquetes, cámara web para foto del visitante, apertura de talanquera por relé). | Fundador / Hardware | 🔴 ABIERTA |
