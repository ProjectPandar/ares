use super::ProfileSliceRequest;

/// The upstream composition of explicitly selected presets: the plater's
/// `PresetBundle::full_fff_config` or the CLI's `--load-settings`
/// (`OrcaSlicer.cpp:2251-2254,3097-3103,3403-3408,3616-3621`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Composition {
    Gui,
    Cli,
}

/// The plater's request: an explicit selection composes as `PresetBundle::full_fff_config`.
#[derive(Clone, Copy, Debug)]
pub struct GuiProfileSliceRequest<'a>(pub ProfileSliceRequest<'a>);

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::ProfileSliceRequest<'_> {}
    impl Sealed for super::GuiProfileSliceRequest<'_> {}
}

/// A selected-profile request and the composition its type names.
pub trait ProfileRequest<'a>: sealed::Sealed + Copy {
    fn request(self) -> ProfileSliceRequest<'a>;
    fn composition(self) -> Composition;
}

impl<'a> ProfileRequest<'a> for ProfileSliceRequest<'a> {
    fn request(self) -> ProfileSliceRequest<'a> {
        self
    }

    fn composition(self) -> Composition {
        Composition::Cli
    }
}

impl<'a> ProfileRequest<'a> for GuiProfileSliceRequest<'a> {
    fn request(self) -> ProfileSliceRequest<'a> {
        self.0
    }

    fn composition(self) -> Composition {
        Composition::Gui
    }
}
