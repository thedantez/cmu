```text
___      ___ ____  ___      _______   ____ ____  _____ ________     ________ ____ ____ ____
\  \    /  / |  | /  /      |  @@  \  |  | |  | /  __/ |_    _|     |_    _| |  | |  | |  |
 \  \  /  /  |  |/  /  ---- |   @  /  |  | |  | \__  \   |  |  ----   |  |   |  | |  | |  |
  \  \/  /   |  |\  \  ---- |  |\  \  |  \_/  |  _/  /   |  |  ----   |  |   |  \_/  | |  |
   \____/    |__| \__\      |__| \__\ |____/|_| /___/    |__|         |__|   |____/|_| |__|
```

# console messenger for VK w/ vim-like controlling

# app features:
  + vim-like controlling (h/j/k/l for navi, normal/insert modes)
  + splitting screen into 2 parts (chats, messages from chat)
  + moving the cursor through messages
  + sending & getting messages by VK's API

# vim keybindings:
  + moving cursor (j/k)
  + opening chat (l/enter)
  + close chat (h)
  + sending message (enter w/ normal mode & opened chat)
  + change mode to insert (i)
  + change mode to normal (esc)
  + quit (q)

# installation:
  + AUR (comming soon)
  + from code

```bash
git clone https://github.com/thedantez/vk-rust-tui.git
cd vk-rust-tui/
cargo build --release

# start
./target/release/vk-rust-tui
```

# dependses & requirements
  + Rust & Cargo (install from [rustup](https://rustup.rs/))
  + token from VK (now by yourself, in features would OAuth)

<div style="text-align: justify;">
    [![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org)
    [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
    [![Github release (latest by date)](https://img.shields.io/github/v/release/thedantez/vk-rust-tui)](https://github.com/thedantez/vk-rust-tui/releases)
</div>
