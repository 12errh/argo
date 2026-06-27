# A-12: Security Model

**Status:** Proposed
**Author:** Argo Core Team
**Created:** 2026-06-27

---

## Summary

Define tool sandboxing implementation, secret handling, MCP authentication, agent isolation rules, and runtime permission enforcement.

## Motivation

Agents execute arbitrary code and access external services. The security model must prevent unauthorized access, protect secrets, and isolate agents from each other.

## Detailed Design

### Tool Sandboxing

```rust
pub struct SandboxConfig {
    pub working_directory: PathBuf,
    pub allowed_paths: Vec<PathBuf>,
    pub allowed_domains: Vec<String>,
    pub allow_network: bool,
    pub allow_subprocess: bool,
    pub max_execution_time: Duration,
    pub max_memory: Option<usize>,
}

pub struct Sandbox {
    config: SandboxConfig,
}

impl Sandbox {
    pub fn check_file_access(&self, path: &Path, operation: FileOperation) -> Result<(), SecurityError> {
        let canonical = path.canonicalize()
            .map_err(|_| SecurityError::PathResolutionFailed { path: path.to_path_buf() })?;

        if !self.config.allowed_paths.iter().any(|allowed| canonical.starts_with(allowed)) {
            return Err(SecurityError::PathDenied { path: path.to_path_buf() });
        }

        match operation {
            FileOperation::Read => Ok(()),
            FileOperation::Write => {
                if self.is_read_only(&canonical) {
                    Err(SecurityError::WriteDenied { path: path.to_path_buf() })
                } else {
                    Ok(())
                }
            }
            FileOperation::Delete => {
                if self.is_protected(&canonical) {
                    Err(SecurityError::DeleteDenied { path: path.to_path_buf() })
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn check_network_access(&self, url: &Url) -> Result<(), SecurityError> {
        if !self.config.allow_network {
            return Err(SecurityError::NetworkDenied { url: url.to_string() });
        }

        let host = url.host_str().ok_or_else(|| SecurityError::InvalidUrl { url: url.to_string() })?;

        if !self.config.allowed_domains.iter().any(|domain| host.ends_with(domain)) {
            return Err(SecurityError::DomainDenied { domain: host.to_string() });
        }

        Ok(())
    }

    pub fn check_execution_time(&self, elapsed: Duration) -> Result<(), SecurityError> {
        if elapsed > self.config.max_execution_time {
            Err(SecurityError::ExecutionTimeout { elapsed })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum SecurityError {
    #[error("Path access denied: {path}")]
    PathDenied { path: PathBuf },

    #[error("Write access denied: {path}")]
    WriteDenied { path: PathBuf },

    #[error("Delete access denied: {path}")]
    DeleteDenied { path: PathBuf },

    #[error("Network access denied: {url}")]
    NetworkDenied { url: String },

    #[error("Domain access denied: {domain}")]
    DomainDenied { domain: String },

    #[error("Execution timeout: {elapsed:?}")]
    ExecutionTimeout { elapsed: Duration },

    #[error("Path resolution failed: {path}")]
    PathResolutionFailed { path: PathBuf },

    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },
}
```

### Secret Management

```rust
pub struct SecretManager {
    env_vars: HashMap<String, String>,
}

impl SecretManager {
    pub fn new() -> Self {
        Self {
            env_vars: std::env::vars().collect(),
        }
    }

    pub fn resolve(&self, input: &str) -> Result<String, SecretError> {
        let re = regex::Regex::new(r"\$\{(\w+)\}").unwrap();

        let resolved = re.replace_all(input, |caps: &regex::Captures| {
            let var_name = &caps[1];
            self.env_vars.get(var_name)
                .map(|v| v.as_str())
                .unwrap_or("")
        }).to_string();

        Ok(resolved)
    }

    pub fn validate(&self, required: &[&str]) -> Result<(), SecretError> {
        for var_name in required {
            if !self.env_vars.contains_key(*var_name) {
                return Err(SecretError::MissingSecret { name: var_name.to_string() });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Error)]
pub enum SecretError {
    #[error("Missing required secret: {name}")]
    MissingSecret { name: String },

    #[error("Secret resolution failed: {reason}")]
    ResolutionFailed { reason: String },
}
```

### MCP Authentication

```rust
pub struct McpAuth {
    config: AuthConfig,
}

impl McpAuth {
    pub fn apply_headers(&self, request: &mut reqwest::RequestBuilder) -> Result<(), SecurityError> {
        match &self.config.auth_type {
            AuthType::Bearer { token } => {
                let resolved = SecretManager::new().resolve(token)?;
                request.header("Authorization", format!("Bearer {}", resolved));
            }
            AuthType::OAuth2 { client_id, client_secret } => {
                let resolved_id = SecretManager::new().resolve(client_id)?;
                let resolved_secret = SecretManager::new().resolve(client_secret)?;
                // OAuth2 token exchange
            }
        }
        Ok(())
    }
}
```

### Agent Isolation

```rust
pub struct AgentIsolation {
    agents: HashMap<String, AgentNamespace>,
}

pub struct AgentNamespace {
    pub short_term_prefix: String,
    pub long_term_namespace: String,
    pub semantic_namespace: String,
}

impl AgentIsolation {
    pub fn create_namespace(&mut self, agent_id: &str, mode: &MemoryMode) -> AgentNamespace {
        match mode {
            MemoryMode::Isolated => AgentNamespace {
                short_term_prefix: format!("argo:agent:{}:", agent_id),
                long_term_namespace: format!("agent_{}", agent_id),
                semantic_namespace: format!("argo_{}_", agent_id),
            },
            MemoryMode::Shared => AgentNamespace {
                short_term_prefix: format!("argo:agent:{}:", agent_id),
                long_term_namespace: "shared".to_string(),
                semantic_namespace: "argo_shared_".to_string(),
            },
            MemoryMode::Persistent => AgentNamespace {
                short_term_prefix: format!("argo:agent:{}:", agent_id),
                long_term_namespace: format!("agent_{}", agent_id),
                semantic_namespace: format!("argo_{}_", agent_id),
            },
        }
    }

    pub fn can_access(&self, agent_id: &str, target: &str, mode: &MemoryMode) -> bool {
        match mode {
            MemoryMode::Isolated | MemoryMode::Persistent => agent_id == target,
            MemoryMode::Shared => true,
        }
    }
}
```

### Permission Enforcement

```rust
pub fn enforce_permissions(tool: &dyn Tool, ctx: &ToolContext, sandbox: &Sandbox) -> Result<(), SecurityError> {
    let perms = tool.permissions();

    if perms.allow_filesystem {
        if let Some(ref working_dir) = perms.working_directory {
            sandbox.check_file_access(working_dir, FileOperation::Read)?;
        }
    }

    if perms.allow_network {
        for domain in &perms.allowed_domains {
            // Validate domain format
        }
    }

    if perms.allow_subprocess {
        // Verify subprocess is allowed in sandbox
    }

    Ok(())
}
```

## Alternatives Considered

1. **OS-level sandboxing (containers, seccomp)**: Strongest isolation, but adds deployment complexity.
2. **WASM sandboxing**: Portable, but limited system access.
3. **No sandboxing**: Simplest, but unacceptable for production use.

## Drawbacks

- Path resolution adds overhead to file operations
- Secret management relies on environment variables (not vault integration)
- Agent isolation depends on correct namespace configuration

## Unresolved Questions

- Should we support vault integration (HashiCorp Vault, AWS Secrets Manager)?
- How to handle secrets in multi-tenant deployments?
- Should we support MAC-based mandatory access control?
