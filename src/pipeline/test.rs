


#[tokio::test]
async fn test_pipeline() {
    use std::time::Duration;
    use tokio::time::sleep;
    use log;
    use crate::log::simple_logger;
    use crate::pipeline::link::{begin, Linkable, Pipeline};
    simple_logger::init().unwrap();

    let p = begin::<i32>()
        .then_result(|x| {
            match x % 2 {
                0 => Ok(x),
                1 => Err(std::io::Error::from_raw_os_error(x).into()),
                _ => { panic!("no way") }
            }
        }).then(|x| {
        log::info!("{} pass by", x);
        x
    })
        .then_async_result(|x| async move {
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


#[tokio::test]
async fn handle_differen_err() {
    use std::fs;
    use serde::{Serialize, Deserialize};
    use crate::pipeline::link::{begin, Linkable, Pipeline};
    #[derive(Serialize, Deserialize)]
    struct Fire {
        a: i32,
        b: String,
    }
    let p = begin().then_async_result(
        |x: i32| async move {
            let _: Fire = serde_json::from_str("adb")?;
            let _ = fs::read("nonono")?;
            Ok(x)
        }
    ).then_result(|x| {
        let _: Fire = serde_json::from_str("adb")?;
        let _ = fs::read("nonono")?;
        Ok(x)
    });
    let r = p.process(10).await;
    println!("{}", r.err().unwrap())
}


