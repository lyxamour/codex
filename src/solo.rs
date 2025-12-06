use crate::ai::AIClient;
use crate::task::{TaskManager, TaskStatus};
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Solo mode step structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoloStep {
    /// Step ID
    id: String,
    /// Step description
    description: String,
    /// Step status
    status: StepStatus,
    /// Step result (if completed)
    result: Option<String>,
    /// Step error (if failed)
    error: Option<String>,
    /// Step execution time in seconds
    execution_time: Option<f64>,
}

/// Solo mode step status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum StepStatus {
    Todo,
    InProgress,
    Done,
    Failed,
}

/// Solo mode task structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoloTask {
    /// Task ID
    id: String,
    /// Original task description
    original_task: String,
    /// Task steps
    steps: Vec<SoloStep>,
    /// Task status
    status: TaskStatus,
    /// Task creation time
    created_at: i64,
    /// Task completion time
    completed_at: Option<i64>,
    /// Final result
    final_result: Option<String>,
}

/// Solo agent for autonomous task execution
pub struct SoloAgent {
    ai_client: AIClient,
    task_manager: TaskManager,
}

impl Default for SoloAgent {
    fn default() -> Self {
        Self::new().expect("Failed to create solo agent")
    }
}

impl SoloAgent {
    /// Create a new solo agent
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            ai_client: AIClient::new()?,
            task_manager: TaskManager::new()?,
        })
    }

    /// Execute a task in solo mode
    pub async fn execute(&mut self, task: &str, max_steps: u32) -> Result<String, Box<dyn Error>> {
        println!("Starting solo mode for task: {}", task);
        println!("Maximum steps: {}", max_steps);

        // Step 1: Decompose task into steps
        let steps = self.decompose_task(task, max_steps).await?;
        println!("Decomposed into {} steps:", steps.len());
        for (i, step) in steps.iter().enumerate() {
            println!("{}. {}", i + 1, step.description);
        }

        // Step 2: Execute steps sequentially
        let mut results = Vec::new();

        for (i, mut step) in steps.into_iter().enumerate() {
            println!(
                "\n=== Step {}: {} ({}/{}) ===",
                i + 1,
                step.description,
                i + 1,
                max_steps
            );

            // Execute step
            match self.execute_step(&step.description).await {
                Ok(result) => {
                    println!("✓ Step completed successfully");
                    println!("Result: {:.200}...", result);
                    step.status = StepStatus::Done;
                    step.result = Some(result.clone());
                    results.push(result.clone());
                }
                Err(e) => {
                    println!("✗ Step failed: {}", e);
                    step.status = StepStatus::Failed;
                    step.error = Some(e.to_string());
                    return Err(format!("Solo mode failed at step {}: {}", i + 1, e).into());
                }
            }
        }

        // Step 3: Synthesize final result
        let final_result = self.synthesize_result(task, &results).await?;
        println!("\n=== Final Result ===");
        println!("{}", final_result);

        Ok(final_result)
    }

    /// Decompose a task into steps
    async fn decompose_task(
        &self,
        task: &str,
        max_steps: u32,
    ) -> Result<Vec<SoloStep>, Box<dyn Error>> {
        let prompt = format!(
            "Decompose the following task into a sequence of {} or fewer concrete, actionable steps:\n\n{}\n\nEach step should be clear, specific, and focused on a single action. \
            The steps should be ordered logically to achieve the task goal. \
            Return ONLY the steps as a numbered list, one step per line, \
            starting with 1. and followed by the step description. \
            Do not include any additional explanation or introduction.",
            max_steps,
            task
        );

        let response = self.ai_client.generate_response(&prompt, None).await?;
        let content = response.content().to_string();

        // Parse steps from response
        let mut steps = Vec::new();
        let lines = content.lines();

        for (i, line) in lines.enumerate() {
            if i >= max_steps as usize {
                break;
            }

            // Skip empty lines
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse numbered steps
            if let Some(step_desc) = line.strip_prefix(|c: char| c.is_ascii_digit()) {
                let step_desc = step_desc.trim_start_matches(".").trim_start();
                if !step_desc.is_empty() {
                    steps.push(SoloStep {
                        id: format!("step_{}", i + 1),
                        description: step_desc.to_string(),
                        status: StepStatus::Todo,
                        result: None,
                        error: None,
                        execution_time: None,
                    });
                }
            }
        }

        if steps.is_empty() {
            return Err("Failed to decompose task into steps".into());
        }

        Ok(steps)
    }

    /// Execute a single step
    async fn execute_step(&self, step: &str) -> Result<String, Box<dyn Error>> {
        let prompt = format!(
            "Execute the following step and provide the result. \
            This is part of a larger task, so make sure the result is complete and self-contained.\n\nStep: {}\n\nReturn ONLY the result of executing this step, \
            no additional explanation or context.",
            step
        );

        let response = self.ai_client.generate_response(&prompt, None).await?;
        Ok(response.content().to_string())
    }

    /// Synthesize final result from step results
    async fn synthesize_result(
        &self,
        original_task: &str,
        results: &[String],
    ) -> Result<String, Box<dyn Error>> {
        let results_text = results.join("\n\n---\n\n");

        let prompt = format!(
            "Synthesize the following step results into a final, comprehensive result for the original task.\n\nOriginal Task: {}\n\nStep Results:\n{}\n\nProvide a clear, well-structured final result \
            that addresses the original task completely. \
            The result should be a cohesive summary of what was accomplished through the steps.",
            original_task,
            results_text
        );

        let response = self.ai_client.generate_response(&prompt, None).await?;
        Ok(response.content().to_string())
    }

    /// Execute a task with real-world actions (experimental)
    pub async fn execute_with_actions(
        &mut self,
        task: &str,
        max_steps: u32,
    ) -> Result<String, Box<dyn Error>> {
        println!(
            "Starting enhanced solo mode with real-world actions for task: {}",
            task
        );
        println!("Maximum steps: {}", max_steps);

        // Decompose task
        let steps = self.decompose_task(task, max_steps).await?;

        // Create a task in the task manager
        let task = self
            .task_manager
            .create(format!("Solo Mode: {}", task), "high".to_string())?;

        // Execute steps with real-world actions
        for (i, step) in steps.into_iter().enumerate() {
            println!(
                "\n=== Step {}: {} ({}/{}) ===",
                i + 1,
                step.description,
                i + 1,
                max_steps
            );

            // Check if step requires any real-world actions
            // This is a placeholder for future implementation
            // that could execute shell commands, edit files, etc.
            println!("Checking for real-world actions...");

            // For now, just use AI to execute the step
            let result = self.execute_step(&step.description).await?;
            println!("✓ Step completed successfully: {:.50}...", result);
        }

        // Update task status to done
        self.task_manager
            .update(&task.id, None, Some("done".to_string()), None)?;

        // Synthesize final result
        let final_result = format!(
            "Task completed successfully. Check the task manager for details: {}",
            task.id
        );

        Ok(final_result)
    }

    /// Execute a task with parallel step execution (experimental)
    pub async fn execute_parallel(
        &mut self,
        task: &str,
        max_steps: u32,
        max_parallel: usize,
    ) -> Result<String, Box<dyn Error>> {
        println!("Starting parallel solo mode for task: {}", task);
        println!(
            "Maximum steps: {}, Maximum parallel: {}",
            max_steps, max_parallel
        );

        // Decompose task
        let steps = self.decompose_task(task, max_steps).await?;

        // Create a shared AI client wrapped in Arc<Mutex> (暂时未使用)
        // 使用Arc::clone而不是AIClient::clone
        // let shared_ai = Arc::new(Mutex::new(self.ai_client.clone())); // TODO: 主人~ 未来实现并行执行时需要使用这个共享AI客户端哦
        let mut step_results = Vec::new();

        // Execute steps sequentially (parallel execution needs redesign)
        for step in steps {
            let result = self.execute_step(&step.description).await?;
            step_results.push(result);
        }

        // Synthesize final result
        let final_result = self.synthesize_result(task, &step_results).await?;

        Ok(final_result)
    }
}

/// Helper function to execute a command (experimental)
async fn execute_command(command: &str) -> Result<String, Box<dyn Error>> {
    use std::process::Command;

    println!("Executing command: {}", command);

    let output = Command::new("bash").arg("-c").arg(command).output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Command failed: {}", stderr).into())
    }
}

/// Helper function to edit a file (experimental)
async fn edit_file(path: &str, content: &str) -> Result<String, Box<dyn Error>> {
    use std::fs;

    println!("Editing file: {}", path);
    fs::write(path, content)?;
    Ok(format!("File {} updated successfully", path))
}

/// Helper function to read a file (experimental)
async fn read_file(path: &str) -> Result<String, Box<dyn Error>> {
    use std::fs;

    println!("Reading file: {}", path);
    let content = fs::read_to_string(path)?;
    Ok(content)
}
