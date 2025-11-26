# a-back

```docker-compose up -d``` - start PostgreSQL server

```cargo install sqlx-cli``` - if not installed

```sqlx migrate run``` - migration script

```cargo watch -q -c -w src/ -x run``` - run for dev

```cargo r -r``` - run for prod

```./chromedriver --port=9515 --disable-gpu --dns-prefetch-disable --disable-extensions --no-sandbox enable-automation``` - run chrome driver, если вылетает, нужно обновить на более новую версию chromedriver-mac-x64 