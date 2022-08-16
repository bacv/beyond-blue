mod behaviour;
mod peerid;
mod swarm;

pub use behaviour::*;
pub use peerid::*;
pub use swarm::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
