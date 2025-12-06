//! Shell命令工具
//!
//! 执行Shell命令并返回结果

use crate::error::AppResult;

/// 执行Shell命令
pub fn execute_command(command: &str, cwd: Option<&str>) -> AppResult<String> {
    use std::process::Command;

    // 解析命令和参数
    let mut parts = match shell_words::split(command) {
        Ok(parts) => parts,
        Err(e) => {
            return Err(crate::error::AppError::command(&format!(
                "命令解析失败: {}",
                e
            )));
        }
    };

    if parts.is_empty() {
        return Ok("".to_string());
    }

    let cmd = parts.remove(0);

    // 创建命令对象
    let mut command = Command::new(cmd);

    // 添加命令参数
    command.args(parts);

    // 设置工作目录
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    // 执行命令并获取输出
    let output = command.output()?;

    if output.status.success() {
        // 命令执行成功，返回标准输出
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        // 命令执行失败，返回错误信息
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(crate::error::AppError::command(&stderr))
    }
}
