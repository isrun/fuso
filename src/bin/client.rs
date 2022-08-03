use std::net::IpAddr;

use clap::Parser;
use fuso::Socket;

#[derive(Parser)]
struct FusoArgs {
    /// 映射成功,实际访问端口
    #[clap(
        long,
        visible_alias = "bind",
        visible_short_alias = 'b',
        default_value = "0"
    )]
    visit_port: u16,
    /// 桥接监听地址
    #[clap(long, default_value = "127.0.0.1")]
    bridge_listen: IpAddr,
    /// 桥接监听端口
    #[clap(long)]
    bridge_port: Option<u16>,
    /// 服务端地址
    #[clap(long, default_value = "127.0.0.1")]
    server_host: String,
    /// 服务端端口
    #[clap(long, default_value = "6722")]
    server_port: u16,
    #[clap(long, default_value = "127.0.0.1")]
    /// 转发地址
    forward_host: String,
    /// 转发端口
    #[clap(long, default_value = "80")]
    forward_port: u16,
    /// 日志级别
    #[cfg(debug_assertions)]
    #[clap(long, default_value = "debug")]
    log_level: log::LevelFilter,
    /// 日志级别
    #[cfg(not(debug_assertions))]
    #[clap(long, default_value = "info")]
    log_level: log::LevelFilter,
}

#[cfg(feature = "fuso-rt-tokio")]
#[tokio::main]
async fn main() -> fuso::Result<()> {
    use std::time::Duration;

    use fuso::{penetrate::PenetrateRsaAndAesHandshake, TokioAccepter, TokioPenetrateConnector};

    let args = FusoArgs::parse();

    env_logger::builder()
        .filter_module("fuso", args.log_level)
        .default_format()
        .format_module_path(false)
        .init();

    let fuso = fuso::builder_client_with_tokio()
        .using_handshake(PenetrateRsaAndAesHandshake::Client)
        .using_penetrate(
            Socket::tcp(args.visit_port),
            Socket::tcp((args.forward_host, args.forward_port)),
        )
        .maximum_retries(None)
        .heartbeat_delay(Duration::from_secs(60))
        .maximum_wait(Duration::from_secs(10))
        .build(
            Socket::tcp((args.server_host, args.server_port)),
            TokioPenetrateConnector::new().await?,
        );

    let fuso = match args.bridge_port {
        None => fuso.run(),
        Some(port) => fuso
            .using_bridge(Socket::tcp((args.bridge_listen, port)), TokioAccepter)
            .run(),
    };

    fuso.await
}

#[cfg(feature = "fuso-web")]
#[tokio::main]
async fn main() {}

#[cfg(feature = "fuso-api")]
#[tokio::main]
async fn main() {}

#[cfg(feature = "fuso-rt-smol")]
fn main() -> fuso::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .default_format()
        .format_module_path(false)
        .init();

    smol::block_on(async move {
        use fuso::SmolPenetrateConnector;

        fuso::builder_client_with_smol()
            .build(
                Socket::Tcp(8888.into()),
                PenetrateClientFactory {
                    connector_factory: Arc::new(SmolPenetrateConnector),
                    socket: {
                        (
                            Socket::Tcp(([0, 0, 0, 0], 9999).into()),
                            Socket::Tcp(([127, 0, 0, 1], 22).into()),
                        )
                    },
                },
            )
            .run()
            .await
    })
}
