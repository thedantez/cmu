```text
   _________    ____                 ___     ___
  /         \  /    \___________    /   \   /   \
 /     _____/  |                \   |   |   |   |
/    /         |    /\     /\    |  |   |   |   |
\    \_____    |   |  |   |  |   |  |   \___/   |
 \          \  |   |  |   |  |   |  |           |
  \_________/  \___/  \___/  \___/  \_______/\__/
  
  Central Messaging Unit
```
A lightweight messenger hub to communicate through one app with vim controls

[![Github release (latest by date)](https://img.shields.io/github/v/release/thedantez/cmu)](https://github.com/thedantez/cmu/releases)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
## ⚠️ This app is still in development many features are not implemented
## What CMU can offer?
Not many messenger wrappers are able to run in terminal and even fewer have **vim-like controlling**, this app can offer the experience, additionally as its second feature it supports **multiple messengers** so there is no need to open many other apps when you can have all in one with shared style

## Implemented messengers
+ VK ✅ (Authorization is handled by Kate Mobile)
+ Telegram ❌
+ Other? ❌

## Planned features
  + vim-like controlling (hjkl for navigation, normal/insert modes) ✅
  + notifications from every messenger ❌
  + quickly switch between messengers in an opened window ❌
  + support for user themes ❌
  + markdown support ❌

## Implemented basic messenger features 
  + List messages and chats ✅
  + Sending messages ✅
  + Reply, delete, edit and forward messages ❌
  + Message reactions ❌
  + Mention users in group chats ❌
  + Show user status ❌
  + View images and videos ❌
  + Send documents ❌
  + Search messages in chat ❌

## Installation
```bash
cd <path>/cmu
git clone https://github.com/thedantez/cmu.git
cargo install --path .
```
The app is not yet published anywhere except github

## Requirements
+ Rust & Cargo (install from [rustup](https://rustup.rs/))
