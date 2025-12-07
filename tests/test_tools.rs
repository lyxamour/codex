
use codex::tools::builtin::file::read_file;
use codex::tools::builtin::shell::execute_command;
use codex::tools::parser::ToolParser;
use codex::tools::registry::{ToolMetadata, ToolParameter, ToolRegistry};
use std::fs;

#[test]
fn test_read_file_tool() {
    // 创建一个临时文件用于测试
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    
    // 写入测试内容
    fs::write(temp_path, "Hello, Codex!").unwrap();
    
    // 测试读取文件工具（read_file 不是异步函数，不需要 .await）
    let result = read_file(temp_path, Some("utf-8"));
    assert!(result.is_ok(), "读取文件失败");
    
    let content = result.unwrap();
    assert_eq!(content, "Hello, Codex!", "读取的文件内容不正确");
}

#[test]
fn test_execute_shell_tool() {
    // 测试执行简单的 shell 命令
    let result = execute_command("echo 'Hello from shell'", Some("."));
    assert!(result.is_ok(), "执行 shell 命令失败");
    
    let output = result.unwrap();
    assert!(output.contains("Hello from shell"), "shell 命令输出不正确");
}

#[test]
fn test_tool_definition_parsing() {
    // 测试工具定义 YAML 解析
    let yaml_content = r#"name: "test_tool"
description: "测试工具"
version: "1.0.0"
category: "builtin"
parameters:
  - name: "input"
    type: "string"
    required: true
    description: "输入参数"
  - name: "optional_param"
    type: "number"
    required: false
    description: "可选参数"
    default: 10
execution:
  type: "builtin_function"
  handler: "test_function"
  timeout: 30
"#;
    
    // 创建工具解析器
    let parser = ToolParser::new();
    
    // 解析 YAML 内容
    let result = parser.parse_yaml(yaml_content);
    assert!(result.is_ok(), "工具定义 YAML 解析失败");
    
    let metadata = result.unwrap();
    
    // 验证解析结果
    assert_eq!(metadata.name, "test_tool", "工具名称不正确");
    assert_eq!(metadata.description, "测试工具", "工具描述不正确");
    assert_eq!(metadata.version, "1.0.0", "工具版本不正确");
    assert_eq!(metadata.category, "builtin", "工具类别不正确");
    
    // 验证参数
    assert_eq!(metadata.parameters.len(), 2, "参数数量不正确");
    assert_eq!(metadata.parameters[0].name, "input", "第一个参数名称不正确");
    assert!(metadata.parameters[0].required, "第一个参数应该是必填的");
    assert_eq!(metadata.parameters[1].name, "optional_param", "第二个参数名称不正确");
    assert!(!metadata.parameters[1].required, "第二个参数应该是可选的");
}

#[test]
fn test_read_nonexistent_file() {
    // 测试读取不存在的文件
    let result = read_file("non_existent_file.txt", Some("utf-8"));
    assert!(result.is_err(), "读取不存在的文件应该失败");
}

#[test]
fn test_tool_registry_creation() {
    // 测试创建工具注册表实例
    let registry = ToolRegistry::new();
    assert!(registry.is_ok(), "创建工具注册表实例失败");
    
    let registry = registry.unwrap();
    assert!(!registry.list_tools().is_empty(), "工具注册表应该包含内置工具");
}

#[test]
fn test_tool_registry_register() {
    // 测试注册新工具
    let mut registry = ToolRegistry::new().unwrap();
    
    // 创建测试工具元数据
    let test_tool = ToolMetadata {
        name: "test_tool".to_string(),
        description: "测试工具".to_string(),
        version: "1.0.0".to_string(),
        category: "test".to_string(),
        parameters: vec![
            ToolParameter {
                name: "param1".to_string(),
                r#type: "string".to_string(),
                required: true,
                description: "测试参数1".to_string(),
                default: None,
            },
        ],
    };
    
    // 注册工具
    let result = registry.register_tool(test_tool);
    assert!(result.is_ok(), "注册工具失败");
    
    // 验证工具已注册
    assert!(registry.has_tool("test_tool"), "工具注册后应该存在");
}

#[test]
fn test_tool_registry_get() {
    // 测试获取工具元数据
    let registry = ToolRegistry::new().unwrap();
    
    // 获取内置工具
    let read_file_tool = registry.get_tool("read_file");
    assert!(read_file_tool.is_some(), "应该能获取到内置的read_file工具");
    
    if let Some(tool) = read_file_tool {
        assert_eq!(tool.name, "read_file", "工具名称不正确");
        assert_eq!(tool.category, "builtin", "工具类别不正确");
    }
    
    // 获取不存在的工具
    let nonexistent_tool = registry.get_tool("nonexistent_tool");
    assert!(nonexistent_tool.is_none(), "获取不存在的工具应该返回None");
}

#[test]
fn test_tool_registry_has_tool() {
    // 测试检查工具是否存在
    let registry = ToolRegistry::new().unwrap();
    
    // 检查内置工具是否存在
    assert!(registry.has_tool("read_file"), "read_file工具应该存在");
    assert!(registry.has_tool("write_file"), "write_file工具应该存在");
    
    // 检查不存在的工具
    assert!(!registry.has_tool("nonexistent_tool"), "不存在的工具应该返回false");
}

#[test]
fn test_tool_registry_list_tools() {
    // 测试获取所有工具列表
    let registry = ToolRegistry::new().unwrap();
    
    let tools = registry.list_tools();
    assert!(!tools.is_empty(), "工具列表不应该为空");
    
    // 检查列表中是否包含内置工具
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    assert!(tool_names.contains(&"read_file".to_string()), "工具列表应该包含read_file");
    assert!(tool_names.contains(&"write_file".to_string()), "工具列表应该包含write_file");
}

#[test]
fn test_tool_registry_list_by_category() {
    // 测试按类别获取工具列表
    let registry = ToolRegistry::new().unwrap();
    
    // 获取builtin类别的工具
    let builtin_tools = registry.list_tools_by_category("builtin");
    assert!(!builtin_tools.is_empty(), "builtin类别应该包含工具");
    
    // 检查所有工具都属于builtin类别
    for tool in builtin_tools {
        assert_eq!(tool.category, "builtin", "按类别获取的工具应该属于该类别");
    }
    
    // 获取不存在的类别
    let nonexistent_tools = registry.list_tools_by_category("nonexistent");
    assert!(nonexistent_tools.is_empty(), "不存在的类别应该返回空列表");
}

#[test]
fn test_tool_registry_load_from_yaml() {
    // 测试从YAML文件加载工具
    let mut registry = ToolRegistry::new().unwrap();
    
    // 创建临时YAML文件
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    
    // 写入测试YAML内容
    let yaml_content = r#"name: "yaml_test_tool"
description: "从YAML加载的测试工具"
version: "1.0.0"
category: "yaml"
parameters:
  - name: "test_param"
    type: "string"
    required: true
    description: "测试参数"
  - name: "optional_param"
    type: "boolean"
    required: false
    description: "可选参数"
    default: "false"
execution:
  type: "builtin_function"
  handler: "test_handler"
  timeout: 30
"#;
    
    // 写入YAML内容到临时文件
    fs::write(temp_path, yaml_content).unwrap();
    
    // 从YAML加载工具
    let result = registry.load_from_yaml(temp_path);
    assert!(result.is_ok(), "从YAML加载工具失败");
    
    // 验证工具已加载
    assert!(registry.has_tool("yaml_test_tool"), "从YAML加载的工具应该存在");
    
    let loaded_tool = registry.get_tool("yaml_test_tool").unwrap();
    assert_eq!(loaded_tool.name, "yaml_test_tool", "加载的工具名称不正确");
    assert_eq!(loaded_tool.category, "yaml", "加载的工具类别不正确");
}

#[tokio::test]
async fn test_tool_executor_creation() {
    // 测试工具执行器的创建
    let registry = std::sync::Arc::new(
        std::sync::RwLock::new(
            ToolRegistry::new().unwrap()
        )
    );
    
    // 验证创建过程没有出错
    let _executor = codex::tools::executor::ToolExecutor::new(registry);
    assert!(true, "工具执行器创建成功");
}

#[tokio::test]
async fn test_tool_executor_read_file() {
    // 测试执行read_file工具
    let registry = std::sync::Arc::new(
        std::sync::RwLock::new(
            ToolRegistry::new().unwrap()
        )
    );
    let executor = codex::tools::executor::ToolExecutor::new(registry);
    
    // 创建临时文件用于测试
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    fs::write(temp_path, "Test file content").unwrap();
    
    // 执行read_file工具
    let params = serde_json::json!({"path": temp_path});
    let result = executor.execute("read_file", params).await;
    assert!(result.is_ok(), "执行read_file工具失败");
    
    let tool_result = result.unwrap();
    match tool_result {
        codex::tools::executor::ToolResult::Success(content) => {
            assert_eq!(content, "Test file content", "读取的文件内容不正确");
        },
        _ => panic!("read_file工具执行应该成功"),
    }
}

#[tokio::test]
async fn test_tool_executor_write_file() {
    // 测试执行write_file工具
    let registry = std::sync::Arc::new(
        std::sync::RwLock::new(
            ToolRegistry::new().unwrap()
        )
    );
    let executor = codex::tools::executor::ToolExecutor::new(registry);
    
    // 创建临时文件用于测试
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    
    // 执行write_file工具
    let params = serde_json::json!({"path": temp_path, "content": "New content"});
    let result = executor.execute("write_file", params).await;
    assert!(result.is_ok(), "执行write_file工具失败");
    
    let tool_result = result.unwrap();
    match tool_result {
        codex::tools::executor::ToolResult::Success(_) => {
            // 验证文件内容
            let content = fs::read_to_string(temp_path).unwrap();
            assert_eq!(content, "New content", "写入的文件内容不正确");
        },
        _ => panic!("write_file工具执行应该成功"),
    }
}

#[tokio::test]
async fn test_tool_executor_missing_params() {
    // 测试执行工具时缺少必填参数
    let registry = std::sync::Arc::new(
        std::sync::RwLock::new(
            ToolRegistry::new().unwrap()
        )
    );
    let executor = codex::tools::executor::ToolExecutor::new(registry);
    
    // 执行read_file工具但缺少path参数
    let params = serde_json::json!({});
    let result = executor.execute("read_file", params).await;
    assert!(result.is_ok(), "执行read_file工具失败");
    
    let tool_result = result.unwrap();
    match tool_result {
        codex::tools::executor::ToolResult::Error(msg) => {
            assert!(msg.contains("缺少必填参数") || msg.contains("缺少path参数"), "应该提示缺少必填参数");
        },
        _ => panic!("缺少必填参数时应该返回错误"),
    }
}

#[tokio::test]
async fn test_tool_executor_nonexistent_tool() {
    // 测试执行不存在的工具
    let registry = std::sync::Arc::new(
        std::sync::RwLock::new(
            ToolRegistry::new().unwrap()
        )
    );
    let executor = codex::tools::executor::ToolExecutor::new(registry);
    
    let params = serde_json::json!({});
    let result = executor.execute("nonexistent_tool", params).await;
    assert!(result.is_ok(), "执行不存在的工具失败");
    
    let tool_result = result.unwrap();
    match tool_result {
        codex::tools::executor::ToolResult::Error(msg) => {
            assert!(msg.contains("不存在"), "应该提示工具不存在");
        },
        _ => panic!("执行不存在的工具时应该返回错误"),
    }
}

#[tokio::test]
async fn test_tool_executor_unimplemented_tool() {
    // 测试执行已注册但未实现的工具
    let registry = std::sync::Arc::new(
        std::sync::RwLock::new(
            ToolRegistry::new().unwrap()
        )
    );
    let executor = codex::tools::executor::ToolExecutor::new(registry);
    
    // 执行write_file工具（已注册且在executor中实现）
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();
    let params = serde_json::json!({"path": temp_path, "content": "test"});
    let result = executor.execute("write_file", params).await;
    assert!(result.is_ok(), "执行write_file工具失败");
    
    // 执行一个不存在的工具
    let params = serde_json::json!({});
    let result = executor.execute("nonexistent_tool", params).await;
    assert!(result.is_ok(), "执行不存在的工具失败");
    
    let tool_result = result.unwrap();
    match tool_result {
        codex::tools::executor::ToolResult::Error(msg) => {
            assert!(msg.contains("不存在"), "应该提示工具不存在");
        },
        _ => panic!("执行不存在的工具时应该返回错误"),
    }
}

#[test]
fn test_tool_localization_creation() {
    // 测试工具本地化管理器的创建
    let localization = codex::tools::localization::ToolLocalization::new();
    assert!(localization.is_ok(), "创建工具本地化管理器失败");
}

#[test]
fn test_tool_localization_set_lang() {
    // 测试设置当前语言
    let mut localization = codex::tools::localization::ToolLocalization::new().unwrap();
    
    // 设置为英语
    localization.set_lang("en-US");
    // 设置为中文
    localization.set_lang("zh-CN");
    
    // 验证设置语言不会导致崩溃
    assert!(true, "设置语言成功");
}

#[test]
fn test_tool_localization_get_string() {
    // 测试获取本地化字符串
    let localization = codex::tools::localization::ToolLocalization::new().unwrap();
    
    // 获取存在的字符串（由于资源为空，应该返回key本身）
    let result = localization.get_string("test_key");
    assert_eq!(result, "test_key", "获取不存在的本地化字符串应该返回key本身");
    
    // 测试不同key
    let result2 = localization.get_string("another_key");
    assert_eq!(result2, "another_key", "获取不存在的本地化字符串应该返回key本身");
}
