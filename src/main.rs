use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;

mod client;
mod error;
mod server;
mod term;
mod tls;
mod util;

use error::Result;
use term::{restore_termios, set_exit_handler};

#[derive(Debug, Parser)]
#[clap(version, author, about)]
struct Args {
    #[clap(
        short,
        parse(try_from_str),
        value_name = "IP:PORT",
        required_unless_present = "server",
        requires_all = &["cert", "key"],
        conflicts_with_all = &["server", "domain", "notverify", "readonly"],
        help = "Listen address (server, required)"
    )]
    listen: Option<SocketAddr>,

    #[clap(
        short,
        value_name = "HOST:PORT",
        required_unless_present = "listen",
        conflicts_with = "listen",
        help = "Server address to connect (client, required)"
    )]
    server: Option<String>,

    #[clap(
        short,
        parse(try_from_str),
        value_name = "FILE",
        help = "Certificate chain file (server, required)"
    )]
    cert: Option<PathBuf>,

    #[clap(
        short,
        value_name = "FILE",
        parse(try_from_str),
        help = "Private key file (server, required)"
    )]
    key: Option<PathBuf>,

    #[clap(short, help = "Server name to verify (client)")]
    domain: Option<String>,

    #[clap(short, help = "Do not verify the server certificate (client)")]
    notverify: bool,

    #[clap(short, help = "Readonly mode (client)")]
    readonly: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let _ = set_exit_handler();

    let ret = if let Some(listen) = args.listen {
        server::server(listen, &args.cert.unwrap(), &args.key.unwrap()).await
    } else if let Some(server) = args.server {
        let sni = if let Some(domain) = &args.domain {
            domain
        } else {
            server.split(':').next().unwrap_or_default()
        };
        client::client(&server, sni, !args.notverify, args.readonly).await
    } else {
        Ok(())
    };

    let _ = restore_termios();

    ret
}
