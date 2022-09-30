use actix_web::middleware::Logger;
use actix_web::{error, get, web, App, Error, HttpResponse, HttpServer};
use futures::StreamExt;
use serde_json::value::Value;
use std::collections::HashMap;

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[get("/echo")]
async fn echo(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();

    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<HashMap<String, Value>>(&body)?;
    let mut res: HashMap<String, Value> = HashMap::new();
    for (key, value) in &obj {
        match value {
            Value::Number(num) => {
                if key == "created" {
                    println!("{:?} {:?}", value, num.as_f64().unwrap());
                    let new_val = num.as_f64().unwrap() / 4.0;

                    println!("{:?}", new_val);
                    res.insert(
                        key.to_string(),
                        serde_json::to_value::<f64>(new_val).unwrap(),
                    );
                } else {
                    res.insert(key.to_string(), value.clone());
                }
            }
            Value::Object(_obj) => {
                res.insert(key.to_string(), value.clone());
                // TODO
            }
            _ => {
                res.insert(key.to_string(), value.clone());
            }
        }
    }

    print!("{:?}", res);
    Ok(HttpResponse::Ok().json(res)) // <- send response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| App::new().wrap(Logger::default()).service(echo))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
