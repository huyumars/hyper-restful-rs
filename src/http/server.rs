use std::convert::Infallible;
/**
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 * <p>
 * http://www.apache.org/licenses/LICENSE-2.0
 * <p>
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */


use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use futures_util::TryFutureExt;
use hyper::server::conn::AddrIncoming;
use hyper::server::Builder as HyperBuilder;
use hyper::service::{make_service_fn, service_fn};
use hyper::{self, Body, Method, Request, Response, Server};
use log::{info, warn};

use tokio::runtime::{Builder, Runtime};
use tokio::sync::oneshot::{Receiver, Sender};

use crate::http::router::Router;

pub struct HttpServer {
    thread_pool: Runtime,
    tx: Sender<()>,
    rx: Option<Receiver<()>>,
    addr: Option<SocketAddr>,
    router: Arc<Router>,
}

impl HttpServer {
    pub fn new(name: String, router: Router, status_thread_pool_size: usize) -> Result<Self, std::io::Error> {
        let start_name = name.clone();
        let end_name = name.clone();
        let thread_pool = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(status_thread_pool_size)
            .thread_name(name)
            .on_thread_start(move || {
                info!("{} started", start_name);
            })
            .on_thread_stop(move || {
                info!("stopping {}", end_name);
            })
            .build()
            .unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        Ok(HttpServer {
            thread_pool,
            tx,
            rx: Some(rx),
            addr: None,
            router: Arc::new(router),
        })
    }


    pub fn stop(self) {
        let _ = self.tx.send(());
        self.thread_pool.shutdown_timeout(Duration::from_secs(10));
    }

    // Return listening address, this may only be used for outer test
    // to get the real address because we may use "127.0.0.1:0"
    // in test to avoid port conflict.
    pub fn listening_addr(&self) -> SocketAddr {
        self.addr.unwrap()
    }

    async fn handle_request(router: Arc<Router>, method: Method, path: String, req: Request<Body>)
                            -> hyper::Result<Response<Body>> {
        router.process(method, path, req).await
    }

    fn start_serve(&mut self, builder: HyperBuilder<AddrIncoming>) {
        // Start to serve.
        let router_out = self.router.clone();
        let make_service = make_service_fn(move |_conn| {
            let router = router_out.clone();
            async move {
                // This is the request handler.
                Ok::<_, Infallible>(service_fn(move |req| {
                    let path = req.uri().path().to_owned();
                    let method = req.method().to_owned();
                    let r = router.clone();
                    Self::handle_request(r, method, path, req)
                }))
            }
        });
        let server = builder.serve(make_service);

        let rx = self.rx.take().unwrap();
        let graceful = server
            .with_graceful_shutdown(async move {
                let _ = rx.await;
            })
            .map_err(|e| warn!("Status server error: {:?}", e));
        self.thread_pool.spawn(graceful);
    }

    pub fn start(&mut self, status_addr: String) -> Result<(), std::io::Error> {
        let addr = SocketAddr::from_str(&status_addr).unwrap();

        let incoming = {
            let _enter = self.thread_pool.enter();
            AddrIncoming::bind(&addr)
        }
            .unwrap();
        self.addr = Some(incoming.local_addr());

        let server = Server::builder(incoming);
        self.start_serve(server);
        Ok(())
    }
}


// use tokio::time::sleep;
// use crate::http::handler::{GET, POST};
// use crate::http::handler::Filter;
// #[test]
// fn service_test() {
//     let mut r1 = Router::new();
//     let b = GET()
//         .start_with("/hello")
//         .handle_request()
//         .then(|_| "world")
//         .ok();
//     let mut r2 = Router::new();
//     let c = GET()
//         .start_with("/ggg")
//         .handle_request()
//         .then(|_| "lalala")
//         .ok();
//     r1.add(b);
//     r2.add(c);
//     r1 = r1.merge(r2);
//     let mut s = HttpServer::new("my_service".to_string(), r1, 1).unwrap();
//     s.start("127.0.0.1:8080".to_string()).unwrap();
//     let _ = sleep(Duration::from_secs(1));
//     s.stop();
// }
//
// #[test]
// fn json_test() {
//     use serde::{Serialize, Deserialize};
//     #[derive(Serialize, Deserialize)]
//     struct Req {
//         seq: i32,
//         msg: String,
//     }
//
//     #[derive(Serialize)]
//     struct Res {
//         seq: i32,
//         msg: String,
//     }
//     let mut r = Router::new();
//     r.add(
//         POST()
//             .start_with("/reverse")
//             .handle_request()
//             .parse_json()
//             .then_async(|rr: std::result::Result<Req, serde_json::Error>| async move {
//                 let r = rr.unwrap();
//                 Res {
//                     seq: r.seq + 1,
//                     msg: format!("{}!", r.msg),
//                 }
//             })
//             .to_json()
//     );
//     let mut s = HttpServer::new("my_service".to_string(), r, 1).unwrap();
//     s.start("127.0.0.1:8080".to_string()).unwrap();
//     let _ = sleep(Duration::from_secs(1000000));
//     s.stop();
// }