//! Target- and feature-gated parallel iteration matching upstream `tbb::parallel_for`
//! loops with cooperative [`CancelToken`] short-circuiting.

use crate::{CancelToken, SliceError};

pub(crate) fn par_map_try<T, U, F>(
    items: Vec<T>,
    cancel: &CancelToken,
    f: F,
) -> Result<Vec<U>, SliceError>
where
    T: Send,
    U: Send,
    F: Fn(T) -> Result<U, SliceError> + Sync + Send,
{
    cancel.check()?;
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    if items.len() > 1 {
        use rayon::prelude::*;
        return items
            .into_par_iter()
            .map(|item| {
                cancel.check()?;
                f(item)
            })
            .collect();
    }
    items
        .into_iter()
        .map(|item| {
            cancel.check()?;
            f(item)
        })
        .collect()
}

pub(crate) fn par_range_map_try<U, F>(
    len: usize,
    cancel: &CancelToken,
    f: F,
) -> Result<Vec<U>, SliceError>
where
    U: Send,
    F: Fn(usize) -> Result<U, SliceError> + Sync + Send,
{
    cancel.check()?;
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    if len > 1 {
        use rayon::prelude::*;
        return (0..len)
            .into_par_iter()
            .map(|index| {
                cancel.check()?;
                f(index)
            })
            .collect();
    }
    (0..len)
        .map(|index| {
            cancel.check()?;
            f(index)
        })
        .collect()
}

/// Preserve input order when jobs can fail with distinct geometry errors.
pub(crate) fn par_range_map_ordered_try<U, F>(
    len: usize,
    cancel: &CancelToken,
    f: F,
) -> Result<Vec<U>, SliceError>
where
    U: Send,
    F: Fn(usize) -> Result<U, SliceError> + Sync + Send,
{
    let results = par_range_map_try(len, cancel, |index| Ok(f(index)))?;
    cancel.check()?;
    results.into_iter().collect()
}

pub(crate) fn par_for_each_mut_try<T, F>(
    items: &mut [T],
    cancel: &CancelToken,
    f: F,
) -> Result<(), SliceError>
where
    T: Send,
    F: Fn(usize, &mut T) -> Result<(), SliceError> + Sync + Send,
{
    cancel.check()?;
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    if items.len() > 1 {
        use rayon::prelude::*;
        return items.par_iter_mut().enumerate().try_for_each(|(i, item)| {
            cancel.check()?;
            f(i, item)
        });
    }
    items.iter_mut().enumerate().try_for_each(|(i, item)| {
        cancel.check()?;
        f(i, item)
    })
}
