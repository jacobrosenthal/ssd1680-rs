/// Display rotation
#[derive(Clone, Copy, Debug)]
pub enum DisplayRotation {
    /// No rotation, normal display
    Rotate0,
    /// Rotate by 90 degrees clockwise
    Rotate90,
    /// Rotate by 180 degrees clockwise
    Rotate180,
    /// Rotate 270 degrees clockwise
    Rotate270,
}

pub fn find_rotation(
    x: u32,
    y: u32,
    height: u32,
    width: u32,
    rotation: DisplayRotation,
) -> (u32, u32) {
    let nx;
    let ny;
    match rotation {
        DisplayRotation::Rotate0 => {
            nx = x;
            ny = y;
        }
        DisplayRotation::Rotate90 => {
            nx = y;
            ny = width - 1 - x;
        }
        DisplayRotation::Rotate180 => {
            nx = height - 1 - x;
            ny = width - 1 - y;
        }
        DisplayRotation::Rotate270 => {
            nx = width - 1 - y;
            ny = x;
        }
    }
    (nx, ny)
}
