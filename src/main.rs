mod config;
use actix_multipart::Multipart;
use actix_web::*;
use anyhow::{Context, Result};
use chbs::prelude::*;
use chbs::probability::Probability;
use futures::StreamExt;
use std::io::{Read, Write};
use std::path::Path;
use std::str;
use tokio;

const SECRET_TOKEN_HEADER: &'static str = "X-Secret-Token";
const MULTIPART_FIELD_NAME: &'static str = "baste_file";

const BANNER: &'static str = "
██████╗  █████╗ ███████╗████████╗███████╗
██╔══██╗██╔══██╗██╔════╝╚══██╔══╝██╔════╝
██████╔╝███████║███████╗   ██║   █████╗  
██╔══██╗██╔══██║╚════██║   ██║   ██╔══╝  
██████╔╝██║  ██║███████║   ██║   ███████╗
╚═════╝ ╚═╝  ╚═╝╚══════╝   ╚═╝   ╚══════╝
                                         
";

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let cfg = config::Config::load().with_context(|| format!("could not read env"))?;
    ensure_storage_dir(cfg.storage_directory.as_ref().unwrap());

    log::debug!("starting baste with env {:?}", cfg);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(PasteManager {
                secret_token: cfg.secret_token.clone(),
                storage_path: cfg.storage_directory.clone().unwrap(),
            }))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .route("/paste", web::route().guard(guard::Post()).to(paste))
            .service(paste_id)
            .service(root)
    })
    .bind((cfg.address.unwrap(), cfg.port.unwrap()))?
    .run()
    .await?;

    Ok(())
}

#[get("/")]
async fn root() -> String {
    let root_example = include_str!("../README.md");

    let mut full_str = String::new();
    full_str.push_str(BANNER);
    full_str.push_str(root_example);
    return full_str;
}

#[derive(Clone, Debug, Default)]
pub struct PasteManager {
    pub secret_token: String,
    pub storage_path: String,
}

#[get("/{paste_id}")]
async fn paste_id(
    data: web::Data<PasteManager>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let paste_id: String = req.match_info().get("paste_id").unwrap().parse().unwrap();

    let paste_content = match read_paste(&data.storage_path, &paste_id) {
        Ok(content) => content,
        Err(_) => return Err(error::ErrorNotFound("paste not found")),
    };

    Ok(HttpResponse::Ok().body(paste_content))
}

pub async fn paste(
    data: web::Data<PasteManager>,
    mut payload: Multipart,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let secret_token = match req.headers().get(SECRET_TOKEN_HEADER) {
        Some(token) => token,
        None => return Err(error::ErrorBadRequest("missing secret token")),
    };

    if !data.secret_token.eq(secret_token.into()) {
        return Err(error::ErrorBadRequest("wrong token"));
    }

    while let Some(item) = payload.next().await {
        let mut field = item?;
        if !field.name().eq(MULTIPART_FIELD_NAME) {
            continue;
        }

        while let Some(chunk) = field.next().await {
            // just parse the first one
            let phrase = match write_paste((&chunk.unwrap()).to_vec(), &data.storage_path) {
                Ok(phrase) => phrase,
                Err(error) => return Err(error::ErrorInternalServerError(error)),
            };

            return Ok(HttpResponse::Ok().body(phrase));
        }
    }

    Err(error::ErrorBadRequest("no multipart found"))
}

fn phrase() -> String {
    let mut c = chbs::config::BasicConfig::default();
    c.separator = "-".to_owned();
    c.capitalize_words = Probability::Never;
    c.capitalize_first = Probability::Never;

    c.to_scheme().generate()
}

fn write_paste(content: Vec<u8>, base_path: &str) -> Result<String> {
    let base_path = Path::new(base_path);
    let mut fname = phrase();

    let mut full_path = base_path.join(&fname);

    let mut phrase_retries = 0;
    while full_path.exists() {
        fname = phrase();
        full_path = base_path.join(&fname);
        phrase_retries += 1;
    }

    log::debug!(
        "retries done to find a unused paste filename: {}",
        phrase_retries
    );

    let mut file = std::fs::File::create(full_path)?;
    file.write_all(&content)?;

    Ok(fname.clone())
}

fn read_paste(base_path: &str, name: &str) -> Result<Vec<u8>> {
    let base_path = Path::new(base_path);

    let full_path = base_path.join(name);
    let mut file = std::fs::File::open(full_path)?;

    let mut ret = Vec::new();
    file.read_to_end(&mut ret)?;

    Ok(ret)
}

fn ensure_storage_dir(path: &str) {
    if !Path::new(&path).exists() {
        std::fs::create_dir_all(path).unwrap()
    }
}
