use pyo3::prelude::*;

mod agent;
mod config;
mod memory;
mod pool;
mod util;

pub use agent::{Agent, LoopAgent};
pub use config::AgentConfig;
pub use memory::MemoryAccess;
pub use pool::AgentPool;

#[pymodule]
fn argo_agents(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Agent>()?;
    m.add_class::<LoopAgent>()?;
    m.add_class::<AgentPool>()?;
    m.add_class::<AgentConfig>()?;
    m.add_class::<MemoryAccess>()?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}

/// Return the Argo SDK version.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
