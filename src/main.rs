mod api;
mod schema;

use std::io;
use std::sync::Arc;

use diesel::r2d2;
use diesel::prelude::{Connection, PgConnection};
use diesel::r2d2::ConnectionManager;
use ntex::web;
use ntex_files;
use ntex_files::NamedFile;


async fn index() -> io::Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}

async fn js() -> io::Result<NamedFile> {
    Ok(NamedFile::open("static/index.js")?)
}

async fn css() -> io::Result<NamedFile> {
    Ok(NamedFile::open("static/styles.css")?)
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Arc::new(r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool"));

    web::HttpServer::new(move || {
        web::App::new()
            .state(pool.clone())
            .service(ntex_files::Files::new("static", ".").show_files_listing())
            .route("/", web::get().to(index))
            .route("/index.js", web::get().to(js))
            .route("/styles.css", web::get().to(css))
            .service(web::scope("/api").configure(api::ntex_config))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
