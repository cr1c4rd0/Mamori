//! Conversión entre los registros persistidos (`mamori_shared::sync`) y los
//! modelos que muestra la interfaz Slint.

use crate::store::LocalStore;
use crate::sync::SyncView;
use crate::units_crud::units_in_tower;
use crate::{ActiveVisitItem, AppWindow, DetailRow, PendingPackageItem, ResidentItem, TowerItem, UnitItem};
use chrono::Local;
use mamori_shared::sync::{PackageRecord, ResidentRecord, UnitRecord, VisitRecord};
use slint::{Model, ModelRc, VecModel};

fn or_dash(v: &Option<String>, empty: &str) -> slint::SharedString {
    match v {
        Some(s) if !s.trim().is_empty() => s.as_str().into(),
        _ => empty.into(),
    }
}

/// `None` si el texto del formulario está vacío.
pub fn non_empty(s: &str) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}

pub fn unit_item(u: &UnitRecord) -> UnitItem {
    UnitItem {
        id: u.id.as_str().into(),
        tower: u.tower.as_str().into(),
        unit_number: u.unit_number.as_str().into(),
        unit_type: u.unit_type.as_str().into(),
        intercom: or_dash(&u.intercom, "Sin citófono"),
        mobile: or_dash(&u.mobile, "—"),
        landline: or_dash(&u.landline, "—"),
        parking: if u.parking.is_empty() { "Sin celda".into() } else { u.parking.join(", ").into() },
    }
}

fn visit_item(v: &VisitRecord) -> ActiveVisitItem {
    ActiveVisitItem {
        id: v.id.as_str().into(),
        visitor_name: v.visitor_name.as_str().into(),
        document: v.document.as_str().into(),
        destination: v.destination.as_str().into(),
        entry_time: v.entry_at.with_timezone(&Local).format("%H:%M").to_string().into(),
        access_type: v.access_type.as_str().into(),
    }
}

fn package_item(p: &PackageRecord) -> PendingPackageItem {
    PendingPackageItem {
        id: p.id.as_str().into(),
        destination: p.destination.as_str().into(),
        recipient: p.recipient.as_str().into(),
        carrier: p.carrier.as_str().into(),
        description: p.description.as_str().into(),
        received_time: p.received_at.with_timezone(&Local).format("%H:%M").to_string().into(),
    }
}

fn resident_item(r: &ResidentRecord) -> ResidentItem {
    ResidentItem {
        id: r.id.as_str().into(),
        full_name: r.full_name.as_str().into(),
        document: or_dash(&r.document, "Sin documento"),
        unit_label: r.units.join(", ").into(),
        role: r.role.as_str().into(),
        mobile: or_dash(&r.mobile, "—"),
        email: or_dash(&r.email, "—"),
    }
}

/// Residentes que coinciden con la búsqueda (nombre, documento, inmueble, celular o rol).
pub fn filtered_residents(store: &LocalStore, query: &str) -> Vec<ResidentItem> {
    let q = query.trim().to_lowercase();
    store
        .data
        .residents
        .iter()
        .map(resident_item)
        .filter(|r| {
            q.is_empty()
                || [&r.full_name, &r.document, &r.unit_label, &r.mobile, &r.role]
                    .iter()
                    .any(|f| f.to_lowercase().contains(&q))
        })
        .collect()
}

/// Parqueaderos seleccionables, como `P-104 (Carro)`: los libres y, al editar, los que
/// ya tiene el propio inmueble (RN-UN-04).
pub fn available_parking(store: &LocalStore, editing_unit_id: &str) -> Vec<String> {
    store
        .data
        .parking_spots
        .iter()
        .filter(|p| match p.assigned_unit_id.as_deref() {
            None => true,
            Some(owner) => !editing_unit_id.is_empty() && owner == editing_unit_id,
        })
        .map(|p| p.display_name())
        .collect()
}

/// Parqueaderos de un inmueble, como `P-101 (Carro)`, para precargar el formulario de edición.
pub fn unit_parking_display(store: &LocalStore, unit: &UnitRecord) -> Vec<slint::SharedString> {
    unit.parking
        .iter()
        .map(|code| {
            store
                .data
                .parking_spots
                .iter()
                .find(|p| &p.code == code)
                .map(|p| p.display_name())
                .unwrap_or_else(|| code.clone())
                .into()
        })
        .collect()
}

/// Reemplaza el contenido de un `VecModel` existente (las callbacks conservan su referencia).
fn set_rows<T: Clone + 'static>(model: ModelRc<T>, rows: Vec<T>) {
    if let Some(vm) = model.as_any().downcast_ref::<VecModel<T>>() {
        vm.set_vec(rows);
    }
}

/// Vuelve a pintar todas las listas a partir del almacenamiento local.
pub fn refresh_all(ui: &AppWindow, store: &LocalStore) {
    // Torres activas con su contador real (REQ-UN-02). Si la torre seleccionada ya no
    // existe (eliminada o renombrada en otra terminal), se selecciona la primera.
    let towers: Vec<TowerItem> = store
        .data
        .towers
        .iter()
        .filter(|t| t.is_active)
        .map(|t| TowerItem {
            id: t.id.as_str().into(),
            name: t.name.as_str().into(),
            unit_count: units_in_tower(&store.data, &t.name) as i32,
        })
        .collect();
    let selected = ui.get_selected_tower();
    if !towers.iter().any(|t| t.name == selected) {
        ui.set_selected_tower(towers.first().map(|t| t.name.clone()).unwrap_or_default());
    }
    set_rows(ui.get_towers(), towers);

    // Todos los inmuebles activos (KPI y selector de Residentes) y los de la torre (tabla).
    let mut active_units: Vec<&UnitRecord> = store.data.units.iter().filter(|u| u.is_active).collect();
    active_units.sort_by(|a, b| a.tower.cmp(&b.tower).then_with(|| a.unit_number.cmp(&b.unit_number)));
    set_rows(ui.get_units(), active_units.iter().map(|u| unit_item(u)).collect());
    let selected = ui.get_selected_tower();
    set_rows(
        ui.get_tower_units(),
        active_units.iter().filter(|u| u.tower == selected.as_str()).map(|u| unit_item(u)).collect(),
    );

    let mut visits: Vec<&VisitRecord> = store.data.visits.iter().filter(|v| v.exit_at.is_none()).collect();
    visits.sort_by_key(|v| v.entry_at);
    set_rows(ui.get_active_visits(), visits.into_iter().map(visit_item).collect());

    let mut packages: Vec<&PackageRecord> =
        store.data.packages.iter().filter(|p| p.delivered_at.is_none()).collect();
    packages.sort_by_key(|p| p.received_at);
    set_rows(ui.get_pending_packages(), packages.into_iter().map(package_item).collect());

    set_rows(ui.get_residents(), filtered_residents(store, &ui.get_resident_search_query()));

    // Si la Ficha del Inmueble está abierta, se actualiza; si el inmueble ya no existe
    // (eliminado aquí o en otra terminal), se cierra.
    if ui.get_show_unit_detail() && !fill_unit_detail(ui, store, &ui.get_detail_unit().id) {
        ui.set_show_unit_detail(false);
    }
}

fn detail_row(title: impl Into<String>, subtitle: impl Into<String>, meta: impl Into<String>) -> DetailRow {
    DetailRow { title: title.into().into(), subtitle: subtitle.into().into(), meta: meta.into().into() }
}

fn rows_model(rows: Vec<DetailRow>) -> ModelRc<DetailRow> {
    std::rc::Rc::new(VecModel::from(rows)).into()
}

/// Contenido de la Ficha del Inmueble, independiente de la ventana (se puede probar).
pub struct UnitDetail {
    pub unit: UnitItem,
    pub parking: Vec<DetailRow>,
    pub residents: Vec<DetailRow>,
    pub visits: Vec<DetailRow>,
    pub packages: Vec<DetailRow>,
}

/// Llena la Ficha del Inmueble (D-34, REQ-UN-12). Devuelve `false` si el inmueble no
/// existe o está inactivo.
pub fn fill_unit_detail(ui: &AppWindow, store: &LocalStore, unit_id: &str) -> bool {
    let Some(d) = unit_detail(store, unit_id) else {
        return false;
    };
    ui.set_detail_unit(d.unit);
    ui.set_detail_parking(rows_model(d.parking));
    ui.set_detail_residents(rows_model(d.residents));
    ui.set_detail_visits(rows_model(d.visits));
    ui.set_detail_packages(rows_model(d.packages));
    true
}

/// Arma la ficha: contactos, parqueaderos con su tipo, residentes asociados, visitas
/// dentro y paquetes pendientes dirigidos al inmueble. `None` si no existe o está inactivo.
pub fn unit_detail(store: &LocalStore, unit_id: &str) -> Option<UnitDetail> {
    let data = &store.data;
    let unit = data.units.iter().find(|u| u.is_active && u.id == unit_id)?;
    let label = unit.label();
    let time = |t: &chrono::DateTime<chrono::Utc>| t.with_timezone(&Local).format("%d/%m %H:%M").to_string();

    let parking = unit
        .parking
        .iter()
        .map(|code| {
            let spot_type = data
                .parking_spots
                .iter()
                .find(|p| &p.code == code)
                .map(|p| p.spot_type.clone())
                .unwrap_or_default();
            detail_row(code.as_str(), spot_type, "")
        })
        .collect();

    let residents = data
        .residents
        .iter()
        .filter(|r| r.units.iter().any(|l| l == &label))
        .map(|r| {
            let mut subtitle = r.role.clone();
            if let Some(m) = r.mobile.as_deref().filter(|m| !m.trim().is_empty()) {
                subtitle.push_str(&format!("  •  {m}"));
            }
            detail_row(r.full_name.as_str(), subtitle, r.document.clone().unwrap_or_default())
        })
        .collect();

    let mut visits: Vec<&VisitRecord> =
        data.visits.iter().filter(|v| v.exit_at.is_none() && v.destination == label).collect();
    visits.sort_by_key(|v| v.entry_at);
    let visits = visits
        .into_iter()
        .map(|v| detail_row(v.visitor_name.as_str(), format!("{}  •  {}", v.document, v.access_type), format!("Entró {}", time(&v.entry_at))))
        .collect();

    let mut packages: Vec<&PackageRecord> =
        data.packages.iter().filter(|p| p.delivered_at.is_none() && p.destination == label).collect();
    packages.sort_by_key(|p| p.received_at);
    let packages = packages
        .into_iter()
        .map(|p| detail_row(format!("{} — {}", p.carrier, p.recipient), p.description.as_str(), time(&p.received_at)))
        .collect();

    Some(UnitDetail { unit: unit_item(unit), parking, residents, visits, packages })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::temp_dir;

    #[test]
    fn ficha_reune_toda_la_informacion_del_inmueble() {
        let store = LocalStore::open(temp_dir("ficha")).unwrap();

        // u2 = Torre 1 - Apto 102: P-102 y P-102B, residente Mateo, paquete de Mercado Libre.
        let d = unit_detail(&store, "u2").unwrap();
        assert_eq!(d.unit.unit_number, "Apto 102");
        let parking: Vec<(&str, &str)> = d.parking.iter().map(|r| (r.title.as_str(), r.subtitle.as_str())).collect();
        assert_eq!(parking, vec![("P-102", "Carro"), ("P-102B", "Carro")]);
        assert_eq!(d.residents.len(), 1);
        assert_eq!(d.residents[0].title, "Mateo Restrepo Cano");
        assert!(d.residents[0].subtitle.contains("Propietario"));
        assert_eq!(d.packages.len(), 1);
        assert!(d.packages[0].title.starts_with("Mercado Libre"));
        assert!(d.visits.is_empty());

        // u5 = Torre 1 - Apto 202: la visita activa de Laura Sofía Mendoza.
        let d = unit_detail(&store, "u5").unwrap();
        assert_eq!(d.visits.len(), 1);
        assert_eq!(d.visits[0].title, "Laura Sofía Mendoza");
        assert!(d.visits[0].meta.starts_with("Entró"));

        assert!(unit_detail(&store, "no-existe").is_none());
    }
}

pub fn set_sync_indicator(ui: &AppWindow, view: &SyncView) {
    let (state, title, detail) = view.labels();
    ui.set_sync_state(state.into());
    ui.set_sync_title(title.into());
    ui.set_sync_detail(detail.into());
}

/// Id de la fila `idx` de un modelo, si existe.
pub fn row_id<T>(model: &impl Model<Data = T>, idx: i32, id: impl Fn(&T) -> String) -> Option<String> {
    usize::try_from(idx).ok().and_then(|i| model.row_data(i)).map(|row| id(&row))
}
