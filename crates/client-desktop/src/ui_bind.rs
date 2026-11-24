//! Conversión entre los registros persistidos (`mamori_shared::sync`) y los
//! modelos que muestra la interfaz Slint.

use crate::store::LocalStore;
use crate::sync::SyncView;
use crate::{ActiveVisitItem, AppWindow, PendingPackageItem, ResidentItem, UnitItem};
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

/// Parqueaderos sin asignar, como `P-104 (Carro)`.
pub fn available_parking(store: &LocalStore) -> Vec<String> {
    store
        .data
        .parking_spots
        .iter()
        .filter(|p| p.assigned_unit_id.is_none())
        .map(|p| p.display_name())
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
    set_rows(ui.get_units(), store.data.units.iter().map(unit_item).collect());

    let mut visits: Vec<&VisitRecord> = store.data.visits.iter().filter(|v| v.exit_at.is_none()).collect();
    visits.sort_by_key(|v| v.entry_at);
    set_rows(ui.get_active_visits(), visits.into_iter().map(visit_item).collect());

    let mut packages: Vec<&PackageRecord> =
        store.data.packages.iter().filter(|p| p.delivered_at.is_none()).collect();
    packages.sort_by_key(|p| p.received_at);
    set_rows(ui.get_pending_packages(), packages.into_iter().map(package_item).collect());

    set_rows(ui.get_residents(), filtered_residents(store, &ui.get_resident_search_query()));
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
