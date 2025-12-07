
use codex::tools::builtin::file::read_file;
use codex::tools::builtin::shell::execute_command;
use codex::tools::parser::ToolParser;
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
