## Usage

### Star the server
```bash
$ cargo run --bin server
```

or with logs:

```bash
$ RUST_LOG=info cargo run --bin server
```

### Test with the client

```bash
$ cargo run --bin client 127.0.0.1:1222 "foo=bar"
```

## Example output

```bash
$ cargo run --bin client 127.0.0.1:1222 "foo=bar"
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/client '127.0.0.1:1222' foo=bar`
Insert request sent. No response expected.

$ cargo run --bin client 127.0.0.1:1222 "foo"
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/client '127.0.0.1:1222' foo`
Received response: foo=bar
```