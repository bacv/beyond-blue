mod hero;
mod npc;

pub use hero::*;
pub use npc::*;

pub const PIXELS_PER_METER: f32 = 492.3;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
