use std::future::Future;
use crate::pipeline::link::{Error, ErrorFuc, Linkable, Pipeline, Start};

pub struct AsyncConnect<P, F> {
    pub(crate) prev: P,
    pub(crate) next: F,
}

impl<NXT, Fut, F, L> Linkable for AsyncConnect<L, F>
    where F: 'static + Fn(L::OUT) -> Fut + Send + Sync,
          Fut: 'static + Future<Output=NXT> + Send + Sync,
          NXT: Send + Sync,
          L: Linkable + Send + Sync {
    type OUT = NXT;
}

#[async_trait::async_trait]
impl<NXT, Fut, F, P> Pipeline for AsyncConnect<P, F>
    where P: Pipeline + Sync + Send,
          F: Fn(P::OUT) -> Fut + 'static + Send + Sync,
          Fut: Future<Output=NXT> + Send + Sync + 'static,
          NXT: Send + Sync
{
    type IN = P::IN;
    async fn process(self: &Self, input: Self::IN) -> Result<NXT, Error> {
        let out = self.prev.process(input).await?;
        Ok((self.next)(out).await)
    }
}

#[async_trait::async_trait]
impl<NXT, Fut, F, IN> Pipeline for AsyncConnect<Start<IN>, F>
    where F: Fn(IN) -> Fut + 'static + Send + Sync,
          Fut: Future<Output=NXT> + Send + Sync + 'static,
          NXT: Send + Sync,
          IN: Send + Sync
{
    type IN = IN;
    async fn process(self: &Self, input: IN) -> Result<NXT, Error> {
        Ok((self.next)(input).await)
    }
}

impl<NXT, Fut, F, L> Linkable for AsyncConnect<L, ErrorFuc<F>>
    where F: Fn(L::OUT) -> Fut + Send + Sync,
          Fut: Future<Output=Result<NXT, Error>> + Send + Sync + 'static,
          NXT: Send + Sync,
          L: Linkable + Send + Sync {
    type OUT = NXT;
}

#[async_trait::async_trait]
impl<NXT, Fut, F, P> Pipeline for AsyncConnect<P, ErrorFuc<F>>
    where F: Fn(P::OUT) -> Fut + Send + Sync,
          Fut: Future<Output=Result<NXT, Error>> + Send + Sync + 'static,
          NXT: Send + Sync,
          P: Pipeline + Send + Sync {
    type IN = P::IN;
    async fn process(self: &Self, input: Self::IN) -> Result<NXT, Error> {
        let out = self.prev.process(input).await?;
        (self.next.f)(out).await
    }
}

#[async_trait::async_trait]
impl<NXT, Fut, F, IN> Pipeline for AsyncConnect<Start<IN>, ErrorFuc<F>>
    where IN: Sync + Send,
          F: Fn(IN) -> Fut + Sync + Send,
          Fut: Future<Output=Result<NXT, Error>> + Send + Sync + 'static,
          NXT: Send + Sync
{
    type IN = IN;
    async fn process(self: &Self, input: Self::IN) -> Result<NXT, Error> {
        (self.next.f)(input).await
    }
}
