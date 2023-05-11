#[macro_use]
extern crate log;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use env_logger::Env;

async fn index() -> &'static str {
    info!("Request received");
    "Hello, world!"
}

async fn test() -> &'static str {
    info!("Test request received");
    "Test response"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialise le système de journalisation
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Ajout d'un logger de fichier pour le niveau de log "info" ou supérieur
    let file_appender = log4rs::append::file::FileAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("{d} - {m}\n")))
        .build("log/app.log")
        .unwrap();

    let config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build("file", Box::new(file_appender)))
        .build(log4rs::config::Root::builder().appender("file").build(log::LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();

    // Crée et démarre le serveur Actix Web avec l'option CORS activée
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .service(web::resource("/").to(index))
            .service(web::resource("/test").to(test)) //Ajout de la route "/test"
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
