#![recursion_limit = "1024"]
use std::{path::PathBuf, sync::Arc};

use actix::Actor;
use actix_web::{get, route, web::{self, Data}, App, HttpServer, Responder};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use actix_cors::Cors;
use actix_web_lab::respond::Html;

mod graphql;
mod native;
mod model;
mod messages;

#[derive(askama::Template)]
#[template(path = "user.html")]
struct UserTemplate<T: std::fmt::Display> {
    name: T,
    text: T,
}

#[derive(askama::Template)]
#[template(path = "index.html")]
struct Index;

/// GraphQL endpoint
#[route("/graphql", method = "GET", method = "POST")]
async fn graphql_e(schema: web::Data<graphql::SrvsSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// GraphiQL playground UI
#[get("/graphiql")]
async fn graphql_playground() -> impl Responder {
    Html::new(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql"),
    ))
}

#[get("/")]
async fn index() -> impl Responder {
    Index
}

#[derive(serde::Deserialize)]
struct UserQ {
    name: String
}

#[get("/user")]
async fn user(web::Query(UserQ { name }): web::Query<UserQ>) -> impl Responder {
    UserTemplate {
        name: name,
        text: "tvoi deistviya?".into()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let router = |app: actix_web::App<_>| {
        app
        .service(index)
        .service(user)
        .service(graphql_e)
        .service(graphql_playground)
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
        .expect("DATA_FOLDER is not path");

    let native = native::Servers::init(srvrs_dir).start();

    let schema = Arc::new(graphql::schema(native));

    log::info!("starting HTTP server on port {port}");
    log::info!("GraphiQL playground: http://localhost:{port}/graphiql");

    HttpServer::new({
        move || {
            let app = App::new()
                .app_data(Data::from(schema.clone()))
                .service(actix_files::Files::new("/static", "./static"));
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