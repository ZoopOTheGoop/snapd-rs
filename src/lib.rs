use connection::SnapdConnectionManager;
use deadpool::managed::Pool;

pub mod api;
mod connection;

pub struct SnapdClient {
    pool: Pool<SnapdConnectionManager>,
}
