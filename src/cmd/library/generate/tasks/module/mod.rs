use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::Task;
use crate::cmd::library::generate::tasks::module::module_documentation::ModuleDocumentationTask;
use crate::manifest::library::Library;
use crate::manifest::module::Module;
use crate::manifest::package::Package;
use crate::result::Result;

mod module_documentation;

pub fn parse_module(
    _config: &Config,
    _library: &Library,
    _package: &Package,
    _module: &Module,
) -> Result<Vec<Box<dyn Task>>> {
    log::debug!("parse module {}", &_module.urn);
    Ok(vec![Box::from(ModuleDocumentationTask::create(
        _config, _library, _module,
    )?)])
}
