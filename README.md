# history-tidy
Tidy history with hash-tag

## Status
- Under development

## TODO
- Testing
- Implementation of a function to modify and delete hashtag

## Dependence
- Rust
- Bash

## Setup

```bash
$ git clone https://github.com/TakuKitamura/history-tidy.git
$ cd history-tidy
$ cargo build --release
$ cp target/release/history-tidy /usr/local/bin/ 
$ echo 'eval "$(/usr/local/bin/history-tidy -init-bash)"' > ~/.bash_profile
$ source bash_profile
$ ls -a #example #file
$ history-tidy
```