// generate captions for images

use std::ffi::OsStr;

use ab_glyph::*;
use ffmpeg_sidecar::command::FfmpegCommand;
use glyph_brush_layout::{FontId, GlyphPositioner, SectionGeometry};
use image::{imageops, DynamicImage, ImageBuffer, Rgba};

use crate::{
    ffmpeg_babysitter::ffbabysit,
    media_helpers::{get_pixel_size, new_temp_media, Media, MediaType},
};

pub fn caption_media(
    input_text: String,
    media: Media,
    bottom: bool,
    text_color: (u8, u8, u8),
    bg_color: (u8, u8, u8),
) -> Result<Media, crate::Error> {
    // creates and adds a caption to every item in the media.

    // are we trying to caption audio?

    if media.media_type == MediaType::Audio {
        // we cant caption audio.
        return Err("Cannot caption a audio file.".into());
    }

    // make sure the text super long
    const MAX_LEN: usize = 500;
    if input_text.len() > MAX_LEN {
        return Err(format!("Caption cannot be longer than {} characters.", MAX_LEN).into());
    }

    // get the size of the main image, so we can determine how wide our caption needs to be
    let (media_x_res, _media_y_res): (i64, i64) = get_pixel_size(&media)?;

    // load in the font
    // TODO: multiple font selection
    let font_open_sans_bold =
        FontRef::try_from_slice(include_bytes!("fonts/open-sans/OpenSans-Bold.ttf"))?;
    let fonts = &[font_open_sans_bold];

    // now we will layout the text

    // calculate the font size.
    // font size is based on either the width of the image.
    let font_size: f32 = (media_x_res as f32 / 12.0).floor();
    tracing::info!("Font size is {},", font_size);

    // make a text section
    let text_section = glyph_brush_layout::SectionText {
        text: &input_text,
        scale: PxScale::from(font_size),
        font_id: FontId(0), // first font in the font array
    };

    // since the caption is going to be in the middle, we need to know where that is
    let width_center = (media_x_res as f32 / 2.0).ceil();

    // now calculate padding
    // padding is based on font size.
    let vertical_padding: i64 = (font_size * 0.5) as i64;
    // and horiz is based on vert.
    let horizontal_padding: i64 = (vertical_padding as f32 / 2.0) as i64; // is this a pointless cast? idk lmao

    // use that to calculate caption image size, by subbing from main image size.
    let caption_geometry_width: f32 = media_x_res as f32 - (horizontal_padding as f32 * 2.0);

    // set the caption size
    // caption can be as tall as it needs to be
    let caption_geometry = SectionGeometry {
        screen_position: (width_center, 0.0), // center, no padding on top yet TODO:
        bounds: (caption_geometry_width, media_x_res as f32), // set the max width of the caption to be the same as the image width,
                                                              // since we will be actually calculating the size of the image later.
    };

    let glyphs_pre_cal = glyph_brush_layout::Layout::default_wrap()
        .h_align(glyph_brush_layout::HorizontalAlign::Center); // in the middle please.

    // now get the size of the layout
    let layout_size: Rect = glyphs_pre_cal.bounds_rect(&caption_geometry);

    // and finish laying out
    let final_layout = glyphs_pre_cal.calculate_glyphs(fonts, &caption_geometry, &[text_section]);

    // calculate the total height based off of the glyphs
    let mut finding_height: f32 = 0.0;
    for section in final_layout.clone() {
        if let Some(outline) = fonts[section.font_id].outline_glyph(section.glyph) {
            let top = outline.px_bounds().max.y;
            if top > finding_height {
                finding_height = top;
            }
        }
    }

    assert!(
        layout_size.min.x >= 0.0,
        "erm, the min >=0 ! {}",
        layout_size.min.x
    );
    let layout_size_height = finding_height.ceil() as u32;

    // now draw it onto a canvas!
    tracing::info!(
        "Creating image of size {}, {}",
        media_x_res,
        layout_size_height
    );
    let mut caption_image: image::ImageBuffer<Rgba<u8>, Vec<u8>> =
        DynamicImage::new_rgba8(media_x_res.try_into().unwrap(), layout_size_height).to_rgba8();

    // render each letter / glyph
    for section in final_layout {
        // grab the font and compute the path (?)
        if let Some(outline) = fonts[section.font_id].outline_glyph(section.glyph) {
            // let x_offset = section.glyph.position.x as u32;
            // let y_offset = section.glyph.position.y as u32;
            let bounding_box = outline.px_bounds();

            // now actually draw onto the image
            outline.draw(|x, y, coverage| {
                caption_image.put_pixel(
                    // now we need to get the offsets into the image for where to draw this glyph
                    // so we dont just draw on top of ourselves for every character
                    x + bounding_box.min.x as u32,
                    y + bounding_box.min.y as u32,
                    // Turn the coverage into an alpha value
                    Rgba([
                        text_color.0,
                        text_color.1,
                        text_color.2,
                        (coverage * 255.0) as u8,
                    ]),
                )
            });
        };
    }

    // Now that's just the text, throw that on top of a white background.

    // need to add vert padding to vert size, and just set width to image input size

    let bg_color: image::Rgba<u8> = image::Rgba([bg_color.0, bg_color.1, bg_color.2, 255]);
    let mut final_image = ImageBuffer::from_pixel(
        media_x_res as u32,
        caption_image.height() + (vertical_padding * 2) as u32,
        bg_color,
    );

    // find center, since we add padding.
    let caption_image_horiz_center: i64 =
        ((final_image.width() / 2) - (caption_image.width() / 2)).into();
    let caption_image_vert_center: i64 =
        ((final_image.height() / 2) - (caption_image.height() / 2)).into();

    imageops::overlay(
        &mut final_image,
        &caption_image,
        caption_image_horiz_center,
        caption_image_vert_center,
    );

    // now save that image to the disk, then we can stack it on top of our media.

    let pre_extension: &str = "png";
    let caption_extension: &OsStr = OsStr::new(pre_extension);

    // create a temp file to store the output.
    let temp_caption_location = new_temp_media(caption_extension);

    // save the image there
    final_image.save(&temp_caption_location.path).unwrap();

    // now stack it with ffmpeg

    // create a temp file to store the output from _ffmpeg_
    let ffmpeg_extension = media.file_path.path.extension().unwrap();
    let temp_ffmpeg_location = new_temp_media(ffmpeg_extension);

    // now stack the image with ffmpeg

    // put the two paths into a vec, we need to reverse them if this is a bottom caption
    let mut inputs: Vec<&str> = vec![
        temp_caption_location.path.as_path().to_str().unwrap(),
        media.file_path.path.as_path().to_str().unwrap(),
    ];

    // is this a bottom caption?
    if bottom {
        // swap em
        inputs.reverse();
    }

    tracing::info!("Applying caption to media...");

    let output = FfmpegCommand::new()
        .hwaccel(std::env::var("HW_ACCEL").unwrap_or("none".to_string()))
        .input(inputs[0])
        .input(inputs[1])
        .args([
            // stack the media
            "-filter_complex",
            "vstack=inputs=2",
        ])
        .codec_audio("copy") // copy audio codec
        //.output(tempfile_path.to_str().unwrap()) // where is it going?
        .output(temp_ffmpeg_location.path.to_str().unwrap())
        .spawn()
        .unwrap(); // run that sucker

    // wait for that to finish
    ffbabysit(output)?;
    // now build our output!
    tracing::info!("Done!");
    Ok(Media {
        media_type: media.media_type,
        file_path: media.file_path,
        output_tempfile: Some(temp_ffmpeg_location), //Some((dir, ffmpeg_filename)),
    })
}
// #TODO: move testing to reduce duplication
#[test]
fn caption_test() {
    use crate::media_helpers::TempFileHolder;
    use tempfile::TempDir;
    ffmpeg_sidecar::download::auto_download().unwrap();
    //get current path to src
    let src_path = env!("CARGO_MANIFEST_DIR");

    // caption some stuff!
    let baja_cat = Media {
        file_path: TempFileHolder {
            dir: TempDir::new().unwrap(),
            path: format!("{}/src/test_files/bajacat.png", src_path).into(),
        },
        media_type: MediaType::Image,
        output_tempfile: None,
    };
    let jazz = Media {
        file_path: TempFileHolder {
            dir: TempDir::new().unwrap(),
            path: format!("{}/src/test_files/CC0-jazz-guitar.mp3", src_path).into(),
        },
        media_type: MediaType::Audio,
        output_tempfile: None,
    };
    let factorio_gif = Media {
        file_path: TempFileHolder {
            dir: TempDir::new().unwrap(),
            path: format!("{}/src/test_files/factorio-test.gif", src_path).into(),
        },
        media_type: MediaType::Gif,
        output_tempfile: None,
    };
    let video_test = Media {
        file_path: TempFileHolder {
            dir: TempDir::new().unwrap(),
            path: format!("{}/src/test_files/text-video-test.mp4", src_path).into(),
        },
        media_type: MediaType::Video,
        output_tempfile: None,
    };

    // loop over the test files.
    let test_files = [baja_cat, jazz, factorio_gif, video_test];
    for i in test_files {
        let m_type = i.media_type;
        println!("Running {}", i.file_path.path.display());
        let caption_result = caption_media(
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
