mod interface;
use color_eyre::{eyre::WrapErr, Result};
pub use interface::*;

pub type DbClient = tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WizardDatabase {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
}

pub async fn connect_to_database(
    db: &WizardDatabase,
) -> Result<tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>> {
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
