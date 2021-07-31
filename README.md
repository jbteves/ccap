# What is this?
This is a one-off I made, over-engineered very badly, because of a niche
scenario where someone needed to have their video captioned different from
a different batch of videos, which would ultimately be stitched together,
necessitating the adjustment of the caption timestamps.
So this binary file will rread a `.srt` file and replace the timestamps
with one or both of the following:
1. A manually specified offset in milliseconds
1. The last timestamp from a specified file.
It's quite fast, even though you probably don't need that.
To build, install `rust` (available on MacOS via `brew`).
Then,
```
git clone https://github.com/jbteves/offset_caption.git
cd offset_caption
cargo build --release
```
The binary is now in the subdirectory `target/release/offset_caption`.
You can run `offset_caption -h` for full help text.
If you want to make it available, you can run in shell or add to your
.bashrc the following
```
alias offset_caption=/path/to/binary/offset_caption
```
Happy caption timestamp adding!
