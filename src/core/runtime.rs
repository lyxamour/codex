//! 运行时管理
//!
//! 管理应用程序的运行时环境和资源

use crate::error::AppResult;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// 应用程序运行时
pub struct AppRuntime {
    /// Tokio运行时
    runtime: Runtime,
    /// 运行状态标志
    is_running: Arc<AtomicBool>,
}

impl AppRuntime {
    /// 创建新的应用运行时实例
    pub fn new() -> AppResult<Self> {
        // 初始化Tokio运行时
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        Ok(Self {
            runtime,
            is_running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// 获取Tokio运行时引用
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    /// 启动运行时
    pub fn start(&self) {
        self.is_running.store(true, Ordering::SeqCst);
    }

    /// 停止运行时
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// 检查运行时是否正在运行
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// 阻塞运行直到停止
    pub fn block_until_stop(&self) {
        while self.is_running() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
