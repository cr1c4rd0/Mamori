use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// 1. GESTIÓN DE ESTRUCTURA FÍSICA Y UNIDADES RESIDENCIALES (FLP-01)
// ============================================================================

/// Tipo de inmueble en la unidad residencial
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UnitType {
    Apartment,  // Apartamento
    House,      // Casa
    Commercial, // Local comercial
    Office,     // Oficina / Administración
}

/// Torre, bloque o manzana dentro de la copropiedad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerBlock {
    pub id: Uuid,
    pub name: String, // ej: "Torre 1", "Bloque A", "Manzana 3"
    pub description: Option<String>,
}

/// Tipo de celda de parqueadero
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParkingType {
    Car,        // Automóvil
    Motorcycle, // Motocicleta
    Bicycle,    // Bicicleta
    Disabled,   // Movilidad reducida
}

/// Celda o bahía de parqueadero en la copropiedad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkingSpot {
    pub id: Uuid,
    pub code: String,                     // ej: "P-101", "M-04"
    pub parking_type: ParkingType,
    pub assigned_unit_id: Option<Uuid>,   // None = Disponible, Some = Asignado a un inmueble
    pub is_visitor: bool,                 // true = Uso exclusivo para visitas
}

/// Información completa de un predio o inmueble en la unidad residencial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidentialUnit {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub unit_number: String,               // ej: "302", "Casa 14"
    pub unit_type: UnitType,
    pub intercom_extension: Option<String>, // Número de citófono / extensión física
    pub mobile_phone: Option<String>,       // Teléfono celular
    pub landline_phone: Option<String>,     // Teléfono fijo (opcional)
    pub private_parking_spots: Vec<String>, // Lista de celdas privadas asignadas (ej: ["P-101", "M-02"])
    pub is_active: bool,
}

// ============================================================================
// 2. CONTROL DE ACCESOS Y ESTADOS
// ============================================================================

/// Estado de una autorización de acceso para visitantes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccessStatus {
    PendingApproval, // Esperando autorización del residente
    Authorized,      // Autorizado para ingresar
    Denied,          // Acceso denegado
    Inside,          // Actualmente dentro de la unidad
    Completed,       // Salió de la unidad
}

/// Tipo de vehículo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VehicleType {
    Car,
    Motorcycle,
    Bicycle,
    VanTruck,
}

/// Información básica de un visitante
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visitor {
    pub id: Uuid,
    pub national_id: String,     // Documento de identidad / Cédula
    pub full_name: String,       // Nombre completo
    pub company: Option<String>, // Empresa / Contratista (si aplica)
}

/// Registro de entrada o salida de persona o vehículo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLog {
    pub id: Uuid,
    pub unit_id: Uuid,
    pub visitor_id: Option<Uuid>,
    pub vehicle_plate: Option<String>,
    pub vehicle_type: Option<VehicleType>,
    pub status: AccessStatus,
    pub entry_time: DateTime<Utc>,
    pub exit_time: Option<DateTime<Utc>>,
    pub guard_notes: Option<String>,
}

// ============================================================================
// 3. GESTIÓN DE PAQUETERÍA
// ============================================================================

/// Estado de un paquete o encomienda recibida en portería
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageStatus {
    ReceivedAtGate,      // Recibido en portería
    ResidentNotified,    // Residente notificado
    DeliveredToResident, // Entregado al residente
    ReturnedToCourier,   // Devuelto a la empresa de mensajería
}

/// Paquete o encomienda entregada en portería
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDelivery {
    pub id: Uuid,
    pub unit_id: Uuid,
    pub tracking_number: Option<String>,
    pub courier_company: String, // Servientrega, Coordinadora, Amazon, etc.
    pub recipient_name: String,
    pub status: PackageStatus,
    pub received_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub guard_notes: Option<String>,
}
