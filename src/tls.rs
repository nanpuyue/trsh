use openssl::ssl::{SslAcceptor, SslConnector, SslFiletype, SslMethod, SslVerifyMode};
use tokio::net::TcpStream;
use tokio_openssl::{accept, connect, SslStream};

use crate::Result;

pub async fn tls_accept(stream: TcpStream, cert: &str, key: &str) -> Result<SslStream<TcpStream>> {
    let mut ssl_acceptor_builder = SslAcceptor::mozilla_modern(SslMethod::tls_server())?;
    ssl_acceptor_builder.set_certificate_chain_file(cert)?;
    ssl_acceptor_builder.set_private_key_file(key, SslFiletype::PEM)?;

    let ssl_acceptor = ssl_acceptor_builder.build();
    Ok(accept(&ssl_acceptor, stream).await?)
}

pub async fn tls_connect(
    stream: TcpStream,
    sni: &str,
    verify: bool,
) -> Result<SslStream<TcpStream>> {
    let mut connector_builder = SslConnector::builder(SslMethod::tls_client())?;
    if !verify {
        connector_builder.set_verify(SslVerifyMode::NONE);
    }
    let config = connector_builder.build().configure()?;
    Ok(connect(config, sni, stream).await?)
}
