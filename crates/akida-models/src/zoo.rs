//! Akida Model Zoo Manager
//!
//! Provides utilities for downloading, converting, and managing
//! models from the `BrainChip` Akida Model Zoo.
//!
//! ## Model Zoo Models
//!
//! The Akida Model Zoo includes pre-trained models for various tasks:
//!
//! | Model | Task | Size | Accuracy | Power |
//! |-------|------|------|----------|-------|
//! | `AkidaNet` 0.5 | `ImageNet` | 160×160 | 65% top-1 | <300 mW |
//! | DS-CNN | Keyword Spotting | 32 words | 94% | <50 mW |
//! | `ViT` | Vision Transformer | 224×224 | 75% top-1 | ~500 mW |
//! | YOLO | Object Detection | 320×320 | mAP 0.28 | <500 mW |
//!
//! ## Usage
//!
//! ```ignore
//! use akida_models::zoo::{ModelZoo, ZooModel};
//!
//! // Initialize the model zoo
//! let zoo = ModelZoo::new("models/akida")?;
//!
//! // Download a model
//! zoo.download(ZooModel::DsCnnKws)?;
//!
//! // Get local path to model
//! let path = zoo.model_path(ZooModel::DsCnnKws)?;
//! ```

use crate::{AkidaModelError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Models available in the Akida Model Zoo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZooModel {
    /// `AkidaNet` `ImageNet` classifier (0.5 width, 160×160)
    AkidaNet05_160,
    /// `AkidaNet` `ImageNet` classifier (1.0 width, 224×224)
    AkidaNet10_224,
    /// DS-CNN for keyword spotting (32 words)
    DsCnnKws,
    /// `MobileNetV2` for `ImageNet`
    MobileNetV2,
    /// Vision Transformer (tiny)
    ViTTiny,
    /// YOLO object detection
    YoloV8n,
    /// `PointNet++` for 3D point clouds
    PointNetPlusPlus,
    /// DVS Gesture recognition
    DvsGesture,
    /// Event camera model
    EventCamera,
    /// ESN for chaotic function prediction
    EsnChaotic,
}

impl ZooModel {
    /// Get model filename
    pub const fn filename(&self) -> &'static str {
        match self {
            Self::AkidaNet05_160 => "akidanet_05_160.fbz",
            Self::AkidaNet10_224 => "akidanet_10_224.fbz",
            Self::DsCnnKws => "ds_cnn_kws.fbz",
            Self::MobileNetV2 => "mobilenetv2.fbz",
            Self::ViTTiny => "vit_tiny.fbz",
            Self::YoloV8n => "yolov8n.fbz",
            Self::PointNetPlusPlus => "pointnet_plus.fbz",
            Self::DvsGesture => "dvs_gesture.fbz",
            Self::EventCamera => "event_camera.fbz",
            Self::EsnChaotic => "esn_chaotic.fbz",
        }
    }

    /// Get model description
    pub const fn description(&self) -> &'static str {
        match self {
            Self::AkidaNet05_160 => "AkidaNet ImageNet classifier (0.5 width, 160×160)",
            Self::AkidaNet10_224 => "AkidaNet ImageNet classifier (1.0 width, 224×224)",
            Self::DsCnnKws => "DS-CNN keyword spotting (32 words, Speech Commands)",
            Self::MobileNetV2 => "MobileNetV2 ImageNet classifier",
            Self::ViTTiny => "Vision Transformer (tiny) ImageNet classifier",
            Self::YoloV8n => "YOLOv8 nano object detection (COCO)",
            Self::PointNetPlusPlus => "PointNet++ 3D point cloud classification",
            Self::DvsGesture => "DVS Gesture recognition (11 classes)",
            Self::EventCamera => "Event camera object detection",
            Self::EsnChaotic => "Echo State Network for chaotic prediction",
        }
    }

    /// Get expected model size (approximate)
    pub const fn expected_size_bytes(&self) -> usize {
        match self {
            Self::AkidaNet05_160 => 500_000,
            Self::AkidaNet10_224 => 2_000_000,
            Self::DsCnnKws => 100_000,
            Self::MobileNetV2 => 3_500_000,
            Self::ViTTiny => 5_000_000,
            Self::YoloV8n => 3_000_000,
            Self::PointNetPlusPlus => 1_500_000,
            Self::DvsGesture => 800_000,
            Self::EventCamera => 1_200_000,
            Self::EsnChaotic => 200_000,
        }
    }

    /// Get all available models
    pub const fn all() -> &'static [Self] {
        &[
            Self::AkidaNet05_160,
            Self::AkidaNet10_224,
            Self::DsCnnKws,
            Self::MobileNetV2,
            Self::ViTTiny,
            Self::YoloV8n,
            Self::PointNetPlusPlus,
            Self::DvsGesture,
            Self::EventCamera,
            Self::EsnChaotic,
        ]
    }

    /// Get models for `NeuroBench` benchmarks
    pub const fn neurobench_models() -> &'static [Self] {
        &[
            Self::DvsGesture,
            Self::DsCnnKws,
            Self::EsnChaotic,
            Self::EventCamera,
        ]
    }

    /// Get task category
    pub const fn task(&self) -> ModelTask {
        match self {
            Self::AkidaNet05_160 | Self::AkidaNet10_224 | Self::MobileNetV2 | Self::ViTTiny => {
                ModelTask::ImageClassification
            }
            Self::DsCnnKws => ModelTask::KeywordSpotting,
            Self::YoloV8n | Self::EventCamera => ModelTask::ObjectDetection,
            Self::PointNetPlusPlus => ModelTask::PointCloud,
            Self::DvsGesture => ModelTask::GestureRecognition,
            Self::EsnChaotic => ModelTask::TimeSeriesPrediction,
        }
    }
}

/// Model task categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTask {
    /// Image classification
    ImageClassification,
    /// Keyword/speech spotting
    KeywordSpotting,
    /// Object detection
    ObjectDetection,
    /// 3D point cloud processing
    PointCloud,
    /// Gesture recognition (DVS)
    GestureRecognition,
    /// Time series prediction
    TimeSeriesPrediction,
}

/// Model metadata extracted from zoo
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    /// Model enum variant
    pub model: ZooModel,
    /// Local file path
    pub path: PathBuf,
    /// File size in bytes
    pub size_bytes: usize,
    /// Is valid .fbz format
    pub is_valid: bool,
    /// SDK version (if parseable)
    pub sdk_version: Option<String>,
    /// Number of layers
    pub layer_count: Option<usize>,
}

/// Akida Model Zoo manager
///
/// Manages local cache of Akida Model Zoo models.
pub struct ModelZoo {
    /// Local cache directory
    cache_dir: PathBuf,
    /// Cached model metadata
    metadata: HashMap<ZooModel, ModelMetadata>,
}

impl ModelZoo {
    /// Create model zoo with specified cache directory
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be created.
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Create directory if it doesn't exist
        fs::create_dir_all(&cache_dir).map_err(|e| {
            AkidaModelError::loading_failed(format!("Cannot create cache dir: {e}"))
        })?;

        info!("Model zoo cache: {}", cache_dir.display());

        let mut zoo = Self {
            cache_dir,
            metadata: HashMap::new(),
        };

        // Scan for existing models
        zoo.scan();

        Ok(zoo)
    }

    /// Scan cache directory for existing models
    fn scan(&mut self) {
        debug!("Scanning model zoo cache...");

        for model in ZooModel::all() {
            let path = self.cache_dir.join(model.filename());

            if path.exists() {
                match Self::load_metadata(*model, &path) {
                    Ok(meta) => {
                        debug!("Found {}: {} bytes", model.filename(), meta.size_bytes);
                        self.metadata.insert(*model, meta);
                    }
                    Err(e) => {
                        warn!("Invalid model {}: {}", model.filename(), e);
                    }
                }
            }
        }

        info!(
            "Found {} cached models in {}",
            self.metadata.len(),
            self.cache_dir.display()
        );
    }

    /// Load metadata for a model file
    fn load_metadata(model: ZooModel, path: &Path) -> Result<ModelMetadata> {
        let data = fs::read(path)
            .map_err(|e| AkidaModelError::loading_failed(format!("Cannot read model: {e}")))?;

        let size_bytes = data.len();

        // Validate .fbz format (check FlatBuffers magic)
        let is_valid = data.len() >= 4 && data[0..4] == [0x80, 0x44, 0x04, 0x10];

        // Try to extract version (simplified - real impl would parse FlatBuffers)
        let sdk_version = if data.len() > 40 {
            // Version typically at offset 30-40
            data[20..50]
                .windows(6)
                .find(|w| w.iter().all(|&b| b == b'.' || b.is_ascii_digit()))
                .and_then(|w| {
                    std::str::from_utf8(w)
                        .ok()
                        .map(|s| s.trim_end_matches('\0').to_string())
                })
        } else {
            None
        };

        Ok(ModelMetadata {
            model,
            path: path.to_path_buf(),
            size_bytes,
            is_valid,
            sdk_version,
            layer_count: None, // Would require full parsing
        })
    }

    /// Check if model is available locally
    pub fn has_model(&self, model: ZooModel) -> bool {
        self.metadata.contains_key(&model)
    }

    /// Get path to model file
    ///
    /// # Errors
    ///
    /// Returns error if model is not available locally.
    pub fn model_path(&self, model: ZooModel) -> Result<PathBuf> {
        if let Some(meta) = self.metadata.get(&model) {
            Ok(meta.path.clone())
        } else {
            Err(AkidaModelError::loading_failed(format!(
                "Model {} not available. Use download() first.",
                model.filename()
            )))
        }
    }

    /// Get model metadata
    pub fn model_metadata(&self, model: ZooModel) -> Option<&ModelMetadata> {
        self.metadata.get(&model)
    }

    /// List all available models
    pub fn available_models(&self) -> Vec<ZooModel> {
        self.metadata.keys().copied().collect()
    }

    /// Get cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Create a stub model for testing
    ///
    /// Creates a minimal .fbz file that passes format validation.
    /// This is useful for testing without real models.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be written.
    pub fn create_stub_model(&mut self, model: ZooModel) -> Result<PathBuf> {
        let path = self.cache_dir.join(model.filename());

        // Create minimal valid .fbz structure
        let mut data = Vec::with_capacity(1024);

        // FlatBuffers magic
        data.extend_from_slice(&[0x80, 0x44, 0x04, 0x10]);

        // Padding to table offset area
        data.extend_from_slice(&[0x00; 26]);

        // Version string at offset 30
        data.extend_from_slice(b"2.18.2\0");

        // Pad to minimum size
        while data.len() < 256 {
            data.push(0x00);
        }

        // Add layer count marker (simplified)
        data.push(0x01); // 1 layer

        // Pad to expected minimum
        while data.len() < 512 {
            data.push(0x00);
        }

        fs::write(&path, &data)
            .map_err(|e| AkidaModelError::loading_failed(format!("Cannot write stub: {e}")))?;

        info!(
            "Created stub model: {} ({} bytes)",
            path.display(),
            data.len()
        );

        // Update metadata
        let meta = Self::load_metadata(model, &path)?;
        self.metadata.insert(model, meta);

        Ok(path)
    }

    /// Initialize stub models for all `NeuroBench` benchmarks
    ///
    /// # Errors
    ///
    /// Returns error if any stub cannot be created.
    pub fn init_neurobench_stubs(&mut self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        for model in ZooModel::neurobench_models() {
            if !self.has_model(*model) {
                let path = self.create_stub_model(*model)?;
                paths.push(path);
            }
        }

        Ok(paths)
    }

    /// Print zoo status
    pub fn print_status(&self) {
        println!("\nAkida Model Zoo Status");
        println!("{}", "=".repeat(60));
        println!("Cache: {}", self.cache_dir.display());
        println!(
            "Available: {}/{}",
            self.metadata.len(),
            ZooModel::all().len()
        );
        println!();

        for model in ZooModel::all() {
            let status = if let Some(meta) = self.metadata.get(model) {
                format!(
                    "✓ {:>8} bytes (valid: {})",
                    meta.size_bytes,
                    if meta.is_valid { "yes" } else { "no" }
                )
            } else {
                "✗ not downloaded".to_string()
            };

            println!("  {:20} {}", model.filename(), status);
        }

        println!("{}", "=".repeat(60));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_zoo_model_filenames() {
        assert_eq!(ZooModel::DsCnnKws.filename(), "ds_cnn_kws.fbz");
        assert_eq!(ZooModel::DvsGesture.filename(), "dvs_gesture.fbz");
    }

    #[test]
    fn test_zoo_model_all() {
        assert!(ZooModel::all().len() >= 5);
    }

    #[test]
    fn test_model_zoo_creation() {
        let temp_dir = TempDir::new().unwrap();
        let zoo = ModelZoo::new(temp_dir.path()).unwrap();

        assert_eq!(zoo.available_models().len(), 0);
        assert!(!zoo.has_model(ZooModel::DsCnnKws));
    }

    #[test]
    fn test_stub_model_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut zoo = ModelZoo::new(temp_dir.path()).unwrap();

        let path = zoo.create_stub_model(ZooModel::DsCnnKws).unwrap();

        assert!(path.exists());
        assert!(zoo.has_model(ZooModel::DsCnnKws));

        let meta = zoo.model_metadata(ZooModel::DsCnnKws).unwrap();
        assert!(meta.is_valid);
    }

    #[test]
    fn test_neurobench_stubs() {
        let temp_dir = TempDir::new().unwrap();
        let mut zoo = ModelZoo::new(temp_dir.path()).unwrap();

        let paths = zoo.init_neurobench_stubs().unwrap();

        assert!(!paths.is_empty());
        assert!(zoo.has_model(ZooModel::DvsGesture));
        assert!(zoo.has_model(ZooModel::DsCnnKws));
    }
}
