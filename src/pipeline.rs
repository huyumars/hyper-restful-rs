pub mod link;
pub mod test;
pub mod connect;
pub mod async_connect;

// use std::future::Future;
// use std::marker::PhantomData;
// use std::sync::Arc;
// use std::time::Duration;
//
// pub trait Linkable {
//     type OUT: Send + Sync;
//     fn then_async<F, NXT>(self: Self, f: F) -> AsyncConnect<Self, F>
//         where F: Fn(Self::OUT) -> NXT,
//               Self: Sized {
//         AsyncConnect {
//             prev: self,
//             next: f,
//         }
//     }
//     fn then<F, NXT>(self: Self, f: F) -> Connect<Self, F>
//         where F: Fn(Self::OUT) -> NXT,
//               Self: Sized {
//         Connect {
//             prev: self,
//             next: f,
//         }
//     }
// }
//
// #[async_trait::async_trait]
// pub trait Pipeline: Linkable {
//     type IN: Send + Sync;
//     // todo return Result
//     async fn process(self: &Self, input: Self::IN) -> Self::OUT;
// }
//
// pub fn begin<T>() -> Start<T> {
//     return Start {
//         _p: Default::default()
//     };
// }
//
// pub struct Start<T> {
//     _p: PhantomData<T>,
// }
//
// impl<IN: Send + Sync> Linkable for Start<IN> {
//     type OUT = IN;
// }
//
// pub struct AsyncConnect<P, F> {
//     prev: P,
//     next: F,
// }
//
// impl<NXT, Fut, F, L> Linkable for AsyncConnect<L, F>
//     where F: 'static + Fn(L::OUT) -> Fut + Send + Sync,
//           Fut: 'static + Future<Output=NXT> + Send + Sync,
//           NXT: Send + Sync,
//           L: Linkable + Send + Sync {
//     type OUT = NXT;
// }
//
// #[async_trait::async_trait]
// impl<NXT, Fut, F, P> Pipeline for AsyncConnect<P, F>
//     where P: Pipeline + Sync + Send,
//           F: Fn(P::OUT) -> Fut + 'static + Send + Sync,
//           Fut: Future<Output=NXT> + Send + Sync + 'static,
//           NXT: Send + Sync
// {
//     type IN = P::IN;
//     async fn process(self: &Self, input: Self::IN) -> NXT {
//         let out = self.prev.process(input).await;
//         (self.next)(out).await
//     }
// }
//
// #[async_trait::async_trait]
// impl<NXT, Fut, F, IN> Pipeline for AsyncConnect<Start<IN>, F>
//     where F: Fn(IN) -> Fut + 'static + Send + Sync,
//           Fut: Future<Output=NXT> + Send + Sync + 'static,
//           NXT: Send + Sync,
//           IN: Send + Sync
// {
//     type IN = IN;
//     async fn process(self: &Self, input: IN) -> NXT {
//         (self.next)(input).await
//     }
// }
//
// pub struct Connect<P, F> {
//     prev: P,
//     next: F,
// }
//
// impl<NXT, F, L> Linkable for Connect<L, F>
//     where F: Fn(L::OUT) -> NXT + Send + Sync,
//           NXT: Send + Sync,
//           L: Linkable + Send + Sync {
//     type OUT = NXT;
// }
//
// #[async_trait::async_trait]
// impl<NXT, F, P> Pipeline for Connect<P, F>
//     where P: Pipeline + Sync + Send,
//           F: Fn(P::OUT) -> NXT + Sync + Send,
//           NXT: Send + Sync
// {
//     type IN = P::IN;
//     async fn process(self: &Self, input: Self::IN) -> NXT {
//         let out = self.prev.process(input).await;
//         (self.next)(out)
//     }
// }
//
// #[async_trait::async_trait]
// impl<NXT, F, IN> Pipeline for Connect<Start<IN>, F>
//     where F: Fn(IN) -> NXT + Send + Sync,
//           NXT: Send + Sync,
//           IN: Send + Sync
// {
//     type IN = IN;
//     async fn process(self: &Self, input: IN) -> NXT {
//         (self.next)(input)
//     }
// }
//
// #[tokio::test]
// async fn test_pipeline() {
//     use tokio::time::sleep;
//     async fn async_func<T>(v: Vec<T>) -> Vec<T> {
//         v
//     }
//     fn func<T>(v: Vec<T>) -> Vec<T> {
//         v
//     }
//     let b = begin().then_async(|i: i32| async move { i.to_string() });
//     let pipeline = b.then(|s| s.into_bytes())
//         .then(func)
//         .then_async(async_func)
//         .then_async(|mut v| async move {
//             sleep(Duration::from_secs(1)).await;
//             v.reverse();
//             v
//         }).then(|v| String::from_utf8(v))
//         .then(|r| r.unwrap());
//     let pr = Arc::new(pipeline);
//     let mut jhs = Vec::new();
//     for i in 0..1000 {
//         let prc = pr.clone();
//         jhs.push(tokio::spawn(async move {
//             let s = prc.process(i).await;
//             let t = i.to_string().chars().rev().collect::<String>();
//             assert_eq!(s, t);
//         }))
//     }
//     for jh in jhs {
//         let _guard = jh.await;
//     }
// }
//
