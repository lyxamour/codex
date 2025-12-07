//! 文件操作工具
//!
//! 提供文件读取、写入、编辑等操作

use crate::error::AppResult;

/// 读取文件内容
pub fn read_file(path: &str, encoding: Option<&str>) -> AppResult<String> {
    // 使用std::fs::read_to_string()函数读取文件内容
    // 目前暂不处理encoding参数，直接使用系统默认编码
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

/// 写入文件内容
pub fn write_file(path: &str, content: &str, append: bool) -> AppResult<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    // 根据append参数决定是覆盖还是追加写入
    let mut options = OpenOptions::new();

    if append {
        options.append(true);
    } else {
        options.write(true);
        options.truncate(true);
    }

    options.create(true);

    let mut file = options.open(path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}
