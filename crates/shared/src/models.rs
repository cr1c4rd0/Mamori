use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Estado de un paquete o encomienda recibida en portería
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageStatus {
    ReceivedAtGate, // Recibido en portería
    ResidentNotified, // Residente notificado
    DeliveredToResident, // Entregado al residente
    ReturnedToCourier, // Devuelto a la empresa de mensajería
}

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

/// Información básica de una unidad residencial / predio (apartamento / casa)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidentialUnit {
    pub id: Uuid,
    pub block_or_tower: String, // Torre o Bloque
    pub unit_number: String,    // Número de apto o casa
    pub primary_phone: String,  // Teléfono principal / citófono
}

/// Información básica de un visitante
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visitor {
    pub id: Uuid,
    pub national_id: String,    // Documento de identidad / Cédula
    pub full_name: String,      // Nombre completo
    pub company: Option<String>,// Empresa / Contratista (si aplica)
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
