//! `dilate_trimming_region` + `seed_fill_block` (`SupportMaterial.cpp:980-1040`).

/// `dilate_trimming_region` (`SupportMaterial.cpp:980-999`): 8-neighborhood
/// AND across the 3 rows above/at/below — a cell stays set only if all 9
/// neighbours in `trimming` are set. Applied to the trimming mask before
/// `seed_fill_block`.
pub(crate) fn dilate_trimming_region(trimming: &[u8], grid_w: usize, grid_h: usize) -> Vec<u8> {
    let mut dilated = vec![0u8; trimming.len()];
    for r in 1..grid_h - 1 {
        for c in 1..grid_w - 1 {
            let mut addr = c + (r - 1) * grid_w;
            let mut b = trimming[addr - 1] != 0 && trimming[addr] != 0 && trimming[addr + 1] != 0;
            addr += grid_w;
            b = b && trimming[addr - 1] != 0 && trimming[addr] != 0 && trimming[addr + 1] != 0;
            addr += grid_w;
            b = b && trimming[addr - 1] != 0 && trimming[addr] != 0 && trimming[addr + 1] != 0;
            dilated[addr - grid_w] = b as u8;
        }
    }
    dilated
}

/// `seed_fill_block` (`SupportMaterial.cpp:1001-1040`): propagate the
/// rasterized support pixels over each `(oversampling x oversampling)` macro
/// block up to the dilated trimming mask.
#[allow(clippy::too_many_arguments)]
pub(crate) fn seed_fill_block(
    grid: &mut [u8],
    grid_w: usize,
    trimming: &[u8],
    grid_blocks_x: usize,
    grid_blocks_y: usize,
    oversampling: usize,
) {
    let size = oversampling;
    let stride = grid_w;
    for block_r in 0..grid_blocks_y {
        for block_c in 0..grid_blocks_x {
            let base = block_c * size + 1 + (block_r * size + 1) * stride;
            // Top to bottom propagation.
            for r in 0..size {
                if r > 0 {
                    for c in 0..size {
                        propagate(grid, trimming, base, r, c, stride, -(stride as isize));
                    }
                }
                for c in 1..size {
                    propagate(grid, trimming, base, r, c, stride, -1);
                }
                for c in (0..size - 1).rev() {
                    propagate(grid, trimming, base, r, c, stride, 1);
                }
            }
            // Bottom to top propagation.
            for r in (0..size - 1).rev() {
                for c in 0..size {
                    propagate(grid, trimming, base, r, c, stride, stride as isize);
                }
                for c in 1..size {
                    propagate(grid, trimming, base, r, c, stride, -1);
                }
                for c in (0..size - 1).rev() {
                    propagate(grid, trimming, base, r, c, stride, 1);
                }
            }
        }
    }
}

/// `PROPAGATION_STEP` (`:1017-1023`): copy a filled neighbour cell into an
/// un-filled, un-trimmed cell.
fn propagate(
    grid: &mut [u8],
    trimming: &[u8],
    base: usize,
    r: usize,
    c: usize,
    stride: usize,
    offset: isize,
) {
    let addr = base + r * stride + c;
    let addr2 = (addr as isize + offset) as usize;
    if grid[addr2] != 0 && trimming[addr] == 0 && trimming[addr2] == 0 {
        grid[addr] = 1;
    }
}
