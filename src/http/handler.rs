use std::future::Future;

use futures_util::future::ok;
use futures_util::TryStreamExt;
use hyper::{Body, Method, Request, Response, StatusCode};
use serde::{de, Serialize};


use crate::pipeline::connect::Connect;
use crate::pipeline::link;
use crate::pipeline::link::{begin, Linkable, Pipeline, Start};

macro_rules! generate_filter_method {
    ($method:ident) => {
       #[allow(non_snake_case)]
       pub fn $method() -> FilterBase<()> {
            FilterBase {
                method: Method::$method,
                inner: (),
            }
        }
    }
}

type HyperResp = Response<Body>;
type HyperReq = Request<Body>;

generate_filter_method!(GET);
generate_filter_method!(POST);
generate_filter_method!(PUT);

#[async_trait::async_trait]
pub trait Handler: Filter + Send + Sync {
    async fn proc(self: &Self, path: String, body: HyperReq) -> hyper::Result<HyperResp>;
}

pub trait Filter: Select + Send + Sync {
    fn test(self: &Self, path: &str) -> bool;
    fn handle(self: Self) -> EntryBase<Self, Start<(String, HyperReq)>>
        where Self: Sized,
    {
        EntryBase {
            test: self,
            pipeline: begin(),
        }
    }
    fn handle_request(self: Self) -> EntryBase<Self, Connect<Start<(String, HyperReq)>, fn((String, HyperReq)) -> HyperReq>>
        where Self: Sized,
    {
        EntryBase {
            test: self,
            pipeline: begin::<(String, HyperReq)>().then(|(_path, request)| request),
        }
    }
}

pub struct EntryBase<T, P> {
    test: T,
    pipeline: P,
}

impl<T, P> Select for EntryBase<T, P> where P: Send + Sync, T: Filter {
    fn method(self: &Self) -> Method {
        self.test.method()
    }
}

impl<T, P> Filter for EntryBase<T, P> where T: Filter, P: Sync + Send {
    fn test(self: &Self, path: &str) -> bool {
        self.test.test(path)
    }
}

// we need return <impl Pipeline>, and know IN. so we have to define EntryBase<..,Start> specially
impl<T, IN> EntryBase<T, Start<IN>> {
    pub fn then_async<NXT, F, Fut>(self: Self, f: F) -> EntryBase<T, impl Pipeline<IN=IN, OUT=NXT>>
        where F: Fn(IN) -> Fut + 'static + Send + Sync,
              Fut: Future<Output=NXT> + Send + Sync + 'static,
              IN: Send + Sync,
              NXT: Send + Sync,
              Self: Sized {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then_async(f),
        }
    }
    pub fn then_async_result<NXT, F, Fut>(self: Self, f: F) -> EntryBase<T, impl Pipeline<IN=IN, OUT=NXT>>
        where F: Fn(IN) -> Fut + 'static + Send + Sync,
              Fut: Future<Output=Result<NXT, link::Error>> + Send + Sync + 'static,
              IN: Send + Sync,
              NXT: Send + Sync,
              Self: Sized {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then_async_result(f),
        }
    }
    pub fn then<NXT, F>(self: Self, f: F) -> EntryBase<T, impl Pipeline<IN=IN, OUT=NXT>>
        where F: Fn(IN) -> NXT + Send + Sync,
              NXT: Send + Sync,
              IN: Send + Sync,
              Self: Sized {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then(f),
        }
    }
    pub fn then_result<NXT, F>(self: Self, f: F) -> EntryBase<T, impl Pipeline<IN=IN, OUT=NXT>>
        where F: Fn(IN) -> Result<NXT, link::Error> + Send + Sync,
              NXT: Send + Sync,
              IN: Send + Sync,
              Self: Sized {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then_result(f),
        }
    }
}

impl<T, P> EntryBase<T, P>
    where P: Pipeline + Sync + Send {
    pub fn then_async<NXT, F, Fut>(self: Self, f: F) -> EntryBase<T, impl Pipeline<IN=P::IN, OUT=NXT>>
        where F: Fn(P::OUT) -> Fut + 'static + Send + Sync,
              Fut: Future<Output=NXT> + Send + Sync + 'static,
              NXT: Send + Sync,
              Self: Sized {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then_async(f),
        }
    }
    pub fn then_async_result<NXT, F, Fut>(self: Self, f: F) -> EntryBase<T, impl Pipeline<IN=P::IN, OUT=NXT>>
        where F: Fn(P::OUT) -> Fut + 'static + Send + Sync,
              Fut: Future<Output=Result<NXT, link::Error>> + Send + Sync + 'static,
              NXT: Send + Sync,
              Self: Sized {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then_async_result(f),
        }
    }
    pub fn then<NXT, F>(self: Self, f: F) -> EntryBase<T, impl Pipeline<IN=P::IN, OUT=NXT>>
        where F: Fn(P::OUT) -> NXT + Send + Sync,
              NXT: Send + Sync,
              Self: Sized {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then(f),
        }
    }
    pub fn then_result<NXT, F>(self: Self, f: F) -> EntryBase<T, impl Pipeline<IN=P::IN, OUT=NXT>>
        where F: Fn(P::OUT) -> Result<NXT, link::Error> + Send + Sync,
              NXT: Send + Sync,
              Self: Sized {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then_result(f),
        }
    }
}


#[async_trait::async_trait]
impl<T, P> Handler for EntryBase<T, P> where
    T: Filter,
    P: Pipeline<IN=(String, HyperReq), OUT=HyperResp> + Sync + Send, {
    async fn proc(self: &Self, path: String, body: HyperReq) -> hyper::Result<HyperResp> {
        match self.pipeline.process((path, body)).await {
            Ok(t) => Ok(t),
            Err(err) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from(err.to_string())).unwrap())
        }
    }
}

impl<T, P> EntryBase<T, P> where
    T: Filter,
    P: Pipeline + Sync + Send {
    /**
     * before the process
     **/
    pub fn parse_json<NXT>(self: Self) -> EntryBase<T, impl Pipeline<IN=P::IN, OUT=NXT>>
        where NXT: de::DeserializeOwned + Send + Sync,
              P: Pipeline<OUT=HyperReq>
    {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then_async_result(|req| async {
                let mut body = Vec::new();
                req.into_body()
                    .try_for_each(|bytes| {
                        body.extend(bytes);
                        ok(())
                    }).await?;
                Ok(serde_json::from_slice::<NXT>(&body)?)
            }),
        }
    }

    /**
     *  after the process
     **/
    pub fn ok_with_msg(self: Self, msg: &'static str) -> EntryBase<T, impl Pipeline<IN=P::IN, OUT=HyperResp>> {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then(move |_| {
                Response::builder()
                    .status(200).body(Body::from(msg)).unwrap()
            }),
        }
    }

    pub fn ok(self: Self) -> EntryBase<T, impl Pipeline<IN=P::IN, OUT=HyperResp>>
        where P::OUT: Into<Body> {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then(|msg| {
                Response::builder()
                    .status(200).body(msg.into()).unwrap()
            }),
        }
    }

    pub fn ret<IN>(self: Self) -> EntryBase<T, impl Pipeline<IN=P::IN, OUT=HyperResp>>
        where P: Pipeline<OUT=Result<IN, link::Error>>,
              IN: Into<Body>,
    {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then(|result| -> HyperResp {
                match result {
                    Ok(msg) => Response::builder()
                        .status(StatusCode::OK).body(msg.into()).unwrap(),
                    Err(err) => Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from(err.to_string())).unwrap()
                }
            }),
        }
    }

    pub fn to_json(self: Self) -> EntryBase<T, impl Pipeline<IN=P::IN, OUT=HyperResp>>
        where P::OUT: Serialize + 'static
    {
        EntryBase {
            test: self.test,
            pipeline: self.pipeline.then_async_result(|obj| async move {
                let s = serde_json::to_string(&obj)?;
                Ok(Response::new(Body::from(s)))
            }),
        }
    }
}


pub struct FilterBase<T> {
    method: Method,
    inner: T,
}

impl<T> Select for FilterBase<T> {
    fn method(self: &Self) -> Method {
        self.method.clone()
    }
}

impl<T> Filter for FilterBase<T> where T: Fn(&str) -> bool + Sync + Send {
    fn test(self: &Self, path: &str) -> bool {
        (self.inner)(path)
    }
}

impl<T> FilterBase<T> where T: Fn(&str) -> bool {}

impl FilterBase<()> {
    pub fn start_with(self: Self, prefix: &'static str) -> impl Filter {
        FilterBase {
            method: self.method,
            inner: move |path: &str| { path.starts_with(prefix) },
        }
    }

    pub fn eq(self: Self, prefix: &'static str) -> impl Filter {
        FilterBase {
            method: self.method,
            inner: move |path: &str| { path.eq(prefix) },
        }
    }
}

pub trait Select {
    fn method(self: &Self) -> Method;
}



#[tokio::test]
async fn test_router() {
    use std::time::Duration;
    use tokio::time::sleep;
    use crate::http::router::Router;
    async fn pf(tuple: (String, HyperReq)) -> Result<HyperResp, link::Error> {
        let (_, req) = tuple;
        let mut body = Vec::new();
        req.into_body()
            .try_for_each(|bytes| {
                body.extend(bytes);
                ok(())
            })
            .await?;
        let mut sufix = Vec::from("!");
        body.append(&mut sufix);
        Ok(Response::builder()
            .status(200).body(Body::from(body)).unwrap())
    }
    let a = GET().start_with("hello").handle()
        .then_async_result(pf)
        .then(|x| x)
        .then_async(|r| async move {
            sleep(Duration::from_secs(1)).await;
            r
        });
    assert!(a.test("hello world"));
    assert_eq!(a.method(), Method::GET);
    let res = a.proc("aa".to_string(), Request::new(Body::from("aaa"))).await;
    let whole_body = hyper::body::to_bytes(res.unwrap().into_body()).await.unwrap();
    println!("ddd: {}", String::from_utf8(whole_body.to_vec()).unwrap());

    let mut r = Router::new();
    r.add(a);
    let res2 = r.process(Method::GET, "hello world".to_string(), Request::new(Body::from("ccc"))).await;
    let whole_body = hyper::body::to_bytes(res2.unwrap().into_body()).await.unwrap();
    println!("ddd: {}", String::from_utf8(whole_body.to_vec()).unwrap());
}


#[tokio::test]
async fn test_ok() {
    {
        let a = GET().start_with("hello").handle_request().ok_with_msg("");
        assert!(a.test("hello world"));
        assert_eq!(a.method(), Method::GET);
        let res = a.proc("aa".to_string(), Request::new(Body::from("aaa"))).await;
        let whole_body = hyper::body::to_bytes(res.unwrap().into_body()).await.unwrap();
        println!("ddd: {}", String::from_utf8(whole_body.to_vec()).unwrap());
    }
    {
        let a = GET().start_with("hello")
            .handle_request()
            .then(|_| "message")
            .ok();
        assert!(a.test("hello world"));
        assert_eq!(a.method(), Method::GET);
        let res = a.proc("aa".to_string(), Request::new(Body::from("aaa"))).await;
        let whole_body = hyper::body::to_bytes(res.unwrap().into_body()).await.unwrap();
        println!("ddd: {}", String::from_utf8(whole_body.to_vec()).unwrap());
    }
}

#[tokio::test]
async fn test_parse_json() {
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    struct Fire {
        a: i32,
        b: String,
    }

    #[derive(Serialize)]
    struct Hole {
        c: i32,
        d: String,
    }

    let h = GET().eq("hello").handle_request()
        .parse_json()
        .then(|f: Fire| {
            println!("{} {}", f.a, f.b);
            Hole {
                c: f.a,
                d: f.b,
            }
        }).to_json();

    let f = Fire {
        a: 10,
        b: "123".to_string(),
    };
    let fjson = serde_json::to_string(&f).unwrap();
    let res = h.proc("hello".to_string(), Request::new(Body::from(fjson))).await;
    let whole_body = hyper::body::to_bytes(res.unwrap().into_body()).await.unwrap();
    println!("ddd: {}", String::from_utf8(whole_body.to_vec()).unwrap());
}