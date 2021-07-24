#![allow(unused_imports)]

use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Write,
    sync::Arc,
};

use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::standard::{
        buckets::{LimitedFor, RevertBucket},
        help_commands,
        macros::{check, command, group, help, hook},
        Args, CommandGroup, CommandOptions, CommandResult, DispatchError, HelpOptions, Reason,
        StandardFramework,
    },
    http::Http,
    model::{
        channel::{Channel, Message},
        gateway::Ready,
        id::UserId,
        permissions::Permissions,
    },
    utils::Color,
};

use rand::prelude::*;
use serenity::http::routing::Route::ChannelsId;
use serenity::model::id::ChannelId;
use serenity::utils::MessageBuilder;
use tokio::sync::Mutex;

const EMBED_SIDE_COLOR: Color = Color::from_rgb(255, 192, 203);

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[derive(Debug)]
struct BlitzQuote {
    quote: String,
    author: String,
}

impl BlitzQuote {
    fn new<T: Into<String>>(quote: T, author: T) -> BlitzQuote {
        BlitzQuote {
            quote: quote.into(),
            author: author.into(),
        }
    }
}

struct BlitzQuoteContainer;

impl TypeMapKey for BlitzQuoteContainer {
    type Value = Vec<BlitzQuote>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(blitz, whyrust, color)]
struct General;

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("CUTE_BOT_TOKEN").expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix(".")
                // In this case, if "," would be first, a message would never
                // be delimited at ", ", forcing you to trim your arguments if you
                // want to avoid whitespaces at the start of each.
                .delimiters(vec![", ", ","])
                // Sets the bot's owners. These will be used for commands that
                // are owners only.
                .owners(owners)
        })
        .unrecognised_command(unknown_command)
        .group(&GENERAL_GROUP);
    // Set a function that's called whenever a message is not a command.

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));

        let quotes = vec![
            BlitzQuote::new("Rapid and blitz chess is first of all for enjoyment.", "Magnus Carlsen"),
            BlitzQuote::new("Playing rapid chess, one can lose the habit of concentrating for several hours in serious chess. That is why, if a player has big aims, he should limit his rapid play in favour of serious chess.", "Vladimir Kramnik"),
            BlitzQuote::new("He who analyses blitz is stupid.", "Rashid Nezhmetdinov"),
            BlitzQuote::new("Blitz chess kills your ideas.", "Bobby Fischer"),
            BlitzQuote::new("To be honest, I consider [bullet chess] a bit moronic, and therefore I never play it.", "Vladimir Kramnik"),
            BlitzQuote::new("I play way too much blitz chess. It rots the brain just as surely as alcohol.", "Nigel Short"),
            BlitzQuote::new("Blitz is simply a waste of time.", "Vladimir Malakhov"),
            BlitzQuote::new("[Blitz] is just getting positions where you can move fast. I mean, it's not chess.", "Hikaru Nakamura"),
            BlitzQuote::new("Always sack the exchange!", "Ben F6gold"),
        ];

        data.insert::<BlitzQuoteContainer>(quotes);
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
#[aliases("colour")]
async fn color(ctx: &Context, msg: &Message) -> CommandResult {
    let bot_channel_id: i64 = 855703545398427668;
    let desc = format!("You can get cute :sparkles: by using the color commands at <#{}>\nUse `color list` to list all the available colors\nThen `color = [color name or number]` to set your role color!\nIf you'd like a color that is not on the list, let Lucy know!", bot_channel_id);

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Set your own role color!");
                e.color(EMBED_SIDE_COLOR);
                e.description(desc);
                e
            });
            m
        })
        .await
        .expect("error making message");

    Ok(())
}

#[command]
async fn blitz(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let quotes = data
        .get::<BlitzQuoteContainer>()
        .expect("Expected blitz quotes in typemap.");

    let index;
    {
        let mut rng = thread_rng();
        index = rng.gen_range(0..quotes.len());
    }

    let mut desc = String::default();
    write!(desc, "\"{}\"", &quotes[index].quote)?;

    let mut the_quote = String::default();
    write!(the_quote, "- {}", &quotes[index].author)?;

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(EMBED_SIDE_COLOR);
                e.description(desc);
                e.footer(|f| {
                    f.text(the_quote);
                    f
                });
                e
            });
            m
        })
        .await
        .expect("error making message");

    Ok(())
}

#[command]
async fn whyrust(ctx: &Context, msg: &Message) -> CommandResult {
    let title = "Why rust?!";
    let reasons = vec!["Why not?", "Sane defaults", "It is fun!", "cargo"];

    let random_index = thread_rng().gen_range(0..reasons.len());

    let mut choice = String::default();
    write!(choice, "{}", &reasons[random_index])?;

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(EMBED_SIDE_COLOR);
                e.title(title);
                e.description(choice);
                e
            });
            m
        })
        .await
        .expect("error making message");

    Ok(())
}
