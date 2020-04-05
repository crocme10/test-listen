CREATE TABLE ticker (
  id VARCHAR(16) PRIMARY KEY NOT NULL,
  price NUMERIC(2) NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION notify_ticker_update()
RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify(
    'ticker', -- channel
    json_build_object(
      'operation', TG_OP,
      'record', row_to_json(NEW)
    )::text
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER ticker_update
AFTER INSERT OR UPDATE
ON ticker
FOR EACH ROW
EXECUTE PROCEDURE notify_ticker_update()
