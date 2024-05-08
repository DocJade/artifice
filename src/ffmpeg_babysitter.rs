// Make sure ffmpeg isnt silently dying on us.

pub fn ffbabysit(mut baby: ffmpeg_sidecar::child::FfmpegChild) -> Option<crate::Error> {
    // now we shall sit and watch the output stream of the ffmpeg process to see if we get any errors.

    // let the baby do its thing
    let result = baby.wait();
    if result.is_err() {
        // well shoot, we didn't even get to the ffmpeg logs!
        return Some(result.expect_err("have error, so error...").into());
    }

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
        return Some(error_string.into());
    }
    // otherwise, no errors happened! yay!
    None
}
