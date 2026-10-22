use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawPcad2 {
    pub graph: RawNode,
    pub metadata: RawMetadata,
    pub texture: RawTexture,
}

#[derive(Debug, Deserialize)]
pub struct RawMetadata {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct RawNode {
    pub name: String,

    #[serde(default)]
    pub children: Vec<RawNode>,

    pub transform: RawTransform,

    #[serde(default)]
    pub mesh: Option<RawMesh>,
}

#[derive(Debug, Deserialize)]
pub struct RawMesh {
    pub name: String,
    pub vertices: Vec<f32>,
    pub faces: Vec<RawFace>,
}

#[derive(Debug, Deserialize)]
pub struct RawFace {
    pub color: usize,
    pub vertex_ids: Vec<usize>,
    pub uvs: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct RawTransform {
    pub pos: RawVec3,
    pub rot: RawVec3,
    pub scale: RawVec3,
}

#[derive(Debug, Deserialize)]
pub struct RawVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Deserialize)]
pub struct RawTexture {
    pub colors: Vec<[f32; 3]>,
    pub pixels: String,
    pub transparent_color: usize,
    pub background_color: usize,
}

