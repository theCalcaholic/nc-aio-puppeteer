pub mod lib;

const ALLOW_UNSAFE_TLS: bool = true;
const AIO_URL: &str = "https://172.17.0.2";
const AIO_PORT: i16 = 8080;

#[tokio::main]
async fn main() {
    let mut aio_client = lib::AioClient::new(AIO_URL.into(), AIO_PORT, ALLOW_UNSAFE_TLS).unwrap();

    aio_client.login().await.unwrap();

    let response = aio_client
        .request("api/docker/logs?id=nextcloud-aio-mastercontainer", None)
        .await
        .unwrap();

    println!("{}", response.text().await.unwrap())
}
