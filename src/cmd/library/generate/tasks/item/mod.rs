use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::Task;
use crate::cmd::library::generate::tasks::item::element_snippet::{
    ElementSnippetTask, SnippetMode,
};
use crate::cmd::library::generate::tasks::item::item_documentation::ItemDocumentationTask;
use crate::cmd::library::generate::tasks::item::item_icon::ItemIconTask;
use crate::cmd::library::generate::tasks::item::item_source::ItemSourceTask;
use crate::cmd::library::generate::tasks::item::sprite_icon::SpriteIconTask;
use crate::cmd::library::generate::tasks::item::sprite_value::SpriteValueTask;
use crate::cmd::library::manifest::icon::Icon;
use crate::cmd::library::manifest::item::Item;
use crate::cmd::library::manifest::library::Library;
use crate::cmd::library::manifest::module::Module;
use crate::cmd::library::manifest::package::Package;

mod element_snippet;
mod item_documentation;
mod item_icon;
mod item_source;
mod sprite_icon;
mod sprite_value;

pub fn parse_item(
    _config: &Config,
    _library: &Library,
    _package: &Package,
    _module: &Module,
    _item: &Item,
) -> anyhow::Result<Vec<Box<dyn Task>>> {
    log::debug!("parse item {}", &_item.urn);

    let mut tasks: Vec<Box<dyn Task>> = vec![];

    if let Some(icon) = &_item.icon {
        match icon {
            Icon::Source { source } => {
                // create the task to generate the icon
                let item_icon_task = ItemIconTask::create(_config, _library, _item, icon, source)?;
                let sprite_icon_source = item_icon_task.full_destination_image.clone();
                tasks.push(Box::from(item_icon_task));
                // create the tasks to generate the sprite values
                for (sprite_size_name, sprite_size_value) in
                    _library.customization.list_sprite_sizes()
                {
                    // create the task to generate the icon used as input of the sprite value
                    let sprite_icon_task = SpriteIconTask::create(
                        _config,
                        _item,
                        icon,
                        &sprite_icon_source,
                        (sprite_size_name, sprite_size_value),
                    )?;
                    // create the task to generate ans cache the sprite value
                    let sprite_value_task = SpriteValueTask::create(
                        _config,
                        _item,
                        icon,
                        &sprite_icon_task.full_destination_icon.clone(),
                        sprite_size_name,
                    )?;
                    tasks.push(Box::from(sprite_icon_task));
                    tasks.push(Box::from(sprite_value_task));
                }
            }
            Icon::Reference { .. } => {}
        }
    };

    // create the snippet for each element
    for element in _item.elements.iter() {
        // create the local snippet
        tasks.push(Box::from(ElementSnippetTask::create(
            _config,
            _library,
            _package,
            _item,
            element,
            SnippetMode::Local,
        )?));
        // create the remote snippet
        tasks.push(Box::from(ElementSnippetTask::create(
            _config,
            _library,
            _package,
            _item,
            element,
            SnippetMode::Remote,
        )?));
    }

    // create the task to generate the documentation
    tasks.push(Box::from(ItemDocumentationTask::create(
        _config, _library, _item,
    )?));

    // create the task to generate the puml file of the item
    tasks.push(Box::from(ItemSourceTask::create(_config, _item)?));

    Ok(tasks)
}
