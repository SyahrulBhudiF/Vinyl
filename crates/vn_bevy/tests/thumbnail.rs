#[test]
fn save_thumbnail_is_320_by_180_png() {
    let source = image::RgbImage::from_pixel(1280, 720, image::Rgb([16, 32, 48]));
    let thumbnail = image::DynamicImage::ImageRgb8(source)
        .resize_to_fill(320, 180, image::imageops::FilterType::Triangle)
        .to_rgb8();
    let mut cursor = std::io::Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut cursor, image::ImageFormat::Png)
        .unwrap();

    let decoded =
        image::load_from_memory_with_format(&cursor.into_inner(), image::ImageFormat::Png).unwrap();
    assert_eq!((decoded.width(), decoded.height()), (320, 180));
}
