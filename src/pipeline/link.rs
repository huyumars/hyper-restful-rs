use std::future::Future;
use std::marker::PhantomData;
use crate::pipeline::async_connect::AsyncConnect;
use crate::pipeline::connect::Connect;

pub type Error = Box<dyn std::error::Error>;

pub trait Linkable {
    type OUT: Send + Sync;
    fn then_async<F, FUT, NXT>(self: Self, f: F) -> AsyncConnect<Self, F>
        where F: Fn(Self::OUT) -> FUT,
              FUT: Future<Output=NXT> + Send + Sync,
              Self: Sized {
        AsyncConnect {
            prev: self,
            next: f,
        }
    }

    fn then_async_result<F, FUT, NXT>(self: Self, f: F) -> AsyncConnect<Self, ErrorFuc<F>>
        where F: Fn(Self::OUT) -> FUT,
              FUT: Future<Output=Result<NXT, Error>> + Send + Sync,
              Self: Sized {
        AsyncConnect {
            prev: self,
            next: ErrorFuc::new(f),
        }
    }

    fn then<F, NXT>(self: Self, f: F) -> Connect<Self, F>
        where F: Fn(Self::OUT) -> NXT,
              Self: Sized {
        Connect {
            prev: self,
            next: f,
        }
    }

    fn then_result<F, NXT>(self: Self, f: F) -> Connect<Self, ErrorFuc<F>>
        where F: Fn(Self::OUT) -> Result<NXT, Error>,
              Self: Sized {
        Connect {
            prev: self,
            next: ErrorFuc::new(f),
        }
    }
}

#[async_trait::async_trait]
pub trait Pipeline: Linkable {
    type IN: Send + Sync;
    // todo return Result
    async fn process(self: &Self, input: Self::IN) -> Result<Self::OUT, Error>;
}

pub fn begin<T>() -> Start<T> {
    return Start {
        _p: Default::default()
    };
}


pub struct Start<T> {
    _p: PhantomData<T>,
}

impl<IN: Send + Sync> Linkable for Start<IN> {
    type OUT = IN;
}

pub struct ErrorFuc<F> {
    pub f: F,
}

impl<F> ErrorFuc<F> {
    fn new(f: F) -> Self {
        ErrorFuc {
            f
        }
    }
}