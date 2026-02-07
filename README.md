# Pokedex TUI

A terminal-based Pokédex built with Rust. Browse Gen 1 Pokémon, view colored sprite art, check type matchups, and build teams.

## Screens

- **Pokédex** — Scrollable list of 151 Pokémon with search and type labels
- **Detail** — Colored sprite art, base stats with bar charts, abilities, height/weight
- **Type Chart** — 18×18 scrollable type effectiveness matrix
- **Team Builder** — 6-slot teams with Pokémon picker, move selection, and type coverage analysis

## Install

### From source

```bash
git clone https://github.com/rae89/pokedex.git
cd pokedex
cargo build --release
./target/release/pokedex
```

### Pre-built binary (macOS)

Download `pokedex-macos.tar.gz` from [Releases](https://github.com/rae89/pokedex/releases), then:

```bash
tar -xzf pokedex-macos.tar.gz
xattr -d com.apple.quarantine pokedex
./pokedex
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
