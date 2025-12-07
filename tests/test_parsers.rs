use codex::parsers::{initialize_parsers, PARSER_REGISTRY};

#[test]
fn test_parser_registry_initialize() {
    // 初始化解析器注册表
    let result = initialize_parsers();
    assert!(result.is_ok(), "解析器注册表初始化失败");

    // 检查注册表中是否包含所有内置解析器
    let registry = PARSER_REGISTRY.read().unwrap();
    let supported_languages = registry.supported_languages();

    // 应该支持rust、python、javascript、typescript
    assert!(supported_languages.contains(&"rust".to_string()));
    assert!(supported_languages.contains(&"python".to_string()));
    assert!(supported_languages.contains(&"javascript".to_string()));
    assert!(supported_languages.contains(&"typescript".to_string()));

    // 检查支持的扩展名
    let supported_extensions = registry.supported_extensions();
    assert!(supported_extensions.contains(&"rs".to_string()));
    assert!(supported_extensions.contains(&"py".to_string()));
    assert!(supported_extensions.contains(&"js".to_string()));
    assert!(supported_extensions.contains(&"ts".to_string()));
}

#[test]
fn test_rust_parser() {
    // 初始化解析器
    initialize_parsers().unwrap();

    // 获取Rust解析器
    let registry = PARSER_REGISTRY.read().unwrap();
    let rust_parser = registry.get_parser_by_language("rust").unwrap();

    // 测试Rust代码解析
    let rust_code = r#"fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

struct Point {
    x: i32,
    y: i32,
}"#;

    let result = rust_parser.parse_snippet(rust_code);
    assert!(result.is_ok(), "Rust代码解析失败");

    let elements = result.unwrap();
    // 应该至少解析出2个函数
    let functions = elements
        .iter()
        .filter(|e| e.element_type == codex::parsers::CodeElementType::Function)
        .collect::<Vec<_>>();
    assert!(functions.len() >= 2, "应该解析出至少2个函数");

    // 检查函数名是否正确
    let function_names: Vec<_> = functions.iter().map(|f| &f.name[..]).collect();
    assert!(function_names.contains(&"main"), "应该包含main函数");
    assert!(function_names.contains(&"add"), "应该包含add函数");
}

#[test]
fn test_python_parser() {
    // 初始化解析器
    initialize_parsers().unwrap();

    // 获取Python解析器
    let registry = PARSER_REGISTRY.read().unwrap();
    let python_parser = registry.get_parser_by_language("python").unwrap();

    // 测试Python代码解析
    let python_code = r#"def main():
    print("Hello, world!")

def add(a, b):
    return a + b

class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
"#;

    let result = python_parser.parse_snippet(python_code);
    assert!(result.is_ok(), "Python代码解析失败");

    let elements = result.unwrap();
    // 应该至少解析出3个函数（main, add, __init__）
    let functions = elements
        .iter()
        .filter(|e| e.element_type == codex::parsers::CodeElementType::Function)
        .collect::<Vec<_>>();
    assert!(functions.len() >= 3, "应该解析出至少3个函数");

    // 检查函数名是否正确
    let function_names: Vec<_> = functions.iter().map(|f| &f.name[..]).collect();
    assert!(function_names.contains(&"main"), "应该包含main函数");
    assert!(function_names.contains(&"add"), "应该包含add函数");
}

#[test]
fn test_javascript_parser() {
    // 初始化解析器
    initialize_parsers().unwrap();

    // 获取JavaScript解析器
    let registry = PARSER_REGISTRY.read().unwrap();
    let js_parser = registry.get_parser_by_language("javascript").unwrap();

    // 测试JavaScript代码解析
    let js_code = r#"function main() {
    console.log("Hello, world!");
}

const add = (a, b) => {
    return a + b;
};

function multiply(a, b) {
    return a * b;
}"#;

    let result = js_parser.parse_snippet(js_code);
    assert!(result.is_ok(), "JavaScript代码解析失败");

    let elements = result.unwrap();
    // 打印所有解析出的元素，用于调试
    println!("JavaScript解析结果：");
    for element in &elements {
        println!("  {:?}: {}", element.element_type, element.name);
    }
    // 应该至少解析出3个函数
    let functions = elements
        .iter()
        .filter(|e| e.element_type == codex::parsers::CodeElementType::Function)
        .collect::<Vec<_>>();
    println!("解析出的函数数量：{}", functions.len());
    assert!(functions.len() >= 3, "应该解析出至少3个函数");
}

#[test]
fn test_typescript_parser() {
    // 初始化解析器
    initialize_parsers().unwrap();

    // 获取TypeScript解析器
    let registry = PARSER_REGISTRY.read().unwrap();
    let ts_parser = registry.get_parser_by_language("typescript").unwrap();

    // 测试TypeScript代码解析
    let ts_code = r#"function main(): void {
    console.log("Hello, world!");
}

interface Point {
    x: number;
    y: number;
}

const add = (a: number, b: number): number => {
    return a + b;
};
"#;

    let result = ts_parser.parse_snippet(ts_code);
    assert!(result.is_ok(), "TypeScript代码解析失败");

    let elements = result.unwrap();
    // 应该至少解析出2个函数和1个接口
    let functions = elements
        .iter()
        .filter(|e| e.element_type == codex::parsers::CodeElementType::Function)
        .collect::<Vec<_>>();
    let interfaces = elements
        .iter()
        .filter(|e| e.element_type == codex::parsers::CodeElementType::Interface)
        .collect::<Vec<_>>();

    assert!(functions.len() >= 2, "应该解析出至少2个函数");
    assert!(interfaces.len() >= 1, "应该解析出至少1个接口");

    // 检查接口名是否正确
    let interface_names: Vec<_> = interfaces.iter().map(|i| &i.name[..]).collect();
    assert!(interface_names.contains(&"Point"), "应该包含Point接口");
}

#[test]
fn test_parser_by_extension() {
    // 初始化解析器
    initialize_parsers().unwrap();

    // 获取解析器注册表
    let registry = PARSER_REGISTRY.read().unwrap();

    // 测试根据扩展名获取解析器
    let rust_parser = registry.get_parser_by_extension("rs");
    assert!(rust_parser.is_some(), "应该根据.rs扩展名获取到Rust解析器");

    let python_parser = registry.get_parser_by_extension("py");
    assert!(
        python_parser.is_some(),
        "应该根据.py扩展名获取到Python解析器"
    );

    let js_parser = registry.get_parser_by_extension("js");
    assert!(
        js_parser.is_some(),
        "应该根据.js扩展名获取到JavaScript解析器"
    );

    let ts_parser = registry.get_parser_by_extension("ts");
    assert!(
        ts_parser.is_some(),
        "应该根据.ts扩展名获取到TypeScript解析器"
    );
}

#[test]
fn test_parser_by_filename() {
    // 初始化解析器
    initialize_parsers().unwrap();

    // 获取解析器注册表
    let registry = PARSER_REGISTRY.read().unwrap();

    // 测试根据文件名获取解析器
    let rust_parser = registry.get_parser_by_filename("main.rs");
    assert!(
        rust_parser.is_some(),
        "应该根据main.rs文件名获取到Rust解析器"
    );

    let python_parser = registry.get_parser_by_filename("script.py");
    assert!(
        python_parser.is_some(),
        "应该根据script.py文件名获取到Python解析器"
    );

    let js_parser = registry.get_parser_by_filename("app.js");
    assert!(
        js_parser.is_some(),
        "应该根据app.js文件名获取到JavaScript解析器"
    );

    let ts_parser = registry.get_parser_by_filename("component.ts");
    assert!(
        ts_parser.is_some(),
        "应该根据component.ts文件名获取到TypeScript解析器"
    );
}
