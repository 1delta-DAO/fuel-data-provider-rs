use config::{Config, File};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct DefaultConfig {
    pub server_port_http: u16,
    pub db_url: String,
    pub db_username: String,
    pub db_password: String,
    pub db_min_connections: u32,
    pub db_max_connections: u32,
    pub db_sql_logging: bool,
    pub data_refresh_interval: u64,
    pub rpc_url: String,
    pub tx_log_start_block_number: u64,
    pub calculation_window: u16,
    pub cdi_fuel_token_gateway: String,
    pub cdi_fuel_token_gateway_dependency: String,
    pub cdi_mira_amm: String,
    pub mira_swap_event: String,
    pub mira_create_pool: String,
    pub mira_total_supply: String,
    pub mira_pool_max_age: u8,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub default: DefaultConfig,
}

pub enum EnvVar {
    ServerPortHTTP,
    DbUrl,
    DBUsername,
    DBPassword,
    DBMinConnections,
    DBMaxConnections,
    DBSQLLogging,
    DataRefreshInterval,
    RpcUrl,
    TxLogStartBlockNumber,
    CalculationWindow,
    CdiFuelTokenGateway,
    CdiFuelTokenGatewayDependency,
    CdiMiraAmm,
    MiraSwapEvent,
    MiraCreatePool,
    MiraTotalSupply,
    MiraPoolMaxAge,
}

// Implement Display for EnvVar to convert to string
impl fmt::Display for EnvVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            EnvVar::ServerPortHTTP => "SERVER_PORT_HTTP",
            EnvVar::DbUrl => "DB_URL",
            EnvVar::DBUsername => "DB_USERNAME",
            EnvVar::DBPassword => "DB_PASSWORD",
            EnvVar::DBMinConnections => "DB_MIN_CONNECTIONS",
            EnvVar::DBMaxConnections => "DB_MAX_CONNECTIONS",
            EnvVar::DBSQLLogging => "DB_SQL_LOGGING",
            EnvVar::DataRefreshInterval => "DATA_REFRESH_INTERVAL",
            EnvVar::RpcUrl => "RPC_URL",
            EnvVar::TxLogStartBlockNumber => "TX_LOG_START_BLOCK_NUMBER",
            EnvVar::CalculationWindow => "CALCULATION_WINDOW",
            EnvVar::CdiFuelTokenGateway => "CDI_FUEL_TOKEN_GATEWAY",
            EnvVar::CdiFuelTokenGatewayDependency => "CDI_FUEL_TOKEN_GATEWAY_DEPENDENCY",
            EnvVar::CdiMiraAmm => "CDI_MIRA_AMM",
            EnvVar::MiraSwapEvent => "MIRA_SWAP_EVENT",
            EnvVar::MiraCreatePool => "MIRA_CREATE_POOL",
            EnvVar::MiraTotalSupply => "MIRA_TOTAL_SUPPLY",
            EnvVar::MiraPoolMaxAge => "MIRA_POOL_MAX_AGE",
        };
        write!(f, "{}", name)
    }
}

impl EnvVar {
    pub fn get_value<T: std::str::FromStr>(&self, default: T) -> T
    where
        T::Err: std::fmt::Debug,
    {
        env::var(self.to_string())
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(default)
    }
}

// Lazy static configuration loading
pub static CONFIG: Lazy<Arc<AppConfig>> = Lazy::new(|| {
    log::info!("Loading configuration...");
    match load_config_from_env_or_file() {
        Ok(config) => Arc::new(config),
        Err(e) => {
            log::error!("Failed to load configuration: {:?}", e);
            panic!("Failed to load configuration: {:?}", e);
        }
    }
});

pub fn load_config_from_env_or_file() -> Result<AppConfig, Box<dyn Error>> {
    let mut settings = Config::builder()
        .add_source(File::with_name("resources/config.toml").required(false))
        .build()?;

    let mut config: AppConfig = settings.try_deserialize()?;

    // Override with environment variables
    config.default.server_port_http =
        EnvVar::ServerPortHTTP.get_value(config.default.server_port_http);
    config.default.db_url = EnvVar::DbUrl.get_value(config.default.db_url.clone());
    config.default.db_username = EnvVar::DBUsername.get_value(config.default.db_username.clone());
    config.default.db_password = EnvVar::DBPassword.get_value(config.default.db_password.clone());
    config.default.db_min_connections =
        EnvVar::DBMinConnections.get_value(config.default.db_min_connections);
    config.default.db_max_connections =
        EnvVar::DBMaxConnections.get_value(config.default.db_max_connections);
    config.default.db_sql_logging =
        EnvVar::DBSQLLogging.get_value(config.default.db_sql_logging);
    config.default.data_refresh_interval =
        EnvVar::DataRefreshInterval.get_value(config.default.data_refresh_interval);
    config.default.rpc_url = EnvVar::RpcUrl.get_value(config.default.rpc_url.clone());
    config.default.tx_log_start_block_number =
        EnvVar::TxLogStartBlockNumber.get_value(config.default.tx_log_start_block_number);
    config.default.calculation_window =
        EnvVar::CalculationWindow.get_value(config.default.calculation_window);
    config.default.cdi_fuel_token_gateway = EnvVar::CdiFuelTokenGateway.get_value(config.default.cdi_fuel_token_gateway.clone());
    config.default.cdi_fuel_token_gateway_dependency = EnvVar::CdiFuelTokenGatewayDependency.get_value(config.default.cdi_fuel_token_gateway_dependency.clone());
    config.default.cdi_mira_amm = EnvVar::CdiMiraAmm.get_value(config.default.cdi_mira_amm.clone());
    config.default.mira_swap_event = EnvVar::MiraSwapEvent.get_value(config.default.mira_swap_event.clone());
    config.default.mira_create_pool = EnvVar::MiraCreatePool.get_value(config.default.mira_create_pool.clone());
    config.default.mira_total_supply = EnvVar::MiraTotalSupply.get_value(config.default.mira_total_supply.clone());
    config.default.mira_pool_max_age = EnvVar::MiraPoolMaxAge.get_value(config.default.mira_pool_max_age);
    Ok(config)
}