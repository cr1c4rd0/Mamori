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
| **Ventana de Sincronización** | Hora fija del día (03:00, 10:00, 15:00, 22:00) fuera de hora pico en la que la terminal envía y recibe datos del servidor (D-23). | Sincronización |
| **Outbox (Cola de Envío)** | Archivo local con los cambios hechos en la terminal que aún no han sido confirmados por el servidor. Nunca se pierde un cambio: se borra de la cola solo al recibir confirmación. | Sincronización |
| **Nodo** | Cada terminal de portería (o el servidor) dentro del sistema distribuido. Se identifica con un `node_id` único guardado en su carpeta de datos. | Sincronización |
| **Cursor de Sincronización** | Número de secuencia del último cambio del servidor que la terminal ya recibió; permite pedir solo lo nuevo. | Sincronización |
| **SRS** | *Software Requirements Specification* (ISO/IEC/IEEE 29148). | `docs/02_requerimientos/` |
| **SDD** | *Specification-Driven Development* (Desarrollo Guiado por Especificaciones Ejecutables). | `specs/` |
