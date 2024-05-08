## Main goals:
* Faster than MediaForge. Performance should be held to a very high standard.
* Doesn't constantly crash like MediaForge
* Written in [[Rust]].
* Transparent queue, you can see what your current position is in the queue at all times. *1*
* Full coverage unit testing.
* Lots of profiling.
* [[Sharding]], so the bot can be updated with no downtime. (but usually just one shard.)
* Link as input, instead of requiring the media to be in chat.
* Input media directly in command ([attatchment input in slash command](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-type:~:text=ATTACHMENT,attachment%20object))
* Some kind of logging into a database so we can keep track of cpu time usage of functions / heavy users.


## Commands:
* /speechbubble: adds speech bubbles to messages (with transparency!) ? possible with videos that support transparency? does discord even support transparent videos?
* /caption: The usual captioning gag. (top or bottom)
* /invert: invert colors of image or video
* /rotate: rotate the image or video, increments of 90. (possibly every angle in the future?)
* /blur: blur the image / video, adjustable strength
* /crunch: absolutely destroy the bitrate of a video (not constant bitrate, but constant quality.)
* /jpeg: apply jpeg compression artifacting to an image
* /resize: resize an image or video to a specified size or multiplier.
* /reverse: Reverses the playback of a gif or video, or reverse the audio of an audio file
* /slowmo: double every frame to make playback 2x slower on gifs and videos.
* /gifit (Gif It): converts an image or video into a gif. (video might be tricky)
* /squish and /stretch: make images and videos wider / taller.
* /overlay: add two images together, or possibly overlay an image on a video?
* /volume: make a video louder or quieter
* /bass: bass boost a video
* /loud: turn up the volume on the video to make it clip like crazy.
* /mute: remove audio from a video
* /audio: rip the audio from a video and upload it as a mp3
* /video: convert a audio file into a video (black image, just for mobile peeps)
* /pitch: pitch up or down the audio of a video (or an audio file)
* /echo: add reverb to a video/audio 
* /chip: bitcrunch audio 
* /5d: make the audio spin around the headphone channels
* /swap: swap the audio channels 
* /stutter: make's a video stuttery.
* /interlace: halves the fps of a video/gif but adds interlacing.
* /8mb: crunch a file down to 8MB by adjusting bitrate settings and such (not sure how gif handling would work, just crank the lossy till we hit it?).
* /clip: YouTube Clip renderer (Takes in a youtube clip url, spits out a video file)
* /decaption: removes the caption from media.

## Important ideas:
* Unified "Media" type


## Unknowns:
* FFMPEG or GStreamer? Lots of rust bindings for GStreamer, possibly more performant? but FFMPEG has very wide codec support....
* Do we need to handle gif's with our video converter of choice, or use a library like https://lib.rs/crates/gif ?
* does discord even support transparent videos?

### Possible issues:
1. Updating all of the embeds for currently processing items might hit a rate limit.

### Important crates
Coverage testing: https://github.com/taiki-e/cargo-llvm-cov
Better tests: https://nexte.st/
‼️ make sure that coverages is in the unit tests! you need to have 100% coverage all the time! ‼️
Easily parallelize loops: https://lib.rs/crates/rayon\
Temporary files: Either https://lib.rs/crates/tempfile (has unsafe code dependances but that probably doesn't even matter for us) or https://lib.rs/crates/temp-dir (forbids unsafe, not windows compatible)
Database interaction: https://lib.rs/crates/sqlx
Text wrapping (important for captions): https://lib.rs/crates/textwrap
file size to human readable: https://lib.rs/crates/bytesize