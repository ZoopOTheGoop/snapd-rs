use connection::SnapdConnectionManager;
use deadpool::managed::Pool;

mod connection;

pub struct SnapdClient {
    pool: Pool<SnapdConnectionManager>,
}
