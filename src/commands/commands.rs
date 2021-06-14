use teloxide::utils::command::BotCommand;
#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "Ban a user")]
    Ban,
    #[command(description = "Unbans a user")]
    Unban,
    #[command(description = "Mute a user")]
    Mute,
    #[command(description = "Unmute a user")]
    Unmute,
    #[command(description = "Greeting a user who sends /start")]
    Start,
    #[command(description = "Helps with available commands")]
    Help,
    #[command(description = "Kick a user from the group")]
    Kick,
    #[command(description = "Sends info about a user")]
    Info,
    #[command(description = "Kick yourself from a group")]
    Kickme,
    #[command(description = "Pins a message")]
    Pin,
    #[command(description = "Unpins a mentioned message")]
    Unpin,
}
