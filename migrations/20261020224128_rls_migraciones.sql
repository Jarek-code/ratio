-- Bloquear el acceso por la API pública a la tabla de control de SQLx (la CLI entra como postgres y no se ve afectada)
ALTER TABLE _sqlx_migrations ENABLE ROW LEVEL SECURITY;
