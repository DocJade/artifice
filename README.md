# Artifice
A high-performance Discord Media Bot, built with Rust.

## Features

* **Fast:** Major focus on being more responsive than other media manipulation bots.
* ~~**Stability First:** Designed to avoid the crashes that plague MediaForge.~~ (WIP lol)
* **Transparent Queues:** Always know your exact spot in line.
* **Media agnostic:** Processes almost every format of audiovisual files.


### Planned Functions:

* [x] /caption: The usual captioning gag. (top or bottom)
* [x] /rotate: rotate the image or video, in increments of 90.
* [x] /resize: resize an image or video to a specified size or multiplier.
* [ ] /speechbubble: adds speech bubbles to images/gifs (with transparency!)
* [ ] /invert: invert colors of image or video
* [ ] /blur: blur the image/video, adjustable strength
* [ ] /crunch: absolutely destroy the bitrate of a video (not constant bitrate, but constant quality.)
* [ ] /jpeg: apply jpeg compression artifacts to an image
* [ ] /reverse: Reverses the playback of a gif or video, or reverses the audio of an audio file
* [ ] /slowmo: double every frame to make playback 2x slower on gifs and videos.
* [ ] /gifit (Gif It): converts an image or video into a gif.
* [ ] /squish and /stretch: make images and videos wider / taller.
* [ ] /overlay: add two images together, or possibly overlay an image on a video?
* [ ] /volume: make a video louder or quieter
* [ ] /bass: bass boost a video
* [ ] /loud: turn up the volume on the video to make it clip like crazy.
* [ ] /mute: remove audio from a video
* [ ] /audio: rip the audio from a video and upload it as a mp3
* [ ] /video: convert an audio file into a video (black image, just for mobile peeps)
* [ ] /pitch: pitch up or down the audio of a video (or an audio file)
* [ ] /echo: add reverb to a video/audio 
* [ ] /chip: bit-crunch audio 
* [ ] /5d: make the audio spin around the headphone channels
* [ ] /swap: swap the audio channels 
* [ ] /stutter: makes a video stuttery.
* [ ] /interlace: halves the fps of a video/gif but adds interlacing.
* [ ] /8MB: crunch a file down to 8MB by adjusting bitrate settings and such (not sure how gif handling would work, just crank the lossy till we hit it?).
* [ ] /clip: YouTube Clip renderer (Takes in a youtube clip url, spits out a video file)
* [ ] /decaption: removes the caption from media.
* [ ] /subway: add subway surfers gameplay below media.

## Getting Started

### Requirements:
FFMPEG must be installed on the host, although if it is not found, `ffmpeg-sidecar` will try and pull a copy for you.

### Setup:
clone the repo
create a `.env` file that contains your `TOKEN=`, and optional `HW_ACCE=` settings.
`cargo run --release`
It's that simple!

## Contributing
- We welcome contributions! Please refer to the contributing guidelines. (to be added soon (lie))

## License
- todo

## Contact
[DocJade's discord](https://discord.docjade.com/) (there is currently a thread for this project in the #off-topic channel)
