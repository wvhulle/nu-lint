use super::RULE;

#[test]
fn test_detects_stderr_silenced_for_ffmpeg() {
    RULE.assert_detects("ffmpeg -i input.mp4 output.mp4 e>| ignore");
}

#[test]
fn test_detects_stderr_silenced_for_evtest() {
    RULE.assert_detects("evtest /dev/input/event0 e>| ignore");
}

#[test]
fn test_detects_both_streams_silenced_for_command_with_stderr_data() {
    RULE.assert_detects("ffmpeg -i input.mp4 output.mp4 o+e>| ignore");
}

#[test]
fn test_detects_stderr_silenced_in_pipeline() {
    RULE.assert_detects("ffmpeg -i input.mp4 output.mp4 e>| ignore | print 'done'");
}

#[test]
fn test_detects_in_nested_block() {
    RULE.assert_detects(
        r#"
        do {
            ffmpeg -i input.mp4 output.mp4 e>| ignore
        }
        "#,
    );
}

#[test]
fn test_detects_in_closure() {
    RULE.assert_detects(
        r#"
        let cmd = {|| ffmpeg -i input.mp4 output.mp4 e>| ignore }
        "#,
    );
}
