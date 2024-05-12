// Make sure ffmpeg isnt silently dying on us.

pub fn ffbabysit(mut baby: ffmpeg_sidecar::child::FfmpegChild) -> crate::Result {
    // now we shall sit and watch the output stream of the ffmpeg process to see if we get any errors.

    // let the baby do its thing
    let _result = baby.wait()?;
    // now inquire, and just keep the errors.
    let possible_errors: Vec<String> = baby.iter().unwrap().filter_errors().collect();

    // did any errors happen?
    if !possible_errors.is_empty() {
        // an error happened somewhere,
        // let's try to make it look at least presentable
        let mut error_string = String::new();
        for error in possible_errors {
            error_string.push_str(&error);
            error_string.push('\n');
        }
        return Err(error_string.into());
    }
    // otherwise, no errors happened! yay!
    Ok(())
}
