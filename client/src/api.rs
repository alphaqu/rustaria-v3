use std::ops::Deref;
use std::path::PathBuf;
use rustaria::api::Api;
use rustaria::api::registry::Registry;
use eyre::{Context, Result};
use rustaria::api::luna::lib::reload::Reload;
use rustaria::api::luna::lib::stargate::Stargate;

use crate::BlockLayerRendererPrototype;
use crate::render::world::entity::EntityRendererPrototype;

pub struct ClientApi {
    pub api: Api,
    pub c_carrier: ClientCarrier,
}

impl ClientApi {
    pub fn new(run_dir: PathBuf, extra: Vec<PathBuf>) -> Result<ClientApi> {
        Ok(ClientApi {
            api: Api::new(run_dir, extra).wrap_err("Failed to reload common API")?,
            c_carrier: ClientCarrier {
                block_layer_renderer: Registry::empty(),
                entity_renderer: Registry::empty()
            }
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        let mut reload = Reload {
            stargate: Stargate::new(),
            client: true
        };
        // Register client only prototypes
        reload.stargate.register_builder::<BlockLayerRendererPrototype>();
        reload.stargate.register_builder::<EntityRendererPrototype>();

        self.api.reload(&mut reload).wrap_err("Failed to reload")?;

        // collect client only prototypes
        self.c_carrier = ClientCarrier {
            block_layer_renderer: reload.stargate.build_registry(),
            entity_renderer: reload.stargate.build_registry(),
        };
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
    pub block_layer_renderer: Registry<BlockLayerRendererPrototype>,
    pub entity_renderer: Registry<EntityRendererPrototype>,
}
