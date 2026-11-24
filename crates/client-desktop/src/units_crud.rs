//! Reglas del CRUD de Unidades & Predios (inmuebles y torres).
//!
//! Especificación: `specs/reglas_negocio/RN_unidades_y_torres.md`.
//! Las funciones no tocan disco ni UI: validan contra el estado actual y devuelven
//! un [`Plan`] con todos los registros que deben guardarse (y sincronizarse).

use mamori_shared::sync::{DataSnapshot, ParkingSpotRecord, ResidentRecord, TowerRecord, UnitRecord};
use std::collections::HashMap;

/// Tipos de inmueble permitidos (RN-UN-11, alineado con `UnitType`).
pub const UNIT_TYPES: [&str; 4] = ["Apartamento", "Casa", "Local", "Oficina"];

/// Rol con permiso de eliminar (RN-UN-09).
pub const ADMIN_ROLE: &str = "Administrador";

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CrudError {
    #[error("Ingrese el número del inmueble.")]
    MissingUnitNumber,
    #[error("Ingrese el nombre de la torre o bloque.")]
    MissingTowerName,
    #[error("Ya existe el inmueble {0} en {1}.")]
    DuplicateUnit(String, String),
    #[error("Ya existe una torre o bloque llamado {0}.")]
    DuplicateTower(String),
    #[error("Registre al menos un canal de contacto: citófono o celular.")]
    NoContactChannel,
    #[error("Tipo de inmueble no válido: {0}.")]
    InvalidUnitType(String),
    #[error("La torre {0} no existe o fue eliminada.")]
    TowerNotFound(String),
    #[error("El inmueble no existe o fue eliminado.")]
    UnitNotFound,
    #[error("El parqueadero {0} ya está asignado a otro inmueble.")]
    ParkingTaken(String),
    #[error("Solo el Administrador puede eliminar.")]
    NotAllowed,
    #[error("{0} tiene {1} residente(s): reasígnelos antes de eliminar.")]
    UnitHasResidents(String, usize),
    #[error("{0} tiene {1} inmueble(s): elimínelos antes de la torre.")]
    TowerHasUnits(String, usize),
}

/// Datos del formulario de inmueble.
#[derive(Debug, Clone, Default)]
pub struct UnitForm {
    pub unit_number: String,
    pub unit_type: String,
    pub intercom: Option<String>,
    pub mobile: Option<String>,
    pub landline: Option<String>,
    /// Códigos de parqueadero (ej. `P-104`).
    pub parking: Vec<String>,
}

/// Registros a guardar como resultado de una operación.
#[derive(Debug, Default, Clone)]
pub struct Plan {
    pub towers: Vec<TowerRecord>,
    pub units: Vec<UnitRecord>,
    pub parking_spots: Vec<ParkingSpotRecord>,
    pub residents: Vec<ResidentRecord>,
}

fn same(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// "Sin citófono" (o vacío) no cuenta como canal de contacto (RN-UN-02).
fn contact(v: &Option<String>) -> Option<String> {
    v.as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !same(s, "Sin citófono"))
}

fn active_tower<'a>(data: &'a DataSnapshot, name: &str) -> Option<&'a TowerRecord> {
    data.towers.iter().find(|t| t.is_active && same(&t.name, name))
}

fn active_unit<'a>(data: &'a DataSnapshot, id: &str) -> Option<&'a UnitRecord> {
    data.units.iter().find(|u| u.is_active && u.id == id)
}

fn check_unit_type(t: &str) -> Result<String, CrudError> {
    UNIT_TYPES
        .iter()
        .find(|v| same(v, t))
        .map(|v| v.to_string())
        .ok_or_else(|| CrudError::InvalidUnitType(t.to_string()))
}

/// Valida el formulario y la unicidad del número en la torre (RN-UN-01, RN-UN-02, RN-UN-11).
fn validate_form(data: &DataSnapshot, tower: &str, form: &UnitForm, editing_id: Option<&str>) -> Result<(), CrudError> {
    let number = form.unit_number.trim();
    if number.is_empty() {
        return Err(CrudError::MissingUnitNumber);
    }
    check_unit_type(&form.unit_type)?;
    if contact(&form.intercom).is_none() && contact(&form.mobile).is_none() {
        return Err(CrudError::NoContactChannel);
    }
    let duplicate = data.units.iter().any(|u| {
        u.is_active && Some(u.id.as_str()) != editing_id && same(&u.tower, tower) && same(&u.unit_number, number)
    });
    if duplicate {
        return Err(CrudError::DuplicateUnit(number.to_string(), tower.to_string()));
    }
    Ok(())
}

/// Reemplaza etiquetas `Torre - Inmueble` en los residentes; devuelve los que cambiaron.
fn relabel_residents(data: &DataSnapshot, renames: &HashMap<String, String>) -> Vec<ResidentRecord> {
    data.residents
        .iter()
        .filter(|r| r.units.iter().any(|l| renames.contains_key(l)))
        .map(|r| ResidentRecord {
            units: r.units.iter().map(|l| renames.get(l).cloned().unwrap_or_else(|| l.clone())).collect(),
            ..r.clone()
        })
        .collect()
}

/// Asigna/libera parqueaderos para que el inmueble `unit_id` quede exactamente con `codes`.
fn reconcile_parking(data: &DataSnapshot, unit_id: &str, codes: &[String]) -> Result<Vec<ParkingSpotRecord>, CrudError> {
    let mut changes = Vec::new();
    for code in codes {
        let spot = data
            .parking_spots
            .iter()
            .find(|p| same(&p.code, code))
            .ok_or_else(|| CrudError::ParkingTaken(code.clone()))?;
        match spot.assigned_unit_id.as_deref() {
            Some(owner) if owner == unit_id => {}
            Some(_) => return Err(CrudError::ParkingTaken(spot.code.clone())),
            None => changes.push(ParkingSpotRecord { assigned_unit_id: Some(unit_id.to_string()), ..spot.clone() }),
        }
    }
    // Liberar los que tenía y ya no están en la lista (REQ-UN-04).
    for spot in data.parking_spots.iter().filter(|p| p.assigned_unit_id.as_deref() == Some(unit_id)) {
        if !codes.iter().any(|c| same(c, &spot.code)) {
            changes.push(ParkingSpotRecord { assigned_unit_id: None, ..spot.clone() });
        }
    }
    Ok(changes)
}

fn canonical_codes(data: &DataSnapshot, codes: &[String]) -> Vec<String> {
    codes
        .iter()
        .map(|c| {
            data.parking_spots
                .iter()
                .find(|p| same(&p.code, c))
                .map(|p| p.code.clone())
                .unwrap_or_else(|| c.trim().to_string())
        })
        .collect()
}

// ============================================================================
// INMUEBLES
// ============================================================================

pub fn create_unit(data: &DataSnapshot, tower: &str, form: &UnitForm) -> Result<Plan, CrudError> {
    let tower = active_tower(data, tower).ok_or_else(|| CrudError::TowerNotFound(tower.to_string()))?;
    validate_form(data, &tower.name, form, None)?;
    let id = uuid::Uuid::new_v4().to_string(); // RN-SYNC-07
    let parking = canonical_codes(data, &form.parking);
    let parking_spots = reconcile_parking(data, &id, &parking)?;
    let unit = UnitRecord {
        id,
        tower: tower.name.clone(),
        unit_number: form.unit_number.trim().to_string(),
        unit_type: check_unit_type(&form.unit_type)?,
        intercom: contact(&form.intercom),
        mobile: contact(&form.mobile),
        landline: contact(&form.landline),
        parking,
        is_active: true,
    };
    Ok(Plan { units: vec![unit], parking_spots, ..Plan::default() })
}

/// Edita un inmueble (la torre no cambia, DA-09). Actualiza parqueaderos y residentes.
pub fn update_unit(data: &DataSnapshot, id: &str, form: &UnitForm) -> Result<Plan, CrudError> {
    let current = active_unit(data, id).ok_or(CrudError::UnitNotFound)?;
    validate_form(data, &current.tower, form, Some(id))?;
    let parking = canonical_codes(data, &form.parking);
    let parking_spots = reconcile_parking(data, id, &parking)?;
    let unit = UnitRecord {
        unit_number: form.unit_number.trim().to_string(),
        unit_type: check_unit_type(&form.unit_type)?,
        intercom: contact(&form.intercom),
        mobile: contact(&form.mobile),
        landline: contact(&form.landline),
        parking,
        ..current.clone()
    };
    let mut renames = HashMap::new();
    if unit.label() != current.label() {
        renames.insert(current.label(), unit.label()); // RN-UN-10
    }
    Ok(Plan {
        residents: relabel_residents(data, &renames),
        units: vec![unit],
        parking_spots,
        ..Plan::default()
    })
}

/// Residentes asociados a un inmueble.
pub fn residents_of(data: &DataSnapshot, unit: &UnitRecord) -> usize {
    let label = unit.label();
    data.residents.iter().filter(|r| r.units.iter().any(|l| same(l, &label))).count()
}

/// Inactiva un inmueble y libera sus parqueaderos (RN-UN-07, RN-UN-08, RN-UN-09).
pub fn delete_unit(data: &DataSnapshot, id: &str, role: &str) -> Result<Plan, CrudError> {
    if role != ADMIN_ROLE {
        return Err(CrudError::NotAllowed);
    }
    let current = active_unit(data, id).ok_or(CrudError::UnitNotFound)?;
    let residents = residents_of(data, current);
    if residents > 0 {
        return Err(CrudError::UnitHasResidents(current.label(), residents));
    }
    let parking_spots = reconcile_parking(data, id, &[])?;
    let unit = UnitRecord { is_active: false, parking: Vec::new(), ..current.clone() };
    Ok(Plan { units: vec![unit], parking_spots, ..Plan::default() })
}

// ============================================================================
// TORRES / BLOQUES
// ============================================================================

fn check_tower_name(data: &DataSnapshot, name: &str, editing_id: Option<&str>) -> Result<String, CrudError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CrudError::MissingTowerName);
    }
    if data.towers.iter().any(|t| t.is_active && Some(t.id.as_str()) != editing_id && same(&t.name, name)) {
        return Err(CrudError::DuplicateTower(name.to_string()));
    }
    Ok(name.to_string())
}

pub fn create_tower(data: &DataSnapshot, name: &str) -> Result<Plan, CrudError> {
    let name = check_tower_name(data, name, None)?;
    let tower = TowerRecord { id: uuid::Uuid::new_v4().to_string(), name, is_active: true };
    Ok(Plan { towers: vec![tower], ..Plan::default() })
}

/// Renombra una torre y propaga el nombre a sus inmuebles y residentes (RN-TR-03).
pub fn rename_tower(data: &DataSnapshot, id: &str, name: &str) -> Result<Plan, CrudError> {
    let current = data
        .towers
        .iter()
        .find(|t| t.is_active && t.id == id)
        .ok_or_else(|| CrudError::TowerNotFound(id.to_string()))?;
    let name = check_tower_name(data, name, Some(id))?;
    if name == current.name {
        return Ok(Plan::default());
    }
    let mut renames = HashMap::new();
    let units: Vec<UnitRecord> = data
        .units
        .iter()
        .filter(|u| same(&u.tower, &current.name))
        .map(|u| {
            let renamed = UnitRecord { tower: name.clone(), ..u.clone() };
            renames.insert(u.label(), renamed.label());
            renamed
        })
        .collect();
    Ok(Plan {
        towers: vec![TowerRecord { name, ..current.clone() }],
        residents: relabel_residents(data, &renames),
        units,
        ..Plan::default()
    })
}

/// Inmuebles activos de una torre.
pub fn units_in_tower(data: &DataSnapshot, tower: &str) -> usize {
    data.units.iter().filter(|u| u.is_active && same(&u.tower, tower)).count()
}

/// Inactiva una torre sin inmuebles activos (RN-TR-02, propuesta DA-08).
pub fn delete_tower(data: &DataSnapshot, id: &str, role: &str) -> Result<Plan, CrudError> {
    if role != ADMIN_ROLE {
        return Err(CrudError::NotAllowed);
    }
    let current = data
        .towers
        .iter()
        .find(|t| t.is_active && t.id == id)
        .ok_or_else(|| CrudError::TowerNotFound(id.to_string()))?;
    let units = units_in_tower(data, &current.name);
    if units > 0 {
        return Err(CrudError::TowerHasUnits(current.name.clone(), units));
    }
    Ok(Plan { towers: vec![TowerRecord { is_active: false, ..current.clone() }], ..Plan::default() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed() -> DataSnapshot {
        serde_json::from_str(include_str!("../../../specs/mocks_dummy_data/seed_estado_local.json")).unwrap()
    }

    fn form(number: &str) -> UnitForm {
        UnitForm {
            unit_number: number.into(),
            unit_type: "Apartamento".into(),
            mobile: Some("300 000 0000".into()),
            ..UnitForm::default()
        }
    }

    fn apply(data: &mut DataSnapshot, plan: &Plan) {
        for t in &plan.towers {
            match data.towers.iter_mut().find(|x| x.id == t.id) {
                Some(x) => *x = t.clone(),
                None => data.towers.push(t.clone()),
            }
        }
        for u in &plan.units {
            match data.units.iter_mut().find(|x| x.id == u.id) {
                Some(x) => *x = u.clone(),
                None => data.units.push(u.clone()),
            }
        }
        for p in &plan.parking_spots {
            if let Some(x) = data.parking_spots.iter_mut().find(|x| x.code == p.code) {
                *x = p.clone();
            }
        }
        for r in &plan.residents {
            if let Some(x) = data.residents.iter_mut().find(|x| x.id == r.id) {
                *x = r.clone();
            }
        }
    }

    #[test]
    fn crear_asigna_parqueaderos_y_valida_unicidad() {
        let data = seed();
        let mut f = form("Apto 301");
        f.parking = vec!["p-104".into()];
        let plan = create_unit(&data, "Torre 1", &f).unwrap();
        assert_eq!(plan.units[0].parking, vec!["P-104"]);
        assert_eq!(plan.parking_spots[0].assigned_unit_id.as_deref(), Some(plan.units[0].id.as_str()));

        let dup = create_unit(&data, "torre 1", &form(" apto 101 ")).unwrap_err();
        assert!(matches!(dup, CrudError::DuplicateUnit(..)));
    }

    #[test]
    fn exige_canal_de_contacto_y_tipo_valido() {
        let data = seed();
        let mut f = form("Apto 999");
        f.mobile = None;
        f.intercom = Some("Sin citófono".into());
        assert_eq!(create_unit(&data, "Torre 1", &f).unwrap_err(), CrudError::NoContactChannel);
        let mut f = form("Apto 999");
        f.unit_type = "Bodega".into();
        assert!(matches!(create_unit(&data, "Torre 1", &f).unwrap_err(), CrudError::InvalidUnitType(_)));
    }

    #[test]
    fn no_permite_parqueadero_de_otro_inmueble() {
        let data = seed();
        let mut f = form("Apto 301");
        f.parking = vec!["P-101".into()]; // asignado a u1
        assert!(matches!(create_unit(&data, "Torre 1", &f).unwrap_err(), CrudError::ParkingTaken(_)));
    }

    #[test]
    fn editar_libera_parqueaderos_quitados_y_renombra_residentes() {
        let data = seed();
        // u2 = Torre 1 - Apto 102, con P-102 y P-102B, residente r1.
        let mut f = form("Apto 102-B");
        f.parking = vec!["P-102".into(), "M-01".into()];
        let plan = update_unit(&data, "u2", &f).unwrap();
        let freed = plan.parking_spots.iter().find(|p| p.code == "P-102B").unwrap();
        assert_eq!(freed.assigned_unit_id, None);
        let taken = plan.parking_spots.iter().find(|p| p.code == "M-01").unwrap();
        assert_eq!(taken.assigned_unit_id.as_deref(), Some("u2"));
        assert_eq!(plan.residents.len(), 1);
        assert_eq!(plan.residents[0].units, vec!["Torre 1 - Apto 102-B"]);
    }

    #[test]
    fn editar_puede_conservar_su_propio_numero() {
        let data = seed();
        assert!(update_unit(&data, "u1", &form("Apto 101")).is_ok());
    }

    #[test]
    fn eliminar_solo_admin_y_sin_residentes() {
        let mut data = seed();
        assert_eq!(delete_unit(&data, "u1", "Portería").unwrap_err(), CrudError::NotAllowed);
        // u2 tiene al residente r1.
        assert!(matches!(delete_unit(&data, "u2", ADMIN_ROLE).unwrap_err(), CrudError::UnitHasResidents(_, 1)));

        let plan = delete_unit(&data, "u1", ADMIN_ROLE).unwrap();
        assert!(!plan.units[0].is_active);
        assert_eq!(plan.parking_spots[0].code, "P-101");
        assert_eq!(plan.parking_spots[0].assigned_unit_id, None);
        apply(&mut data, &plan);
        // Ya inactivo: se puede crear otro con el mismo número y no cuenta en la torre.
        assert!(create_unit(&data, "Torre 1", &form("Apto 101")).is_ok());
        assert_eq!(units_in_tower(&data, "Torre 1"), 5);
    }

    #[test]
    fn torres_crear_renombrar_y_eliminar() {
        let mut data = seed();
        assert!(matches!(create_tower(&data, " torre 1 ").unwrap_err(), CrudError::DuplicateTower(_)));
        let plan = create_tower(&data, "Torre 5").unwrap();
        apply(&mut data, &plan);

        // Renombrar Torre 1 propaga a sus 6 inmuebles y al residente de Apto 102.
        let plan = rename_tower(&data, "t1", "Torre Uno").unwrap();
        assert_eq!(plan.units.len(), 6);
        assert!(plan.units.iter().all(|u| u.tower == "Torre Uno"));
        assert_eq!(plan.residents[0].units, vec!["Torre Uno - Apto 102"]);
        apply(&mut data, &plan);

        assert!(matches!(delete_tower(&data, "t1", ADMIN_ROLE).unwrap_err(), CrudError::TowerHasUnits(_, 6)));
        assert_eq!(delete_tower(&data, "t2", "Portería").unwrap_err(), CrudError::NotAllowed);
        let plan = delete_tower(&data, "t2", ADMIN_ROLE).unwrap();
        assert!(!plan.towers[0].is_active);
    }
}
