// generate captions for images

use std::ffi::OsStr;

use ffmpeg_sidecar::command::FfmpegCommand;
use image::{imageops, DynamicImage, ImageBuffer, Rgba};
use rusttype::{point, Font, PositionedGlyph, Scale};

use crate::{
    ffmpeg_babysitter::ffbabysit,
    media_helpers::{new_temp_media, Media, MediaType},
};

pub fn caption(
    text: String,
    media: Media,
    bottom: bool,
    text_color: (u8, u8, u8),
    bg_color: (u8, u8, u8),
) -> Result<Media, crate::Error> {
    // creates and adds a caption to every item in the media.

    if media.media_type == MediaType::Audio {
        // we cant caption audio.
        return Err("Cannot caption a audio file.".into());
    }

    // make sure the text isnt stupidly long
    const MAX_LEN: usize = 500;
    if text.len() > MAX_LEN {
        return Err(format!("Caption cannot be longer than {} characters.", MAX_LEN).into());
    }

    // now get the size of the media

    // ask ffprobe

    let media_info = match ffprobe::ffprobe(&media.file_path) {
        // all good
        Ok(info) => info,
        // something broke
        Err(err) => {
            println!("ffprobe failed!");
            return Err(format!("{:#?}", err).to_string().into());
        }
    };

    let media_x_res = media_info.streams[0]
        .coded_width
        .expect("File had no width?");
    // let y_res = media_info.streams[0].coded_height.expect("File had no height?");

    // now process our text to wrap it into lines.

    // based on our set font size, let's calculate how many characters wide our lines should be

    // const FONT_SIZE: u8 = 32;

    // font size is based on the width of the image.
    let font_size = media_x_res / 16;

    let left_right_padding = media_x_res / 20;
    let top_bottom_padding = media_x_res / 30;

    // how much width can we use?
    let workable_width = media_x_res - (left_right_padding * 2) as i64;

    // how many characters wide are we going to make our text?
    // this math was made up on the spot.
    // let workable_character_width: i64 = (workable_width as f32 / (font_size as f32 / 2.5)) as i64;
    let workable_character_width: i64 = (workable_width / font_size) * 3; // ???

    //TODO: smarter character math? or just test it with a bunch of images.
    // make it also take into account the length of the strings? so single word captions arent too small.

    // make sure we have a reasonable amount of room
    if workable_character_width < 10 {
        // less than 10 characters wide is wild. no thanks.
        //TODO: In the future, this should rescale the image instead of failing, make it 2x wider or something
        // TODO: It will recurse into self, so:
        // run resize to resize image (2x? 4x?) then call itself again.

        return Err("Not wide enough to caption.".into());
    }

    let wrapped_text = textwrap::wrap(&text, workable_character_width as usize);

    // now we need to make an image for each one of these rows...

    //TODO: this font is hardcoded, font selection in the future?
    let font_data = include_bytes!("../fonts/open-sans/OpenSans-Bold.ttf");
    // open the font
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Could not load font.");

    // set up the font scale
    let scale: Scale = Scale::uniform(font_size as f32);

    // no idea what this does, guessing its height related.
    let v_metrics = font.v_metrics(scale);

    // now we will loop over every line of text and lay it out

    // laid out glyphs
    let mut layouts: Vec<Vec<PositionedGlyph>> = vec![];

    for line in wrapped_text {
        // idk how cows work rofl
        let text: String = line.into_owned();

        // rustfont has a nice auto-layout thing, but its only for one line at a time
        let laid: Vec<PositionedGlyph> = font
            .layout(&text, scale, point(0.0, 0.0 + v_metrics.ascent))
            .collect();

        // done.
        layouts.push(laid);
    }

    // now we need to know how big each of those layouts is
    // width, height
    let mut layout_sizes: Vec<(u32, u32)> = vec![];

    for layout in &layouts {
        // borrowed with love from
        // https://gitlab.redox-os.org/redox-os/rusttype/-/blob/master/dev/examples/image.rs
        let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
        let glyphs_width = {
            let min_x = layout
                .first()
                .map(|g| g.pixel_bounding_box().unwrap().min.x)
                .unwrap();
            let max_x = layout
                .last()
                .map(|g| g.pixel_bounding_box().unwrap().max.x)
                .unwrap();
            (max_x - min_x) as u32
        };
        layout_sizes.push((glyphs_width, glyphs_height))
    }

    // okay now that we know the size of each layout, we can calculate where the center of each line is
    // might have to come back here for vertical centering, we'll see.
    let mut layout_centers: Vec<u32> = vec![];

    for i in &layout_sizes {
        layout_centers.push(i.0 / 2);
    }

    // now we need to make images for every row, yay.

    let mut line_images: Vec<image::ImageBuffer<Rgba<u8>, Vec<u8>>> = vec![];
    for i in 0..layouts.len() {
        // now make an image of the correct size.
        let layout = &layouts[i];
        let (size_x, size_y) = layout_sizes[i];
        // let center = layout_centers[i];

        // create the image we are going to render into.
        let mut image: image::ImageBuffer<Rgba<u8>, Vec<u8>> =
            DynamicImage::new_rgba8(size_x + 20, size_y + 20).to_rgba8(); // 20px of padding, because the size calculation isn't 100% accurate.

        // stolem!
        // Loop through the glyphs in the text, positing each one on a line
        for glyph in layout {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                // Draw the glyph into the image per-pixel by using the draw closure
                glyph.draw(|x, y, v| {
                    image.put_pixel(
                        // Offset the position by the glyph bounding box
                        x + bounding_box.min.x as u32,
                        y + bounding_box.min.y as u32,
                        // Turn the coverage into an alpha value
                        Rgba([text_color.0, text_color.1, text_color.2, (v * 255.0) as u8]),
                    )
                });
            }
        }

        // now the image should be rendered in. NEXT!
        line_images.push(image);
    }

    // now with all of the images, we need to stack them

    // calculate the size for the new image

    // get widest item in the list, add padding
    // let main_w = layout_sizes.iter().map(|x| x.0).max().unwrap() + (SIDE_PADDING*2) as u32;\

    // actually we dont care, just make it the width of the input
    let main_w = media_x_res as u32;

    // total height, add padding as well.
    let main_h: u32 =
        layout_sizes.iter().map(|y| y.1).sum::<u32>() + (top_bottom_padding * 2) as u32;

    // make the background image with the chosen color
    let white: image::Rgba<u8> = image::Rgba([bg_color.0, bg_color.1, bg_color.2, 255]);
    let mut caption_image = ImageBuffer::from_pixel(main_w, main_h, white);

    // now add the images. making sure to center them.

    // middle of new big image
    let big_middle = main_w / 2;

    // current height offset
    let mut height_in: u32 = left_right_padding as u32; // start with a little padding.

    for i in 0..line_images.len() {
        // place the image
        // get the centered alignment
        let centered = big_middle - layout_centers[i];
        imageops::overlay(
            &mut caption_image,
            &line_images[i],
            centered.into(),
            height_in.into(),
        );
        // now increment the height in by the height of the image
        height_in += layout_sizes[i].1;
    }

    // now we need to stack the image on top of the original source media

    //ffmpeg time!

    // first we need to save our caption somewhere so we can feed it to ffmpeg.
    // caption
    let pre_extension: &str = "png";
    let caption_extension: &OsStr = OsStr::new(pre_extension);

    // create a temp file to store the output.
    let (_caption_dir, temp_caption_filename) = new_temp_media(caption_extension);

    // save the image there
    caption_image.save(&temp_caption_filename).unwrap();

    // create a temp file to store the output from _ffmpeg_
    let ffmpeg_extension = media.file_path.extension().unwrap();
    let (dir, ffmpeg_filename) = new_temp_media(ffmpeg_extension);

    // now stack the image with ffmpeg

    // put the two paths into a vec, we need to reverse them if this is a bottom caption
    let mut inputs: Vec<&str> = vec![
        temp_caption_filename.as_path().to_str().unwrap(),
        media.file_path.as_path().to_str().unwrap(),
    ];

    // is this a bottom caption?
    if bottom {
        // swap em
        inputs.reverse();
    }

    let output = FfmpegCommand::new()
        .hwaccel("auto")
        .input(inputs[0])
        .input(inputs[1])
        .args([
            // stack the media
            "-filter_complex",
            "vstack=inputs=2",
        ])
        .codec_audio("copy") // copy audio codec
        //.output(tempfile_path.to_str().unwrap()) // where is it going?
        .output(ffmpeg_filename.to_str().unwrap())
        .spawn()
        .unwrap(); // run that sucker

    // wait for that to finish
    ffbabysit(output)?;
    // now build our output!
    Ok(Media {
        media_type: media.media_type,
        file_path: media.file_path,
        output_tempfile: Some((dir, ffmpeg_filename)),
    })
}

#[test]
fn caption_test() {
    ffmpeg_sidecar::download::auto_download().unwrap();
    //get current path to src
    let src_path = env!("CARGO_MANIFEST_DIR");

    // caption some stuff!
    let baja_cat = Media {
        file_path: format!("{}/src/test_files/bajacat.png", src_path).into(),
        media_type: MediaType::Image,
        output_tempfile: None,
    };
    let jazz = Media {
        file_path: format!("{}/src/test_files/CC0-jazz-guitar.mp3", src_path).into(),
        media_type: MediaType::Audio,
        output_tempfile: None,
    };
    let factorio_gif = Media {
        file_path: format!("{}/src/test_files/factorio-test.gif", src_path).into(),
        media_type: MediaType::Gif,
        output_tempfile: None,
    };
    let video_test = Media {
        file_path: format!("{}/src/test_files/text-video-test.mp4", src_path).into(),
        media_type: MediaType::Video,
        output_tempfile: None,
    };

    // loop over the test files.
    let test_files = [baja_cat, jazz, factorio_gif, video_test];
    for i in test_files {
        let m_type = i.media_type;
        println!("Running {}", i.file_path.display());
        let caption_result = caption(
            "This is a test caption.".to_string(),
            i,
            false,
            (0, 0, 0),
            (255, 255, 255),
        );
        match caption_result {
            Ok(okay) => {
                println!(
                    "Got output file at {}",
                    okay.output_tempfile.unwrap().1.display()
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
