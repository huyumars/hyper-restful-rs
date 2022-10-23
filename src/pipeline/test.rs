use crate::pipeline::link::{begin, Linkable, Pipeline};

#[tokio::test]
async fn test_pipeline() {
    use std::time::Duration;
    use tokio::time::sleep;
    use log;
    use crate::log::simple_logger;
    simple_logger::init().unwrap();

    let p = begin::<i32>()
        .then_may_fail(|x| {
            match x % 2 {
                0 => Ok(x),
                1 => Err(std::io::Error::from_raw_os_error(x).into()),
                _ => { panic!("no way") }
            }
        }).then(|x| {
        log::info!("{} pass by", x);
        x
    })
        .then_async_may_fail(|x| async move {
            match x % 3 {
                0 => Ok(x),
                1 => Err("something wrong".into()),
                _ => { panic!("no way") }
            }
        })
        .then_async(|x| async move {
            log::info!("{} pass by async", x);
            sleep(Duration::from_secs(1)).await;
            x
        });
    let v = p.process(12).await.unwrap();
    assert_eq!(v, 12);
    let v = p.process(11).await.err().unwrap();
    log::info!("err: {}", v);
    let v = p.process(10).await.err().unwrap();
    log::info!("err: {}", v);
}

