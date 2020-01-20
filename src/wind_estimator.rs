#[derive(Clone, Debug)]
pub struct WindEstimator {
    pub tas_scale: f32,
    pub tas_innovation: f32,
    pub tas_innovation_var: f32
}