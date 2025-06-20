# Who's Turn Is It?!
A TUI to keep track of encounters in D&D. It aims to be Very lightweight and as few bells-and-whistles as possible.

# Configuration
If you want to have a default troop of players you can add a `default.yml` file in either `~/.config/wtii/` on UNIX or
`Documents\wtii\` on Windows. Example of the config file:

```
---
players:
  - name: Player 1
    desc: Some description to Player 1
  - name: Player 2
```

These players will always be loaded as default, it is recommended to have the whole party here.

# Installation
## Linux and MacOS
1. Install Rust and Cargo from https://rustup.rs/
2. Run `./install`, the binary will be placed in `~/.local/bin/wtii`
