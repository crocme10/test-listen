Example of asynchronous notifications for
[tokio-postgres](https://docs.rs/tokio-postgres/0.5.3/tokio_postgres/)

You need to edit the code to modify the connection string, and the channel on which
to listen for notifications.

This example just prints, errr, should print, message like
"Received notification: channel : payload"
