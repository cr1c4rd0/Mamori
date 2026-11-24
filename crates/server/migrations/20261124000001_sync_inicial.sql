-- Esquema inicial de sincronización Local-First (D-27, D-29)
-- Especificación: specs/esquemas_bd/servidor_postgres.md

CREATE TABLE sync_nodes (
    node_id       uuid PRIMARY KEY,
    first_seen_at timestamptz NOT NULL DEFAULT now(),
    last_seen_at  timestamptz NOT NULL DEFAULT now()
);

-- Fuente de verdad: log append-only de cambios recibidos.
CREATE TABLE change_log (
    seq          bigserial PRIMARY KEY,
    change_id    uuid NOT NULL UNIQUE,
    origin_node  uuid NOT NULL REFERENCES sync_nodes(node_id),
    collection   text NOT NULL CHECK (collection IN ('units', 'parking_spots', 'residents', 'visits', 'packages')),
    record_id    text NOT NULL,
    op           text NOT NULL CHECK (op IN ('upsert', 'delete')),
    payload      jsonb,
    created_at   timestamptz NOT NULL,
    received_at  timestamptz NOT NULL DEFAULT now(),
    apply_error  text
);
CREATE INDEX change_log_record_idx ON change_log (collection, record_id);

-- Estado actual materializado (para consultas de otros roles).
CREATE TABLE units (
    id          text PRIMARY KEY,
    tower       text NOT NULL,
    unit_number text NOT NULL,
    unit_type   text NOT NULL,
    intercom    text,
    mobile      text,
    landline    text,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    updated_by  uuid
);

CREATE TABLE parking_spots (
    code             text PRIMARY KEY,
    spot_type        text NOT NULL,
    assigned_unit_id text REFERENCES units(id) ON DELETE SET NULL,
    updated_at       timestamptz NOT NULL DEFAULT now(),
    updated_by       uuid
);

CREATE TABLE residents (
    id         text PRIMARY KEY,
    full_name  text NOT NULL,
    document   text,
    role       text NOT NULL,
    mobile     text,
    email      text,
    updated_at timestamptz NOT NULL DEFAULT now(),
    updated_by uuid
);

CREATE TABLE resident_units (
    resident_id text NOT NULL REFERENCES residents(id) ON DELETE CASCADE,
    unit_label  text NOT NULL,
    PRIMARY KEY (resident_id, unit_label)
);

CREATE TABLE visits (
    id           text PRIMARY KEY,
    visitor_name text NOT NULL,
    document     text NOT NULL,
    destination  text NOT NULL,
    access_type  text NOT NULL,
    entry_at     timestamptz NOT NULL,
    exit_at      timestamptz,
    updated_at   timestamptz NOT NULL DEFAULT now(),
    updated_by   uuid
);
CREATE INDEX visits_activas_idx ON visits (entry_at) WHERE exit_at IS NULL;

CREATE TABLE packages (
    id           text PRIMARY KEY,
    destination  text NOT NULL,
    recipient    text NOT NULL,
    carrier      text NOT NULL,
    description  text NOT NULL,
    received_at  timestamptz NOT NULL,
    delivered_at timestamptz,
    updated_at   timestamptz NOT NULL DEFAULT now(),
    updated_by   uuid
);
CREATE INDEX packages_pendientes_idx ON packages (received_at) WHERE delivered_at IS NULL;
