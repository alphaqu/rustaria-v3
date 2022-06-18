use eyre::{Context, Result};
use rustaria::api::luna::lib::reload::Reload;
use rustaria::api::luna::lib::stargate::Stargate;
use rustaria::api::registry::{IdTable, Registry};
use rustaria::api::Api;
use rustaria::world::chunk::layer::BlockLayerPrototype;
use rustaria::world::entity::prototype::EntityPrototype;
use std::collections::HashSet;
use std::mem::replace;
use std::ops::Deref;
use std::path::PathBuf;

use crate::render::atlas::Atlas;
use crate::render::world::chunk::layer::LuaBlockLayerRendererPrototype;
use crate::render::world::entity::{EntityRendererPrototype, LuaEntityRendererPrototype};
use crate::{BlockLayerRendererPrototype, Frontend};

pub struct ClientApi {
    pub c_carrier: ClientCarrier,
    pub api: Api,
    pub atlas: Option<Atlas>,
}

impl ClientApi {
    pub fn new(run_dir: PathBuf, extra: Vec<PathBuf>) -> Result<ClientApi> {
        Ok(ClientApi {
            api: Api::new(run_dir, extra).wrap_err("Failed to reload common API")?,
            c_carrier: ClientCarrier {
                block_layer_renderer: Default::default(),
                entity_renderer: Default::default(),
            },
            atlas: None,
        })
    }

    pub fn reload(&mut self, frontend: &Frontend) -> Result<()> {
        let mut reload = Reload {
            stargate: Stargate::new(),
            client: true,
        };
        // Register client only prototypes
        reload
            .stargate
            .register_builder::<LuaBlockLayerRendererPrototype>();
        reload
            .stargate
            .register_builder::<LuaEntityRendererPrototype>();

        // reload server stuff
        let mut hasher = self.api.reload(&mut reload).wrap_err("Failed to reload")?;

        let mut sprites = HashSet::new();
        let block_layers = reload
            .stargate
            .build_registry::<LuaBlockLayerRendererPrototype>(&self.api.luna.lua, &mut hasher)?;

        let entities = reload
            .stargate
            .build_registry::<LuaEntityRendererPrototype>(&self.api.luna.lua, &mut hasher)?;

        for (_, prototype) in block_layers.table.iter() {
            prototype.get_sprites(&mut sprites);
        }

        for (_, prototype) in entities.table.iter() {
            prototype.get_sprites(&mut sprites);
        }

        let atlas = Atlas::new(frontend, self, sprites)?;

        let mut block_layer_renderer = Vec::new();
        for (id, _, _) in self.api.carrier.block_layer.iter() {
            block_layer_renderer.push((id, None));
        }
        for (_, identifier, prototype) in block_layers.into_iter() {
            if let Some(id) = self.api.carrier.block_layer.get_id(&identifier) {
                let prototype = prototype
                    .bake(
                        &self.api.luna.lua,
                        &atlas,
                        self.api.carrier.block_layer.get(id),
                    )
                    .wrap_err_with(|| format!("Failed to bake {}", identifier))?;
                let _ = replace(&mut block_layer_renderer[id.index()], (id, Some(prototype)));
            }
        }

        let mut entity_renderer = Vec::new();
        for (id, _, _) in self.api.carrier.entity.iter() {
            entity_renderer.push((id, None));
        }
        for (_, identifier, prototype) in entities.into_iter() {
            if let Some(id) = self.api.carrier.entity.get_id(&identifier) {
                let _ = replace(&mut entity_renderer[id.index()], (id, Some(prototype.bake(&atlas))));
            }
        }

        self.atlas = Some(atlas);
        self.c_carrier = ClientCarrier {
            block_layer_renderer: block_layer_renderer.into_iter().collect(),
            entity_renderer: entity_renderer.into_iter().collect(),
        };

        self.api.finalize_reload(hasher);

        Ok(())
    }
}

impl Deref for ClientApi {
    type Target = Api;

    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

pub struct ClientCarrier {
    pub block_layer_renderer: IdTable<BlockLayerPrototype, Option<BlockLayerRendererPrototype>>,
    pub entity_renderer: IdTable<EntityPrototype, Option<EntityRendererPrototype>>,
}
