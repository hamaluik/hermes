//! Wizard commands for populating HL7 messages with data from the database.
//!
//! Wizards simplify the creation of test HL7 messages by providing sample patient,
//! visit, and interface data that can be applied to message templates. This is useful
//! during development and testing when realistic-looking data is needed.
//!
//! # Available Wizards
//! * **Patient Wizard** - Search for sample patients and populate PID segments
//! * **Visit Wizard** - Search for sample visits and populate PV1 segments
//! * **Interface Wizard** - Query sample interfaces and populate MSH segments
//!
//! # Database Connection
//! The wizards accept database connection parameters for compatibility, but currently
//! return hardcoded sample data. This allows testing message composition workflows
//! without requiring an actual database connection.

mod interface;
pub use interface::*;
mod patient;
pub use patient::*;
mod visit;
pub use visit::*;

/// Database connection configuration for wizard commands.
///
/// This struct is passed from the frontend to wizard commands. Currently the
/// connection is not actually used - wizards return sample data instead.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WizardDatabase {
    /// Database server hostname or IP address
    pub host: String,
    /// Database server port (typically 1433 for SQL Server)
    pub port: u16,
    /// Database name
    pub database: String,
    /// Database authentication username
    pub user: String,
    /// Database authentication password
    pub password: String,
}
