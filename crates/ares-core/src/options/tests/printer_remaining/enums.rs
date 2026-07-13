use super::super::super::{
    AuthorizationType, NozzleVolumeType, PrintHostType, PrinterTechnology,
};
use crate::GCodeThumbnailFormat;

#[test]
fn printer_options_remaining_enum_tokens_are_exact() {
    assert_tokens(&[("FFF", PrinterTechnology::Fff), ("SLA", PrinterTechnology::Sla)]);
    assert_tokens(&[
        ("Standard", NozzleVolumeType::Standard),
        ("High Flow", NozzleVolumeType::HighFlow),
    ]);
    assert_tokens(&[("key", AuthorizationType::Key), ("user", AuthorizationType::User)]);
    assert_tokens(&[
        ("PNG", GCodeThumbnailFormat::Png),
        ("JPG", GCodeThumbnailFormat::Jpg),
        ("QOI", GCodeThumbnailFormat::Qoi),
        ("BTT_TFT", GCodeThumbnailFormat::BttTft),
        ("COLPIC", GCodeThumbnailFormat::ColPic),
    ]);
    assert_tokens(&[
        ("prusalink", PrintHostType::PrusaLink),
        ("prusaconnect", PrintHostType::PrusaConnect),
        ("octoprint", PrintHostType::OctoPrint),
        ("crealityprint", PrintHostType::CrealityPrint),
        ("duet", PrintHostType::Duet),
        ("flashair", PrintHostType::FlashAir),
        ("astrobox", PrintHostType::AstroBox),
        ("repetier", PrintHostType::Repetier),
        ("mks", PrintHostType::Mks),
        ("esp3d", PrintHostType::Esp3d),
        ("obico", PrintHostType::Obico),
        ("flashforge", PrintHostType::Flashforge),
        ("simplyprint", PrintHostType::SimplyPrint),
        ("elegoolink", PrintHostType::ElegooLink),
        ("3dprinteros", PrintHostType::ThreeDPrinterOs),
        ("moonraker", PrintHostType::Moonraker),
    ]);

    for invalid in ["fff", "Normal", "HighFlow", "ColPic", "password", "klipper"] {
        assert!(serde_json::from_str::<PrinterTechnology>(&format!(r#""{invalid}""#)).is_err());
        assert!(serde_json::from_str::<NozzleVolumeType>(&format!(r#""{invalid}""#)).is_err());
        assert!(serde_json::from_str::<AuthorizationType>(&format!(r#""{invalid}""#)).is_err());
        assert!(serde_json::from_str::<GCodeThumbnailFormat>(&format!(r#""{invalid}""#)).is_err());
        assert!(serde_json::from_str::<PrintHostType>(&format!(r#""{invalid}""#)).is_err());
    }
}

fn assert_tokens<T>(cases: &[(&str, T)])
where
    T: Copy + std::fmt::Debug + PartialEq + serde::Serialize + serde::de::DeserializeOwned,
{
    for &(wire, expected) in cases {
        let json = format!(r#""{wire}""#);
        assert_eq!(serde_json::from_str::<T>(&json).unwrap(), expected);
        assert_eq!(serde_json::to_string(&expected).unwrap(), json);
    }
}
