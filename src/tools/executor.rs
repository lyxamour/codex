//! 工具执行器
//!
//! 执行注册的工具并处理执行结果

use crate::error::AppResult;
use crate::tools::registry::ToolMetadata;

/// 工具执行结果
pub enum ToolResult {
    /// 成功执行
    Success(String),
    /// 执行失败
    Error(String),
    /// 超时
    Timeout,
}

/// 工具执行器
pub struct ToolExecutor {
    /// 工具注册表引用
    registry: std::sync::Arc<std::sync::RwLock<crate::tools::registry::ToolRegistry>>,
}

impl ToolExecutor {
    /// 创建新的工具执行器实例
    pub fn new(
        registry: std::sync::Arc<std::sync::RwLock<crate::tools::registry::ToolRegistry>>,
    ) -> Self {
        Self { registry }
    }

    /// 执行工具
    pub async fn execute(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> AppResult<ToolResult> {
        // 1. 检查工具是否存在
        let registry = self
            .registry
            .read()
            .map_err(|e| crate::error::AppError::tool(&format!("无法获取工具注册表: {}", e)))?;

        let tool_metadata = match registry.get_tool(tool_name) {
            Some(tool) => tool,
            None => return Ok(ToolResult::Error(format!("工具 {} 不存在", tool_name))),
        };

        // 2. 验证参数
        if let Err(e) = self.validate_params(tool_metadata, &params) {
            return Ok(ToolResult::Error(format!("参数验证失败: {}", e)));
        }

        // 3. 执行工具
        match tool_name {
            "read_file" => self.execute_read_file(params).await,
            "write_file" => self.execute_write_file(params).await,
            _ => Ok(ToolResult::Error(format!("工具 {} 暂未实现", tool_name))),
        }
    }

    /// 验证参数
    fn validate_params(
        &self,
        tool_metadata: &crate::tools::registry::ToolMetadata,
        params: &serde_json::Value,
    ) -> Result<(), String> {
        // TODO: 主人~ 这里需要实现参数验证逻辑
        // 提示：检查必填参数是否存在，参数类型是否正确

        // 检查必填参数
        for param in &tool_metadata.parameters {
            if param.required {
                if !params.get(&param.name).is_some() {
                    return Err(format!("缺少必填参数: {}", param.name));
                }
            }
        }

        Ok(())
    }

    /// 执行read_file工具
    async fn execute_read_file(&self, params: serde_json::Value) -> AppResult<ToolResult> {
        // TODO: 主人~ 这里需要实现read_file工具执行逻辑
        // 提示：调用crate::tools::builtin::file::read_file函数
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or(crate::error::AppError::tool("缺少path参数"))?;
        let encoding = params
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("utf-8");

        match crate::tools::builtin::file::read_file(path, Some(encoding)) {
            Ok(content) => Ok(ToolResult::Success(content)),
            Err(e) => Ok(ToolResult::Error(format!("读取文件失败: {}", e))),
        }
    }

    /// 执行write_file工具
    async fn execute_write_file(&self, params: serde_json::Value) -> AppResult<ToolResult> {
        // TODO: 主人~ 这里需要实现write_file工具执行逻辑
        // 提示：调用crate::tools::builtin::file::write_file函数
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or(crate::error::AppError::tool("缺少path参数"))?;
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or(crate::error::AppError::tool("缺少content参数"))?;
        let append = params
            .get("append")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        match crate::tools::builtin::file::write_file(path, content, append) {
            Ok(_) => Ok(ToolResult::Success("文件写入成功".to_string())),
            Err(e) => Ok(ToolResult::Error(format!("文件写入失败: {}", e))),
        }
    }
}
