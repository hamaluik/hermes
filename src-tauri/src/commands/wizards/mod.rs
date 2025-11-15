//! Wizard commands for populating HL7 messages with data from the database.
//!
//! Wizards simplify the creation of test HL7 messages by allowing users to search for
//! real patient, visit, and interface data and apply it to message templates. This is
//! particularly useful during development and testing when realistic data is needed.
//!
//! # Available Wizards
//! * **Patient Wizard** - Search for patients and populate PID segments
//! * **Visit Wizard** - Search for visits and populate PV1 segments
//! * **Interface Wizard** - Query interfaces and populate MSH segments
//!
//! # Database Connection
//! All wizards require a connection to the HL7 system SQL Server database. The connection
//! is established using the `WizardDatabase` configuration and Tiberius client library.

use color_eyre::{eyre::WrapErr, Result};

mod interface;
pub use interface::*;
mod patient;
pub use patient::*;
mod visit;
pub use visit::*;

/// Type alias for the database client used by all wizard commands.
///
/// Uses the Tiberius SQL Server client wrapped in a Tokio compatibility layer.
pub type DbClient = tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>;

/// database connection configuration.
///
/// This struct is passed from the frontend to wizard commands and contains
/// all necessary connection parameters for SQL Server authentication.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WizardDatabase {
    /// Database server hostname or IP address
    pub host: String,
    /// Database server port (typically 1433 for SQL Server)
    pub port: u16,
    /// Database name (typically the database)
    pub database: String,
    /// SQL Server authentication username
    pub user: String,
    /// SQL Server authentication password
    pub password: String,
}

/// Establish a connection to the database.
///
/// This function creates a new database connection for each call. Connections are
/// not pooled or cached, which is acceptable for wizard commands that are invoked
/// infrequently during message composition.
///
/// # Security Considerations
/// * Uses SQL Server authentication (not Windows authentication)
/// * Certificate validation is disabled via `trust_cert()`
/// * Encryption is disabled (NotSupported)
///
/// These settings are appropriate for development/testing against internal databases,
/// but should be revisited if connecting to production systems or over untrusted networks.
///
/// # Arguments
/// * `db` - Database connection configuration
///
/// # Returns
/// * `Ok(DbClient)` - Connected database client
/// * `Err` - Connection error with context
pub async fn connect_to_database(db: &WizardDatabase) -> Result<DbClient> {
    use tiberius::{AuthMethod, Client, Config, EncryptionLevel};
    use tokio_util::compat::TokioAsyncWriteCompatExt;

    let mut config = Config::new();
    config.host(db.host.clone());
    config.port(db.port);
    config.authentication(AuthMethod::sql_server(db.user.clone(), db.password.clone()));
    config.database(db.database.clone());
    config.trust_cert(); // TODO: probably make this configurable, along with cert file
    config.encryption(EncryptionLevel::NotSupported);

    let tcp = tokio::net::TcpStream::connect(config.get_addr())
        .await
        .wrap_err_with(|| "Failed to connect to database")?;

    let client = Client::connect(config, tcp.compat_write())
        .await
        .wrap_err("Failed to establish database connection")?;

    Ok(client)
}
