use dotenv;
use futures::future;
use futures::{stream, FutureExt, Stream, StreamExt, TryStreamExt};
use snafu::{Backtrace, NoneError, ResultExt, Snafu};
use std::env;
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

    #[snafu(display("Env Variable Error: {}", detail))]
    #[snafu(visibility(pub))]
    EnvError { detail: String },

    #[snafu(display("DB Connection Error: {}", source))]
    #[snafu(visibility(pub))]
    DBError {
        source: tokio_postgres::Error,
        backtrace: Backtrace,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Read the file ./postgres/database.env to extract user, password, and database name
    let dbenv = env::current_dir()
        .and_then(|d| Ok(d.join("postgres").join("database.env")))
        .context(IOError)?;
    dotenv::from_path(dbenv.as_path())
        .or(Err(NoneError))
        .context(EnvError {
            detail: String::from("database env"),
        })?;

    // Build the connection string
    let connstr = format!(
        "postgresql://{user}:{pwd}@localhost/{db}",
        user = dotenv::var("POSTGRES_USER")
            .or(Err(NoneError))
            .context(EnvError {
                detail: String::from("POSTGRES_USER")
            })?,
        pwd = dotenv::var("POSTGRES_PASSWORD")
            .or(Err(NoneError))
            .context(EnvError {
                detail: String::from("POSTGRES_PASSWORD")
            })?,
        db = dotenv::var("POSTGRES_DB")
            .or(Err(NoneError))
            .context(EnvError {
                detail: String::from("POSTGRES_DB")
            })?,
    );

    let (client, connection) = connect_raw(&connstr).await?;

    let mut stream = get_stream(&client, connection).await?;

    loop {
        match stream.try_next().await {
            Ok(n) => {
                if let Some(msg) = n {
                    println!("Received notification: {}", msg);
                }
            }
            Err(err) => {
                println!("Received error: {}", err);
            }
        }
    }

    // Ok(())
}

async fn get_stream(
    client: &Client,
    mut connection: Connection<TcpStream, NoTlsStream>,
) -> Result<impl Stream<Item = Result<String, Error>> + Send + 'static, Error> {
    let (tx, rx) = futures::channel::mpsc::unbounded();
    let stream = stream::poll_fn(move |cx| connection.poll_message(cx)).map_err(|e| panic!(e));
    let connection = stream.forward(tx).map(|r| r.expect("stream forward"));
    tokio::spawn(connection);
    println!("Spawned dedicated connection for postgres notifications");

    client
        .batch_execute("LISTEN ticker;")
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
    // Here we extract the host and port from the connection string.
    // Note that the port may not necessarily be explicitely specified,
    // the port 5432 is used by default.
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
