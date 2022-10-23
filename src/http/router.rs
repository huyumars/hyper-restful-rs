use std::collections::HashMap;
use hyper::{Body, Method, Request, Response, StatusCode};
use crate::http::handler::Handler;

pub struct Router {
    entries: HashMap<Method, Vec<Box<dyn Handler>>>,
}


impl Router {
    pub fn new() -> Self {
        Router {
            entries: HashMap::new()
        }
    }

    pub(crate) fn err_response<T>(status_code: StatusCode, message: T) -> Response<Body>
        where
            T: Into<Body>,
    {
        Response::builder()
            .status(status_code)
            .body(message.into())
            .unwrap()
    }

    pub fn add(self: &mut Self, handler: impl Handler + 'static) {
        let method = handler.method();
        let handler_box = Box::new(handler);
        match self.entries.get_mut(&method) {
            None => {
                self.entries.insert(method, vec![handler_box]);
            }
            Some(l) => {
                l.push(handler_box);
            }
        }
    }

    pub fn merge(mut self: Self, other: Router) -> Self {
        for (method, handle)in other.entries {
           match self.entries.get_mut(&method) {
               None => {
                   self.entries.insert(method, handle);
               }
               Some(exist) => {
                   exist.extend(handle);
               }
           }
        }
        self
    }

    pub async fn process(self: &Self, method: Method, path: String, body: Request<Body>) -> hyper::Result<Response<Body>> {
        match self.entries.get(&method) {
            None => Ok(Self::err_response(
                StatusCode::METHOD_NOT_ALLOWED,
                "method not allowed",
            )),
            Some(list_of_processor) => {
                match list_of_processor.into_iter().filter(|p| p.test(&path)).nth(0) {
                    None => Ok(Self::err_response(
                        StatusCode::NOT_FOUND,
                        "not found",
                    )),
                    Some(processor) => {
                        match processor.proc(path, body).await {
                            Ok(t) => Ok(t),
                            Err(e) => Ok(Self::err_response(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                e.to_string()
                            ))
                        }
                    }
                }
            }
        }
    }
}