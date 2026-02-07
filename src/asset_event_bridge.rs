use bevy::prelude::*;

#[derive(Event)]
pub struct AssetLoadedEvent<A>
where
    A: Asset,
{
    pub asset_id: AssetId<A>,
}

// bridge method because we can't observe asset events yet
// https://github.com/bevyengine/bevy/issues/16041
pub fn bridge_asset_events<A>(mut events: MessageReader<AssetEvent<A>>, mut commands: Commands)
where
    A: Asset,
{
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            debug!("bridging asset load for {}", id);
            commands.trigger(AssetLoadedEvent { asset_id: *id });
        }
    }
}
