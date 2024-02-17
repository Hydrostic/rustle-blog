use strum_macros::IntoStaticStr;

#[derive(IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum ImageExt{
    Jpeg,
    Png,
    Gif,
    Bmp,
    Webp,
    Tiff,
    Unknown
}

pub fn image_sniff(data: [u8;8]) -> ImageExt{
    if data[..2] == [0xFF, 0xD8] {
        ImageExt::Jpeg
    } else if data[..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]{
        ImageExt::Png
    } else if data[..4] == [0x47, 0x49, 0x46, 0x38]{
        ImageExt::Gif
    } else if data[..2] == [0x42, 0x4D]{
        ImageExt::Bmp
    } else if data[..4] == [0x52, 0x49, 0x46, 0x46]{
        ImageExt::Webp
    } else if data[..4] == [0x49, 0x49, 0x2A, 0x00]{
        ImageExt::Tiff
    } else if data[..4] == [0x4D, 0x4D, 0x00, 0x2A]{
        ImageExt::Tiff
    } else {
        ImageExt::Unknown
    }
}