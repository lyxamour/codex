use codex::config::loader::ConfigLoader;
use std::fs;

#[test]
fn test_config_loading() {
    // 测试配置加载功能
    let config_loader = ConfigLoader::new();
    let result = config_loader.load(None);
    assert!(result.is_ok(), "配置加载失败");
    let config = result.unwrap();

    // 检查配置是否包含默认值
    assert!(!config.app.name.is_empty(), "应用名称为空");
    assert!(!config.app.version.is_empty(), "应用版本为空");

    // 尝试创建数据目录并检查
    let data_dir = &config.app.data_dir;
    if let Err(e) = fs::create_dir_all(data_dir) {
        panic!("无法创建数据目录: {}", e);
    }
    assert!(data_dir.exists(), "数据目录不存在");
}
