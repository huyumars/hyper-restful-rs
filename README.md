### example
start a service serve for process json
```rust
use std::time::Duration;
use serde::{Serialize, Deserialize};
use tokio::time::sleep;
use hyper_restful_rs::http::handler::POST;
use hyper_restful_rs::http::router::Router;
use hyper_restful_rs::http::server::HttpServer;
use hyper_restful_rs::http::handler::Filter;

#[derive(Serialize, Deserialize)]
struct Req {
    seq: i32,
    msg: String,
}

#[derive(Serialize)]
struct Res {
    seq: i32,
    msg: String,
}



#[tokio::main]
async fn main() {
    let mut r = Router::new();
    r.add(
        POST()
            .start_with("/reverse")
            .handle_request()
            .parse_json()
            .then_async(|r: Req| async move {
                Res {
                    seq: r.seq + 1,
                    msg: format!("{}!", r.msg),
                }
            })
            .to_json()
    );
    let mut s = HttpServer::new("my_service".to_string(), r, 1).unwrap();
    s.start("127.0.0.1:8080".to_string()).unwrap();
    loop {
        sleep(Duration::from_secs(10)).await
    }
}
```