// media stuff!!

use std::path::PathBuf;

use ffmpeg_sidecar::command::FfmpegCommand;

use tempfile::NamedTempFile;

// this is our main media type, all media is converted into this format before being passed to a function.

#[derive(Debug)]
pub struct Media {
    // The type of the media
    media_type: MediaType,
    // the path to the temporary file
    file_path: PathBuf,
    // output path!
    output_tempfile: tempfile::NamedTempFile
}

#[derive(Debug, PartialEq)]
enum MediaType {
    Video,
    Gif,
    Image,
    Audio
}

pub fn resize_media(input: Media, x_size: u16, y_size: u16) -> Result<Media, crate::Error> {
    // This function takes in a media file, and resizes it to be of certain dimensions.

    // Set boundaries for how small and big media can become
    const MIN_SIZE: u16 = 1;
    const MAX_SIZE: u16 = 8000;

    // check to make sure the input sizes haven't exceeded our boundaries.
    if !(x_size <= MAX_SIZE && y_size <= MAX_SIZE && x_size >= MIN_SIZE) {
        return Err("Invalid media dimensions!".into());
    }

    // Make sure the media isn't a audio file, because we cant resize that.

    if input.media_type == MediaType::Audio {
        // Cant resize audio.
        return Err("Cannot resize a audio file.".into());
    }

    // Media is of a good size, now to process it.

    // create a tempfile to store the output.
    let mut tempfile = NamedTempFile::new()?;
    // where does that live
    let tempfile_path = tempfile.path();

    // Do the actual resizing.
    // every arg gets a separate line for readability instead of an array.

    let mut output = FfmpegCommand::new()
    .hwaccel("auto")
    .input(input.file_path.as_path().to_str().unwrap()) // input file
    .arg(format!("-s {}x{}", x_size, y_size)) // set the dimensions
    .codec_audio("copy") // copy audio codec
    .output(tempfile_path.to_str().unwrap()) // where is it going?
    .spawn().unwrap(); // run that sucker

    output.wait().unwrap(); // wait for that sucker to finish

    // now build our output

    Ok(Media {
        media_type: input.media_type,
        file_path: input.file_path,
        output_tempfile: tempfile
    })
}