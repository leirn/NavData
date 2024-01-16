// Database
pub const ERROR_SQLITE_ACCESS: &str = "Error while accessing SQLite connection";
pub const CSV_FORMAT_ERROR: &str = "CSV file does not have the right format";

// Parameters
pub const PARAM_DATABASE_PATH: &str = "DATABASE_PATH";
pub const DEFAULT_DATABASE: &str = ":memory:";
pub const PARAM_HOST: &str = "HOST";
pub const PARAM_PORT: &str = "PORT";
pub const PORT_NOT_SET: &str = "$PORT is not set";
pub const PORT_ERROR: &str = "$PORT cannot be converted to uint_16";
pub const HOST_NOT_SET: &str = "$HOST is not set";
