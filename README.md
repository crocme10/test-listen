# Asynchronous Notifications for [tokio-postgres](https://docs.rs/tokio-postgres/0.5.3/tokio_postgres/)

This project builds a tiny database, with a trigger on update, which sends notifications on
a channel.

We expect that if on one hand we update the rows in the table, then on the other we would receive
notifications if we listen to the specified channel.

## Setup

### Database

We can use `docker-compose` to quickly spin a postgresql database:

```
cd postgres
docker-compose up
```

The connection point for this database is stored in `postgres/database.env`.

In another terminal window, we use the script `postgres/provision.sh` to create and populate the
database.

```
cd postgres
./provision.sh
```

This script will get the database schema from `postgres/build.sql`, and insert a few rows.
The table is really simple, it consists in a string id, a price, and a timestamp.

At this point, we can run a check to see if the notifications mechanism works correctly. In one
window, we open a postgresql shell, and subscribe to the channel.

```
psql -h localhost -U [POSTGRES_USER] -d [POSTGRES_DB]

sql > LISTEN ticker;
```

In another terminal, we use the script `postgres/update.sh` to update the database:

```
cd postgres
./update.sh AAA 3.14
```

Nothing happens in the postgres shell, but if you update the terminal (by typing `;`), you'll
get the update:

```
Asynchronous notification "ticker" with payload "{"operation" : "UPDATE", "record" :
  {"id":"AAA","price":3.14,"updated_at":"2020-04-05T18:40:22.109379+00:00"}}"
  received from server process with PID 74
```

### Rust client

The rust client uses `tokio-postgres` to connect to the database, subscribe to the `ticker`
channel, and print notifications received.

You'll need to compile the client, and run it. It reads the database information from
the `postgres/database.env` file, so you don't need to pass any information.

```
cargo run --release
```

