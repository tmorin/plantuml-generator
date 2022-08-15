use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::Task;
use crate::cmd::library::generate::tasks::library::library_bootstrap::LibraryBootstrapTask;
use crate::cmd::library::generate::tasks::library::library_documentation::LibraryDocumentationTask;
use crate::cmd::library::generate::tasks::library::library_summary::LibrarySummaryTask;
use crate::manifest::library::Library;
use crate::result::Result;

mod library_bootstrap;
mod library_documentation;
mod library_summary;

pub fn parse_library(config: &Config, library: &Library) -> Result<Vec<Box<dyn Task>>> {
    log::debug!("parse library {}", &library.name);
    Ok(vec![
        Box::from(LibraryBootstrapTask::create(config, library)?),
        Box::from(LibraryDocumentationTask::create(config, library)?),
        Box::from(LibrarySummaryTask::create(config, library)?),
    ])
}
