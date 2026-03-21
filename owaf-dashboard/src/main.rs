use rinja::Template;
use salvo::prelude::*;
use salvo::serve_static::StaticDir;

#[derive(Template)]
#[template(path = "log_list_page.html")]
struct LogListPage {}

#[handler]
pub async fn render_logs(res: &mut Response) {
    let tpl = LogListPage {};
    res.render(Text::Html(tpl.render().unwrap()));
}

#[derive(Template)]
#[template(path = "proxy_config_page.html")]
struct ProxyConfigPage {}

#[handler]
pub async fn render_proxy_config(res: &mut Response) {
    let tpl = ProxyConfigPage {};
    res.render(Text::Html(tpl.render().unwrap()));
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let router = Router::new()
        .push(Router::with_path("dashboard/logs").get(render_logs))
        .push(Router::with_path("dashboard/proxy-config").get(render_proxy_config))
        .push(
            Router::with_path("assets/<**path>").get(
                StaticDir::new(vec!["owaf-core/assets", "../owaf-core/assets"])
                    .defaults("index.html")
                    .auto_list(true),
            ),
        );

    let listen_addr = "127.0.0.1:8010";
    tracing::info!("owaf-dashboard listening on http://{}", listen_addr);
    let acceptor = TcpListener::new(listen_addr).bind().await;
    Server::new(acceptor).serve(router).await;
}
