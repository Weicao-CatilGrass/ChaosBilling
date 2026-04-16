# Chaos Billing

A command-line bill splitting tool

## Installation
```bash
cargo install --path .
```

## Usage
```bash
# Add a bill
cobill add Alice --paid 20 --for Alice Bob --reason Food

# List bills
cobill ls

# Optimize splits
cobill ls --optimize

# Edit
cobill edit # or `cobill vi/vim/nvim/helix/nano`

# Clear
cobill clear
```

## License
This project uses the [WTFPL](./LICENSE)
