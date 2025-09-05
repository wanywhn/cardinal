use objc2::rc::Retained;
use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSImage, NSWorkspace};
use objc2_foundation::{NSData, NSDictionary, NSSize, NSString};

// https://stackoverflow.com/questions/73062803/resizing-nsimage-keeping-aspect-ratio-reducing-the-image-size-while-trying-to-sc
pub fn icon_of_path(path: &str) -> Option<Vec<u8>> {
    objc2::rc::autoreleasepool(|_| -> Option<Vec<u8>> {
        let path_ns = NSString::from_str(path);
        let image = unsafe { NSWorkspace::sharedWorkspace().iconForFile(&path_ns) };

        let png_data: Retained<NSData> = (|| -> Option<_> {
            unsafe {
                // https://stackoverflow.com/questions/66270656/macos-determine-real-size-of-icon-returned-from-iconforfile-method
                for image in image.representations().iter() {
                    let size = image.size();
                    if size.width > 31.0
                        && size.height > 31.0
                        && size.width < 33.0
                        && size.height < 33.0
                    {
                        // println!("representation: {}x{}", size.width, size.height);
                        let new_image = NSImage::imageWithSize_flipped_drawingHandler(
                            NSSize::new(size.width, size.height),
                            false,
                            &block2::RcBlock::new(move |rect| {
                                image.drawInRect(rect);
                                true.into()
                            }),
                        );
                        return Some(
                            NSBitmapImageRep::imageRepWithData(&*new_image.TIFFRepresentation()?)?
                                .representationUsingType_properties(
                                    NSBitmapImageFileType::PNG,
                                    &NSDictionary::new(),
                                )?,
                        );
                    }
                }
            }
            // zoom in and you will see that the small icon in Finder is 32x32, here we keep it at 64x64 for better visibility
            let (new_width, new_height) = unsafe {
                let width = 32.0;
                let height = 32.0;
                // keep aspect ratio
                let old_width = image.size().width;
                let old_height = image.size().height;
                let ratio_x = width / old_width;
                let ratio_y = height / old_height;
                let ratio = if ratio_x < ratio_y { ratio_x } else { ratio_y };
                (old_height * ratio, old_width * ratio)
            };
            unsafe {
                let new_image = NSImage::imageWithSize_flipped_drawingHandler(
                    NSSize::new(new_width, new_height),
                    false,
                    &block2::RcBlock::new(move |rect| {
                        image.drawInRect(rect);
                        true.into()
                    }),
                );
                return Some(
                    NSBitmapImageRep::imageRepWithData(&*new_image.TIFFRepresentation()?)?
                        .representationUsingType_properties(
                            NSBitmapImageFileType::PNG,
                            &NSDictionary::new(),
                        )?,
                );
            }
        })()?;
        Some(png_data.to_vec())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_of_file_normal() {
        let pwd = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let data = icon_of_path(&pwd).unwrap();
        std::fs::write("/tmp/icon.png", data).unwrap();
    }

    #[test]
    #[ignore]
    fn test_icon_of_file_leak() {
        let pwd = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        loop {
            for _ in 0..100 {
                let _data = icon_of_path(&pwd).unwrap();
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
