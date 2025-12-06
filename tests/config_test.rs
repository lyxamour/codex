
use codex::config::loader::load_config;

#[test]
fn test_config_loading() {
    // 测试配置加载功能
    let result = load_config();
    assert!(result.is_ok(), "配置加载失败");
    let config = result.unwrap();
    
    // 检查配置是否包含默认值
    assert!(!config.app.name.is_empty(), "应用名称为空");
    assert!(!config.app.version.is_empty(), "应用版本为空");
    assert!(config.app.data_dir.exists(), "数据目录不存在");
}
