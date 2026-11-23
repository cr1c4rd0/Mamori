# Glosario Terminológico del Negocio y Dominio

**Proyecto:** Mamori  
**Dominio:** Seguridad y Control de Acceso para Unidades Residenciales  

| Término / Sigla | Definición en el Contexto de Mamori | Módulos / Entidades Relacionadas |
| :--- | :--- | :--- |
| **Unidad Residencial** | Conjunto cerrado, edificio, urbanización o condominio que agrupa predios habitacionales. | Configuración / Multi-tenant |
| **Predio / Inmueble** | Casa, apartamento, penthouse o torre que conforma la unidad residencial. | Módulo Residentes |
| **Terminal de Portería** | Equipo de cómputo físico con Windows en la garita/portería donde los vigilantes operan el cliente de escritorio en Rust. | Cliente Windows (`client-desktop`) |
| **Residente** | Persona propietaria o arrendataria registrada que habita un predio residencial. | Módulo Residentes |
| **Visitante** | Persona no residente que solicita ingreso temporal autorizado a un predio. | Módulo Ingresos / Salidas |
| **Contratista / Personal de Servicio** | Personal con autorización temporal o periódica para prestar servicios dentro de la unidad. | Módulo Seguridad y Accesos |
| **Paquetería / Encomienda** | Correspondencia, entrega de mensajería o delivery recibido en portería y pendiente de entrega al residente. | Módulo Paquetería |
| **Control Vehicular** | Registro de placa, tipo de vehículo y asignación de parqueadero (propio o de visitantes). | Módulo Vehículos |
| **Local-First / Modo Offline** | Capacidad de la terminal de portería para operar sin internet y sincronizar registros al restaurar conexión. | Arquitectura de Resiliencia |
| **SRS** | *Software Requirements Specification* (ISO/IEC/IEEE 29148). | `docs/02_requerimientos/` |
| **SDD** | *Specification-Driven Development* (Desarrollo Guiado por Especificaciones Ejecutables). | `specs/` |
