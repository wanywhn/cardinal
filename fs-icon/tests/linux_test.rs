
#[cfg(target_os = "linux")]
#[test]
fn test_icon_functions_exist() {
    // Just make sure the functions are callable without crashing
    use fs_icon::{icon_of_path, scale_with_aspect_ratio};
    
    // Test scale function
    let (w, h) = scale_with_aspect_ratio(100.0, 50.0, 50.0, 50.0);
    assert_eq!(w, 50.0); // Width should be scaled to 50
    assert_eq!(h, 25.0); // Height should be scaled proportionally to 25
    
    // Test with a path (may return None if no icon is found, but shouldn't crash)
    let _result = icon_of_path("/tmp");
    // Result can be Some or None depending on system, but shouldn't panic
}