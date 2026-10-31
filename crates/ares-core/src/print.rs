pub(crate) mod validate;

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
