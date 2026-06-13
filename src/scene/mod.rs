use crate::game::resources::GameResources;

pub trait Scene {
    fn process_events(&mut self, res: &mut GameResources);
}
