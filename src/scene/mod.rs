use crate::game::resources::Resources;

pub trait Scene {
    fn process_events(&mut self, res: &mut Resources);
}
