# Captain Caption
This is a program designed to help people edit video captions.
Currently the following formats are supported:
- .srt
- .vtt
and the following edit operations are supported:
- concatenating multiple caption files (useful for when you stitch multiple videos into one)
- converting captions between srt and vtt formats
- crop captions between two timestamps or millisecond values
- get information about a caption file, including speaker talk time
- offset a caption by some amount of time (useful for if you place a video of known length and NO caption before the one you're editing)

This was done mostly because at the time I was learning some Rust and
wanted to try my hand at writing a full program.
If you would like a format added or have a feature request feel free to
open an issue.
