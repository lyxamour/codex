use chrono::{Utc, TimeZone};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use uuid::Uuid;

/// Task status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl From<&str> for TaskStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "todo" | "pending" => TaskStatus::Todo,
            "in-progress" | "progress" | "doing" => TaskStatus::InProgress,
            "done" | "completed" => TaskStatus::Done,
            _ => TaskStatus::Todo,
        }
    }
}

impl From<TaskStatus> for &'static str {
    fn from(status: TaskStatus) -> Self {
        match status {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in-progress",
            TaskStatus::Done => "done",
        }
    }
}

/// Task priority enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

impl From<&str> for TaskPriority {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => TaskPriority::Low,
            "medium" => TaskPriority::Medium,
            "high" => TaskPriority::High,
            _ => TaskPriority::Medium,
        }
    }
}

impl From<TaskPriority> for &'static str {
    fn from(priority: TaskPriority) -> Self {
        match priority {
            TaskPriority::Low => "low",
            TaskPriority::Medium => "medium",
            TaskPriority::High => "high",
        }
    }
}

/// Task structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task ID
    pub id: String,
    /// Task description
    pub description: String,
    /// Task status
    pub status: TaskStatus,
    /// Task priority
    pub priority: TaskPriority,
    /// Creation timestamp (Unix seconds)
    pub created_at: i64,
    /// Last update timestamp (Unix seconds)
    pub updated_at: i64,
    /// Completion timestamp (Unix seconds, if completed)
    pub completed_at: Option<i64>,
    /// Task tags
    pub tags: Vec<String>,
    /// Task notes
    pub notes: String,
}

/// Task actions (same as in main.rs for now)
#[derive(Debug, Clone, Subcommand)]
pub enum TaskActions {
    Create { description: String, priority: String },
    List { status: Option<String> },
    Update { 
        id: String, 
        description: Option<String>, 
        status: Option<String>, 
        priority: Option<String> 
    },
    Delete { id: String },
}

/// Task manager for handling CRUD operations
pub struct TaskManager {
    tasks: HashMap<String, Task>,
    storage_path: String,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new().expect("Failed to create task manager")
    }
}

impl TaskManager {
    /// Create a new task manager
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let storage_path = "~/.codex_tasks.json";
        let expanded_path = shellexpand::tilde(storage_path).to_string();
        
        let tasks = Self::load_tasks(&expanded_path)?;
        
        Ok(Self {
            tasks,
            storage_path: expanded_path,
        })
    }
    
    /// Create a new task manager with custom storage path
    pub fn with_storage_path(path: &str) -> Result<Self, Box<dyn Error>> {
        let expanded_path = shellexpand::tilde(path).to_string();
        
        let tasks = Self::load_tasks(&expanded_path)?;
        
        Ok(Self {
            tasks,
            storage_path: expanded_path,
        })
    }
    
    /// Load tasks from storage
    fn load_tasks(path: &str) -> Result<HashMap<String, Task>, Box<dyn Error>> {
        let path = Path::new(path);
        
        if path.exists() {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let tasks: HashMap<String, Task> = serde_json::from_reader(reader)?;
            Ok(tasks)
        } else {
            Ok(HashMap::new())
        }
    }
    
    /// Save tasks to storage
    fn save_tasks(&self) -> Result<(), Box<dyn Error>> {
        let file = File::create(&self.storage_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.tasks)?;
        Ok(())
    }
    
    /// Create a new task
    pub fn create(&mut self, description: String, priority: String) -> Result<Task, Box<dyn Error>> {
        let now = chrono::Utc::now().timestamp();
        let priority = TaskPriority::from(priority.as_str());
        
        let task = Task {
            id: Uuid::new_v4().to_string(),
            description,
            status: TaskStatus::Todo,
            priority,
            created_at: now,
            updated_at: now,
            completed_at: None,
            tags: Vec::new(),
            notes: String::new(),
        };
        
        self.tasks.insert(task.id.clone(), task.clone());
        self.save_tasks()?;
        
        Ok(task)
    }
    
    /// List tasks with optional status filter
    pub fn list(&self, status: Option<&str>) -> Result<Vec<Task>, Box<dyn Error>> {
        let mut tasks: Vec<_> = self.tasks.values().cloned().collect();
        
        // Filter by status if provided
        if let Some(status_str) = status {
            let status = TaskStatus::from(status_str);
            tasks.retain(|task| task.status == status);
        }
        
        // Sort by priority and then by creation time
        tasks.sort_by(|a, b| {
            // First sort by priority (high to low)
            let priority_order = |p: TaskPriority| match p {
                TaskPriority::High => 0,
                TaskPriority::Medium => 1,
                TaskPriority::Low => 2,
            };
            
            let priority_cmp = priority_order(a.priority).cmp(&priority_order(b.priority));
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }
            
            // Then sort by creation time (newest first)
            b.created_at.cmp(&a.created_at)
        });
        
        Ok(tasks)
    }
    
    /// Update a task
    pub fn update(
        &mut self, 
        id: &str, 
        description: Option<String>, 
        status: Option<String>, 
        priority: Option<String>
    ) -> Result<Task, Box<dyn Error>> {
        let task = self.tasks.get_mut(id)
            .ok_or(format!("Task with ID '{}' not found", id))?;
        
        let now = chrono::Utc::now().timestamp();
        task.updated_at = now;
        
        // Update description if provided
        if let Some(description) = description {
            task.description = description;
        }
        
        // Update status if provided
        if let Some(status_str) = status {
            let new_status = TaskStatus::from(status_str.as_str());
            
            // Update completed_at if status changed to/from Done
            if new_status == TaskStatus::Done && task.status != TaskStatus::Done {
                task.completed_at = Some(now);
            } else if new_status != TaskStatus::Done && task.status == TaskStatus::Done {
                task.completed_at = None;
            }
            
            task.status = new_status;
        }
        
        // Update priority if provided
        if let Some(priority_str) = priority {
            task.priority = TaskPriority::from(priority_str.as_str());
        }
        
        let updated_task = task.clone();
        self.save_tasks()?;
        
        Ok(updated_task)
    }
    
    /// Delete a task
    pub fn delete(&mut self, id: &str) -> Result<(), Box<dyn Error>> {
        if self.tasks.remove(id).is_none() {
            return Err(format!("Task with ID '{}' not found", id).into());
        }
        
        self.save_tasks()?;
        Ok(())
    }
    
    /// Get a task by ID
    pub fn get(&self, id: &str) -> Option<&Task> {
        self.tasks.get(id)
    }
    
    /// Add a tag to a task
    pub fn add_tag(&mut self, id: &str, tag: &str) -> Result<Task, Box<dyn Error>> {
        let task = self.tasks.get_mut(id)
            .ok_or(format!("Task with ID '{}' not found", id))?;
        
        if !task.tags.contains(&tag.to_string()) {
            task.tags.push(tag.to_string());
            task.updated_at = chrono::Utc::now().timestamp();
        }
        
        let updated_task = task.clone();
        self.save_tasks()?;
        
        Ok(updated_task)
    }
    
    /// Remove a tag from a task
    pub fn remove_tag(&mut self, id: &str, tag: &str) -> Result<Task, Box<dyn Error>> {
        let task = self.tasks.get_mut(id)
            .ok_or(format!("Task with ID '{}' not found", id))?;
        
        task.tags.retain(|t| t != tag);
        task.updated_at = chrono::Utc::now().timestamp();
        
        let updated_task = task.clone();
        self.save_tasks()?;
        
        Ok(updated_task)
    }
    
    /// Add notes to a task
    pub fn add_notes(&mut self, id: &str, notes: &str) -> Result<Task, Box<dyn Error>> {
        let task = self.tasks.get_mut(id)
            .ok_or(format!("Task with ID '{}' not found", id))?;
        
        task.notes = notes.to_string();
        task.updated_at = chrono::Utc::now().timestamp();
        
        let updated_task = task.clone();
        self.save_tasks()?;
        
        Ok(updated_task)
    }
    
    /// Get statistics about tasks
    pub fn get_stats(&self) -> TaskStats {
        let total = self.tasks.len();
        let mut todo = 0;
        let mut in_progress = 0;
        let mut done = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;
        
        for task in self.tasks.values() {
            match task.status {
                TaskStatus::Todo => todo += 1,
                TaskStatus::InProgress => in_progress += 1,
                TaskStatus::Done => done += 1,
            }
            
            match task.priority {
                TaskPriority::High => high += 1,
                TaskPriority::Medium => medium += 1,
                TaskPriority::Low => low += 1,
            }
        }
        
        TaskStats {
            total,
            todo,
            in_progress,
            done,
            high,
            medium,
            low,
        }
    }
}

/// Task statistics structure
#[derive(Debug, Clone, Copy)]
pub struct TaskStats {
    pub total: usize,
    pub todo: usize,
    pub in_progress: usize,
    pub done: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

/// Display task in a formatted way
pub fn format_task(task: &Task) -> String {
    let status_str: &str = task.status.into();
    let priority_str: &str = task.priority.into();
    
    let mut formatted = format!(
        "ID: {}\n", task.id
    );
    formatted.push_str(&format!("Description: {}\n", task.description));
    formatted.push_str(&format!("Status: {}\n", status_str));
    formatted.push_str(&format!("Priority: {}\n", priority_str));
    formatted.push_str(&format!("Created: {}\n", 
        chrono::Utc.timestamp(task.created_at, 0)
            .format("%Y-%m-%d %H:%M:%S")
    ));
    formatted.push_str(&format!("Updated: {}\n", 
        chrono::Utc.timestamp(task.updated_at, 0)
            .format("%Y-%m-%d %H:%M:%S")
    ));
    
    if let Some(completed_at) = task.completed_at {
        formatted.push_str(&format!("Completed: {}\n", 
            chrono::Utc.timestamp(completed_at, 0)
                .format("%Y-%m-%d %H:%M:%S")
        ));
    }
    
    if !task.tags.is_empty() {
        formatted.push_str(&format!("Tags: {}\n", task.tags.join(", ")));
    }
    
    if !task.notes.is_empty() {
        formatted.push_str(&format!("Notes: {}\n", task.notes));
    }
    
    formatted
}
