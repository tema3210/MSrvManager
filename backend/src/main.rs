#![recursion_limit = "1024"]
use std::{fmt::Display, path::PathBuf, sync::Arc, time::Duration};

use actix::Actor;

use actix_web::{get, guard, middleware::{ErrorHandlerResponse, ErrorHandlers}, route, web::{self, Data}, App, HttpRequest, HttpResponse, HttpServer, Responder};
use askama::Template;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use actix_cors::Cors;

pub mod graphql;
pub mod native;
pub mod model;
pub mod messages;
pub mod instance;
pub mod rcon;
pub mod utils;

#[derive(serde::Deserialize)]
struct IdParams {
    name: String,
}

#[derive(askama::Template)]
#[template(path = "page.html")]
struct Page<C: Display,T: Display> {
    chunk: C,
    title: T,
    content: serde_json::Value
}

#[route("/graphql", method = "GET", method = "HEAD", method = "POST")]
async fn graphql_e(
    schema: web::Data<graphql::SrvsSchema>, 
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_ws(
    schema: web::Data<graphql::SrvsSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> actix_web::Result<HttpResponse> {
    GraphQLSubscription::new(graphql::SrvsSchema::clone(&*schema))
        .start(&req, payload)
        .map_err(|err| {
            log::error!("WebSocket subscription error: {}", err);
            actix_web::error::ErrorInternalServerError(err)
        })
}

#[get("/")]
async fn index() -> impl Responder {
    Page {
        chunk: "index.js",
        title: "Servers",
        content: serde_json::json!({})
    }
}

#[get("/create")]
async fn create() -> impl Responder {
    Page {
        chunk: "create.js",
        title: "Create server",
        content: serde_json::json!({})
    }
}

#[get("/rcon")]
async fn command(info: web::Query<IdParams>) -> impl Responder {
    Page {
        chunk: "rcon.js",
        title: "RCON",
        content: serde_json::json!({
            "name": info.name
        })
    }
}

#[get("/alter")]
async fn alter(info: web::Query<IdParams>) -> impl Responder {
    Page {
        chunk: "alter.js",
        title: format!("Alter server {}",&info.name),
        content: serde_json::json!({
            "name": info.name
        })
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let router = |app: actix_web::App<_>| {
        app
        .wrap(
            ErrorHandlers::new()
                .default_handler(|r| {
                    let (req,res) = r.into_parts();
                    
                    let error = Page {
                        title: res.status().as_str().to_owned(),
                        chunk: "error.js",
                        content: serde_json::json!({
                            "msg": "Error has occured",
                            "color": "red",
                            "fontSize": "2em",
                            "fontStyle": "italic",
                            "title": res.status().canonical_reason().unwrap_or("Unknown error")
                        })
                    };

                    let res = match error.render() {
                        Ok(body) => HttpResponse::build(res.status()).body(body),
                        Err(_) => HttpResponse::InternalServerError().finish(),
                    };
    
                    let res = actix_web::dev::ServiceResponse::new(req, res)
                        .map_into_boxed_body()
                        .map_into_right_body();
    
                    Ok(ErrorHandlerResponse::Response(res))
                })
        )
        .service(
            web::resource("/graphql_ws")
                .guard(guard::Get())
                .guard(guard::Header("upgrade", "websocket"))
                .to(graphql_ws),
        )
        .service(graphql_e)
        .service(index)
        .service(create)
        .service(alter)
        .service(command)
    };

    simple_logger::SimpleLogger::new().env().init().unwrap();

    let port = std::env::var("PORT")
        .expect("no port specified")
        .parse::<u16>()
        .expect("bad port format");

    let password = std::env::var("PASSWORD")
        .expect("no password specified");

    let addr = std::env::var("ADDR")
        .expect("no addr specified")
        .parse::<std::net::Ipv4Addr>()
        .expect("bad addr format");

    let srvrs_dir = std::env::var("DATA_FOLDER")
        .expect("no DATA_FOLDER specified")
        .parse::<PathBuf>()
        .expect("DATA_FOLDER is not a path");

    let rcons = match std::env::var("RCON_RANGE").expect("has to have rcon range").split('.').filter(|s| !s.is_empty()).collect::<Vec<_>>()[..] {
        [l,r] => {
            let l = l.parse().expect("bad rcon left bound");
            let r = r.parse().expect("bad rcon right bound");
            l..r
        },
        _ => panic!("bad RCON_RANGE format")
    };

    let ports = match std::env::var("PORT_RANGE").expect("has to have port range").split('.').filter(|s| !s.is_empty()).collect::<Vec<_>>()[..] {
        [l,r] => {
            let l = l.parse().expect("bad port left bound");
            let r = r.parse().expect("bad port right bound");
            l..r
        },
        _ => panic!("bad PORT_RANGE format")
    };

    let timeout = std::env::var("TIMEOUT")
        .expect("no timeout specified")
        .parse::<u64>()
        .expect("bad timeout format");

    let timeout = Duration::from_secs(timeout);

    let native = native::Servers::new(srvrs_dir,rcons,ports,timeout,password.clone()).start();

    let native_clone = native.clone();
    
    std::thread::spawn(move || {
        let interval = std::time::Duration::from_secs(4) + std::time::Duration::from_millis(500);
        loop {
            std::thread::sleep(interval);
            native_clone.do_send(messages::Tick);
        }
    });

    let schema = Arc::new(graphql::schema(native,password));

    log::info!("starting HTTP server on port {port}");

    HttpServer::new({
        move || {
            let app = App::new()
                .app_data(Data::from(schema.clone()))
                .service(
                    actix_files::Files::new("/static", "./static")
                        .prefer_utf8(true)
                );
            let app = router(app)
                .wrap(Cors::permissive());
                // .wrap(actix_web::middleware::Logger::default());
            app
        }
    })
    .workers(4)
    .bind((addr, port))?
    .run()
    .await
}