use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// プロセス管理構造体
pub struct ProcessManager {
	processes: Arc<Mutex<HashMap<String, Child>>>,
}

impl ProcessManager {
	pub fn new() -> Self {
		Self {
			processes: Arc::new(Mutex::new(HashMap::new())),
		}
	}

	/// コマンドを実行し、プロセスを管理下に置く
	///
	/// # Arguments
	/// * `rule_id` - ルールの識別子（ログ用）
	/// * `command` - 実行するコマンド
	/// * `port` - PORT環境変数に設定する値（Optionの場合あり）
	///
	/// # Returns
	/// 成功時はOk(()), 失敗時はエラーメッセージ
	pub fn spawn_command(
		&self,
		rule_id: String,
		command: &str,
		port: Option<u16>,
	) -> Result<(), String> {
		// Format: rebab: PORT=3000 echo Frontend server started
		let log_message = if let Some(port_value) = port {
			format!("PORT={} {}", port_value, command)
		} else {
			command.to_string()
		};
		crate::log::log(&log_message);

		// Parse command into program and arguments
		let parts: Vec<&str> = command.split_whitespace().collect();
		if parts.is_empty() {
			return Err("Empty command".to_string());
		}

		let program = parts[0];
		let args = &parts[1..];

		// Build command
		let mut cmd = Command::new(program);
		cmd.args(args);
		cmd.stdout(Stdio::piped());
		cmd.stderr(Stdio::piped());
		cmd.stdin(Stdio::null());

		// Set PORT environment variable
		if let Some(port_value) = port {
			cmd.env("PORT", port_value.to_string());
		}

		// Spawn process
		match cmd.spawn() {
			Ok(mut child) => {
				// Take stdout and stderr for streaming
				let stdout = child.stdout.take();
				let stderr = child.stderr.take();

				// Spawn thread to stream stdout
				if let Some(stdout) = stdout {
					let rule_id_clone = rule_id.clone();
					thread::spawn(move || {
						stream_output(BufReader::new(stdout), rule_id_clone);
					});
				}

				// Spawn thread to stream stderr
				if let Some(stderr) = stderr {
					let rule_id_clone = rule_id.clone();
					thread::spawn(move || {
						stream_output(BufReader::new(stderr), rule_id_clone);
					});
				}

				let mut processes = self.processes.lock().unwrap();
				processes.insert(rule_id, child);
				Ok(())
			}
			Err(e) => {
				let error_msg = format!("Failed to execute command [{}]: {}", rule_id, e);
				crate::log::log(&error_msg);
				Err(error_msg)
			}
		}
	}

	/// Check all process states and fail if any has exited
	///
	/// # Returns
	/// Ok(()) if all processes are still running, Err if any has exited
	pub fn check_all(&self) -> Result<(), String> {
		let mut processes = self.processes.lock().unwrap();
		let mut exited_rules = Vec::new();

		for (rule_id, child) in processes.iter_mut() {
			match child.try_wait() {
				Ok(Some(status)) => {
					// Any process exit (success or failure) triggers shutdown
					let msg = if status.success() {
						format!(
							"Process [{}] exited successfully (exit code: {:?})",
							rule_id,
							status.code()
						)
					} else {
						format!(
							"Process [{}] exited with error (exit code: {:?})",
							rule_id,
							status.code()
						)
					};
					crate::log::log(&msg);
					exited_rules.push(rule_id.clone());
				}
				Ok(None) => {
					// Process still running
				}
				Err(e) => {
					let msg = format!("Failed to check process [{}]: {}", rule_id, e);
					crate::log::log(&msg);
					exited_rules.push(rule_id.clone());
				}
			}
		}

		if !exited_rules.is_empty() {
			Err(format!(
				"The following processes exited: {}",
				exited_rules.join(", ")
			))
		} else {
			Ok(())
		}
	}

	/// Terminate all processes
	pub fn terminate_all(&self) {
		crate::log::log("Terminating all processes...");
		let mut processes = self.processes.lock().unwrap();

		for (rule_id, child) in processes.iter_mut() {
			crate::log::log(&format!("Terminating process [{}]...", rule_id));
			let _ = child.kill();
			let _ = child.wait();
		}

		processes.clear();
		crate::log::log("All processes terminated");
	}
}

impl Drop for ProcessManager {
	fn drop(&mut self) {
		self.terminate_all();
	}
}

/// Stream output from a child process
fn stream_output<R: std::io::Read>(reader: BufReader<R>, rule_id: String) {
	for line in reader.lines() {
		match line {
			Ok(line) => {
				// Format: rebab: rule_id: line content
				crate::log::log(&format!("{}: {}", rule_id, line));
			}
			Err(_) => break,
		}
	}
}
