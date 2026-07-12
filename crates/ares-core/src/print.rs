use crate::{
    ExtrusionEntityCollection, ExtrusionPath, ExtrusionRole, Layer, LayerContours, LayerPrintPaths,
    SliceError, Surface, SurfaceType,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrintRegion {
    id: usize,
}

impl PrintRegion {
    pub const fn new(id: usize) -> Self {
        Self { id }
    }

    pub const fn id(&self) -> usize {
        self.id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerRegion {
    region_id: usize,
    slices: Vec<Surface>,
    perimeters: ExtrusionEntityCollection,
    fills: ExtrusionEntityCollection,
    extras: ExtrusionEntityCollection,
}

impl LayerRegion {
    pub fn new(
        region_id: usize,
        slices: Vec<Surface>,
        perimeters: ExtrusionEntityCollection,
        fills: ExtrusionEntityCollection,
        extras: ExtrusionEntityCollection,
    ) -> Self {
        Self {
            region_id,
            slices,
            perimeters,
            fills,
            extras,
        }
    }

    pub const fn region_id(&self) -> usize {
        self.region_id
    }

    pub fn slices(&self) -> &[Surface] {
        &self.slices
    }

    pub const fn perimeters(&self) -> &ExtrusionEntityCollection {
        &self.perimeters
    }

    pub const fn fills(&self) -> &ExtrusionEntityCollection {
        &self.fills
    }

    pub const fn extras(&self) -> &ExtrusionEntityCollection {
        &self.extras
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrintLayer {
    id: usize,
    print_z: f64,
    height: f64,
    regions: Vec<LayerRegion>,
}

impl PrintLayer {
    pub fn new(id: usize, print_z: f64, height: f64, regions: Vec<LayerRegion>) -> Self {
        Self {
            id,
            print_z,
            height,
            regions,
        }
    }

    pub const fn id(&self) -> usize {
        self.id
    }

    pub const fn print_z(&self) -> f64 {
        self.print_z
    }

    pub const fn height(&self) -> f64 {
        self.height
    }

    pub fn regions(&self) -> &[LayerRegion] {
        &self.regions
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrintObject {
    layers: Vec<PrintLayer>,
}

impl PrintObject {
    pub fn new(layers: Vec<PrintLayer>) -> Self {
        Self { layers }
    }

    pub fn layers(&self) -> &[PrintLayer] {
        &self.layers
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Print {
    objects: Vec<PrintObject>,
    regions: Vec<PrintRegion>,
}

impl Print {
    pub fn new(objects: Vec<PrintObject>, regions: Vec<PrintRegion>) -> Self {
        Self { objects, regions }
    }

    pub fn objects(&self) -> &[PrintObject] {
        &self.objects
    }

    pub fn regions(&self) -> &[PrintRegion] {
        &self.regions
    }
}

pub fn build_print_domain(
    layers: &[Layer],
    layer_contours: &[LayerContours],
    layer_print_paths: &[LayerPrintPaths],
) -> Result<Print, SliceError> {
    if layers.len() != layer_contours.len() || layers.len() != layer_print_paths.len() {
        return Err(SliceError::InvalidInput(
            "layer, contour and print path counts must match".to_owned(),
        ));
    }

    let print_layers = layers
        .iter()
        .zip(layer_contours.iter())
        .zip(layer_print_paths.iter())
        .map(|((layer, contours), print_paths)| build_print_layer(layer, contours, print_paths))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Print::new(
        vec![PrintObject::new(print_layers)],
        vec![PrintRegion::new(0)],
    ))
}

fn build_print_layer(
    layer: &Layer,
    contours: &LayerContours,
    print_paths: &LayerPrintPaths,
) -> Result<PrintLayer, SliceError> {
    if layer.id() != contours.layer_id()
        || layer.id() != print_paths.layer_id()
        || layer.print_z() != contours.print_z()
        || layer.print_z() != print_paths.print_z()
    {
        return Err(SliceError::InvalidInput(
            "layer, contour and print path metadata must match".to_owned(),
        ));
    }

    let slices = contours
        .contours()
        .iter()
        .cloned()
        .map(|contour| Surface::new(SurfaceType::Perimeter, contour))
        .collect();
    let mut perimeters = Vec::new();
    let mut fills = Vec::new();
    let mut extras = Vec::new();

    for path in print_paths.paths() {
        let role = ExtrusionRole::from_print_path_role(path.role());
        let extrusion_path = ExtrusionPath::new(role, path.points().to_vec())?;
        if role.is_perimeter() {
            perimeters.push(extrusion_path);
        } else if role.is_infill() {
            fills.push(extrusion_path);
        } else {
            extras.push(extrusion_path);
        }
    }

    Ok(PrintLayer::new(
        layer.id(),
        layer.print_z(),
        layer.height(),
        vec![LayerRegion::new(
            0,
            slices,
            ExtrusionEntityCollection::from_paths(perimeters),
            ExtrusionEntityCollection::from_paths(fills),
            ExtrusionEntityCollection::from_paths(extras),
        )],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Contour, Point2, PrintPath, PrintPathRole};

    #[test]
    fn builds_default_print_object_region_layers_and_surfaces() {
        let print =
            build_print_domain(&sample_layers(), &sample_contours(), &sample_paths()).unwrap();

        assert_eq!(print.objects().len(), 1);
        assert_eq!(print.regions(), &[PrintRegion::new(0)]);
        let layers = print.objects()[0].layers();
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].id(), 0);
        assert_eq!(layers[0].print_z(), 0.2);
        assert_eq!(layers[0].height(), 0.2);
        assert_eq!(layers[0].regions().len(), 1);
        assert_eq!(layers[0].regions()[0].region_id(), 0);
        assert_eq!(layers[0].regions()[0].slices().len(), 1);
        assert_eq!(
            layers[0].regions()[0].slices()[0].surface_type(),
            SurfaceType::Perimeter
        );
    }

    #[test]
    fn splits_paths_into_perimeters_fills_and_extras_in_order() {
        let print = build_print_domain(
            &sample_layers()[..1],
            &sample_contours()[..1],
            &sample_paths()[..1],
        )
        .unwrap();
        let region = &print.objects()[0].layers()[0].regions()[0];

        assert_eq!(
            region.perimeters().paths()[0].role(),
            ExtrusionRole::ExternalPerimeter
        );
        assert_eq!(
            region.fills().paths()[0].role(),
            ExtrusionRole::InternalInfill
        );
        assert_eq!(region.fills().paths()[1].role(), ExtrusionRole::SolidInfill);
        assert_eq!(
            region.fills().paths()[2].role(),
            ExtrusionRole::BridgeInfill
        );
        assert_eq!(region.extras().paths()[0].role(), ExtrusionRole::Skirt);
        assert_eq!(region.extras().paths()[1].role(), ExtrusionRole::Brim);
    }

    #[test]
    fn rejects_count_mismatch() {
        assert!(matches!(
            build_print_domain(&sample_layers(), &sample_contours()[..1], &sample_paths()),
            Err(SliceError::InvalidInput(_))
        ));
    }

    #[test]
    fn rejects_layer_metadata_mismatch() {
        let contours = vec![LayerContours::new(7, 0.2, vec![square_contour()])];

        assert_eq!(
            build_print_domain(&sample_layers()[..1], &contours, &sample_paths()[..1]),
            Err(SliceError::InvalidInput(
                "layer, contour and print path metadata must match".to_owned()
            ))
        );
    }

    fn sample_layers() -> Vec<Layer> {
        vec![Layer::new(0, 0.2, 0.2), Layer::new(1, 0.2, 0.4)]
    }

    fn sample_contours() -> Vec<LayerContours> {
        vec![
            LayerContours::new(0, 0.2, vec![square_contour()]),
            LayerContours::new(1, 0.4, Vec::new()),
        ]
    }

    fn sample_paths() -> Vec<LayerPrintPaths> {
        vec![
            LayerPrintPaths::new(
                0,
                0.2,
                vec![
                    path(PrintPathRole::Skirt),
                    path(PrintPathRole::Brim),
                    path(PrintPathRole::ExternalPerimeter),
                    path(PrintPathRole::SparseInfill),
                    path(PrintPathRole::SolidInfill),
                    path(PrintPathRole::Bridge),
                ],
            ),
            LayerPrintPaths::new(1, 0.4, Vec::new()),
        ]
    }

    fn path(role: PrintPathRole) -> PrintPath {
        PrintPath::new(role, vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)]).unwrap()
    }

    fn square_contour() -> Contour {
        Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
        ])
    }
}
