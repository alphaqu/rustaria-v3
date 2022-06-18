use std::collections::HashMap;
use std::io;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;

use eyre::{Context, Result};
use rayon::{ThreadPool, ThreadPoolBuilder};
use tracing::debug;

use crate::api::luna::glue::Glue;
use crate::api::luna::lib::reload::Reload;
use crate::api::luna::Luna;
use crate::api::plugin::Plugin;
use crate::api::registry::Registry;
use crate::multi_deref_fields;
use crate::ty::identifier::Identifier;
use crate::ty::MultiDeref;
use crate::util::blake3::{Blake3Hash, Hasher};
use crate::world::chunk::block::LuaBlockPrototype;
use crate::world::chunk::layer::{BlockLayerPrototype, LuaBlockLayerPrototype};
use crate::world::entity::prototype::EntityPrototype;

pub mod luna;
pub mod plugin;
pub mod prototype;
pub mod registry;
pub mod util;

pub struct Api {
    pub carrier: Carrier,
    pub resources: Plugins,
    pub thread_pool: Arc<ThreadPool>,
    pub luna: Luna,
    pub hash: Option<Blake3Hash>,
}

impl Api {
    pub fn new(run_dir: PathBuf, extra: Vec<PathBuf>) -> Result<Api> {
        let plugins_path = run_dir.join("./plugins");
        if !plugins_path.exists() {
            std::fs::create_dir_all(&plugins_path).wrap_err("Could not create dirs.")?;
        }

        let mut paths: Vec<PathBuf> = std::fs::read_dir(plugins_path)?
            .flatten()
            .map(|entry| entry.path())
            .collect();
        paths.extend(extra);

        let mut plugins = HashMap::new();
        for path in paths {
            if path.is_dir()
                || (path.is_file()
                    && path
                        .extension()
                        .map(|extention| extention.to_str().unwrap() == "zip")
                        .unwrap_or(false))
            {
                let plugin = Plugin::new(&path)?;
                plugins.insert(plugin.id.clone(), plugin);
            }
        }

        let resources = Plugins {
            plugins: Arc::new(plugins),
        };
        Ok(Api {
            luna: Luna::new(&resources)?,
            carrier: Carrier {
                block_layer: Registry::default(),
                entity: Registry::default(),
            },
            resources,
            thread_pool: Arc::new(ThreadPoolBuilder::new().build()?),
            hash: None,
        })
    }

    pub fn reload(&mut self, reload: &mut Reload) -> Result<Hasher> {
        self.hash = None;

        // Prepare for reload
        reload.stargate.register_builder::<LuaBlockLayerPrototype>();
        reload.stargate.register_builder::<EntityPrototype>();

        {
            let glue = Glue::new(reload);
            self.luna.lua.globals().set("reload", glue.clone())?;

            for plugin in self.resources.plugins.values() {
                debug!("Reloading {}", &plugin.id);
                let identifier = Identifier::new("main.lua");
                let data = self
                    .resources
                    .get_resource(ResourceKind::Source, &identifier)
                    .wrap_err(format!(
                        "Could not find entrypoint \"main.lua\" for plugin {}",
                        plugin.id
                    ))?;

                self.luna.load(&identifier, &data)?.exec()?;
            }
        }

        let mut hasher = Hasher::new();

        let registry = reload
            .stargate
            .build_registry::<LuaBlockLayerPrototype>(&self.luna.lua, &mut hasher)?;

        let block_layer = registry
            .table
            .into_iter()
            .zip(registry.id_to_ident.into_iter());

        let mut out = Vec::new();
        for ((id, prototype), (_, identifier)) in block_layer {
            out.push((id.build(), identifier, prototype.bake()?));
        }
        let block_layer = out.into_iter().collect();

        self.carrier = Carrier {
            block_layer,
            entity: reload
                .stargate
                .build_registry::<EntityPrototype>(&self.luna.lua, &mut hasher)?,
        };

        Ok(hasher)
    }

    pub fn finalize_reload(&mut self, hasher: Hasher) {
        self.hash = Some(hasher.finalize());
    }
}
pub enum ResourceKind {
    Assets,
    Source,
}

#[derive(Clone)]
pub struct Plugins {
    pub plugins: Arc<HashMap<String, Plugin>>,
}

impl Plugins {
    pub fn get_resource(&self, kind: ResourceKind, location: &Identifier) -> io::Result<Vec<u8>> {
        let plugin = self.plugins.get(&location.namespace).ok_or_else(|| {
            io::Error::new(
                ErrorKind::NotFound,
                format!("Plugin {} does not exist", location.namespace),
            )
        })?;

        let prefix = match kind {
            ResourceKind::Assets => "assets",
            ResourceKind::Source => "src",
        };
        plugin.archive.get(&format!("{}/{}", prefix, location.path))
    }
}

pub struct Carrier {
    pub block_layer: Registry<BlockLayerPrototype>,
    pub entity: Registry<EntityPrototype>,
}

multi_deref_fields!(Carrier {
    block_layer: Registry<BlockLayerPrototype>,
    entity: Registry<EntityPrototype>
});
