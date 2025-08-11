use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub camera: CameraConfig,
    pub movement: MovementConfig,
    pub rendering: RenderingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub resizable: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CameraConfig {
    pub fov_degrees: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MovementConfig {
    pub move_speed: f32,
    pub rotation_speed: f32,
    pub mouse_sensitivity: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RenderingConfig {
    pub clear_color: [f32; 4],
    pub vsync: bool,
    pub msaa_samples: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                width: 800,
                height: 600,
                title: "Demo Engine".to_string(),
                resizable: true,
            },
            camera: CameraConfig {
                fov_degrees: 45.0,
                znear: 0.1,
                zfar: 100.0,
            },
            movement: MovementConfig {
                move_speed: 5.0,
                rotation_speed: 1.0,
                mouse_sensitivity: 0.001,
            },
            rendering: RenderingConfig {
                clear_color: [0.5, 0.2, 0.2, 1.0],
                vsync: true,
                msaa_samples: 1,
            },
        }
    }
}

impl AppConfig {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let constant = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&constant)?;
        Ok(config)
    }

    pub fn load_or_default(path: &str) -> Self {
        if let Ok(home) = std::env::current_dir() {
            let config_path = home.join(path);
            if let Ok(config) = Self::load_from_file(&config_path.to_string_lossy()) {
                return config;
            }
        }

        Self::default()
    }

    #[allow(dead_code)]
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path_buf = std::path::Path::new(path);
        if let Some(parent) = path_buf.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> AppConfig {
        AppConfig {
            window: WindowConfig {
                width: 1920,
                height: 1080,
                title: "Test Demo Engine".to_string(),
                resizable: false,
            },
            camera: CameraConfig {
                fov_degrees: 60.0,
                znear: 0.05,
                zfar: 500.0,
            },
            movement: MovementConfig {
                move_speed: 8.0,
                rotation_speed: 1.5,
                mouse_sensitivity: 0.002,
            },
            rendering: RenderingConfig {
                clear_color: [0.1, 0.2, 0.3, 1.0],
                vsync: false,
                msaa_samples: 4,
            },
        }
    }

    #[test]
    fn test_default_config_values() {
        let config = AppConfig::default();

        // Window設定のテスト
        assert_eq!(config.window.width, 800);
        assert_eq!(config.window.height, 600);
        assert_eq!(config.window.title, "Demo Engine");
        assert!(config.window.resizable);

        // Camera設定のテスト
        assert_eq!(config.camera.fov_degrees, 45.0);
        assert_eq!(config.camera.znear, 0.1);
        assert_eq!(config.camera.zfar, 100.0);

        // Movement設定のテスト
        assert_eq!(config.movement.move_speed, 5.0);
        assert_eq!(config.movement.rotation_speed, 1.0);
        assert_eq!(config.movement.mouse_sensitivity, 0.001);

        // Rendering設定のテスト
        assert_eq!(config.rendering.clear_color, [0.5, 0.2, 0.2, 1.0]);
        assert!(config.rendering.vsync);
        assert_eq!(config.rendering.msaa_samples, 1);
    }

    #[test]
    fn test_save_and_load_specific_values() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // 特定の値を持つ設定を作成
        let original_config = create_test_config();

        // 保存
        assert!(
            original_config
                .save_to_file(config_path.to_str().unwrap())
                .is_ok()
        );

        // 読み込み
        let loaded_config = AppConfig::load_from_file(config_path.to_str().unwrap()).unwrap();

        // Window設定の比較
        assert_eq!(loaded_config.window.width, 1920);
        assert_eq!(loaded_config.window.height, 1080);
        assert_eq!(loaded_config.window.title, "Test Demo Engine");
        assert!(!loaded_config.window.resizable);

        // Camera設定の比較
        assert_eq!(loaded_config.camera.fov_degrees, 60.0);
        assert_eq!(loaded_config.camera.znear, 0.05);
        assert_eq!(loaded_config.camera.zfar, 500.0);

        // Movement設定の比較
        assert_eq!(loaded_config.movement.move_speed, 8.0);
        assert_eq!(loaded_config.movement.rotation_speed, 1.5);
        assert_eq!(loaded_config.movement.mouse_sensitivity, 0.002);

        // Rendering設定の比較
        assert_eq!(loaded_config.rendering.clear_color, [0.1, 0.2, 0.3, 1.0]);
        assert!(!loaded_config.rendering.vsync);
        assert_eq!(loaded_config.rendering.msaa_samples, 4);
    }

    #[test]
    fn test_toml_format_content() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("format_test.toml");

        let config = create_test_config();
        config.save_to_file(config_path.to_str().unwrap()).unwrap();

        // 生成されたTOMLファイルの内容を確認
        let content = std::fs::read_to_string(&config_path).unwrap();

        // 各セクションの存在確認
        assert!(content.contains("[window]"));
        assert!(content.contains("[camera]"));
        assert!(content.contains("[movement]"));
        assert!(content.contains("[rendering]"));

        // 具体的な値の確認
        assert!(content.contains("width = 1920"));
        assert!(content.contains("height = 1080"));
        assert!(content.contains("title = \"Test Demo Engine\""));
        assert!(content.contains("resizable = false"));

        assert!(content.contains("fov_degrees = 60.0"));
        assert!(content.contains("znear = 0.05"));
        assert!(content.contains("zfar = 500.0"));

        assert!(content.contains("move_speed = 8.0"));
        assert!(content.contains("rotation_speed = 1.5"));
        assert!(content.contains("mouse_sensitivity = 0.002"));

        // TOMLでは配列の表現が異なる可能性があるため、個別にチェック
        assert!(content.contains("clear_color = ["));
        assert!(content.contains("0.1"));
        assert!(content.contains("0.2"));
        assert!(content.contains("0.3"));
        assert!(content.contains("1.0"));
        assert!(content.contains("vsync = false"));
        assert!(content.contains("msaa_samples = 4"));
    }

    #[test]
    fn test_save_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("deep")
            .join("nested")
            .join("config.toml");

        let config = create_test_config();

        // 存在しない深いディレクトリに保存
        assert!(config.save_to_file(nested_path.to_str().unwrap()).is_ok());
        assert!(nested_path.exists());

        // 保存された内容の確認
        let loaded = AppConfig::load_from_file(nested_path.to_str().unwrap()).unwrap();
        assert_eq!(loaded.window.width, 1920);
        assert_eq!(loaded.camera.fov_degrees, 60.0);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent.toml");

        // 存在しないファイルの読み込みはエラーになるべき
        assert!(AppConfig::load_from_file(nonexistent_path.to_str().unwrap()).is_err());
    }

    #[test]
    fn test_load_or_default_fallback() {
        // 実際のプロジェクトルートにconfig.tomlがない場合のテスト
        // このテストは実際の環境に依存するため、条件付きで実行

        let config = AppConfig::load_or_default("config.toml");

        // デフォルト値またはファイルから読み込んだ値が返されるべき
        assert!(config.window.width > 0);
        assert!(config.window.height > 0);
        assert!(!config.window.title.is_empty());
        assert!(config.camera.fov_degrees > 0.0);
        assert!(config.camera.znear > 0.0);
        assert!(config.camera.zfar > config.camera.znear);
    }

    #[test]
    fn test_invalid_toml_content() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");

        // 無効なTOMLファイルを作成
        std::fs::write(&config_path, "invalid toml content [[[").unwrap();

        // 無効なTOMLファイルの読み込みはエラーになるべき
        assert!(AppConfig::load_from_file(config_path.to_str().unwrap()).is_err());
    }
}
