use crate::server::ChannelsClient;
use crate::turtle_core::control::TurtControl;
use crate::turtle_core::navigation::TurtNavigation;
use crate::TurtleIdentifier;

pub fn turtle_registered(identifier: TurtleIdentifier, channels_client: ChannelsClient) {
    let turt = TurtControl::new(
        channels_client.0.clone(),
        &channels_client.1);
    let mut nav = TurtNavigation::new(
        identifier,
        &turt,
        true,
        channels_client.0.clone(),
        &channels_client.1);

    nav.gps_init();

    turt.disconnect();
}
