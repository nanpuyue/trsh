use std::pin::Pin;

use openssl::base64::encode_block;
use openssl::hash::MessageDigest;
use openssl::ssl::{
    Ssl, SslAcceptor, SslConnector, SslContext, SslFiletype, SslMethod, SslVerifyMode,
};
use tokio::net::TcpStream;
use tokio_openssl::SslStream;

use crate::error::Result;

pub fn context_digest(ctx: &SslContext) -> Result<String> {
    let digest = ctx.certificate().unwrap().digest(MessageDigest::sha256())?;
    Ok(encode_block(digest.as_ref()))
}

pub fn peer_digest(tls: &SslStream<TcpStream>) -> Result<String> {
    let digest = tls
        .ssl()
        .peer_certificate()
        .unwrap()
        .digest(MessageDigest::sha256())?;
    Ok(encode_block(digest.as_ref()))
}

pub fn acceptor_context(cert: &str, key: &str) -> Result<SslContext> {
    let mut acceptor_builder = SslAcceptor::mozilla_modern(SslMethod::tls_server())?;
    acceptor_builder.set_certificate_chain_file(cert)?;
    acceptor_builder.set_private_key_file(key, SslFiletype::PEM)?;

    let context = acceptor_builder.build().into_context();
    Ok(context)
}

pub async fn tls_accept(stream: TcpStream, ctx: &SslContext) -> Result<SslStream<TcpStream>> {
    let mut ssl_stream = SslStream::new(Ssl::new(ctx.as_ref())?, stream)?;

    Pin::new(&mut ssl_stream).accept().await?;
    Ok(ssl_stream)
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
    let ssl = config.into_ssl(sni)?;
    let mut ssl_stream = SslStream::new(ssl, stream)?;

    Pin::new(&mut ssl_stream).connect().await?;
    Ok(ssl_stream)
}
