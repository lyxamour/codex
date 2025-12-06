
use codex::config::loader::ConfigLoader;
use codex::config::validator::ConfigValidator;
use std::env;

#[test]
fn test_default_config() {
    // 创建配置加载器
    let loader = ConfigLoader::new();
    
    // 加载默认配置
    let config = loader.load(None).unwrap();
    
    // 验证配置
    let validator = ConfigValidator::new();
    let result = validator.validate(&config);
    
    assert!(result.is_ok(), "默认配置应该验证通过");
    
    // 检查默认值
    assert_eq!(config.app.name, "codex");
    assert_eq!(config.app.language, "en");
    assert_eq!(config.ai.default_platform, "openai");
}

#[test]
fn test_env_vars_override() {
    // 设置环境变量
    env::set_var("CODEX_APP_LANGUAGE", "zh-CN");
    env::set_var("CODEX_AI_DEFAULT_MODEL", "gpt-4");
    env::set_var("CODEX_UI_THEME", "dark");
    
    // 创建配置加载器
    let loader = ConfigLoader::new();
    
    // 加载配置（应该从环境变量覆盖）
    let config = loader.load(None).unwrap();
    
    // 验证环境变量覆盖是否生效
    assert_eq!(config.app.language, "zh-CN");
    assert_eq!(config.ai.openai.unwrap().default_model, "gpt-4");
    assert_eq!(config.ui.theme, "dark");
    
    // 清理环境变量
    env::remove_var("CODEX_APP_LANGUAGE");
    env::remove_var("CODEX_AI_DEFAULT_MODEL");
    env::remove_var("CODEX_UI_THEME");
}

#[test]
fn test_config_validation() {
    // 创建配置加载器
    let loader = ConfigLoader::new();
    
    // 加载默认配置
    let config = loader.load(None).unwrap();
    
    // 创建配置验证器
    let validator = ConfigValidator::new();
    
    // 验证配置
    let result = validator.validate(&config);
    
    // 默认配置应该验证通过
    assert!(result.is_ok(), "默认配置应该验证通过");
}

#[test]
fn test_is_first_run() {
    // 创建配置加载器
    let loader = ConfigLoader::new();
    
    // 检查不存在的配置文件是否被认为是首次运行
    let is_first_run = loader.is_first_run(Some("non_existent_config.yaml"));
    assert!(is_first_run, "不存在的配置文件应该被认为是首次运行");
}
