use crate::pipeline::link::{Error, ErrorFuc, Linkable, Pipeline, Start};

pub struct Connect<P, F> {
    pub(crate) prev: P,
    pub(crate) next: F,
}

impl<NXT, F, L> Linkable for Connect<L, F>
    where F: Fn(L::OUT) -> NXT + Send + Sync,
          NXT: Send + Sync,
          L: Linkable + Send + Sync {
    type OUT = NXT;
}

#[async_trait::async_trait]
impl<NXT, F, P> Pipeline for Connect<P, F>
    where P: Pipeline + Sync + Send,
          F: Fn(P::OUT) -> NXT + Sync + Send,
          NXT: Send + Sync
{
    type IN = P::IN;
    async fn process(self: &Self, input: Self::IN) -> Result<NXT, Error> {
        let out = self.prev.process(input).await?;
        Ok((self.next)(out))
    }
}


impl<NXT, F, P> Linkable for Connect<P, ErrorFuc<F>>
    where F: Fn(P::OUT) -> Result<NXT, Error> + Send + Sync,
          NXT: Send + Sync,
          P: Linkable + Send + Sync {
    type OUT = NXT;
}

#[async_trait::async_trait]
impl<NXT, F, P> Pipeline for Connect<P, ErrorFuc<F>>
    where P: Pipeline + Sync + Send,
          F: Fn(P::OUT) -> Result<NXT, Error> + Sync + Send,
          NXT: Send + Sync
{
    type IN = P::IN;
    async fn process(self: &Self, input: Self::IN) -> Result<NXT, Error> {
        let out = self.prev.process(input).await?;
        (self.next.f)(out)
    }
}

#[async_trait::async_trait]
impl<NXT, F, IN> Pipeline for Connect<Start<IN>, ErrorFuc<F>>
    where IN: Sync + Send,
          F: Fn(IN) -> Result<NXT, Error> + Sync + Send,
          NXT: Send + Sync
{
    type IN = IN;
    async fn process(self: &Self, input: Self::IN) -> Result<NXT, Error> {
        (self.next.f)(input)
    }
}

#[async_trait::async_trait]
impl<NXT, F, IN> Pipeline for Connect<Start<IN>, F>
    where F: Fn(IN) -> NXT + Send + Sync,
          NXT: Send + Sync,
          IN: Send + Sync
{
    type IN = IN;
    async fn process(self: &Self, input: IN) -> Result<NXT, Error> {
        Ok((self.next)(input))
    }
}