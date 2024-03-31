use crate::{fursona::Fursona, Context, Error};
use poise::serenity_prelude as serenity;
use rand::prelude::SliceRandom;
use std::thread;

/// A simple ping command that responds with "Pong!" and the bot's latency.
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let latency = ctx.ping().await;

    let response = format!("Pong! Latency: `{:?}`", latency);

    ctx.say(response).await?;

    Ok(())
}

/// The parent command for fursona-related commands.
#[poise::command(
    slash_command,
    subcommands("view_fursona", "create_fursona", "random_fursona")
)]
pub async fn fursona(_: Context<'_>) -> Result<(), Error> {
    // This will never be called, because `subcommand_required` parameter is set
    Ok(())
}

/// A command to view a user's fursona.
#[poise::command(slash_command, rename = "view")]
pub async fn view_fursona(
    ctx: Context<'_>,
    #[description = "User to view fursona of"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let fursona = {
        let fursonas = ctx.data().fursonas.lock().unwrap();
        let u = user.as_ref().unwrap_or_else(|| ctx.author());

        fursonas.get(&u.id).cloned()
    };

    if let Some(user) = user {
        if let Some(fursona) = fursona {
            // FIXME: #3 Formatting is incorrect
            let response = format!(
                "{}'s fursona is a **{}** with a **{}** body type, **{}** markings, and the following accessories: {:?}. Their personality is: **{}**",
                user.name,
                fursona.species,
                fursona.body_type,
                fursona.markings,
                fursona.accessories,
                fursona.personality,
            );

            ctx.say(response).await?;
        } else {
            ctx.say(format!("{} doesn't have a fursona set!", user.name))
                .await?;
        }
    } else {
        if let Some(fursona) = fursona {
            // FIXME: #2 Formatting is incorrect
            let response = format!(
                "Your fursona is a **{}** with a **{}** body type, **{}** markings, and the following accessories: {:?}. Their personality is: **{}**",
                fursona.species,
                fursona.body_type,
                fursona.markings,
                fursona.accessories,
                fursona.personality,
            );

            ctx.say(response).await?;
        } else {
            // TODO: #1 Add a button to create a fursona
            ctx.say("You don't have a fursona yet!").await?;
        }
    }

    Ok(())
}

/// A command to create a fursona.
#[poise::command(slash_command, rename = "create")]
pub async fn create_fursona(ctx: Context<'_>) -> Result<(), Error> {
    // Check if user already has a fursona set
    let fursona = {
        let fursonas = ctx.data().fursonas.lock().unwrap();
        let u = ctx.author();

        fursonas.get(&u.id).cloned()
    };

    if let Some(_) = fursona {
        let reply = {
            let components = vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("create_fursona")
                    .label("Create New Fursona")
                    .style(serenity::ButtonStyle::Primary),
                serenity::CreateButton::new("cancel")
                    .label("Cancel")
                    .style(serenity::ButtonStyle::Danger),
            ])];

            poise::CreateReply::default()
                .ephemeral(true)
                .content("You already have a fursona set!")
                .components(components)
        };

        let reply_msg = ctx.send(reply).await?;

        while let Some(interaction) = serenity::ComponentInteractionCollector::new(ctx)
            .author_id(ctx.author().id)
            .channel_id(ctx.channel_id())
            .timeout(std::time::Duration::from_secs(60))
            .await
        {
            match interaction.data.custom_id.as_str() {
                "create_fursona" => {
                    interaction.defer(ctx).await?;

                    // Create a fursona with the user's input
                    create_new_fursona(ctx, Some(reply_msg)).await?;

                    return Ok(());
                }
                "cancel" => {
                    cancel_action(ctx, reply_msg).await?;

                    return Ok(());
                }
                _ => {}
            }
        }

        let reply = poise::CreateReply::default()
            .content("Timed out")
            .components(vec![]);

        reply_msg.edit(ctx, reply).await?;
    } else {
        // Create a fursona with the user's input
        create_new_fursona(ctx, None).await?;
    }

    Ok(())
}

async fn create_new_fursona(
    ctx: Context<'_>,
    msg: Option<poise::ReplyHandle<'_>>,
) -> Result<(), Error> {
    let msg = match msg {
        Some(msg) => msg,
        None => {
            let reply = poise::CreateReply::default()
                .ephemeral(true)
                .content("Let's create a new fursona!")
                .components(vec![]);

            let msg = ctx.send(reply).await?;

            thread::sleep(std::time::Duration::from_secs(1));

            msg
        }
    };

    // Dropdowns for species, body type, accessories, markings, and personality
    // REWRITE: This is a mess, refactor this into a more readable and maintainable state

    // Species
    let species_options = vec![
        serenity::CreateSelectMenuOption::new("Dog", "dog"),
        serenity::CreateSelectMenuOption::new("Cat", "cat"),
        serenity::CreateSelectMenuOption::new("Fox", "fox"),
        serenity::CreateSelectMenuOption::new("Wolf", "wolf"), // TODO: #4 Add more species
    ];

    let mut selected_species = String::new();

    let species_page = poise::CreateReply::default()
        .ephemeral(true)
        .content("What species is your fursona?")
        .components(vec![
            serenity::CreateActionRow::SelectMenu(
                serenity::CreateSelectMenu::new(
                    "species",
                    serenity::CreateSelectMenuKind::String {
                        options: species_options,
                    },
                )
                .placeholder("Select a species")
                .min_values(1)
                .max_values(1),
            ),
            serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("back")
                    .label("Go Back")
                    .style(serenity::ButtonStyle::Primary)
                    .disabled(true),
                serenity::CreateButton::new("next")
                    .label("Next")
                    .style(serenity::ButtonStyle::Success),
                serenity::CreateButton::new("cancel")
                    .label("Cancel")
                    .style(serenity::ButtonStyle::Danger),
            ]),
        ]);

    // body type
    let body_type_options = vec![
        serenity::CreateSelectMenuOption::new("Slim", "slim"),
        serenity::CreateSelectMenuOption::new("Average", "average"),
        serenity::CreateSelectMenuOption::new("Muscular", "muscular"),
        serenity::CreateSelectMenuOption::new("Fluffy", "fluffy"),
        serenity::CreateSelectMenuOption::new("Chubby", "chubby"),
    ];

    let mut selected_body_type = String::new();

    let body_type_page = poise::CreateReply::default()
        .ephemeral(true)
        .content("What body type is your fursona?")
        .components(vec![
            serenity::CreateActionRow::SelectMenu(
                serenity::CreateSelectMenu::new(
                    "body_type",
                    serenity::CreateSelectMenuKind::String {
                        options: body_type_options,
                    },
                )
                .placeholder("Select a body type")
                .min_values(1)
                .max_values(1),
            ),
            serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("back")
                    .label("Go Back")
                    .style(serenity::ButtonStyle::Primary),
                serenity::CreateButton::new("next")
                    .label("Next")
                    .style(serenity::ButtonStyle::Success),
                serenity::CreateButton::new("cancel")
                    .label("Cancel")
                    .style(serenity::ButtonStyle::Danger),
            ]),
        ]);

    // markings
    let markings_options = vec![
        serenity::CreateSelectMenuOption::new("None", "none"),
        serenity::CreateSelectMenuOption::new("Stripes", "stripes"),
        serenity::CreateSelectMenuOption::new("Spots", "spots"),
        serenity::CreateSelectMenuOption::new("Solid", "solid"),
    ];

    let mut selected_markings = String::new();

    let markings_page = poise::CreateReply::default()
        .ephemeral(true)
        .content("What markings does your fursona have?")
        .components(vec![
            serenity::CreateActionRow::SelectMenu(
                serenity::CreateSelectMenu::new(
                    "markings",
                    serenity::CreateSelectMenuKind::String {
                        options: markings_options,
                    },
                )
                .placeholder("Select markings")
                .min_values(1)
                .max_values(1),
            ),
            serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("back")
                    .label("Go Back")
                    .style(serenity::ButtonStyle::Primary),
                serenity::CreateButton::new("next")
                    .label("Next")
                    .style(serenity::ButtonStyle::Success),
                serenity::CreateButton::new("cancel")
                    .label("Cancel")
                    .style(serenity::ButtonStyle::Danger),
            ]),
        ]);

    // accessories
    let accessories_options = vec![
        serenity::CreateSelectMenuOption::new("Glasses", "glasses"),
        serenity::CreateSelectMenuOption::new("Scarf", "scarf"),
        serenity::CreateSelectMenuOption::new("Hat", "hat"),
        serenity::CreateSelectMenuOption::new("Collar", "collar"),
        serenity::CreateSelectMenuOption::new("Jewellery", "jewellery"),
        serenity::CreateSelectMenuOption::new("Wings", "wings"),
        serenity::CreateSelectMenuOption::new("Tail", "tail"),
        serenity::CreateSelectMenuOption::new("Horns", "horns"),
    ];

    let mut selected_accessories = Vec::new();

    let accessories_page = poise::CreateReply::default()
        .ephemeral(true)
        .content("What accessories does your fursona have?")
        .components(vec![
            serenity::CreateActionRow::SelectMenu(
                serenity::CreateSelectMenu::new(
                    "accessories",
                    serenity::CreateSelectMenuKind::String {
                        options: accessories_options,
                    },
                )
                .placeholder("Select accessories")
                .min_values(0)
                .max_values(3),
            ),
            serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("back")
                    .label("Go Back")
                    .style(serenity::ButtonStyle::Primary),
                serenity::CreateButton::new("next")
                    .label("Next")
                    .style(serenity::ButtonStyle::Success),
                serenity::CreateButton::new("cancel")
                    .label("Cancel")
                    .style(serenity::ButtonStyle::Danger),
            ]),
        ]);

    // personality
    let personality_options = vec![
        serenity::CreateSelectMenuOption::new("Shy", "shy"),
        serenity::CreateSelectMenuOption::new("Friendly", "friendly"),
        serenity::CreateSelectMenuOption::new("Silly", "silly"),
        serenity::CreateSelectMenuOption::new("Brave", "brave"),
        serenity::CreateSelectMenuOption::new("Caring", "caring"),
        serenity::CreateSelectMenuOption::new("Mischievous", "mischievous"),
    ];

    let mut selected_personality = String::new();

    let personality_page = poise::CreateReply::default()
        .ephemeral(true)
        .content("What is your fursona's personality?")
        .components(vec![
            serenity::CreateActionRow::SelectMenu(
                serenity::CreateSelectMenu::new(
                    "personality",
                    serenity::CreateSelectMenuKind::String {
                        options: personality_options,
                    },
                )
                .placeholder("Select personality")
                .min_values(1)
                .max_values(1),
            ),
            serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("back")
                    .label("Go Back")
                    .style(serenity::ButtonStyle::Primary),
                serenity::CreateButton::new("create")
                    .label("Create Fursona")
                    .style(serenity::ButtonStyle::Success),
                serenity::CreateButton::new("cancel")
                    .label("Cancel")
                    .style(serenity::ButtonStyle::Danger),
            ]),
        ]);

    // page management

    let mut page = 0;
    let pages = vec![
        species_page,
        body_type_page,
        markings_page,
        accessories_page,
        personality_page,
    ];

    // Reply to the user with the first page listen for interactions

    let reply = pages[page].clone();

    msg.edit(ctx, reply).await?;

    while let Some(interaction) = serenity::ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(300))
        .await
    {
        match interaction.data.custom_id.as_str() {
            "species" => {
                selected_species = match &interaction.data.kind {
                    serenity::ComponentInteractionDataKind::StringSelect { values } => {
                        values[0].clone()
                    }
                    _ => panic!("unexpected interaction data kind"),
                };

                let reply = pages[page].clone().content(format!(
                    "What species is your fursona? Selected: {selected_species}"
                ));

                msg.edit(ctx, reply).await?;
            }
            "body_type" => {
                selected_body_type = match &interaction.data.kind {
                    serenity::ComponentInteractionDataKind::StringSelect { values } => {
                        values[0].clone()
                    }
                    _ => panic!("unexpected interaction data kind"),
                };

                let reply = pages[page].clone().content(format!(
                    "What body type is your fursona? Selected: {selected_body_type}"
                ));

                msg.edit(ctx, reply).await?;
            }
            "markings" => {
                selected_markings = match &interaction.data.kind {
                    serenity::ComponentInteractionDataKind::StringSelect { values } => {
                        values[0].clone()
                    }
                    _ => panic!("unexpected interaction data kind"),
                };

                let reply = pages[page].clone().content(format!(
                    "What markings does your fursona have? Selected: {selected_markings}"
                ));

                msg.edit(ctx, reply).await?;
            }
            "accessories" => {
                selected_accessories = match &interaction.data.kind {
                    serenity::ComponentInteractionDataKind::StringSelect { values } => {
                        values.clone()
                    }
                    _ => panic!("unexpected interaction data kind"),
                };

                // FIXME: #5 Formatting is incorrect
                let reply = pages[page].clone().content(format!(
                    "What accessories does your fursona have? Selected: {:?}",
                    selected_accessories
                ));

                msg.edit(ctx, reply).await?;
            }
            "personality" => {
                selected_personality = match &interaction.data.kind {
                    serenity::ComponentInteractionDataKind::StringSelect { values } => {
                        values[0].clone()
                    }
                    _ => panic!("unexpected interaction data kind"),
                };

                let reply = pages[page].clone().content(format!(
                    "What is your fursona's personality? Selected: {selected_personality}"
                ));

                msg.edit(ctx, reply).await?;
            }
            "back" => {
                if page > 0 {
                    page -= 1;
                }

                // FIXME: #6 This is will not display the selected values
                let reply = pages[page].clone();

                msg.edit(ctx, reply).await?;
            }
            "next" => {
                if page < pages.len() - 1 {
                    page += 1;
                }

                let reply = pages[page].clone();

                msg.edit(ctx, reply).await?;
            }
            "cancel" => {
                cancel_action(ctx, msg).await?;

                return Ok(());
            }
            "create" => {
                // Create the fursona
                let fursona = Fursona::new(
                    selected_species,
                    selected_body_type,
                    selected_markings,
                    selected_accessories,
                    selected_personality,
                );

                {
                    let mut fursonas = ctx.data().fursonas.lock().unwrap();

                    fursonas.insert(ctx.author().id, fursona);
                }

                let reply = poise::CreateReply::default()
                    .ephemeral(true)
                    .content("Fursona created!")
                    .components(vec![]);

                msg.edit(ctx, reply).await?;

                return Ok(());
            }
            _ => {}
        }

        interaction.defer(ctx).await?;
    }

    let reply = poise::CreateReply::default()
        .content("Timed out")
        .components(vec![]);

    msg.edit(ctx, reply).await?;

    Ok(())
}

async fn cancel_action(ctx: Context<'_>, msg: poise::ReplyHandle<'_>) -> Result<(), Error> {
    let reply = poise::CreateReply::default()
        .content("Action cancelled")
        .components(vec![]);

    msg.edit(ctx, reply).await?;

    Ok(())
}

/// A command to generate a random fursona.
#[poise::command(slash_command, rename = "random")]
pub async fn random_fursona(ctx: Context<'_>) -> Result<(), Error> {
    // Check if user already has a fursona set
    let fursona = {
        let fursonas = ctx.data().fursonas.lock().unwrap();
        let u = ctx.author();

        fursonas.get(&u.id).cloned()
    };

    if let Some(_) = fursona {
        let reply = {
            let components = vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("create_fursona")
                    .label("Create Random Fursona")
                    .style(serenity::ButtonStyle::Primary),
                serenity::CreateButton::new("cancel")
                    .label("Cancel")
                    .style(serenity::ButtonStyle::Danger),
            ])];

            poise::CreateReply::default()
                .ephemeral(true)
                .content("You already have a fursona set!")
                .components(components)
        };

        let reply_msg = ctx.send(reply).await?;

        while let Some(interaction) = serenity::ComponentInteractionCollector::new(ctx)
            .author_id(ctx.author().id)
            .channel_id(ctx.channel_id())
            .timeout(std::time::Duration::from_secs(60))
            .await
        {
            match interaction.data.custom_id.as_str() {
                "create_fursona" => {
                    interaction.defer(ctx).await?;

                    // Create a random fursona
                    create_random_fursona(ctx, Some(reply_msg)).await?;

                    return Ok(());
                }
                "cancel" => {
                    cancel_action(ctx, reply_msg).await?;

                    return Ok(());
                }
                _ => {}
            }
        }

        let reply = poise::CreateReply::default()
            .content("Timed out")
            .components(vec![]);

        reply_msg.edit(ctx, reply).await?;
    } else {
        // Create a random fursona
        create_random_fursona(ctx, None).await?;
    }

    Ok(())
}

async fn create_random_fursona(
    ctx: Context<'_>,
    msg: Option<poise::ReplyHandle<'_>>,
) -> Result<(), Error> {
    let msg = match msg {
        Some(msg) => msg,
        None => {
            let reply = poise::CreateReply::default()
                .ephemeral(true)
                .content("Creating Random Fursona...")
                .components(vec![]);

            let msg = ctx.send(reply).await?;

            thread::sleep(std::time::Duration::from_secs(1));

            msg
        }
    };

    // Generate a random fursona
    let species = ["Dog", "Cat", "Fox", "Wolf"];
    let body_type = ["Slim", "Average", "Muscular", "Fluffy", "Chubby"];
    let markings = ["None", "Stripes", "Spots", "Solid"];
    let accessories = [
        "Glasses",
        "Scarf",
        "Hat",
        "Collar",
        "Jewellery",
        "Wings",
        "Tail",
        "Horns",
    ];
    let personality = ["Shy", "Friendly", "Silly", "Brave", "Caring", "Mischievous"];

    let fursona = Fursona::new(
        species[rand::random::<usize>() % species.len()].to_string(),
        body_type[rand::random::<usize>() % body_type.len()].to_string(),
        markings[rand::random::<usize>() % markings.len()].to_string(),
        accessories
            .choose_multiple(&mut rand::thread_rng(), rand::random::<usize>() % 3)
            .map(|s| s.to_string())
            .collect(),
        personality[rand::random::<usize>() % personality.len()].to_string(),
    );

    {
        let mut fursonas = ctx.data().fursonas.lock().unwrap();

        fursonas.insert(ctx.author().id, fursona);
    }

    let reply = poise::CreateReply::default()
        .ephemeral(true)
        .content("Random Fursona created!")
        .components(vec![]);

    msg.edit(ctx, reply).await?;

    Ok(())
}
