// Database
pub const ERROR_SQLITE_ACCESS: &str = "Error while accessing SQLite connection";
pub const CSV_FORMAT_ERROR: &str = "CSV file does not have the right format";

// Parameters
pub const PARAM_DATABASE_PATH: &str = "DATABASE_PATH";
pub const DEFAULT_DATABASE: &str = ":memory:";
pub const PARAM_TOKEN_LIST: &str = "TOKEN_LIST";
pub const TOKEN_COOKIE: &str = "navaid_auth_token";
pub const PARAM_HOST: &str = "HOST";
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const PARAM_PORT: &str = "PORT";
pub const DEFAULT_PORT: &str = "8080";
pub const PORT_ERROR: &str = "$PORT cannot be converted to uint_16";

// HTTP
pub const HTTP_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
