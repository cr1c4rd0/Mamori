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
| **D-07** | 2026-11-22 | Fundador (Usuario) | **Contrato de Traspaso SDD ➔ Lead Rust Engineer:** El rol de *Lead Rust Engineer* es un experto integral en desarrollo y arquitectura de software. Implementa todo lo que el *Especificador Técnico (SDD)* libere en `specs/`. | Principio estricto de Spec-to-Code en el monorepo. |
| **D-08** | 2026-11-22 | Fundador (Usuario) | **Actores Iniciales del Sistema:** Se confirman como usuarios primarios del MVP: (1) **El Guarda de Portería** (operador táctico en garita con terminal Windows) y (2) **El Administrador de la Unidad Residencial** (configuración de predios, censo y auditoría). | Formalizado en `docs/01_descubrimiento/actores_y_perfiles.md`. |
| **D-09** | 2026-11-22 | Fundador (Usuario) | **Reglas de Configuración de Unidades e Inmuebles (`FLP-01`):** (1) Estructura jerárquica Copropiedad ➔ Torres/Bloques ➔ Inmuebles (Apartamentos/Casas). (2) Todo inmueble debe contar con al menos un canal de contacto: Citófono/Extensión, Celular (en caso de carecer de citófono físico) o Fijo. (3) Asignación de 0 a N parqueaderos privados (bahías de auto o moto) por inmueble. | Actualizado en `docs/01_descubrimiento/flujos_de_proceso/01_flujo_gestion_unidades.md` y modelos en `crates/shared/src/models.rs`. |

---

## 2. Decisiones Abiertas (`DA-XX` — 🔴 Pendientes por Definir)

| ID | Fecha Apertura | Dominio / Tema | Pregunta o Definición Pendiente | Responsable | Estado |
| :---: | :---: | :--- | :--- | :---: | :---: |
| **DA-01** | 2026-11-22 | Resiliencia de Red | ¿Las terminales de portería deben soportar **operación offline (Local-First)** con SQLite local si se cae el internet, sincronizando en diferido con el servidor central? | Fundador / Requerimientos | 🔴 ABIERTA (Propuesta: Sí, obligatorio) |
| **DA-02** | 2026-11-22 | Framework UI Cliente | Para la interfaz webview de Tauri v2: ¿Preferir **React + Tailwind** o **Svelte + Tailwind** para la UI de los vigilantes? | Fundador / UI | 🔴 ABIERTA |
| **DA-03** | 2026-11-22 | Acceso Administrador | ¿El Administrador utilizará la **misma aplicación de escritorio** (iniciando sesión con rol de Administrador que desbloquea los módulos de configuración y reportes) o prefieres un panel web? | Fundador / Arquitectura | 🔴 ABIERTA |
| **DA-04** | 2026-11-22 | Notificación a Residentes | ¿Cuál será el canal principal de notificación para el MVP cuando llegue visita o paquete? (WhatsApp API, Notificación Push a app móvil, Telegram o SMS). | Fundador / Requerimientos | 🔴 ABIERTA |
| **DA-05** | 2026-11-22 | Autenticación en Portería | ¿Cómo se autentican los guardias al cambiar de turno en la terminal de garita? (PIN rápido numérico de 4 dígitos, usuario/contraseña o tarjeta RFID). | Fundador / UX Portería | 🔴 ABIERTA |
