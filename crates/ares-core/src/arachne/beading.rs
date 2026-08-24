pub(crate) mod base;
mod distributed;
pub(crate) mod factory;
mod limited;
mod outer_inset;
mod redistribute;
mod widening;

#[cfg(test)]
pub(crate) use widening::WideningBeadingStrategy;

#[cfg(test)]
mod tests;
