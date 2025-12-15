use std::env;
use std::error::Error;
use std::sync::Arc;

use ::zbus::Connection;
use asusd::aura_manager::DeviceManager;
use asusd::config::Config;
use asusd::{print_board_info, DBUS_NAME};
use config_traits::{StdConfig, StdConfigLoad2};
use futures_util::lock::Mutex;
use log::info;
use zbus::fdo::ObjectManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // console_subscriber::init();
    let mut logger = env_logger::Builder::new();
    logger
        .parse_default_env()
        .target(env_logger::Target::Stdout)
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Debug)
        .init();

    let is_service = match env::var_os("IS_SERVICE") {
        Some(val) => val == "1",
        None => true,
    };

    if !is_service {
        println!("asusd schould be only run from the right systemd service");
        println!(
            "do not run in your terminal, if you need an logs please use journalctl -b -u asusd"
        );
        println!("asusd will now exit");
        return Ok(());
    }

    info!("       daemon v{}", asusd::VERSION);
    info!("    rog-anime v{}", rog_anime::VERSION);
    info!("    rog-slash v{}", rog_slash::VERSION);
    info!("     rog-aura v{}", rog_aura::VERSION);
    info!("rog-platform v{}", rog_platform::VERSION);

    start_daemon().await?;
    Ok(())
}

/// The actual main loop for the daemon
async fn start_daemon() -> Result<(), Box<dyn Error>> {
    // let supported = SupportedFunctions::get_supported();
    print_board_info();
    // println!("{:?}", supported.supported_functions());

    // Start zbus server
    let server = Connection::system().await?;
    server.object_server().at("/", ObjectManager).await.unwrap();

    let config = Config::new().load();
    let config = Arc::new(Mutex::new(config));

    let _ = DeviceManager::new(server.clone()).await?;

    // Request dbus name after finishing initalizing all functions
    server.request_name(DBUS_NAME).await?;

    info!("Startup success, begining dbus server loop");
    loop {
        // This is just a blocker to idle and ensure the reator reacts
        server.executor().tick().await;
    }
}
