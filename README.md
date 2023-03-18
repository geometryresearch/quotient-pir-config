# quotient-pir-config

[`quotient-pir`](https://github.com/geometryresearch/quotient-pir/) requires
serialised `Config` data to instantiate `QuotientTracker` objects.

```js
var data = fs.readFileSync('configs/config_32');
const server = new wasm_module.QuotientTracker(data);
```

This repository contains code for a binary that can generate such config files.

## Usage

First, clone this repository and build the binary:

```bash
cargo build --release
```

Run the `config` binary:

```
cargo run --release --bin config -- -l 10 --ptau 10.ptau --output config_10
```
