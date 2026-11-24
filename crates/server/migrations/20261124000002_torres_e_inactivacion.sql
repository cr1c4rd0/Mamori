-- CRUD de Unidades & Predios (D-30, D-31)
-- Especificación: specs/esquemas_bd/servidor_postgres.md · specs/reglas_negocio/RN_unidades_y_torres.md

-- Nueva colección sincronizable: torres / bloques.
ALTER TABLE change_log DROP CONSTRAINT change_log_collection_check;
ALTER TABLE change_log ADD CONSTRAINT change_log_collection_check
    CHECK (collection IN ('towers', 'units', 'parking_spots', 'residents', 'visits', 'packages'));

CREATE TABLE towers (
    id         text PRIMARY KEY,
    name       text NOT NULL,
    is_active  boolean NOT NULL DEFAULT true,
    updated_at timestamptz NOT NULL DEFAULT now(),
    updated_by uuid
);
-- RN-TR-01: nombre único entre torres activas.
CREATE UNIQUE INDEX towers_nombre_activo_uq ON towers (lower(name)) WHERE is_active;

-- Eliminar = inactivar (D-31, RN-UN-N01).
ALTER TABLE units ADD COLUMN is_active boolean NOT NULL DEFAULT true;
CREATE INDEX units_activos_torre_idx ON units (tower) WHERE is_active;
