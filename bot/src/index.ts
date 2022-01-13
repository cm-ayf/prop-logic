import { ApplicationCommandData, Client, Intents } from "discord.js";
import dotenv from "dotenv";
import { main } from "prop-logic";

dotenv.config();

const GUILD_ID = process.env.GUILD_ID ?? process.exit(1);
const BOT_TOKEN = process.env.BOT_TOKEN ?? process.exit(1);


const client = new Client({ intents: [
  Intents.FLAGS.GUILD_MESSAGES
]});

const commands: ApplicationCommandData[] = [
  {
    name: 'solve',
    description: 'solves propositional logic (best effort)',
    options: [
      {
        name: 'input',
        description: 'logic to solve',
        type: 'STRING',
        required: true
      },
      {
        name: 'tex',
        description: 'outputs in TeX',
        type: 'BOOLEAN',
        required: false,
      }
    ]
  },
  {
    name: 'help',
    description: 'shows help'
  }
];

client.on('ready', client => {
  console.log(`logged in as ${client.user.tag}`);

  client.guilds.fetch(GUILD_ID)
    .then(guild => guild.commands.set(commands))
    .then(() => console.log('command initialized.'), console.error);
})

client.on('interactionCreate', async interaction => {
  if (!interaction.isCommand()) return;
  switch (interaction.commandName) {
    case 'solve': {
      await interaction.deferReply();
      let input = interaction.options.getString('input', true);
      let tex = interaction.options.getBoolean('tex') ?? false;
      try {
        interaction.editReply(`\`\`\`${tex ? 'latex' : ''}\n${main(input, tex)}\`\`\``);
      } catch (e) {
        interaction.editReply(`${e}`);
      }
      break;
    }
    case 'help': {
      interaction.reply({
        content: 'https://github.com/cm-ayf/prop-logic/tree/wasm-discord-bot#使い方',
        ephemeral: true
      });
      break;
    }
  }
});

process.on('SIGINT', async () => {
  await client.application?.commands.set([], GUILD_ID);
  process.exit(0);
});

client.login(BOT_TOKEN);