use crate::serenity_prelude as serenity;

/// Message builder that abstracts over prefix and application command responses
#[derive(Default)]
pub struct CreateReply<'a> {
    /// Message content
    pub content: Option<String>,
    /// Embed, if present
    pub embed: Option<serenity::CreateEmbed>,
    /// Message attachments
    pub attachments: Vec<serenity::AttachmentType<'a>>,
    /// Whether the message is ephemeral (only has an effect in application commands)
    pub ephemeral: bool,
    /// Message components, that is, buttons
    pub components: Option<serenity::CreateComponents>,
}

impl<'a> CreateReply<'a> {
    /// Set the content of the message.
    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
        self.content = Some(content.into());
        self
    }

    /// Set an embed for the message.
    ///
    /// Any previously set embed will be overwritten.
    pub fn embed(
        &mut self,
        f: impl FnOnce(&mut serenity::CreateEmbed) -> &mut serenity::CreateEmbed,
    ) -> &mut Self {
        let mut embed = serenity::CreateEmbed::default();
        f(&mut embed);
        self.embed = Some(embed);
        self
    }

    /// Set an embed for the message.
    ///
    /// Any previously set embed will be overwritten.
    pub fn components(
        &mut self,
        f: impl FnOnce(&mut serenity::CreateComponents) -> &mut serenity::CreateComponents,
    ) -> &mut Self {
        let mut components = serenity::CreateComponents::default();
        f(&mut components);
        self.components = Some(components);
        self
    }

    /// Add an attachment.
    ///
    /// This will not have an effect in a slash command's initial response!
    pub fn attachment(&mut self, attachment: serenity::AttachmentType<'a>) -> &mut Self {
        self.attachments.push(attachment);
        self
    }

    /// Toggles whether the message is an ephemeral response (only invoking user can see it).
    ///
    /// This only has an effect in slash commands!
    pub fn ephemeral(&mut self, ephemeral: bool) -> &mut Self {
        self.ephemeral = ephemeral;
        self
    }
}

/// Returned from [`send_reply`] to retrieve the sent message object.
///
/// For prefix commands, you can retrieve the sent message directly. For slash commands, Discord
/// requires a network request.
pub enum ReplyHandle<'a> {
    /// When sending a normal message, Discord returns the message object directly
    Prefix(serenity::Message),
    /// When sending an application command response, you need to request the message object
    /// seperately
    Application {
        /// Serenity HTTP instance that can be used to request the interaction response message
        /// object
        http: &'a serenity::Http,
        /// Interaction which contains the necessary data to request the interaction response
        /// message object
        interaction: &'a serenity::ApplicationCommandInteraction,
    },
}

impl ReplyHandle<'_> {
    /// Retrieve the message object of the sent reply.
    ///
    /// Only needs to do an HTTP request in the application command response case
    pub async fn message(self) -> Result<serenity::Message, serenity::Error> {
        match self {
            Self::Prefix(msg) => Ok(msg),
            Self::Application { http, interaction } => {
                interaction.get_interaction_response(http).await
            }
        }
    }
}

/// Send a message in the given context: normal message if prefix command, interaction response
/// if application command.
///
/// If you just want to send a string, use [`say_reply`].
///
/// ```rust,no_run
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let ctx: poise::Context<'_, (), ()> = todo!();
/// poise::send_reply(ctx, |f| f
///     .content("Works for slash and prefix commands")
///     .embed(|f| f
///         .title("Much versatile, very wow")
///         .description("I need more documentation ok?")
///     )
///     .ephemeral(true) // this one only applies in application commands though
/// ).await?;
/// # Ok(()) }
/// ```
pub async fn send_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    builder: impl for<'a, 'b> FnOnce(&'a mut CreateReply<'b>) -> &'a mut CreateReply<'b>,
) -> Result<ReplyHandle<'_>, serenity::Error> {
    Ok(match ctx {
        crate::Context::Prefix(ctx) => {
            ReplyHandle::Prefix(crate::send_prefix_reply(ctx, builder).await?)
        }
        crate::Context::Application(ctx) => {
            crate::send_application_reply(ctx, builder).await?;
            ReplyHandle::Application {
                interaction: ctx.interaction,
                http: &ctx.discord.http,
            }
        }
    })
}

/// Shorthand of [`send_reply`] for text-only messages
pub async fn say_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    text: impl Into<String>,
) -> Result<ReplyHandle<'_>, serenity::Error> {
    send_reply(ctx, |m| m.content(text.into())).await
}
