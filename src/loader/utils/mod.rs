//! Useful functionalities for loading FBX.


/// Applies the given function to each polygon.
///
/// Returns `Ok(n)` on success where `n` is the number of not completed vrtices at pvi's tail,
/// returns `Err(e)` if `f` returned `Err(_)`.
///
/// If `f` returned `Err(_)`, the loop will no longer be continued.
///
/// This can be used for polygon vertex indices of FBX 7.4 or later.
pub fn for_each_polygon<E, F>(pvi: &[i32], mut f: F) -> Result<usize, E>
where
    F: FnMut(&[u32]) -> Result<(), E>,
{
    let mut polys = Vec::new();
    let mut pvi_iter = pvi.into_iter();
    'all: loop {
        let mut polygon_closed = false;
        'get_poly: for &i in &mut pvi_iter {
            if i < 0 {
                polys.push(!i as u32);
                polygon_closed = true;
                break 'get_poly;
            } else {
                polys.push(i as u32);
            }
        }
        // Note that if `polys.is_empty() == true` then `polygon_closed == false`.
        if !polygon_closed {
            // Polygon is not closed (i.e. the last pvi is not negative) or there are no more pvi.
            break 'all;
        }
        f(&polys)?;
        polys.clear();
    }
    Ok(polys.len())
}
