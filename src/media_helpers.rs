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
    output_tempfile: Option<tempfile::NamedTempFile>
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
    let tempfile = NamedTempFile::new()?;
    // where does that live
    let tempfile_path = tempfile.path();

    // Do the actual resizing.
    // every arg gets a separate line for readability instead of an array.

    let mut output = FfmpegCommand::new()
    .hwaccel("auto")
    .input(input.file_path.as_path().to_str().unwrap()) // input file
    .args([ // set the dimensions
        "-vf",
        &format!("scale={}:{}", x_size, y_size)
    ]) 
    .codec_audio("copy") // copy audio codec
    //.output(tempfile_path.to_str().unwrap()) // where is it going?
    .output(tempfile_path.to_str().unwrap())
    .spawn().unwrap(); // run that sucker

    output.wait()?; // wait for it to finish.
    //TODO: make a helper that makes sure that FFMPEG didnt die.

    // now build our output

    Ok(Media {
        media_type: input.media_type,
        file_path: input.file_path,
        output_tempfile: Some(tempfile)
    })
}

#[test]
fn resize_test() {
    ffmpeg_sidecar::download::auto_download().unwrap();
    //get current path to src
    let srcpath = env!("CARGO_MANIFEST_DIR");
    // try resizing a few things
    let baja_cat = Media {
        file_path: format!("{}\\src\\test_files\\bajacat.png", srcpath).into(), //TODO: does this work on linux?
        media_type: MediaType::Image,
        output_tempfile: None
    };
    let result = resize_media(baja_cat, 128, 128);
    match result {
        Ok(media) => {},
        Err(e) => panic!("{}", e),
    }
}
