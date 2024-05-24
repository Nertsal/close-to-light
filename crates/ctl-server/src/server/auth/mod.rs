mod discord;
mod native;

use super::*;

pub fn router() -> Router {
    native::router().merge(discord::router())
}
