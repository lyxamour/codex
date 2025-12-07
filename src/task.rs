use clap::{arg, Subcommand};
use serde::{Deserialize, Serialize};
use sled::{Db, IVec};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// 任务状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 待办任务
    Todo,
    /// 进行中任务
    InProgress,
    /// 已完成任务
    Completed,
    /// 已取消任务
    Cancelled,
}

/// 任务优先级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    /// 低优先级
    Low,
    /// 中优先级
    Medium,
    /// 高优先级
    High,
    /// 紧急优先级
    Urgent,
}

/// 任务结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务唯一标识符
    pub id: String,
    /// 任务标题
    pub title: String,
    /// 任务描述
    pub description: Option<String>,
    /// 任务状态
    pub status: TaskStatus,
    /// 任务优先级
    pub priority: TaskPriority,
    /// 任务创建时间（Unix时间戳）
    pub created_at: u64,
    /// 任务更新时间（Unix时间戳）
    pub updated_at: u64,
    /// 任务截止时间（Unix时间戳，可选）
    pub due_at: Option<u64>,
    /// 任务依赖的其他任务ID列表
    pub dependencies: Vec<String>,
    /// 任务标签列表
    pub tags: Vec<String>,
    /// 任务完成时间（Unix时间戳，可选）
    pub completed_at: Option<u64>,
}

impl Task {
    /// 创建新任务
    pub fn new(title: String, description: Option<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        
        Self {
            id: format!("task_{}_{}", now, rand::random::<u32>()),
            title,
            description,
            status: TaskStatus::Todo,
            priority: TaskPriority::Medium,
            created_at: now,
            updated_at: now,
            due_at: None,
            dependencies: Vec::new(),
            tags: Vec::new(),
            completed_at: None,
        }
    }
    
    /// 更新任务状态
    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        
        // 如果任务完成，记录完成时间
        if status == TaskStatus::Completed {
            self.completed_at = Some(self.updated_at);
        } else {
            self.completed_at = None;
        }
    }
    
    /// 更新任务优先级
    pub fn update_priority(&mut self, priority: TaskPriority) {
        self.priority = priority;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
    }
    
    /// 添加任务依赖
    pub fn add_dependency(&mut self, dependency_id: String) {
        if !self.dependencies.contains(&dependency_id) {
            self.dependencies.push(dependency_id);
            self.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
        }
    }
    
    /// 移除任务依赖
    pub fn remove_dependency(&mut self, dependency_id: &str) {
        if let Some(index) = self.dependencies.iter().position(|id| id == dependency_id) {
            self.dependencies.remove(index);
            self.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
        }
    }
    
    /// 添加任务标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
        }
    }
    
    /// 移除任务标签
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(index) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(index);
            self.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
        }
    }
}

/// 任务管理器结构体
#[derive(Clone)]
pub struct TaskManager {
    /// 任务存储实例
    storage: Arc<RwLock<TaskStorage>>,
}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, sled::Error> {
        let storage = Arc::new(RwLock::new(TaskStorage::new(path)?));
        Ok(Self {
            storage,
        })
    }
    
    /// 创建新任务
    pub fn create_task(&self, title: String, description: Option<String>) -> Result<Task, sled::Error> {
        let mut task = Task::new(title, description);
        self.storage.write().unwrap().save_task(&task)?;
        Ok(task)
    }
    
    /// 获取任务
    pub fn get_task(&self, id: &str) -> Result<Option<Task>, sled::Error> {
        self.storage.read().unwrap().get_task(id)
    }
    
    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Result<Vec<Task>, sled::Error> {
        self.storage.read().unwrap().get_all_tasks()
    }
    
    /// 更新任务
    pub fn update_task(&self, task: &Task) -> Result<(), sled::Error> {
        self.storage.write().unwrap().update_task(task)
    }
    
    /// 删除任务
    pub fn delete_task(&self, id: &str) -> Result<Option<Task>, sled::Error> {
        self.storage.write().unwrap().delete_task(id)
    }
    
    /// 根据状态获取任务
    pub fn get_tasks_by_status(&self, status: TaskStatus) -> Result<Vec<Task>, sled::Error> {
        self.storage.read().unwrap().get_tasks_by_status(status)
    }
    
    /// 根据优先级获取任务
    pub fn get_tasks_by_priority(&self, priority: TaskPriority) -> Result<Vec<Task>, sled::Error> {
        self.storage.read().unwrap().get_tasks_by_priority(priority)
    }
    
    /// 根据标签获取任务
    pub fn get_tasks_by_tag(&self, tag: &str) -> Result<Vec<Task>, sled::Error> {
        self.storage.read().unwrap().get_tasks_by_tag(tag)
    }
}

/// 任务操作枚举（用于CLI子命令）
#[derive(Debug, Clone, Subcommand)]
pub enum TaskActions {
    /// 添加任务
    Add {
        /// 任务标题
        title: String,
        
        /// 任务描述（可选）
        #[arg(long, short)]
        description: Option<String>,
        
        /// 任务优先级（可选）
        #[arg(long, short)]
        priority: Option<String>,
        
        /// 任务截止时间（可选，格式：YYYY-MM-DD HH:MM）
        #[arg(long)]
        due: Option<String>,
    },
    
    /// 列出任务
    List {
        /// 按状态筛选（可选）
        #[arg(long, short)]
        status: Option<String>,
        
        /// 按优先级筛选（可选）
        #[arg(long, short)]
        priority: Option<String>,
        
        /// 显示详细信息
        #[arg(long, short)]
        details: bool,
    },
    
    /// 标记任务为完成
    Complete {
        /// 任务ID
        id: String,
    },
    
    /// 删除任务
    Delete {
        /// 任务ID
        id: String,
    },
    
    /// 更新任务
    Update {
        /// 任务ID
        id: String,
        
        /// 新的任务标题（可选）
        #[arg(long, short)]
        title: Option<String>,
        
        /// 新的任务描述（可选）
        #[arg(long)]
        description: Option<String>,
        
        /// 新的任务状态（可选）
        #[arg(long)]
        status: Option<String>,
        
        /// 新的任务优先级（可选）
        #[arg(long)]
        priority: Option<String>,
        
        /// 新的任务截止时间（可选，格式：YYYY-MM-DD HH:MM）
        #[arg(long)]
        due: Option<String>,
    },
    
    /// 查看任务详情
    Show {
        /// 任务ID
        id: String,
    },
}

/// 任务操作trait
pub trait TaskManagerActions {
    /// 创建任务
    fn create_task(&self, title: String, description: Option<String>) -> Result<Task, sled::Error>;
    
    /// 获取任务
    fn get_task(&self, id: &str) -> Result<Option<Task>, sled::Error>;
    
    /// 获取所有任务
    fn get_all_tasks(&self) -> Result<Vec<Task>, sled::Error>;
    
    /// 更新任务
    fn update_task(&self, task: &Task) -> Result<(), sled::Error>;
    
    /// 删除任务
    fn delete_task(&self, id: &str) -> Result<Option<Task>, sled::Error>;
}

impl TaskManagerActions for TaskManager {
    fn create_task(&self, title: String, description: Option<String>) -> Result<Task, sled::Error> {
        self.create_task(title, description)
    }
    
    fn get_task(&self, id: &str) -> Result<Option<Task>, sled::Error> {
        self.get_task(id)
    }
    
    fn get_all_tasks(&self) -> Result<Vec<Task>, sled::Error> {
        self.get_all_tasks()
    }
    
    fn update_task(&self, task: &Task) -> Result<(), sled::Error> {
        self.update_task(task)
    }
    
    fn delete_task(&self, id: &str) -> Result<Option<Task>, sled::Error> {
        self.delete_task(id)
    }
}

/// 任务调度器结构体
pub struct TaskScheduler {
    /// 任务管理器实例
    task_manager: Arc<TaskManager>,
    /// 最大重试次数
    max_retries: u32,
    /// 重试间隔（秒）
    retry_interval: u32,
}

impl TaskScheduler {
    /// 创建新的任务调度器
    pub fn new(task_manager: Arc<TaskManager>, max_retries: u32, retry_interval: u32) -> Self {
        Self {
            task_manager,
            max_retries,
            retry_interval,
        }
    }
    
    /// 调度任务执行
    pub fn schedule_task(&self, task_id: &str) -> Result<(), sled::Error> {
        // 这里实现任务调度逻辑
        // 1. 检查任务是否存在
        // 2. 检查任务依赖是否已完成
        // 3. 根据优先级调度任务
        // 4. 处理任务超时
        // 5. 实现重试机制
        
        let task = self.task_manager.get_task(task_id)?
            .unwrap_or_else(|| panic!("Task not found: {}", task_id));
        
        // 检查任务依赖
        if !self.are_dependencies_completed(&task) {
            // 依赖未完成，暂时不调度
            return Ok(());
        }
        
        // 标记任务为进行中
        let mut task = task.clone();
        task.update_status(TaskStatus::InProgress);
        self.task_manager.update_task(&task)?;
        
        // 这里应该启动一个异步任务来执行实际的任务逻辑
        // 为了简化，我们现在只更新任务状态
        
        Ok(())
    }
    
    /// 检查任务依赖是否已完成
    fn are_dependencies_completed(&self, task: &Task) -> bool {
        for dep_id in &task.dependencies {
            if let Some(dep_task) = self.task_manager.get_task(dep_id).unwrap() {
                if dep_task.status != TaskStatus::Completed {
                    return false;
                }
            } else {
                // 依赖任务不存在，视为未完成
                return false;
            }
        }
        true
    }
    
    /// 处理任务完成
    pub fn complete_task(&self, task_id: &str) -> Result<(), sled::Error> {
        let task = self.task_manager.get_task(task_id)?
            .unwrap_or_else(|| panic!("Task not found: {}", task_id));
        
        let mut task = task.clone();
        task.update_status(TaskStatus::Completed);
        self.task_manager.update_task(&task)?;
        
        // 任务完成后，检查是否有依赖于该任务的其他任务可以调度
        self.schedule_dependent_tasks(task_id)?;
        
        Ok(())
    }
    
    /// 处理任务失败
    pub fn fail_task(&self, task_id: &str) -> Result<(), sled::Error> {
        let task = self.task_manager.get_task(task_id)?
            .unwrap_or_else(|| panic!("Task not found: {}", task_id));
        
        // 这里可以实现重试逻辑
        // 目前简化处理，直接标记为失败
        let mut task = task.clone();
        task.update_status(TaskStatus::Cancelled);
        self.task_manager.update_task(&task)?;
        
        Ok(())
    }
    
    /// 调度依赖于指定任务的其他任务
    fn schedule_dependent_tasks(&self, completed_task_id: &str) -> Result<(), sled::Error> {
        // 获取所有任务
        let all_tasks = self.task_manager.get_all_tasks()?;
        
        // 查找依赖于已完成任务的所有任务
        for task in all_tasks {
            if task.dependencies.contains(&completed_task_id.to_string()) {
                // 检查该任务的所有依赖是否都已完成
                if self.are_dependencies_completed(&task) {
                    // 调度该任务
                    self.schedule_task(&task.id)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// 获取待调度的任务列表（按优先级排序）
    pub fn get_pending_tasks(&self) -> Result<Vec<Task>, sled::Error> {
        let mut pending_tasks = self.task_manager.get_tasks_by_status(TaskStatus::Todo)?;
        
        // 按优先级排序
        pending_tasks.sort_by(|a, b| {
            // 优先级顺序：Urgent > High > Medium > Low
            let priority_order = |p: &TaskPriority| match p {
                TaskPriority::Urgent => 0,
                TaskPriority::High => 1,
                TaskPriority::Medium => 2,
                TaskPriority::Low => 3,
            };
            priority_order(&a.priority).cmp(&priority_order(&b.priority))
        });
        
        Ok(pending_tasks)
    }
}

/// 任务存储结构体
pub struct TaskStorage {
    /// Sled数据库实例
    db: Db,
}

impl TaskStorage {
    /// 创建新的任务存储
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, sled::Error> {
        let db = Db::open(path)?;
        Ok(Self { db })
    }
    
    /// 保存任务
    pub fn save_task(&self, task: &Task) -> Result<(), sled::Error> {
        let task_data = serde_json::to_vec(task).expect("Failed to serialize task");
        self.db.insert(task.id.as_bytes(), task_data)?;
        Ok(())
    }
    
    /// 根据ID获取任务
    pub fn get_task(&self, id: &str) -> Result<Option<Task>, sled::Error> {
        match self.db.get(id.as_bytes())? {
            Some(task_data) => {
                let task: Task = serde_json::from_slice(&task_data).expect("Failed to deserialize task");
                Ok(Some(task))
            },
            None => Ok(None),
        }
    }
    
    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Result<Vec<Task>, sled::Error> {
        let mut tasks = Vec::new();
        
        for result in self.db.iter() {
            let (_, task_data) = result?;
            let task: Task = serde_json::from_slice(&task_data).expect("Failed to deserialize task");
            tasks.push(task);
        }
        
        Ok(tasks)
    }
    
    /// 删除任务
    pub fn delete_task(&self, id: &str) -> Result<Option<Task>, sled::Error> {
        match self.db.remove(id.as_bytes())? {
            Some(task_data) => {
                let task: Task = serde_json::from_slice(&task_data).expect("Failed to deserialize task");
                Ok(Some(task))
            },
            None => Ok(None),
        }
    }
    
    /// 更新任务
    pub fn update_task(&self, task: &Task) -> Result<(), sled::Error> {
        self.save_task(task)
    }
    
    /// 根据状态获取任务
    pub fn get_tasks_by_status(&self, status: TaskStatus) -> Result<Vec<Task>, sled::Error> {
        let all_tasks = self.get_all_tasks()?;
        let filtered_tasks = all_tasks
            .into_iter()
            .filter(|task| task.status == status)
            .collect();
        Ok(filtered_tasks)
    }
    
    /// 根据优先级获取任务
    pub fn get_tasks_by_priority(&self, priority: TaskPriority) -> Result<Vec<Task>, sled::Error> {
        let all_tasks = self.get_all_tasks()?;
        let filtered_tasks = all_tasks
            .into_iter()
            .filter(|task| task.priority == priority)
            .collect();
        Ok(filtered_tasks)
    }
    
    /// 根据标签获取任务
    pub fn get_tasks_by_tag(&self, tag: &str) -> Result<Vec<Task>, sled::Error> {
        let all_tasks = self.get_all_tasks()?;
        let filtered_tasks = all_tasks
            .into_iter()
            .filter(|task| task.tags.contains(&tag.to_string()))
            .collect();
        Ok(filtered_tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_creation() {
        let task = Task::new("Test Task".to_string(), Some("Test Description".to_string()));
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, Some("Test Description".to_string()));
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, TaskPriority::Medium);
        assert!(task.dependencies.is_empty());
        assert!(task.tags.is_empty());
    }
    
    #[test]
    fn test_task_status_update() {
        let mut task = Task::new("Test Task".to_string(), None);
        task.update_status(TaskStatus::InProgress);
        assert_eq!(task.status, TaskStatus::InProgress);
        
        task.update_status(TaskStatus::Completed);
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());
        
        task.update_status(TaskStatus::InProgress);
        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.completed_at.is_none());
    }
    
    #[test]
    fn test_task_priority_update() {
        let mut task = Task::new("Test Task".to_string(), None);
        task.update_priority(TaskPriority::High);
        assert_eq!(task.priority, TaskPriority::High);
        
        task.update_priority(TaskPriority::Urgent);
        assert_eq!(task.priority, TaskPriority::Urgent);
    }
    
    #[test]
    fn test_task_dependencies() {
        let mut task = Task::new("Test Task".to_string(), None);
        let dependency_id = "task_1234567890_12345".to_string();
        
        task.add_dependency(dependency_id.clone());
        assert!(task.dependencies.contains(&dependency_id));
        
        task.remove_dependency(&dependency_id);
        assert!(!task.dependencies.contains(&dependency_id));
    }
    
    #[test]
    fn test_task_tags() {
        let mut task = Task::new("Test Task".to_string(), None);
        let tag = "work".to_string();
        
        task.add_tag(tag.clone());
        assert!(task.tags.contains(&tag));
        
        task.remove_tag(&tag);
        assert!(!task.tags.contains(&tag));
    }
    
    #[test]
    fn test_task_storage() {
        // 创建临时目录用于测试
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        
        // 创建任务存储
        let storage = TaskStorage::new(temp_dir.path()).expect("Failed to create task storage");
        
        // 创建测试任务
        let mut task = Task::new("Test Task".to_string(), Some("Test Description".to_string()));
        task.add_tag("test".to_string());
        task.add_tag("work".to_string());
        
        // 保存任务
        storage.save_task(&task).expect("Failed to save task");
        
        // 根据ID获取任务
        let retrieved_task = storage.get_task(&task.id).expect("Failed to get task").expect("Task not found");
        assert_eq!(retrieved_task.id, task.id);
        assert_eq!(retrieved_task.title, task.title);
        assert_eq!(retrieved_task.description, task.description);
        assert_eq!(retrieved_task.tags, task.tags);
        
        // 获取所有任务
        let all_tasks = storage.get_all_tasks().expect("Failed to get all tasks");
        assert_eq!(all_tasks.len(), 1);
        
        // 更新任务
        task.update_status(TaskStatus::InProgress);
        task.update_priority(TaskPriority::High);
        storage.update_task(&task).expect("Failed to update task");
        
        let updated_task = storage.get_task(&task.id).expect("Failed to get updated task").expect("Updated task not found");
        assert_eq!(updated_task.status, TaskStatus::InProgress);
        assert_eq!(updated_task.priority, TaskPriority::High);
        
        // 根据状态获取任务
        let in_progress_tasks = storage.get_tasks_by_status(TaskStatus::InProgress).expect("Failed to get in progress tasks");
        assert_eq!(in_progress_tasks.len(), 1);
        
        let todo_tasks = storage.get_tasks_by_status(TaskStatus::Todo).expect("Failed to get todo tasks");
        assert_eq!(todo_tasks.len(), 0);
        
        // 根据优先级获取任务
        let high_priority_tasks = storage.get_tasks_by_priority(TaskPriority::High).expect("Failed to get high priority tasks");
        assert_eq!(high_priority_tasks.len(), 1);
        
        // 根据标签获取任务
        let test_tag_tasks = storage.get_tasks_by_tag("test").expect("Failed to get tasks by tag");
        assert_eq!(test_tag_tasks.len(), 1);
        
        let work_tag_tasks = storage.get_tasks_by_tag("work").expect("Failed to get tasks by tag");
        assert_eq!(work_tag_tasks.len(), 1);
        
        // 删除任务
        let deleted_task = storage.delete_task(&task.id).expect("Failed to delete task").expect("Deleted task not found");
        assert_eq!(deleted_task.id, task.id);
        
        // 确认任务已删除
        let deleted_check = storage.get_task(&task.id).expect("Failed to check deleted task");
        assert!(deleted_check.is_none());
        
        let all_tasks_after_delete = storage.get_all_tasks().expect("Failed to get all tasks after delete");
        assert_eq!(all_tasks_after_delete.len(), 0);
    }
}