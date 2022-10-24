### example
start a service serve for process json
```rust
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
```