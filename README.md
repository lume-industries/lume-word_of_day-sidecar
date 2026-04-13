# lume-word_of_day-sidecar

Fetches Merriam-Webster Word of the Day RSS feed + dictionary API definitions.

Produces `WordPayload` payloads conforming to the VZGLYD sidecar channel ABI.

This sidecar is designed to be reusable. Any slide can depend on it via git and receive data payloads through the standard channel ABI.

## Poll Interval

Every 24 hours.

## Payload Format

`WordPayload` serialized as JSON bytes.

## Environment Variables

| Variable | Description |
|---|---|

## Usage

Build the sidecar:

```bash
cargo build --target wasm32-wasip1 --release
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
