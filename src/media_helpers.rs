// media stuff!!

use std::{ffi::OsStr, path::PathBuf};

use ffmpeg_sidecar::command::FfmpegCommand;

use rand::Rng;
use tempfile::TempDir;

use crate::ffmpeg_babysitter::ffbabysit;

// this is our main media type, all media is converted into this format before being passed to a function.

#[derive(Debug)]
pub struct Media {
    // The type of the media
    media_type: MediaType,
    // the path to the temporary file
    file_path: PathBuf,
    // output path!
    output_tempfile: Option<(TempDir, PathBuf)>, //TODO: Should this be moved into its own type?
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum MediaType {
    Video,
    Gif,
    Image,
    Audio,
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

    // get the extension of the input file
    let extension = input.file_path.extension().unwrap();

    // create a tempfile to store the output.
    let (dir, filename) = new_temp_media(extension);

    // Do the actual resizing.
    // every arg gets a separate line for readability instead of an array.

    let output = FfmpegCommand::new()
        .hwaccel("auto")
        .input(input.file_path.as_path().to_str().unwrap()) // input file
        .args([
            // set the dimensions
            "-vf",
            &format!("scale={}:{}", x_size, y_size),
        ])
        .codec_audio("copy") // copy audio codec
        //.output(tempfile_path.to_str().unwrap()) // where is it going?
        .output(filename.to_str().unwrap())
        .spawn()
        .unwrap(); // run that sucker

    // wait for that to finish
    let waited = ffbabysit(output);
    // did that work?
    if let Some(babysitting_error) = waited {
        // no, it did not.
        return Err(babysitting_error)
    }

    // now build our output

    Ok(Media {
        media_type: input.media_type,
        file_path: input.file_path,
        output_tempfile: Some((dir, filename)),
    })
}


// create a temporary output file in a tmp folder
fn new_temp_media(extension: &OsStr) -> (TempDir, PathBuf) {
    // make a new file with a random name inside a temp folder
    // ! the TempDir is passed with the path to the file to ensure
    // ! it does not go out of scope, but IDK if there is a better
    // ! way to do this. #TODO: investigate?

    // generate some random numbers to serve as the filename
    let mut file_name: [u8; 5] = [0,0,0,0,0];
    rand::thread_rng().fill(&mut file_name);
    let stringed_name: String = file_name.iter().map(ToString::to_string).collect();

    // now make that temp file
    // Create a directory inside of `std::env::temp_dir()`.
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join(format!("{}.{}", stringed_name, extension.to_str().unwrap()));

    // Done!
    (dir, file_path)
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
        output_tempfile: None,
    };
    let jazz = Media {
        file_path: format!("{}\\src\\test_files\\CC0-jazz-guitar.mp3", srcpath).into(), //TODO: does this work on linux?
        media_type: MediaType::Audio,
        output_tempfile: None,
    };
    let factorio_gif = Media {
        file_path: format!("{}\\src\\test_files\\factorio-test.gif", srcpath).into(), //TODO: does this work on linux?
        media_type: MediaType::Gif,
        output_tempfile: None,
    };
    let video_test = Media {
        file_path: format!("{}\\src\\test_files\\text-video-test.mp4", srcpath).into(), //TODO: does this work on linux?
        media_type: MediaType::Video,
        output_tempfile: None,
    };

    // loop over the test files.
    let testfiles = [baja_cat, jazz, factorio_gif, video_test];
    for i in testfiles {
        let m_type = i.media_type;
        println!("Running {}", i.file_path.display());
        let resize_result = resize_media(i, 128, 128);
        match resize_result {
            Ok(okay) => {
                println!("Got output file at {}", okay.output_tempfile.unwrap().1.display());
            }
            Err(e) => {
                // only the audio file should panic
                if m_type != MediaType::Audio {
                    panic!("{e}")
                }
            }
        }
    }
}
