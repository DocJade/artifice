// media stuff!!

extern crate reqwest;
use std::fs::File;
use std::io::{self, Write};
use std::{ffi::OsStr, path::PathBuf};

use ffmpeg_sidecar::command::FfmpegCommand;

use poise::serenity_prelude::{MessageId, MessagePagination};
use rand::Rng;
use tempfile::TempDir;
use tracing::info;

use crate::ffmpeg_babysitter::ffbabysit;

use crate::Context;

use poise::serenity_prelude::model::prelude::Message;

// this is our main media type, all media is converted into this format before being passed to a function.

#[derive(Debug)]
pub struct Media {
    // The type of the media
    pub media_type: MediaType,
    // the path to the temporary file
    pub file_path: TempFileHolder,
    // output path!
    pub output_tempfile: Option<TempFileHolder>,
}

#[derive(Debug)]
pub struct TempFileHolder {
    pub dir: TempDir,
    pub path: PathBuf
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MediaType {
    Video,
    Gif,
    Image,
    Audio,
    Unknown
}

impl MediaType {
    pub fn from(thingie: String) -> Option<MediaType> {
        // dafuq is this
        // did we get anything?
        if thingie.is_empty() {
            // no stuffs
            return None
        }

        // what kinda mf file is this
        return match thingie.split('/').next().unwrap() {
            "video" => Some(MediaType::Video),
            "audio" => Some(MediaType::Audio),
            "image" => {
                // is this a gif?
                if thingie.split('/').last().unwrap() == "gif" {
                    // yes
                    Some(MediaType::Gif)
                } else {
                    // erm no
                    Some(MediaType::Image)
                }
            },
            other => None
        };
        
    }
}

pub fn resize_media(input: Media, mut x_size: u16, mut y_size: u16) -> Result<Media, crate::Error> {
    // This function takes in a media file, and resizes it to be of certain dimensions.

    // TODO: fix transparency on gifs (currently adds a white background)

    // h264 gets very angry if the sizes are not divisible by 2, therefore we must check

    // make sure that sucker is even
    if (x_size % 2) == 1 {
        //odd! add one.
        x_size += 1;
    }
    if y_size != 0 && (y_size % 2) == 1 {
        //odd! add one.
        x_size += 1;
    }

    // if the x size is 0, we will automatically rescale using the y size and aspect ratio.

    if x_size == 0 {
        // need to calculate new size.
        // get the size of input media
        let (x_media_size, y_media_size): (i64, i64) = get_pixel_size(&input)?;
        let aspect_ratio: f32 = x_media_size as f32 / y_media_size as f32;

        // now multiply and round to get our new y size
        x_size = (aspect_ratio * y_size as f32).round() as u16;
        // make sure that sucker is even
        if (x_size % 2) == 1 {
            //odd! add one.
            x_size += 1;
        }
    };

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
    let extension = input.file_path.path.extension().unwrap();

    // create a tempfile to store the output.
    let dir = new_temp_media(extension);

    // Do the actual resizing.
    // every arg gets a separate line for readability instead of an array.

    let output = FfmpegCommand::new()
        .hwaccel(std::env::var("HW_ACCEL").unwrap_or("none".to_string()))
        .input(input.file_path.path.as_path().to_str().unwrap()) // input file
        .args([
            // set the dimensions
            "-vf",
            &format!("scale={}:{}", x_size, y_size),
        ])
        .codec_audio("copy") // copy audio codec
        //.output(tempfile_path.to_str().unwrap()) // where is it going?
        .output(dir.path.to_str().unwrap())
        .spawn()
        .unwrap(); // run that sucker

    // wait for that to finish
    ffbabysit(output)?;

    // now build our output

    Ok(Media {
        media_type: input.media_type,
        file_path: input.file_path,
        output_tempfile: Some(TempFileHolder{
            dir: dir.dir, // durrrrr
            path: dir.path,
        }),
    })
}

// create a temporary output file in a tmp folder
pub fn new_temp_media(extension: &OsStr) -> TempFileHolder {
    // make a new file with a random name inside a temp folder
    // ! the TempDir is passed with the path to the file to ensure
    // ! it does not go out of scope, but IDK if there is a better
    // ! way to do this. #TODO: investigate?

    // generate some random numbers to serve as the filename
    let mut file_name: [u8; 5] = [0, 0, 0, 0, 0];
    rand::thread_rng().fill(&mut file_name);
    let stringed_name: String = file_name.iter().map(ToString::to_string).collect();

    // now make that temp file
    // Create a directory inside of `std::env::temp_dir()`.
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir
        .path()
        .join(format!("{}.{}", stringed_name, extension.to_str().unwrap()));

    // Done!
    TempFileHolder {
        dir,
        path: file_path
    }
}

// create a temporary folder by itself.
// just a shortcut.
pub fn new_temp_dir() -> TempDir {
    return tempfile::tempdir().unwrap();
}

/// get the screen size of a media file
/// returns (x,y)
pub fn get_pixel_size(input: &Media) -> Result<(i64, i64), crate::Error> {
    // ask ffprobe
    let media_info = match ffprobe::ffprobe(&input.file_path.path) {
        // all good
        Ok(info) => info,
        // something broke
        Err(err) => {
            tracing::info!("ffprobe size failed!");
            return Err(format!("{:#?}", err).to_string().into());
        }
    };

    // now pull out the x and y

    let size_x: i64 = media_info.streams[0].width.expect("Media had unknown x size!");
    let size_y: i64 = media_info.streams[0].height.expect("Media had unknown y size!");


    Ok((size_x,size_y))
}

// looks for a media file in the chat history.
pub async fn find_media(ctx: Context<'_>) -> crate::Result<Option<Media>> {
    // TODO: gifs from tenor.
    info!("Looking for media...");
    // now we shall take that mf context and look for some media
    let channel_id: poise::serenity_prelude::model::prelude::ChannelId = ctx.channel_id();
    let http = ctx.http();
    // get the message id that this context came from
    let start: MessageId = ctx.id().into();

    let search_params: MessagePagination = MessagePagination::Before(start);
    // get the last 50 messages
    let messages: Vec<Message> = http.get_messages(channel_id, Some(search_params), Some(50)).await?; // TODO: is this too many messages????!?!??!
    info!("Got 50 messages...");

    let mut url: String = String::new();
    let mut media_type: MediaType = MediaType::Unknown;

    // do these messages have media tho
    for msg in messages {
        if msg.attachments.len() == 1 {
            info!("Found a message with media!");
            // wowie boys we got medias
            let find_type = msg.attachments.first().unwrap().content_type.as_ref().unwrap().clone();
            info!("{find_type}");

            // is this a thing we can actually use
            let maybe_media_type = MediaType::from(find_type);
            // can we use it?
            if maybe_media_type.is_none() {
                // nuh uh
                info!("...but it was something we couldn't use.");
                continue;
            }
            info!("Good media file!");
            // cool we can use it!
            media_type = maybe_media_type.unwrap();
            url = msg.attachments.first().unwrap().url.clone();
            break
        }
    }
    info!("Finished looping over messages...");

    // got anything?
    if url.is_empty() || media_type == MediaType::Unknown {
        // no :(
            info!("didn't find anything.");
        return Ok(None)
    }

    // ok cool we got a media, time to download it

    // first, get a folder to store it
    let tempdir = new_temp_dir();

    // now actually download the file
    info!("Downloading a media file...");
    let resp = reqwest::get(url.clone()).await?;

    // split out the filename
    // TODO: cleanup?
    let filename = url
    .split('/')
    .last()
    .unwrap()
    .split('?')
    .next()
    .unwrap();

    // Create the target file path within the folder
    let file_path = tempdir.path().join(filename);
    info!("{}",file_path.to_str().unwrap());
    
    // Get the response body as bytes
    let data = resp.bytes().await?;
    
    // Open the target file for writing
    let mut out = File::create(file_path.clone())?;
    
    // Copy downloaded data to the file
    out.write_all(&data)?;
    
    // in theory we have the file now.
    info!("Finished downloading {}", filename);

    // return that sucker!
    Ok(Some(Media { media_type, file_path: TempFileHolder { dir: tempdir, path: file_path.into() }, output_tempfile: None }))
}

#[test]
fn resize_test() {
    ffmpeg_sidecar::download::auto_download().unwrap();
    //get current path to src
    let srcpath = env!("CARGO_MANIFEST_DIR");
    // try resizing a few things
    let baja_cat = Media {
        file_path: TempFileHolder{ dir: TempDir::new().unwrap(), path: format!("{}/src/test_files/bajacat.png", srcpath).into() },
        media_type: MediaType::Image,
        output_tempfile: None,
    };
    let jazz = Media {
        file_path: TempFileHolder{ dir: TempDir::new().unwrap(), path: format!("{}/src/test_files/CC0-jazz-guitar.mp3", srcpath).into() },
        media_type: MediaType::Audio,
        output_tempfile: None,
    };
    let factorio_gif = Media {
        file_path: TempFileHolder{ dir: TempDir::new().unwrap(), path: format!("{}/src/test_files/factorio-test.gif", srcpath).into() },
        media_type: MediaType::Gif,
        output_tempfile: None,
    };
    let video_test = Media {
        file_path: TempFileHolder{ dir: TempDir::new().unwrap(), path: format!("{}/src/test_files/text-video-test.mp4", srcpath).into() },
        media_type: MediaType::Video,
        output_tempfile: None,
    };

    // loop over the test files.
    let testfiles = [baja_cat, jazz, factorio_gif, video_test];
    for i in testfiles {
        let m_type = i.media_type;
        println!("Running {}", i.file_path.path.display());
        let resize_result = resize_media(i, 128, 128);
        match resize_result {
            Ok(okay) => {
                println!(
                    "Got output file at {}",
                    okay.output_tempfile.unwrap().path.display()
                );
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

#[test]
fn auto_resize_test() {
    ffmpeg_sidecar::download::auto_download().unwrap();
    //get current path to src
    let srcpath = env!("CARGO_MANIFEST_DIR");
    // try resizing a few things
    let baja_cat = Media {
        file_path: TempFileHolder{ dir: TempDir::new().unwrap(), path: format!("{}/src/test_files/bajacat.png", srcpath).into() },
        media_type: MediaType::Image,
        output_tempfile: None,
    };
    // let jazz = Media {
    //     file_path: TempFileHolder{ dir: TempDir::new().unwrap(), path: format!("{}/src/test_files/CC0-jazz-guitar.mp3", srcpath).into() },
    //     media_type: MediaType::Audio,
    //     output_tempfile: None,
    // };
    let factorio_gif = Media {
        file_path: TempFileHolder{ dir: TempDir::new().unwrap(), path: format!("{}/src/test_files/factorio-test.gif", srcpath).into() },
        media_type: MediaType::Gif,
        output_tempfile: None,
    };
    let video_test = Media {
        file_path: TempFileHolder{ dir: TempDir::new().unwrap(), path: format!("{}/src/test_files/text-video-test.mp4", srcpath).into() },
        media_type: MediaType::Video,
        output_tempfile: None,
    };

    // loop over the test files.
    let testfiles = [baja_cat, /*jazz,*/ factorio_gif, video_test];
    for i in testfiles {
        let m_type = i.media_type;
        println!("Running {}", i.file_path.path.display());
        let resize_result = resize_media(i, 167, 0);
        match resize_result {
            Ok(okay) => {
                println!(
                    "Got output file at {}",
                    okay.output_tempfile.unwrap().path.display()
                );
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