# pod-server

A simple implementation of a web service facilitating the Proof-of-device
at the service provider's end.

## Basic usage

The minimal config file to run the server consists of the IAS valid API key,
and 32-byte long private key for signing cookies:

```toml
api_key = "0123456abcdef"
cookie_key = "0123456abcdef"
```

By default, the server binds itself to `127.0.0.1:8080` address. You can tweak
it by appending a `[server]` section to the config file:

```toml
api_key = "0123456abcdef"
cookie_key = "0123456abcdef"

[server]
address = "127.0.0.1"
port = 8080
```

The server uses [`diesel`] crate as the ORM and SQLite as the database. In order
to configure the database for use with the app, firstly, you need to specify the
path to the database file in `.env` file:

```
DATABASE_URL=mock.db
```

Secondly, before actually firing up the server for the first time, you need to
init the db:

```
diesel setup
```

Then, and this step should occur whenver you want to redo the tables, re-run
the migrations:

```
diesel migration run
```

Finally, when invoking the server from the command line, you are required to
specify the path to the config file:

```
cargo run -- config.toml -v
```

### RESTful API

The service strives to communicate with JSON only, hence, if there is an error,
its description will be communication inside the message's body encoded as JSON.

Example response content:

```json
{
  "description": "Something went wrong!"
}
```

#### Available entrypoints:

* GET `/` -- dummy route which will respond with a JSON indicating whether the user
  is authenticated or not.


* POST `/register` -- route required to register a new user with the service. The
  expected content body is a JSON containing at least two fields: `login` and
  `quote`.

  Example request content:

  ```json
  {
    "login": "johndoe",
    "quote": "AAAAA...AAA",
    "nonce": "AAAAA"
  }
  ```

  Nonce is optional, however both quote and nonce should be base64 encoded.

  Upon successful registration, a `200` message will be generated with empty
  body.

* GET `/auth` -- route required to initiate challenge-response protocol in order
  to authenticate the registered user using their SGX enclave (which holds a
  private ED25519 key). The challenge which the user is required to signed using
  their enclave will be provided inside content as JSON inside `challenge` field.
  The challenge will be base64 encoded.

  Example response content:

  ```json
  {
    "challenge": "AAAA...AAA"
  }
  ```

* POST `/auth` -- route where the signed `challenge` should be sent to. The signed
  challenge should be sent inside JSON under `response` field, and should be base64
  encoded. Additinally, the user should supply their login as part of the JSON.

  Example request content:

  ```json
  {
    "login": "johndoe",
    "response": "AAAA...AAA"
  }
  ```

## Testing with simple test client

For some end-to-end testing, you can use the provided simple test client which
exposes two bits of functionality: registering and authenticating with the
service.

The former takes a username and path to IAS Quote as arguments with
optional nonce:

```
cargo run --example test_client -- register johndoe <path_to_quote> --nonce "deadbeef123456"
```

This command will initiate a POST request to `/register` entrypoint.

The latter on the other hand takes only your username as an argument:

```
cargo run --example test_client -- authenticate johndoe
```

This command initiates 3 requests: a GET to `/auth` to obtain a challenge,
a POST to `/auth` to validate the challenge and authenticate with the
service, and finally a GET to `/` to verify that we've indeed successfully
signed in.
