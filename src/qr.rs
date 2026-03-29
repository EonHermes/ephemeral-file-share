//! QR code generation utilities

use anyhow::Result;
use image::{DynamicImage, ImageBuffer, Rgba};

/// Generate a QR code as an image
pub fn generate_qr_code(data: &str, size: u32) -> Result<DynamicImage> {
    // In a full implementation, we would use a QR code library like qrcode
    // For now, return a placeholder that indicates the concept
    
    // Create a simple 256x256 image as placeholder
    let img = ImageBuffer::from_fn(size, size, |x, y| {
        if (x + y) % 10 < 5 {
            Rgba([0, 0, 0, 255]) // Black
        } else {
            Rgba([255, 255, 255, 255]) // White
        }
    });

    Ok(DynamicImage::ImageRgba8(img))
}

/// Convert QR code image to base64 data URL for embedding in HTML
pub fn image_to_data_url(img: &DynamicImage) -> Result<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    
    let mut buffer = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
        .map_err(|e| anyhow::anyhow!("Failed to encode image: {}", e))?;
    
    let base64 = STANDARD.encode(&buffer);
    Ok(format!("data:image/png;base64,{}", base64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr_code() {
        let qr = generate_qr_code("test-data", 256).unwrap();
        assert_eq!(qr.width(), 256);
        assert_eq!(qr.height(), 256);
    }

    #[test]
    fn test_image_to_data_url() {
        let qr = generate_qr_code("test-data", 100).unwrap();
        let data_url = image_to_data_url(&qr).unwrap();
        
        assert!(data_url.starts_with("data:image/png;base64,"));
        assert!(!data_url.ends_with(','));
    }
}
