use codex::config::loader::ConfigLoader;
use codex::config::validator::ConfigValidator;
use std::env;

#[test]
fn test_default_config() {
    // 保存原始环境变量
    let original_theme = env::var("CODEX_UI_THEME").ok();
    let original_language = env::var("CODEX_APP_LANGUAGE").ok();
    let original_model = env::var("CODEX_AI_OPENAI_DEFAULT_MODEL").ok();

    // 清除所有可能影响测试的环境变量
    std::env::remove_var("CODEX_APP_LANGUAGE");
    std::env::remove_var("CODEX_AI_OPENAI_DEFAULT_MODEL");
    std::env::remove_var("CODEX_UI_THEME");
    std::env::remove_var("CODEX_UI_ANIMATIONS");
    std::env::remove_var("CODEX_UI_COLORED");
    std::env::remove_var("CODEX_UI_FONT_SIZE");

    // 创建配置加载器
    let loader = ConfigLoader::new();

    // 直接获取默认配置，而不通过load方法，这样可以避免环境变量影响
    let config = loader.get_default_config();

    // 检查默认值
    assert_eq!(config.app.name, "codex");
    assert_eq!(config.app.language, "en");
    assert_eq!(config.ai.default_platform, "openai");
    assert_eq!(config.ui.theme, "default");

    // 恢复原始环境变量
    if let Some(theme) = original_theme {
        env::set_var("CODEX_UI_THEME", theme);
    } else {
        env::remove_var("CODEX_UI_THEME");
    }

    if let Some(language) = original_language {
        env::set_var("CODEX_APP_LANGUAGE", language);
    } else {
        env::remove_var("CODEX_APP_LANGUAGE");
    }

    if let Some(model) = original_model {
        env::set_var("CODEX_AI_OPENAI_DEFAULT_MODEL", model);
    } else {
        env::remove_var("CODEX_AI_OPENAI_DEFAULT_MODEL");
    }
}

#[test]
fn test_env_vars_override() {
    // 保存原始环境变量
    let original_theme = env::var("CODEX_UI_THEME").ok();
    let original_language = env::var("CODEX_APP_LANGUAGE").ok();
    let original_model = env::var("CODEX_AI_OPENAI_DEFAULT_MODEL").ok();

    // 创建一个临时目录，确保没有现有的配置文件干扰测试
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_config_path = temp_dir.path().join("config.yaml");

    // 设置环境变量
    env::set_var("CODEX_APP_LANGUAGE", "zh-CN");
    env::set_var("CODEX_AI_OPENAI_DEFAULT_MODEL", "gpt-4");
    env::set_var("CODEX_UI_THEME", "dark");

    // 创建配置加载器
    let loader = ConfigLoader::new();

    // 获取默认配置并保存到临时文件
    let default_config = loader.get_default_config();

    // 将默认配置写入临时文件
    std::fs::write(
        &temp_config_path,
        serde_yaml::to_string(&default_config).unwrap(),
    )
    .unwrap();

    // 从临时文件加载配置（应该会被环境变量覆盖）
    let config = loader
        .load(Some(temp_config_path.to_str().unwrap()))
        .unwrap();

    // 验证环境变量覆盖是否生效
    assert_eq!(config.app.language, "zh-CN");
    assert_eq!(config.ai.openai.unwrap().default_model, "gpt-4");
    assert_eq!(config.ui.theme, "dark");

    // 恢复原始环境变量
    if let Some(theme) = original_theme {
        env::set_var("CODEX_UI_THEME", theme);
    } else {
        env::remove_var("CODEX_UI_THEME");
    }

    if let Some(language) = original_language {
        env::set_var("CODEX_APP_LANGUAGE", language);
    } else {
        env::remove_var("CODEX_APP_LANGUAGE");
    }

    if let Some(model) = original_model {
        env::set_var("CODEX_AI_OPENAI_DEFAULT_MODEL", model);
    } else {
        env::remove_var("CODEX_AI_OPENAI_DEFAULT_MODEL");
    }
}

#[test]
fn test_config_validation() {
    // 创建配置加载器
    let loader = ConfigLoader::new();

    // 加载默认配置
    let mut config = loader.load(None).unwrap();

    // 为了验证通过，需要设置一个有效的API密钥
    config.ai.openai = Some(codex::config::app::OpenAIConfig {
        api_key: "test_api_key".to_string(),
        default_model: "gpt-4o".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
    });

    // 创建配置验证器
    let validator = ConfigValidator::new();

    // 验证配置
    let result = validator.validate(&config);

    // 配置应该验证通过
    assert!(result.is_ok(), "配置应该验证通过");
}

#[test]
fn test_is_first_run() {
    // 创建配置加载器
    let loader = ConfigLoader::new();

    // 检查不存在的配置文件是否被认为是首次运行
    let is_first_run = loader.is_first_run(Some("non_existent_config.yaml"));
    assert!(is_first_run, "不存在的配置文件应该被认为是首次运行");
}
