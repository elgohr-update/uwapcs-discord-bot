use serenity::{
    model::{channel, channel::Message},
    prelude::*,
    utils::MessageBuilder,
};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::config;

macro_rules! e {
    ($error: literal, $x:expr) => {
        match $x {
            Ok(_) => (),
            Err(why) => eprintln!($error, why),
        }
    };
}

pub struct Commands;
impl Commands {
    pub fn move_something(ctx: Context, msg: Message, content: &str) {
        let motion = content;
        if motion.len() > 0 {
            create_motion(&ctx, &msg, motion);
        } else {
            e!(
                "Error sending message: {:?}",
                msg.channel_id.say(
                    &ctx.http,
                    "If there's something you want to motion, put it after the !move keyword",
                )
            );
        }
    }
    pub fn motion(ctx: Context, msg: Message, _content: &str) {
        e!("Error sending message: {:?}",
                msg.channel_id.say(
                &ctx.http,
                "I hope you're not having a motion. You may have wanted to !move something instead."
            ));
    }
    pub fn poll(ctx: Context, msg: Message, content: &str) {
        let topic = content;
        if topic.len() > 0 {
            create_poll(&ctx, &msg, topic);
        } else {
            e!(
                "Error sending message: {:?}",
                msg.channel_id.say(
                    &ctx.http,
                    "If there's something you want to motion, put it after the !move keyword",
                )
            );
        }
    }
    pub fn cowsay(ctx: Context, msg: Message, content: &str) {
        let mut text = content.to_owned();
        text.escape_default();
        // Guess what buddy! You definitely are passing a string to cowsay
        text.insert(0, '\'');
        text.insert(text.len(), '\'');
        let output = std::process::Command::new("cowsay")
            .arg(text)
            .output()
            // btw, if we can't execute cowsay we crash
            .expect("failed to execute cowsay");
        let mut message = MessageBuilder::new();
        message.push_codeblock(
            String::from_utf8(output.stdout).expect("unable to parse stdout to String"),
            None,
        );
        e!(
            "Error sending message: {:?}",
            msg.channel_id.say(&ctx.http, message.build())
        );
    }
}

fn create_motion(ctx: &Context, msg: &Message, topic: &str) {
    println!("{} created a motion {}", msg.author.name, topic);
    if let Err(why) = msg.delete(ctx) {
        println!("Error deleting motion prompt: {:?}", why);
    }
    match msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|embed| {
            embed.author(|a| {
                a.name(&msg.author.name);
                a.icon_url(
                    msg.author
                        .static_avatar_url()
                        .expect("Expected author to have avatar"),
                );
                a
            });
            embed.colour(serenity::utils::Colour::GOLD);
            embed.title(format!("Motion to {}", topic));
            let mut desc = MessageBuilder::new();
            desc.role(config::VOTE_ROLE);
            desc.push(" take a look at this motion from ");
            desc.mention(&msg.author);
            embed.description(desc.build());
            embed.field("Status", "Under Consideration", true);
            embed.field("Votes", "For: 0\nAgainst: 0\nAbstain: 0", true);
            embed.timestamp(msg.timestamp.to_rfc3339());
            embed
        });
        m.reactions(vec![
            config::FOR_VOTE,
            config::AGAINST_VOTE,
            config::ABSTAIN_VOTE,
            config::APPROVE_REACT,
            config::DISAPPROVE_REACT,
        ]);
        m
    }) {
        Err(why) => {
            println!("Error creating motion: {:?}", why);
        }
        Ok(_) => {}
    }
}

fn create_poll(ctx: &Context, msg: &Message, topic: &str) {
    println!("{} created a poll {}", msg.author.name, topic);
    match msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|embed| {
            embed.author(|a| {
                a.name(&msg.author.name);
                a.icon_url(
                    msg.author
                        .static_avatar_url()
                        .expect("Expected author to have avatar"),
                );
                a
            });
            embed.colour(serenity::utils::Colour::BLUE);
            embed.title(format!("Poll {}", topic));
            let mut desc = MessageBuilder::new();
            desc.mention(&msg.author);
            desc.push(" wants to know what you think.");
            embed.description(desc.build());
            embed.timestamp(msg.timestamp.to_rfc3339());
            embed
        });
        m.reactions(vec![
            config::APPROVE_REACT,
            config::DISAPPROVE_REACT,
            config::UNSURE_REACT,
        ]);
        m
    }) {
        Err(why) => {
            println!("Error sending message: {:?}", why);
        }
        Ok(_) => {
            if let Err(why) = msg.delete(ctx) {
                println!("Error deleting motion prompt: {:?}", why);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct MotionInfo {
    votes: HashMap<&'static str, Vec<serenity::model::user::User>>,
}

lazy_static! {
    static ref MOTIONS_CACHE: Mutex<HashMap<serenity::model::id::MessageId, MotionInfo>> =
        Mutex::new(HashMap::new());
}

fn get_cached_motion(ctx: &Context, msg: &Message) -> MotionInfo {
    let mut cached_motions = MOTIONS_CACHE.lock().unwrap();
    if !cached_motions.contains_key(&msg.id) {
        println!("Initialising representation of motion {:?}", msg.id);
        let this_motion = MotionInfo {
            votes: {
                let mut m = HashMap::new();
                m.insert(
                    config::FOR_VOTE,
                    msg.reaction_users(ctx, config::FOR_VOTE, None, None)
                        .unwrap(),
                );
                m.insert(
                    config::AGAINST_VOTE,
                    msg.reaction_users(ctx, config::AGAINST_VOTE, None, None)
                        .unwrap(),
                );
                m.insert(
                    config::ABSTAIN_VOTE,
                    msg.reaction_users(ctx, config::ABSTAIN_VOTE, None, None)
                        .unwrap(),
                );
                m
            },
        };
        cached_motions.insert(msg.id, this_motion);
    }
    return (*cached_motions.get(&msg.id).unwrap()).clone();
}
fn set_cached_motion(id: &serenity::model::id::MessageId, motion_info: MotionInfo) {
    if let Some(motion) = MOTIONS_CACHE.lock().unwrap().get_mut(id) {
        *motion = motion_info;
    } else {
        println!("{}", "Couldn't find motion in cache to set");
    }
}

fn update_motion(
    ctx: &Context,
    msg: &mut Message,
    user: &serenity::model::user::User,
    change: &str,
    reaction: channel::Reaction,
) {
    let motion_info: MotionInfo = get_cached_motion(ctx, msg);

    let for_votes = motion_info.votes.get(config::FOR_VOTE).unwrap().len() as isize - 1;
    let against_votes = motion_info.votes.get(config::AGAINST_VOTE).unwrap().len() as isize - 1;
    let abstain_votes = motion_info.votes.get(config::ABSTAIN_VOTE).unwrap().len() as isize - 1;

    let has_tiebreaker = |users: &Vec<serenity::model::user::User>| {
        users.iter().any(|u| {
            u.has_role(ctx, config::SERVER_ID, config::TIEBREAKER_ROLE)
                .unwrap()
        })
    };

    let for_strength = for_votes as f32
        + (if has_tiebreaker(motion_info.votes.get(config::FOR_VOTE).unwrap()) {
            0.5
        } else {
            0.0
        });
    let against_strength = against_votes as f32
        + (if has_tiebreaker(motion_info.votes.get(config::AGAINST_VOTE).unwrap()) {
            0.5
        } else {
            0.0
        });
    let abstain_strength = abstain_votes as f32
        + (if has_tiebreaker(motion_info.votes.get(config::ABSTAIN_VOTE).unwrap()) {
            0.5
        } else {
            0.0
        });

    let old_embed = msg.embeds[0].clone();
    let topic = old_embed.clone().title.unwrap();

    println!(
        "  {:10} {:6} {} on {}",
        user.name,
        change,
        reaction.emoji.as_data().as_str(),
        topic
    );

    let update_status = |e: &mut serenity::builder::CreateEmbed,
                         status: &str,
                         last_status_full: String,
                         topic: &str| {
        let last_status = last_status_full.lines().next().expect("No previous status");
        if last_status == status {
            e.field("Status", last_status_full, true);
        } else {
            e.field(
                "Status",
                format!("{}\n_was_ {}", status, last_status_full),
                true,
            );
            println!("Motion to {} now {}", topic, status);
            //
            let mut message = MessageBuilder::new();
            message.push_bold(topic);
            message.push(" is now ");
            message.push_bold(status);
            message.push_italic(format!(" (was {})", last_status));
            if let Err(why) = config::ANNOUNCEMENT_CHANNEL.say(&ctx.http, message.build()) {
                println!("Error sending message: {:?}", why);
            };
        }
    };

    if let Err(why) = msg.edit(ctx, |m| {
        m.embed(|e| {
            e.author(|a| {
                let old_author = old_embed.clone().author.expect("Expected author in embed");
                a.name(old_author.name);
                a.icon_url(
                    old_author
                        .icon_url
                        .expect("Expected embed author to have icon"),
                );
                a
            });
            e.title(&topic);
            e.description(old_embed.description.unwrap());
            let last_status_full = old_embed
                .fields
                .iter()
                .filter(|f| f.name == "Status")
                .next()
                .expect("No previous status")
                .clone()
                .value;
            if for_strength > (config::VOTE_POOL_SIZE / 2) as f32 {
                e.colour(serenity::utils::Colour::TEAL);
                update_status(e, "Passed", last_status_full, &topic);
            } else if against_strength + abstain_strength > (config::VOTE_POOL_SIZE / 2) as f32 {
                e.colour(serenity::utils::Colour::RED);
                update_status(e, "Failed", last_status_full, &topic);
            } else {
                e.colour(serenity::utils::Colour::GOLD);
                update_status(e, "Under Consideration", last_status_full, &topic);
            }
            e.field(
                format!(
                    "Votes ({}/{})",
                    for_votes + against_votes + abstain_votes,
                    config::VOTE_POOL_SIZE
                ),
                format!(
                    "For: {}\nAgainst: {}\nAbstain: {}",
                    for_votes, against_votes, abstain_votes
                ),
                true,
            );
            e.timestamp(
                old_embed
                    .timestamp
                    .expect("Expected embed to have timestamp"),
            );
            e
        })
    }) {
        println!("Error updating motion: {:?}", why);
    }
}

pub fn reaction_add(ctx: Context, add_reaction: channel::Reaction) {
    match add_reaction.message(&ctx.http) {
        Ok(mut message) => {
            if message.author.id.0 == config::BOT_ID {
                if let Ok(user) = add_reaction.user(&ctx) {
                    match user.has_role(&ctx, config::SERVER_ID, config::VOTE_ROLE) {
                        Ok(true) => {
                            for react in
                                [config::FOR_VOTE, config::AGAINST_VOTE, config::ABSTAIN_VOTE]
                                    .iter()
                                    .filter(|r| r != &&add_reaction.emoji.as_data().as_str())
                            {
                                for a_user in
                                    message.reaction_users(&ctx, *react, None, None).unwrap()
                                {
                                    if a_user.id.0 == user.id.0 {
                                        if let Err(why) = add_reaction.delete(&ctx) {
                                            println!("Error deleting react: {:?}", why);
                                        };
                                        return;
                                    }
                                }
                            }
                            if !config::ALLOWED_REACTS
                                .contains(&add_reaction.emoji.as_data().as_str())
                            {
                                if let Err(why) = add_reaction.delete(&ctx) {
                                    println!("Error deleting react: {:?}", why);
                                };
                                return;
                            }
                            if user.id.0 != config::BOT_ID {
                                let mut motion_info = get_cached_motion(&ctx, &message);
                                if let Some(vote) = motion_info
                                    .votes
                                    .get_mut(add_reaction.emoji.as_data().as_str())
                                {
                                    vote.retain(|u| u.id != user.id);
                                    vote.push(user.clone());
                                }
                                set_cached_motion(&message.id, motion_info);
                                update_motion(&ctx, &mut message, &user, "add", add_reaction);
                            }
                        }
                        Ok(false) => {
                            if user.id.0 != config::BOT_ID {
                                if ![config::APPROVE_REACT, config::DISAPPROVE_REACT]
                                    .contains(&add_reaction.emoji.as_data().as_str())
                                {
                                    if let Err(why) = add_reaction.delete(&ctx) {
                                        println!("Error deleting react: {:?}", why);
                                    };
                                    return;
                                }
                            }
                        }
                        Err(why) => {
                            println!("Error getting user role: {:?}", why);
                        }
                    }
                }
            }
        }
        Err(why) => {
            println!("Error processing react: {:?}", why);
        }
    }
}

pub fn reaction_remove(ctx: Context, removed_reaction: channel::Reaction) {
    match removed_reaction.message(&ctx.http) {
        Ok(mut message) => {
            if message.author.id.0 == config::BOT_ID {
                if let Ok(user) = removed_reaction.user(&ctx) {
                    let mut motion_info = get_cached_motion(&ctx, &message);
                    if let Some(vote) = motion_info
                        .votes
                        .get_mut(removed_reaction.emoji.as_data().as_str())
                    {
                        vote.retain(|u| u.id != user.id);
                    }
                    set_cached_motion(&message.id, motion_info);
                    update_motion(&ctx, &mut message, &user, "remove", removed_reaction);
                }
            }
        }
        Err(why) => {
            println!("Error getting user role: {:?}", why);
        }
    }
}