use super::room_object_def::RoomObject;

pub struct Dummy {

}

// Dummy should only be constructed and held temporarily. It should never actually be used in the main room object
// logic. Therefore, all these methods are unimplemented.
impl RoomObject for Dummy {
    fn get_metadata(&self) -> &super::room_object_def::RoomObjectMetadata {
        unimplemented!()
    }

    fn act1(&mut self, _ctx: &mut super::room_object_def::Act1Context) -> super::room_object_def::Act1Response {
        unimplemented!()
    }

    fn draw(&mut self, _ctx: &mut crate::rolag3::floor::draw::DrawContext) {
        unimplemented!()
    }

    fn handle_collision(&mut self, _ctx: &mut super::room_object_def::HandleCollisionContext) -> super::room_object_def::HandleCollisionResponse {
        unimplemented!()
    }
}