# Speed Reader
- ts pretty cool ngl its like you click enter once opening the app right and then you like get to read shit faster cause like your eyes don't have to stray so like its better yk and its like cli so you dont need a gui thing right so its pretty lit ngl

## Compiling and running
- you need rust btw
```
git clone https://github.com/taice/speedreader
cd speedreader
cargo build --release

# if you want to copy the binary to path
cp target/release/speedreader $HOME/.cargo/bin

# otherwise just run it like this
./target/release/speedreader
```

## Sick Features
- saving data in a data directory regardless of platform
- settings for selecting speed and stuff

### how to use ts
- press enter to load text from clipboard
- press space to pause/play
- press esc to toggle the settings menu, navigate/change with arrow keys
- press b/B to go back a word or a sentence respectively
- in the settings you can select the word at which the reader should start by doing enter on the words box
