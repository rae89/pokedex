# Pokedex TUI

A terminal-based Pokédex built with Rust. Browse Gen 1 Pokémon, view colored sprite art, check type matchups, and build teams.

## Screens

- **Pokédex** — Scrollable list of 151 Pokémon with search and type labels
- **Detail** — Colored sprite art, base stats with bar charts, abilities, height/weight
- **Type Chart** — 18×18 scrollable type effectiveness matrix
- **Team Builder** — 6-slot teams with Pokémon picker, move selection, and type coverage analysis

## Install

### Pre-built binary (macOS)

Download `pokedex-macos.tar.gz` from [Releases](https://github.com/rae89/pokedex/releases), then:

```bash
tar -xzf pokedex-macos.tar.gz
xattr -d com.apple.quarantine pokedex
./pokedex
```

> The `xattr` command removes the macOS quarantine flag that blocks downloaded binaries. If you get "No such xattr", the file is already fine to run.

### Build from source

Requires [Rust](https://rustup.rs/) (1.70+).

```bash
git clone https://github.com/rae89/pokedex.git
cd pokedex
cargo build --release
./target/release/pokedex
```

#### Build a universal macOS binary (Apple Silicon + Intel)

```bash
rustup target add x86_64-apple-darwin
cargo build --release
cargo build --release --target x86_64-apple-darwin
lipo -create target/release/pokedex target/x86_64-apple-darwin/release/pokedex -output pokedex
```

The resulting `pokedex` binary works on both Apple Silicon and Intel Macs.

## Testing

Run the unit test suite:

```bash
cargo test
```

Generate code coverage report:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage/
```

## Controls

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Cycle screens |
| `1`–`4` | Jump to screen |
| `↑↓` / `jk` | Navigate lists |
| `/` | Search |
| `Enter` | Select / view details |
| `Esc` | Back / close modal |
| `a` | Add Pokémon to team (detail screen) |
| `d` | Remove from team (team builder) |
| `n` | New team |
| `←→` / `hl` | Switch teams / scroll type chart |
| `q` | Quit |

## How it works

- Data is fetched from [PokéAPI](https://pokeapi.co/) and cached locally for instant subsequent loads
- Sprites are rendered as colored Unicode half-block characters (`▀▄█`) with true-color RGB
- Teams are saved to `~/Library/Caches/pokemon-tui/teams.json` (macOS)

## Tech Stack

Rust, Ratatui, Crossterm, Tokio, Reqwest, Serde, image
