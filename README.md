# rust_sandbox

Run the local analyzer service:

```sh
cargo run
```

Analyze pasted clipboard text by sending it as the raw request body:

```sh
curl -X POST http://127.0.0.1:7878/analyze-clipboard \
  --data-binary $'hello\u00a0world\u200b'
```

The endpoint returns JSON with character and byte positions, line and column numbers, code points,
Unicode-style names, marker labels, and descriptions for hidden characters, formatting markers,
non-breaking spaces, control characters, and other invisible symbols.
