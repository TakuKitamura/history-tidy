# history-tidy
Tidy history with hash-tag

- YouTube


[![YouTube Movie](http://img.youtube.com/vi/5RiUXg75OUs/0.jpg)](https://www.youtube.com/watch?v=5RiUXg75OUs)

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