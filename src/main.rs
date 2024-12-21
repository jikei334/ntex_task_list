mod api;

use std::io;
use ntex::web;
use ntex_files;
use ntex_files::NamedFile;


async fn index() -> io::Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}

async fn js() -> io::Result<NamedFile> {
    Ok(NamedFile::open("static/index.js")?)
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    web::HttpServer::new(|| {
        web::App::new()
            .service(ntex_files::Files::new("static", ".").show_files_listing())
            .route("/", web::get().to(index))
            .route("/index.js", web::get().to(js))
            .service(web::scope("/api").configure(api::ntex_config))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
