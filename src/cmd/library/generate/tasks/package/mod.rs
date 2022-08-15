use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::Task;
use crate::cmd::library::generate::tasks::package::package_bootstrap::PackageBootstrapTask;
use crate::cmd::library::generate::tasks::package::package_documentation::PackageDocumentationTask;
use crate::cmd::library::generate::tasks::package::package_embedded::{
    EmbeddedMode, PackageEmbeddedTask,
};
use crate::cmd::library::generate::tasks::package::package_example::PackageExampleTask;
use crate::manifest::library::Library;
use crate::manifest::package::Package;
use crate::result::Result;

mod package_bootstrap;
mod package_documentation;
mod package_embedded;
mod package_example;

pub fn parse_package(
    _config: &Config,
    _library: &Library,
    _package: &Package,
) -> Result<Vec<Box<dyn Task>>> {
    log::debug!("parse package {}", &_package.urn);
    let mut tasks: Vec<Box<dyn Task>> = vec![];

    for example in _package.examples.iter() {
        tasks.push(Box::from(PackageExampleTask::create(
            _config, _library, _package, example,
        )?));
    }

    tasks.push(Box::from(PackageBootstrapTask::create(_config, _package)?));
    if !_package.rendering.skip_embedded {
        tasks.push(Box::from(PackageEmbeddedTask::create(
            _config,
            _package,
            EmbeddedMode::Single,
        )?));
        tasks.push(Box::from(PackageEmbeddedTask::create(
            _config,
            _package,
            EmbeddedMode::Full,
        )?));
    }
    tasks.push(Box::from(PackageDocumentationTask::create(
        _config, _library, _package,
    )?));

    Ok(tasks)
}
