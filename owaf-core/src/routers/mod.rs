use rust_embed::RustEmbed;
use salvo::prelude::*;
use salvo::serve_static::{static_embed, EmbeddedFileExt};

mod demo;

#[derive(RustEmbed)]
#[folder = "assets"]
struct Assets;

pub fn root() -> Router {
    let favicon = Assets::get("favicon.ico")
        .expect("favicon not found")
        .into_handler();
    let router = Router::new()
        .hoop(Logger::new())
        .push(Router::with_path("favicon.ico").get(favicon))
        .push(Router::with_path("{**rest}").goal(demo::hello))
        .push(Router::with_path("assets/{**rest}").get(static_embed::<Assets>()));
    let doc = OpenApi::new("salvo web api", "0.0.1").merge_router(&router);
    router
        .unshift(doc.into_router("/api-doc/openapi.json"))
        .unshift(Scalar::new("/api-doc/openapi.json").into_router("scalar"))
}
