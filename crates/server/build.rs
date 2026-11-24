// Recompila el servidor cuando se agrega o cambia una migración: `sqlx::migrate!`
// las incrusta en el binario al compilar.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
