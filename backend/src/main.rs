#![recursion_limit = "1024"]
use std::{fmt::Display, path::PathBuf, sync::Arc, time::Duration};

use actix::Actor;
use actix_web::{get, middleware::{ErrorHandlerResponse, ErrorHandlers}, route, web::{self, Data}, App, HttpServer, Responder};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use actix_cors::Cors;

pub mod graphql;
pub mod native;
pub mod model;
pub mod messages;
pub mod instance;

#[derive(askama::Template)]
#[template(path = "page.html")]
struct Page<C: Display,T: Display, S: Display> {
    chunk: C,
    title: T,
    content: S
}

#[derive(askama::Template)]
#[template(path = "error.html")]
struct ErrorPage<T: Display, M: Display> {
    title: T,
    message: M,
}

/// GraphQL endpoint
#[route("/graphql", method = "GET", method = "POST", method = "HEAD")]
async fn graphql_e(schema: web::Data<graphql::SrvsSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[get("/")]
async fn index() -> impl Responder {
    Page {
        chunk: "index.js",
        title: "Index",
        content: ""
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let router = |app: actix_web::App<_>| {
        app
        .wrap(
            ErrorHandlers::new()
                .default_handler( |r| {
                    let (req,res) = r.into_parts();
                    
                    let error = ErrorPage {
                        title: res.status().as_str().to_owned(),
                        message: "cannot satisfy req"
                    };

                    let res = error.respond_to(&req);

                    let res = actix_web::dev::ServiceResponse::new(req, res)
                        .map_into_boxed_body()
                        .map_into_right_body();

                    Ok(ErrorHandlerResponse::Response(res))
                })
        )
        .service(graphql_e)
        .service(index)
    };

    simple_logger::SimpleLogger::new().env().init().unwrap();

    let port = std::env::var("PORT")
        .expect("no port specified")
        .parse::<u16>()
        .expect("bad port format");
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

    let native = native::Servers::init(srvrs_dir,rcons,ports).expect("cannot init native service").start();

    // the timer for actor
    std::thread::spawn({
        let native = native.clone();
        move || {
            loop {
                native.do_send(messages::Tick);
                std::thread::sleep(Duration::from_secs(3));
            }
        }
    });

    let schema = Arc::new(graphql::schema(native));

    log::info!("starting HTTP server on port {port}");
    log::info!("GraphiQL playground: http://localhost:{port}/graphiql");

    HttpServer::new({
        move || {
            let app = App::new()
                .app_data(Data::from(schema.clone()))
                .service(
                    actix_files::Files::new("/static", "./static")
                        .prefer_utf8(true)
                );
            let app = router(app)
                .wrap(Cors::permissive())
                .wrap(actix_web::middleware::Logger::default());
            app
        }
    })
    .workers(4)
    .bind((addr, port))?
    .run()
    .await
}