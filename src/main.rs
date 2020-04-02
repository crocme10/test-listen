use futures::future;
use futures::{stream, FutureExt, Stream, StreamExt, TryStreamExt};
use snafu::{Backtrace, ResultExt, Snafu};
use std::io;
use tokio::net::TcpStream;
use tokio_postgres::tls::{NoTls, NoTlsStream};
use tokio_postgres::{config::Host, AsyncMessage, Client, Config, Connection};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("User Error: {}", details))]
    #[snafu(visibility(pub))]
    UserError { details: String },

    #[snafu(display("IO Error: {}", source))]
    #[snafu(visibility(pub))]
    IOError {
        source: io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("DB Connection Error: {}", source))]
    #[snafu(visibility(pub))]
    DBError {
        source: tokio_postgres::Error,
        backtrace: Backtrace,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let connstr = String::from("postgresql://user:pwd@postgres/journal");
    let mut stream = get_stream(&connstr).await?;

    while let Ok(n) = stream.try_next().await {
        if let Some(msg) = n {
            println!("Received notification: {}", msg);
        }
    }

    Ok(())
}

async fn get_stream(
    connstr: &str,
) -> Result<impl Stream<Item = Result<String, Error>> + Send + 'static, Error> {
    let (client, mut connection) = connect_raw(connstr).await?;

    let (tx, rx) = futures::channel::mpsc::unbounded();
    let stream = stream::poll_fn(move |cx| connection.poll_message(cx)).map_err(|e| panic!(e));
    let connection = stream.forward(tx).map(|r| r.expect("stream forward"));
    tokio::spawn(connection);
    println!("Spawned dedicated connection for postgres notifications");

    client
        .batch_execute("LISTEN documents;")
        .await
        .context(DBError)?;

    Ok(rx.filter_map(|m| match m {
        AsyncMessage::Notification(n) => {
            println!("Received notification on channel: {}", n.channel());
            future::ready(Some(Ok(format!("{} : {}", n.channel(), n.payload()))))
        }
        _ => {
            println!("Received something on channel that is not a notification.");
            future::ready(None)
        }
    }))
}

async fn connect_raw(s: &str) -> Result<(Client, Connection<TcpStream, NoTlsStream>), Error> {
    let config = s.parse::<Config>().context(DBError)?;
    let host = config
        .get_hosts()
        .first()
        .ok_or(Error::UserError {
            details: String::from("Missing host"),
        })
        .and_then(|h| match h {
            Host::Tcp(remote) => Ok(remote),
            Host::Unix(_) => Err(Error::UserError {
                details: String::from("No local socket"),
            }),
        })?;
    let port = config.get_ports().first().ok_or(Error::UserError {
        details: String::from("Missing port"),
    })?;

    let conn = format!("{}:{}", host, port);
    println!("Connecting to {}", conn);
    let socket = TcpStream::connect(conn).await.context(IOError)?;
    config.connect_raw(socket, NoTls).await.context(DBError)
}
