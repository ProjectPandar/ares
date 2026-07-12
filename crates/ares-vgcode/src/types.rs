// Ports rendering-neutral data from AGPL-licensed OrcaSlicer `src/libvgcode/include/Types.hpp` and `src/Types.cpp`.

pub type Vec3 = [f32; 3];
pub type Mat4x4 = [f32; 16];
pub type Color = [u8; 3];
pub type Palette = Vec<Color>;
pub type AABox = [Vec3; 2];
pub type Interval = [usize; 2];

pub const DEFAULT_TRAVELS_RADIUS_MM: f32 = 0.1;
pub const MIN_TRAVELS_RADIUS_MM: f32 = 0.05;
pub const MAX_TRAVELS_RADIUS_MM: f32 = 1.0;
pub const DEFAULT_WIPES_RADIUS_MM: f32 = 0.1;
pub const MIN_WIPES_RADIUS_MM: f32 = 0.05;
pub const MAX_WIPES_RADIUS_MM: f32 = 1.0;
pub const DUMMY_COLOR: Color = [64, 64, 64];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ViewType {
    Summary,
    FeatureType,
    ColorPrint,
    Speed,
    ActualSpeed,
    Height,
    Width,
    VolumetricFlowRate,
    ActualVolumetricFlowRate,
    LayerTimeLinear,
    LayerTimeLogarithmic,
    FanSpeed,
    Temperature,
    PressureAdvance,
    Acceleration,
    Jerk,
    Tool,
}

impl ViewType {
    pub const COUNT: usize = 17;

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MoveType {
    Noop,
    Retract,
    Unretract,
    Seam,
    ToolChange,
    ColorChange,
    PausePrint,
    CustomGCode,
    Travel,
    Wipe,
    Extrude,
}

impl MoveType {
    pub const COUNT: usize = 11;

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GCodeExtrusionRole {
    None,
    Perimeter,
    ExternalPerimeter,
    OverhangPerimeter,
    InternalInfill,
    SolidInfill,
    TopSolidInfill,
    Ironing,
    BridgeInfill,
    GapFill,
    Skirt,
    SupportMaterial,
    SupportMaterialInterface,
    WipeTower,
    Custom,
    BottomSurface,
    InternalBridgeInfill,
    Brim,
    SupportTransition,
    Mixed,
}

impl GCodeExtrusionRole {
    pub const COUNT: usize = 20;

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OptionType {
    Travels,
    Wipes,
    Retractions,
    Unretractions,
    Seams,
    ToolChanges,
    ColorChanges,
    PausePrints,
    CustomGCodes,
}

impl OptionType {
    pub const COUNT: usize = 9;

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TimeMode {
    Normal,
    Stealth,
}

impl TimeMode {
    pub const COUNT: usize = 2;

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorRangeType {
    Linear,
    Logarithmic,
}

impl ColorRangeType {
    pub const COUNT: usize = 2;

    pub const fn index(self) -> usize {
        self as usize
    }
}

pub const fn move_type_to_option(move_type: MoveType) -> Option<OptionType> {
    match move_type {
        MoveType::Travel => Some(OptionType::Travels),
        MoveType::Wipe => Some(OptionType::Wipes),
        MoveType::Retract => Some(OptionType::Retractions),
        MoveType::Unretract => Some(OptionType::Unretractions),
        MoveType::Seam => Some(OptionType::Seams),
        MoveType::ToolChange => Some(OptionType::ToolChanges),
        MoveType::ColorChange => Some(OptionType::ColorChanges),
        MoveType::PausePrint => Some(OptionType::PausePrints),
        MoveType::CustomGCode => Some(OptionType::CustomGCodes),
        MoveType::Noop | MoveType::Extrude => None,
    }
}

pub fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    [
        lerp_channel(c1[0], c2[0], t),
        lerp_channel(c1[1], c2[1], t),
        lerp_channel(c1[2], c2[2], t),
    ]
}

fn lerp_channel(c1: u8, c2: u8, t: f32) -> u8 {
    ((1.0 - t) * f32::from(c1) + t * f32::from(c2)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_discriminants_and_counts_match_upstream_order() {
        assert_eq!(ViewType::Summary as u8, 0);
        assert_eq!(ViewType::Tool as u8, 16);
        assert_eq!(ViewType::COUNT, 17);
        assert_eq!(MoveType::Noop as u8, 0);
        assert_eq!(MoveType::Extrude as u8, 10);
        assert_eq!(MoveType::COUNT, 11);
        assert_eq!(GCodeExtrusionRole::None as u8, 0);
        assert_eq!(GCodeExtrusionRole::Mixed as u8, 19);
        assert_eq!(GCodeExtrusionRole::COUNT, 20);
        assert_eq!(OptionType::Travels as u8, 0);
        assert_eq!(OptionType::CustomGCodes as u8, 8);
        assert_eq!(OptionType::COUNT, 9);
        assert_eq!(TimeMode::Normal as u8, 0);
        assert_eq!(TimeMode::Stealth as u8, 1);
        assert_eq!(TimeMode::COUNT, 2);
        assert_eq!(ColorRangeType::Linear as u8, 0);
        assert_eq!(ColorRangeType::Logarithmic as u8, 1);
        assert_eq!(ColorRangeType::COUNT, 2);
        assert_eq!(TimeMode::Stealth.index(), 1);
    }

    #[test]
    fn move_types_map_to_matching_option_types() {
        assert_eq!(
            move_type_to_option(MoveType::Travel),
            Some(OptionType::Travels)
        );
        assert_eq!(move_type_to_option(MoveType::Wipe), Some(OptionType::Wipes));
        assert_eq!(
            move_type_to_option(MoveType::Retract),
            Some(OptionType::Retractions)
        );
        assert_eq!(
            move_type_to_option(MoveType::Unretract),
            Some(OptionType::Unretractions)
        );
        assert_eq!(move_type_to_option(MoveType::Seam), Some(OptionType::Seams));
        assert_eq!(
            move_type_to_option(MoveType::ToolChange),
            Some(OptionType::ToolChanges)
        );
        assert_eq!(
            move_type_to_option(MoveType::ColorChange),
            Some(OptionType::ColorChanges)
        );
        assert_eq!(
            move_type_to_option(MoveType::PausePrint),
            Some(OptionType::PausePrints)
        );
        assert_eq!(
            move_type_to_option(MoveType::CustomGCode),
            Some(OptionType::CustomGCodes)
        );
        assert_eq!(move_type_to_option(MoveType::Noop), None);
        assert_eq!(move_type_to_option(MoveType::Extrude), None);
    }

    #[test]
    fn color_lerp_clamps_and_truncates_channels() {
        assert_eq!(
            lerp_color([0, 100, 200], [100, 200, 250], -0.5),
            [0, 100, 200]
        );
        assert_eq!(
            lerp_color([0, 100, 200], [100, 200, 250], 1.5),
            [100, 200, 250]
        );
        assert_eq!(lerp_color([0, 0, 0], [255, 127, 1], 0.5), [127, 63, 0]);
    }
}
